//! Construction errors for domain quantities.

use rust_decimal::Decimal;
use thiserror::Error;

/// Failed to construct a validated quantity.
#[derive(Debug, Error, Clone, PartialEq)]
pub enum DomainError {
    /// Utilization is outside `(0, 1]`.
    #[error("utilization must be in (0, 1], got {0}")]
    InvalidUtilization(Decimal),
    /// Discount rate is negative.
    #[error("discount rate must be >= 0, got {0}")]
    InvalidDiscountRate(Decimal),
    /// Economic life is not a positive integer.
    #[error("economic life must be a positive integer, got {0}")]
    InvalidYears(u32),
    /// Kilowatt value is NaN or infinite.
    #[error("kilowatt must be a finite f64, got {0}")]
    NonFiniteKilowatt(f64),
    /// Duration is NaN or infinite.
    #[error("hours must be a finite f64, got {0}")]
    NonFiniteHours(f64),
    /// PUE is NaN or infinite.
    #[error("PUE must be a finite f64, got {0}")]
    NonFinitePue(f64),
}
