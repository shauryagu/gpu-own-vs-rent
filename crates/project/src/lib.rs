//! Invert report assembly. Gate 4 owns Θ surfaces.

use domain::{
    implied_salvage, leftover, IdentityError, ObservedSpot, ThetaExResidual, Usd, UsdPerGpuHour,
};

/// Leftover \(L\) and salvage \(R^{\star}\). Never one “implied residual.”
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct NamedInverses {
    pub leftover: UsdPerGpuHour,
    pub implied_salvage: Usd,
}

impl NamedInverses {
    pub fn compute(obs: ObservedSpot, ex: &ThetaExResidual) -> Result<Self, IdentityError> {
        Ok(Self {
            leftover: leftover(obs, ex)?,
            implied_salvage: implied_salvage(obs, ex)?,
        })
    }
}
