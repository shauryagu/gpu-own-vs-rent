//! GPU identifiers and the H100-equivalent hour hole.

use rust_decimal::Decimal;

/// Free OCPI SKUs. A100 SXM4 is named so invert can fail closed later.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum GpuModel {
    H100Sxm,
    H200,
    B200,
    A100Sxm4,
    Rtx5090,
}

/// Quality-adjusted hour. Gate 1 maps only H100 SXM as the identity.
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub struct H100eHour(Decimal);

impl H100eHour {
    pub(crate) fn from_inner(hours: Decimal) -> Self {
        Self(hours)
    }

    pub(crate) fn inner(self) -> Decimal {
        self.0
    }
}
