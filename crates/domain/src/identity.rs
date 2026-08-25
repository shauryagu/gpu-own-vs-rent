//! Discrete NPV identity. No I/O.

use thiserror::Error;

/// Failed to evaluate the identity or the energy product.
#[derive(Debug, Error, Clone, PartialEq)]
pub enum IdentityError {
    /// A physics product was NaN or infinite at the `f64` → `Decimal` step.
    #[error("physics quantity is not finite")]
    NonFinitePhysics,
}
