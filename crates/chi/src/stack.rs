use std::fs;
use std::path::{Path, PathBuf};
use std::str::FromStr;

use anyhow::{anyhow, bail, Context, Result};
use clap::{Args, ValueEnum};
use domain::{
    DiscountRate, GpuModel, Kilowatt, Pue, ThetaExResidual, Usd, UsdPerGpuHour, UsdPerKwh,
    Utilization, Years,
};
use ingest::cache::gpu_slug;
use ingest::epoch::{parse_ml_hardware_csv, row_for_gpu};
use ingest::lmp::parse_lmp_wrapper;
use ingest::ocpi_daily::{gpu_model_from_ocpi_name, parse_daily_index_wrapper};
use project::{default_pue_grid, CostStack, NamedEnergy};
use rust_decimal::prelude::ToPrimitive;
use rust_decimal::Decimal;

#[derive(Clone, Copy, Debug, Default, ValueEnum)]
enum OutputFormat {
    #[default]
    Text,
    Json,
}

#[derive(Args, Debug)]
pub struct StackArgs {
    /// OCPI GPU name, e.g. "H100 SXM".
    #[arg(long)]
    gpu: String,

    /// Fixture directory. Stack \(S\) is `{fixture-dir}/ocpi/daily-index/{slug}.json` only.
    #[arg(long)]
    fixture_dir: PathBuf,

    /// Declared purchase \(P\) in integer USD cents. Not Epoch release price.
    #[arg(long)]
    purchase_cents: i64,

    /// Declared economic life \(T\) in years.
    #[arg(long)]
    life_years: u32,

    /// Declared utilization \(u\) in (0, 1].
    #[arg(long)]
    utilization: String,

    /// Declared annual discount rate \(r\). Zero is allowed.
    #[arg(long)]
    discount_rate: String,

    /// Named LMP fixture path (USD/MWh wrapper). Mutually exclusive with --energy-usd-per-kwh.
    #[arg(long)]
    lmp_fixture: Option<PathBuf>,

    /// Energy price \(\pi\) in USD/kWh. Requires --energy-source when no --lmp-fixture.
    #[arg(long)]
    energy_usd_per_kwh: Option<String>,

    /// Human-readable source label for --energy-usd-per-kwh.
    #[arg(long)]
    energy_source: Option<String>,

    /// Datacenter PUE (wall ÷ IT). Repeatable; omit ⇒ default grid 1.0, 1.2, 1.5.
    #[arg(long, action = clap::ArgAction::Append)]
    pue: Vec<f64>,

    #[arg(long, value_enum, default_value_t = OutputFormat::Text)]
    format: OutputFormat,
}

