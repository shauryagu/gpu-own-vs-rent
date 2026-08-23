//! Observed rental. Invert \(S\) is daily-index only.

use crate::gpu::GpuModel;
use crate::money::UsdPerGpuHour;
use crate::time::ValidOn;

/// Which OCPI series an [`ObservedSpot`] came from.
///
/// `OcpiCurrent` and `OcpiDailyHistory` are absent on purpose: mixing hourly
/// current or rounded history into invert is unrepresentable.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum SpotSeries {
    /// Authoritative invert \(S\).
    OcpiDailyIndex,
}

/// One daily-index settle. `FetchedAt` lives on the ingest wrapper, not here.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct ObservedSpot {
    pub gpu: GpuModel,
    pub series: SpotSeries,
    pub valid_on: ValidOn,
    pub price: UsdPerGpuHour,
}
