use std::fs;
use std::path::{Path, PathBuf};
use std::str::FromStr;

use anyhow::{anyhow, bail, Context, Result};
use clap::{Args, ValueEnum};
use domain::{
    capital_rent, energy_per_gpu_hour, fair_rent, DiscountRate, GpuModel, Kilowatt, Pue, Theta,
    ThetaExResidual, Usd, UsdPerGpuHour, UsdPerKwh, Utilization, Years,
};
use ingest::cache::gpu_slug;
use ingest::epoch::{parse_ml_hardware_csv, row_for_gpu};
use ingest::ocpi_daily::{gpu_model_from_ocpi_name, parse_daily_index_wrapper};
use project::NamedInverses;
use rust_decimal::prelude::ToPrimitive;
use rust_decimal::Decimal;
use time::UtcOffset;

#[derive(Clone, Copy, Debug, Default, ValueEnum)]
enum OutputFormat {
    #[default]
    Text,
    Json,
}

#[derive(Args, Debug)]
pub struct InvertArgs {
    /// OCPI GPU name, e.g. "H100 SXM".
    #[arg(long)]
    gpu: String,

    /// Fixture directory. Invert \(S\) is `{fixture-dir}/ocpi/daily-index/{slug}.json` only.
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

    /// Declared salvage \(R\) in integer USD cents. Required to print \(F(\theta)\).
    #[arg(long)]
    residual_cents: Option<i64>,

    /// Energy price \(\pi\). Omitted ⇒ 0; the TDP·h·PUE product is still evaluated.
    #[arg(long)]
    energy_usd_per_kwh: Option<String>,

    #[arg(long, value_enum, default_value_t = OutputFormat::Text)]
    format: OutputFormat,
}

pub fn run(args: InvertArgs) -> Result<()> {
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
    let pue = Pue::try_new(1.0)?;
    let pi = match &args.energy_usd_per_kwh {
        Some(text) => UsdPerKwh::try_from(parse_decimal(text, "energy-usd-per-kwh")?)?,
        None => UsdPerKwh::from_cents(0),
    };
    let energy = energy_per_gpu_hour(tdp, pi, pue)?;

    let purchase = Usd::from_cents(args.purchase_cents);
    let life = Years::try_new(args.life_years)?;
    let utilization = Utilization::try_new(parse_decimal(&args.utilization, "utilization")?)?;
    let discount = DiscountRate::try_new(parse_decimal(&args.discount_rate, "discount-rate")?)?;

    let ex = ThetaExResidual {
        purchase,
        life,
        utilization,
        energy,
        discount,
    };
    let inverses = NamedInverses::compute(record.spot, &ex)?;
    let f_capital = capital_rent(&ex)?;
    let hours = ex.utilized_hours_per_year()?;

    let accounting_life = Years::try_new(6)?;
    let accounting_ex = ThetaExResidual {
        purchase,
        life: accounting_life,
        utilization,
        energy,
        discount,
    };
    let accounting_theta = Theta {
        purchase,
        life: accounting_life,
        utilization,
        salvage: Usd::from_cents(0),
        energy,
        discount,
    };
    let f_acct = fair_rent(&accounting_theta)?;
    let accounting_inverses = NamedInverses::compute(record.spot, &accounting_ex)?;

    let forward = match args.residual_cents {
        Some(cents) => {
            let theta = Theta {
                purchase,
                life,
                utilization,
                salvage: Usd::from_cents(cents),
                energy,
                discount,
            };
            Some(fair_rent(&theta)?)
        }
        None => None,
    };

    let source = Path::new(&args.fixture_dir)
        .join("ocpi/daily-index")
        .join(format!("{slug}.json"));
    let fetched_at = format_fetched_at(record.fetched_at.get())?;
    let valid_on = record.spot.valid_on.get();
    let s_token = record.spot.price.amount().to_string();

    match args.format {
        OutputFormat::Text => print!(
            "{}",
            render_text(TextReport {
                gpu: &args.gpu,
                s: record.spot.price,
                s_token: &s_token,
                valid_on,
                fetched_at: &fetched_at,
                source: &source,
                purchase,
                life,
                utilization,
                energy,
                tdp_kw,
                pi,
                discount,
                hours,
                f_capital,
                inverses,
                forward,
                f_acct,
                accounting_inverses,
            })
        ),
        OutputFormat::Json => print!(
            "{}",
            render_json(JsonReport {
                gpu: &args.gpu,
                s_token: &s_token,
                valid_on,
                fetched_at: &fetched_at,
                source: &source,
                purchase,
                life,
                utilization,
                energy,
                tdp_kw,
                pi,
                discount,
                f_capital,
                inverses,
                forward,
                f_acct,
                accounting_inverses,
                residual_cents: args.residual_cents,
            })?
        ),
    }
    Ok(())
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

fn format_fetched_at(when: time::OffsetDateTime) -> Result<String> {
    let when = when.to_offset(UtcOffset::UTC);
    Ok(format!(
        "{:04}-{:02}-{:02}T{:02}:{:02}:{:02}.{:03}Z",
        when.year(),
        u8::from(when.month()),
        when.day(),
        when.hour(),
        when.minute(),
        when.second(),
        when.millisecond(),
    ))
}

fn json_rate(rate: UsdPerGpuHour) -> String {
    rate.amount().round_dp(12).to_string()
}

fn json_usd(usd: Usd) -> String {
    usd.amount().round_dp(12).to_string()
}

struct TextReport<'a> {
    gpu: &'a str,
    s: UsdPerGpuHour,
    s_token: &'a str,
    valid_on: time::Date,
    fetched_at: &'a str,
    source: &'a Path,
    purchase: Usd,
    life: Years,
    utilization: Utilization,
    energy: UsdPerGpuHour,
    tdp_kw: f64,
    pi: UsdPerKwh,
    discount: DiscountRate,
    hours: domain::GpuHour,
    f_capital: UsdPerGpuHour,
    inverses: NamedInverses,
    forward: Option<UsdPerGpuHour>,
    f_acct: UsdPerGpuHour,
    accounting_inverses: NamedInverses,
}

