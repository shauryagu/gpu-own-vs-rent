//! H100 → H100e identity. FLOPS maps for other SKUs are Gate 5.

use crate::gpu::{GpuModel, H100eHour};
use crate::qty::GpuHour;
use thiserror::Error;

/// Conversion that is not implemented in this gate.
#[derive(Debug, Error, Clone, PartialEq, Eq)]
pub enum ConvertError {
    /// Only [`GpuModel::H100Sxm`] is the identity in Gate 1.
    #[error("GPU model {0:?} is not in the Gate 1 H100e identity")]
    NotInMvp(GpuModel),
}

/// Map GPU-hours into H100-equivalent hours.
///
/// H100 SXM is the identity. Every other SKU is [`ConvertError::NotInMvp`].
pub fn to_h100e(hours: GpuHour, gpu: GpuModel) -> Result<H100eHour, ConvertError> {
    match gpu {
        GpuModel::H100Sxm => Ok(H100eHour::from_inner(hours.amount())),
        other => Err(ConvertError::NotInMvp(other)),
    }
}

/// Treat an H100e hour as an H100 SXM GPU-hour.
pub fn from_h100e_as_h100(h: H100eHour) -> GpuHour {
    GpuHour::new(h.inner())
}
