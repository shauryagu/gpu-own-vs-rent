//! Declared parameter vectors. `Theta` is the forward path; salvage is required.

use crate::money::{Usd, UsdPerGpuHour};
use crate::qty::{DiscountRate, GpuHour, Utilization, Years};
use rust_decimal::Decimal;

/// Civil hours per year. \( H = 8760 \).
pub const HOURS_PER_YEAR: GpuHour = GpuHour::new(Decimal::from_parts(8760, 0, 0, false, 0));

/// Fully declared \(\theta = (P, T, u, R, e, r)\). Forward path for \(F(\theta)\).
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Theta {
    pub purchase: Usd,
    pub life: Years,
    pub utilization: Utilization,
    pub salvage: Usd,
    pub energy: UsdPerGpuHour,
    pub discount: DiscountRate,
}

/// \(\theta\) without salvage. Inverse path for leftover and implied salvage.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct ThetaExResidual {
    pub purchase: Usd,
    pub life: Years,
    pub utilization: Utilization,
    pub energy: UsdPerGpuHour,
    pub discount: DiscountRate,
}