pub fn run(args: StackArgs) -> Result<()> {
    let named_energy = resolve_named_energy(&args)?;

    let gpu =
        gpu_model_from_ocpi_name(&args.gpu).ok_or_else(|| anyhow!("unknown gpu {:?}", args.gpu))?;

    let slug = gpu_slug(&args.gpu);
    let spot_path = args
        .fixture_dir
        .join("ocpi/daily-index")
        .join(format!("{slug}.json"));
    let spot_bytes = fs::read(&spot_path)
        .with_context(|| format!("missing ocpi.daily-index fixture {}", spot_path.display()))?;
    let record = parse_daily_index_wrapper(&spot_bytes)?;
    if record.spot.gpu != gpu {
        bail!(
            "daily-index gpu {:?} does not match --gpu {:?}",
            ocpi_name(record.spot.gpu),
            args.gpu
        );
    }

    let epoch_path = args.fixture_dir.join("epoch/ml_hardware.excerpt.csv");
    let epoch_bytes = fs::read(&epoch_path)
        .with_context(|| format!("missing Epoch excerpt {}", epoch_path.display()))?;
    let rows = parse_ml_hardware_csv(&epoch_bytes)?;
    let epoch = row_for_gpu(gpu, &rows, true)?;
    let tdp_kw = (epoch.tdp_w / Decimal::from(1000))
        .to_f64()
        .ok_or_else(|| anyhow!("Epoch TDP is not a finite kW"))?;
    let tdp = Kilowatt::try_new(tdp_kw)?;

    let pues = resolve_pues(&args.pue)?;

    let purchase = Usd::from_cents(args.purchase_cents);
    let life = Years::try_new(args.life_years)?;
    let utilization = Utilization::try_new(parse_decimal(&args.utilization, "utilization")?)?;
    let discount = DiscountRate::try_new(parse_decimal(&args.discount_rate, "discount-rate")?)?;

    // capital_ex.energy is ignored by CostStack::compute; each row sets e from TDP·π·PUE.
    let capital_ex = ThetaExResidual {
        purchase,
        life,
        utilization,
        energy: UsdPerGpuHour::from_cents(0),
        discount,
    };

    let stack = CostStack::compute(record.spot, capital_ex, tdp, named_energy, &pues)?;

    let source = Path::new(&args.fixture_dir)
        .join("ocpi/daily-index")
        .join(format!("{slug}.json"));
    let valid_on = record.spot.valid_on.get();
    let s_token = record.spot.price.amount().to_string();

    match args.format {
        OutputFormat::Text => print!(
            "{}",
            render_text(TextReport {
                gpu: &args.gpu,
                s: stack.spot,
                s_token: &s_token,
                valid_on,
                source: &source,
                tdp_kw,
                stack: &stack,
            })
        ),
        OutputFormat::Json => print!(
            "{}",
            render_json(JsonReport {
                gpu: &args.gpu,
                s_token: &s_token,
                valid_on,
                source: &source,
                tdp_kw,
                stack: &stack,
            })?
        ),
    }
    Ok(())
}

fn resolve_named_energy(args: &StackArgs) -> Result<NamedEnergy> {
    let has_lmp = args.lmp_fixture.is_some();
    let has_pi = args.energy_usd_per_kwh.is_some();
    let has_src = args.energy_source.is_some();

    match (has_lmp, has_pi, has_src) {
        (true, false, false) => {
            let path = args.lmp_fixture.as_ref().expect("lmp checked");
            let bytes = fs::read(path)
                .with_context(|| format!("missing LMP fixture {}", path.display()))?;
            let record = parse_lmp_wrapper(&bytes)?;
            Ok(NamedEnergy {
                usd_per_kwh: record.usd_per_kwh,
                lmp_usd_per_mwh: Some(record.lmp_usd_per_mwh),
                source: format!("{} {} {}", record.iso, record.node, record.valid_on.get()),
            })
        }
        (false, true, true) => {
            let text = args.energy_usd_per_kwh.as_ref().expect("pi checked");
            let source = args.energy_source.as_ref().expect("source checked").clone();
            let usd_per_kwh = UsdPerKwh::try_from(parse_decimal(text, "energy-usd-per-kwh")?)?;
            Ok(NamedEnergy {
                usd_per_kwh,
                lmp_usd_per_mwh: None,
                source,
            })
        }
        (true, true, _) | (true, _, true) => {
            bail!("use exactly one energy identity: --lmp-fixture or (--energy-usd-per-kwh and --energy-source)")
        }
        (false, true, false) | (false, false, true) => {
            bail!(
                "named π requires both --energy-usd-per-kwh and --energy-source (or pass --lmp-fixture)"
            )
        }
        (false, false, false) => {
            bail!(
                "named π required: pass --lmp-fixture, or both --energy-usd-per-kwh and --energy-source"
            )
        }
    }
}

fn resolve_pues(values: &[f64]) -> Result<Vec<Pue>> {
    if values.is_empty() {
        return Ok(default_pue_grid().to_vec());
    }
    let mut out = Vec::with_capacity(values.len());
    for &pue_value in values {
        if !pue_value.is_finite() || pue_value <= 0.0 {
            bail!("--pue must be a finite number > 0, got {pue_value}");
        }
        out.push(Pue::try_new(pue_value)?);
    }
    Ok(out)
}

fn parse_decimal(text: &str, flag: &str) -> Result<Decimal> {
    Decimal::from_str(text).with_context(|| format!("invalid --{flag} {text:?}"))
}

