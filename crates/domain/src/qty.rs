//! Physical and calendar quantities. Not money.

use crate::error::DomainError;
use rust_decimal::Decimal;
use std::ops::Mul;

/// Utilized or civil GPU-hours. Not a wall-clock [`Hours`] duration.
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub struct GpuHour(Decimal);

/// Positive integer economic life in years. Zero is rejected at construction.
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub struct Years(u32);

/// Fraction of civil hours the GPU is on and earning. Open at 0, closed at 1.
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub struct Utilization(Decimal);

/// Annual discount rate. Zero is allowed (annuity factor becomes `T`).
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub struct DiscountRate(Decimal);

/// Thermal design power. Physics; `f64` is allowed here.
#[derive(Clone, Copy, Debug, PartialEq, PartialOrd)]
pub struct Kilowatt(f64);

/// Wall-clock duration in hours. Not [`GpuHour`].
#[derive(Clone, Copy, Debug, PartialEq, PartialOrd)]
pub struct Hours(f64);

/// Power usage effectiveness. Physics; `f64` is allowed here.
#[derive(Clone, Copy, Debug, PartialEq, PartialOrd)]
pub struct Pue(f64);

impl GpuHour {
    pub fn new(hours: Decimal) -> Self {
        Self(hours)
    }

    pub fn amount(self) -> Decimal {
        self.0
    }
}

impl From<i64> for GpuHour {
    fn from(hours: i64) -> Self {
        Self(Decimal::from(hours))
    }
}

impl Years {
    pub fn try_new(years: u32) -> Result<Self, DomainError> {
        if years == 0 {
            return Err(DomainError::InvalidYears(years));
        }
        Ok(Self(years))
    }

    pub fn get(self) -> u32 {
        self.0
    }
}

impl Utilization {
    pub fn try_new(value: Decimal) -> Result<Self, DomainError> {
        if value <= Decimal::ZERO || value > Decimal::ONE {
            return Err(DomainError::InvalidUtilization(value));
        }
        Ok(Self(value))
    }

    pub fn amount(self) -> Decimal {
        self.0
    }
}

impl DiscountRate {
    pub fn try_new(rate: Decimal) -> Result<Self, DomainError> {
        if rate < Decimal::ZERO {
            return Err(DomainError::InvalidDiscountRate(rate));
        }
        Ok(Self(rate))
    }

    pub fn amount(self) -> Decimal {
        self.0
    }
}

impl Kilowatt {
    pub fn try_new(kw: f64) -> Result<Self, DomainError> {
        if !kw.is_finite() {
            return Err(DomainError::NonFiniteKilowatt(kw));
        }
        Ok(Self(kw))
    }

    pub fn get(self) -> f64 {
        self.0
    }
}

impl Hours {
    pub fn try_new(hours: f64) -> Result<Self, DomainError> {
        if !hours.is_finite() {
            return Err(DomainError::NonFiniteHours(hours));
        }
        Ok(Self(hours))
    }

    pub fn get(self) -> f64 {
        self.0
    }
}

impl Pue {
    pub fn try_new(pue: f64) -> Result<Self, DomainError> {
        if !pue.is_finite() {
            return Err(DomainError::NonFinitePue(pue));
        }
        Ok(Self(pue))
    }

    pub fn get(self) -> f64 {
        self.0
    }
}

impl Mul<GpuHour> for Utilization {
    type Output = GpuHour;

    fn mul(self, rhs: GpuHour) -> Self::Output {
        GpuHour(self.0 * rhs.0)
    }
}

impl Mul<Hours> for Kilowatt {
    type Output = f64;

    fn mul(self, rhs: Hours) -> Self::Output {
        self.0 * rhs.0
    }
}
