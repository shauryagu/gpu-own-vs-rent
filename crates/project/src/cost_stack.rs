//! Cost-stack projection: \( S = F_{\mathrm{capital}} + e + L \).

use domain::{
    capital_rent, energy_per_gpu_hour, leftover, IdentityError, Kilowatt, ObservedSpot, Pue,
    ThetaExResidual, UsdPerGpuHour, UsdPerKwh,
};
use rust_decimal::Decimal;

/// Named electricity price. Amount is USD/kWh (already converted from LMP).
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct NamedEnergy {
    pub usd_per_kwh: UsdPerKwh,
    pub lmp_usd_per_mwh: Option<String>,
    pub source: String,
}

/// One PUE's split of observed rent. Rates only (`UsdPerGpuHour`).
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct CostStackRow {
    pub pue: Pue,
    pub energy: UsdPerGpuHour,
    pub capital: UsdPerGpuHour,
    pub leftover: UsdPerGpuHour,
    /// \( e / S \). Dimensionless; “small or not next to OCPI.”
    pub energy_share: Decimal,
}

/// Panel of leftover identity at a named \(\pi\) and one or more PUEs.
#[derive(Clone, Debug, PartialEq)]
pub struct CostStack {
    pub spot: UsdPerGpuHour,
    pub named_energy: NamedEnergy,
    pub rows: Vec<CostStackRow>,
}

impl CostStack {
    /// `capital_ex.energy` is ignored; each row sets \( e \) from TDP · π · PUE.
    pub fn compute(
        obs: ObservedSpot,
        capital_ex: ThetaExResidual,
        tdp: Kilowatt,
        named_energy: NamedEnergy,
        pues: &[Pue],
    ) -> Result<Self, IdentityError> {
        let mut rows = Vec::with_capacity(pues.len());
        for &pue in pues {
            let energy = energy_per_gpu_hour(tdp, named_energy.usd_per_kwh, pue)?;
            let mut ex = capital_ex;
            ex.energy = energy;
            let capital = capital_rent(&ex)?;
            let leftover_rate = leftover(obs, &ex)?;
            let energy_share = if obs.price.amount().is_zero() {
                Decimal::ZERO
            } else {
                energy.amount() / obs.price.amount()
            };
            rows.push(CostStackRow {
                pue,
                energy,
                capital,
                leftover: leftover_rate,
                energy_share,
            });
        }
        Ok(Self {
            spot: obs.price,
            named_energy,
            rows,
        })
    }
}
