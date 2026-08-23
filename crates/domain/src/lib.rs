//! Quantities and identity. No I/O.

mod convert;
mod error;
mod gpu;
mod money;
mod qty;
mod spot;
mod time;

pub use convert::{from_h100e_as_h100, to_h100e, ConvertError};
pub use error::DomainError;
pub use gpu::{GpuModel, H100eHour};
pub use money::{Usd, UsdPerGpuHour, UsdPerKwh};
pub use qty::{DiscountRate, GpuHour, Hours, Kilowatt, Pue, Utilization, Years};
pub use spot::{ObservedSpot, SpotSeries};
pub use time::{AsOf, FetchedAt, ValidOn};