fn render_text(r: TextReport<'_>) -> String {
    let mut out = String::new();
    out.push_str(&format!("chi invert  gpu={}  format=text\n\n", r.gpu));
    out.push_str(&format!(
        "S (OCPI daily-index)      {} USD / GPU-hour\n",
        r.s
    ));
    out.push_str(&format!("  valid_on                {}\n", r.valid_on));
    out.push_str(&format!("  fetched_at              {}\n", r.fetched_at));
    out.push_str("  series                  ocpi.daily-index\n");
    out.push_str(&format!(
        "  source                  {}\n\n",
        r.source.display()
    ));
    let salvage_note = if r.forward.is_some() {
        ""
    } else {
        "   [no salvage declared]"
    };
    out.push_str(&format!("Declared (P, T, u, e, r){salvage_note}\n"));
    out.push_str(&format!(
        "  P                       {} USD / GPU     [declared point; not an MSRP]\n",
        r.purchase
    ));
    out.push_str(&format!(
        "  T                       {} years\n",
        r.life.get()
    ));
    out.push_str(&format!(
        "  u                       {}\n",
        r.utilization.amount()
    ));
    out.push_str(&format!(
        "  e                       {} USD / GPU-hour    [{} kW · 1 h · {} USD/kWh · PUE 1.0]\n",
        r.energy,
        r.tdp_kw,
        r.pi.amount()
    ));
    out.push_str(&format!(
        "  r                       {} / year\n",
        r.discount.amount()
    ));
    out.push_str(&format!(
        "  h                       {} GPU-hour / year\n",
        r.hours.amount()
    ));
    out.push_str(&format!(
        "  F_capital               {} USD / GPU-hour\n\n",
        r.f_capital
    ));
    out.push_str("Inverse\n");
    out.push_str(&format!(
        "  L leftover              {} USD / GPU-hour  [S − F_capital − e; rises with S]\n",
        r.inverses.leftover
    ));
    out.push_str(&format!(
        "  R* salvage              {} USD / GPU    [NPV break-even salvage; falls with S]\n\n",
        r.inverses.implied_salvage
    ));
    if let Some(f) = r.forward {
        let f_minus_s = f.amount() - Decimal::from_str(r.s_token).unwrap_or(r.s.amount());
        out.push_str("Forward\n");
        out.push_str(&format!("  F(θ)                    {} USD / GPU-hour\n", f));
        out.push_str(&format!(
            "  F(θ) − S                {} USD / GPU-hour\n\n",
            UsdPerGpuHour::try_from(f_minus_s).unwrap_or(f)
        ));
    }
    out.push_str("Accounting point  T=6y, R=0  (same P, u, e, r)\n");
    out.push_str(&format!(
        "  F_acct                  {} USD / GPU-hour\n",
        r.f_acct
    ));
    let f_acct_minus_s = r.f_acct.amount() - r.s.amount();
    out.push_str(&format!(
        "  F_acct − S              {} USD / GPU-hour\n",
        UsdPerGpuHour::try_from(f_acct_minus_s).unwrap_or(r.f_acct)
    ));
    out.push_str(&format!(
        "  L leftover at T=6       {} USD / GPU-hour\n",
        r.accounting_inverses.leftover
    ));
    out.push_str(&format!(
        "  R* salvage at T=6       {} USD / GPU\n\n",
        r.accounting_inverses.implied_salvage
    ));
    out.push_str("Notes\n");
    out.push_str("  Hourly current was not used.\n");
    out.push_str("  Daily history was not used (that series rounds S to 2.88).\n");
    if r.forward.is_none() {
        out.push_str("  F(θ) omitted: --residual-cents not set; F(R*)=S is a tautology.\n");
    }
    out.push_str("  Negative R* means scrap-zero ownership still beats renting at this S.\n");
    out.push_str("  Half-life was not estimated.\n");
    out
}