fn ocpi_name(gpu: GpuModel) -> &'static str {
    match gpu {
        GpuModel::H100Sxm => "H100 SXM",
        GpuModel::H200 => "H200",
        GpuModel::B200 => "B200",
        GpuModel::A100Sxm4 => "A100 SXM4",
        GpuModel::Rtx5090 => "RTX 5090",
    }
}

fn json_rate(rate: UsdPerGpuHour) -> String {
    rate.amount().round_dp(12).to_string()
}

fn format_pue(pue: f64) -> String {
    let text = pue.to_string();
    if text.contains('.') {
        text
    } else {
        format!("{text}.0")
    }
}

struct TextReport<'a> {
    gpu: &'a str,
    s: UsdPerGpuHour,
    s_token: &'a str,
    valid_on: time::Date,
    source: &'a Path,
    tdp_kw: f64,
    stack: &'a CostStack,
}

fn render_text(r: TextReport<'_>) -> String {
    let mut out = String::new();
    out.push_str(&format!("chi stack  gpu={}  format=text\n\n", r.gpu));
    out.push_str(&format!(
        "S (OCPI daily-index)      {} USD / GPU-hour   [token {}]\n",
        r.s, r.s_token
    ));
    out.push_str(&format!("  valid_on                {}\n", r.valid_on));
    out.push_str("  series                  ocpi.daily-index\n");
    out.push_str(&format!(
        "  source                  {}\n\n",
        r.source.display()
    ));

    let ne = &r.stack.named_energy;
    out.push_str("Named π\n");
    if let Some(mwh) = &ne.lmp_usd_per_mwh {
        out.push_str(&format!("  LMP                     {} USD/MWh\n", mwh));
    }
    out.push_str(&format!(
        "  π                       {} USD/kWh\n",
        ne.usd_per_kwh.amount()
    ));
    out.push_str(&format!("  source                  {}\n", ne.source));
    out.push_str(&format!("  TDP                     {} kW\n\n", r.tdp_kw));

    out.push_str("PUE | e | F_capital | L leftover | e/S\n");
    for row in &r.stack.rows {
        out.push_str(&format!(
            "{} | {} | {} | {} | {}\n",
            format_pue(row.pue.get()),
            row.energy,
            row.capital,
            row.leftover,
            row.energy_share
        ));
    }
    out.push('\n');
    out.push_str("Notes\n");
    out.push_str("  Invert S is OCPI daily-index; hourly current was not used.\n");
    out.push_str("  Leftover L is unexplained rent after capital and energy.\n");
    out.push_str("  Salvage is not in this panel.\n");
    out
}

struct JsonReport<'a> {
    gpu: &'a str,
    s_token: &'a str,
    valid_on: time::Date,
    source: &'a Path,
    tdp_kw: f64,
    stack: &'a CostStack,
}

fn render_json(r: JsonReport<'_>) -> Result<String> {
    let ne = &r.stack.named_energy;
    let rows: Vec<serde_json::Value> = r
        .stack
        .rows
        .iter()
        .map(|row| {
            serde_json::json!({
                "pue": row.pue.get(),
                "energy_usd_per_gpu_hour": json_rate(row.energy),
                "capital_rent_usd_per_gpu_hour": json_rate(row.capital),
                "leftover_usd_per_gpu_hour": json_rate(row.leftover),
                "energy_share": row.energy_share.round_dp(12).to_string(),
            })
        })
        .collect();

    let value = serde_json::json!({
        "gpu": r.gpu,
        "spot": {
            "series": "ocpi.daily-index",
            "s_usd_per_gpu_hour": r.s_token,
            "valid_on": r.valid_on.to_string(),
            "source": r.source.display().to_string(),
        },
        "energy": {
            "usd_per_kwh": ne.usd_per_kwh.amount().to_string(),
            "lmp_usd_per_mwh": ne.lmp_usd_per_mwh,
            "source": ne.source,
            "tdp_kw": r.tdp_kw,
        },
        "rows": rows,
    });
    Ok(format!("{value}\n"))
}
