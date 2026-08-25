//! Quantities and identity. No I/O.

mod convert;
mod energy;
mod error;
mod gpu;
mod identity;
mod money;
mod qty;
mod spot;
mod theta;
mod time;

pub use convert::{from_h100e_as_h100, to_h100e, ConvertError};
pub use energy::energy_per_gpu_hour;
pub use error::DomainError;
pub use gpu::{GpuModel, H100eHour};
pub use identity::{capital_rent, fair_rent, implied_salvage, leftover, IdentityError};
pub use money::{Usd, UsdPerGpuHour, UsdPerKwh};
pub use qty::{DiscountRate, GpuHour, Hours, Kilowatt, Pue, Utilization, Years};
pub use spot::{ObservedSpot, SpotSeries};
pub use theta::{Theta, ThetaExResidual, HOURS_PER_YEAR};
pub use time::{AsOf, FetchedAt, ValidOn};