struct JsonReport<'a> {
    gpu: &'a str,
    s_token: &'a str,
    valid_on: time::Date,
    fetched_at: &'a str,
    source: &'a Path,
    purchase: Usd,
    life: Years,
    utilization: Utilization,
    energy: UsdPerGpuHour,
    tdp_kw: f64,
    pi: UsdPerKwh,
    discount: DiscountRate,
    f_capital: UsdPerGpuHour,
    inverses: NamedInverses,
    forward: Option<UsdPerGpuHour>,
    f_acct: UsdPerGpuHour,
    accounting_inverses: NamedInverses,
    residual_cents: Option<i64>,
}

fn render_json(r: JsonReport<'_>) -> Result<String> {
    let salvage_usd = r
        .residual_cents
        .map(|cents| Usd::from_cents(cents).to_string());
    let forward = match r.forward {
        Some(f) => {
            let f_minus_s = f.amount() - Decimal::from_str(r.s_token).unwrap_or(f.amount());
            serde_json::json!({
                "f_usd_per_gpu_hour": json_rate(f),
                "f_minus_s": json_rate(UsdPerGpuHour::try_from(f_minus_s)?),
            })
        }
        None => serde_json::Value::Null,
    };
    let value = serde_json::json!({
        "gpu": r.gpu,
        "spot": {
            "series": "ocpi.daily-index",
            "s_usd_per_gpu_hour": r.s_token,
            "valid_on": r.valid_on.to_string(),
            "fetched_at": r.fetched_at,
            "source": r.source.display().to_string(),
        },
        "declared": {
            "purchase_usd": r.purchase.to_string(),
            "life_years": r.life.get(),
            "utilization": r.utilization.amount().to_string(),
            "salvage_usd": salvage_usd,
            "energy_usd_per_gpu_hour": format!("{}", r.energy),
            "energy_factors": {
                "tdp_kw": r.tdp_kw,
                "hours": 1.0,
                "usd_per_kwh": r.pi.amount().to_string(),
                "pue": 1.0,
            },
            "discount_rate": r.discount.amount().to_string(),
            "capital_rent_usd_per_gpu_hour": json_rate(r.f_capital),
        },
        "forward": forward,
        "inverse": {
            "leftover_usd_per_gpu_hour": json_rate(r.inverses.leftover),
            "implied_salvage_usd": json_usd(r.inverses.implied_salvage),
        },
        "accounting": {
            "life_years": 6,
            "salvage_usd": Usd::from_cents(0).to_string(),
            "f_usd_per_gpu_hour": json_rate(r.f_acct),
            "f_minus_s": json_rate(UsdPerGpuHour::try_from(r.f_acct.amount() - Decimal::from_str(r.s_token)?)?),
            "leftover_usd_per_gpu_hour": json_rate(r.accounting_inverses.leftover),
            "implied_salvage_usd": json_usd(r.accounting_inverses.implied_salvage),
        }
    });
    Ok(format!("{value}\n"))
}
