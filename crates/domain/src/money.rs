//! USD money newtypes. JSON is a decimal string; a JSON number is a schema error.
//!
//! There is no public `f64 → Usd` constructor. Arithmetic uses `rust_decimal`.
//!
//! Illegal unit joins do not type-check:
//!
//! ```compile_fail
//! use domain::{Usd, UsdPerGpuHour};
//! let _ = Usd::from_cents(1) + UsdPerGpuHour::from_cents(1);
//! ```
//!
//! ```compile_fail
//! use domain::{GpuHour, Usd};
//! let _ = Usd::from_cents(1) + GpuHour::from(1_i64);
//! ```
//!
//! ```compile_fail
//! use domain::{GpuHour, UsdPerGpuHour};
//! let _ = UsdPerGpuHour::from_cents(1) + GpuHour::from(1_i64);
//! ```
//!
//! ```compile_fail
//! use domain::{Kilowatt, UsdPerKwh};
//! let _ = Kilowatt::try_new(1.0).unwrap() + UsdPerKwh::from_cents(1);
//! ```
//!
//! ```compile_fail
//! use domain::{to_h100e, GpuHour, GpuModel};
//! let hours = GpuHour::from(1_i64);
//! let h100e = to_h100e(hours, GpuModel::H100Sxm).unwrap();
//! let _ = hours + h100e;
//! ```

use crate::error::DomainError;
use crate::qty::GpuHour;
use rust_decimal::Decimal;
use serde::de::{self, Visitor};
use serde::{Deserialize, Deserializer, Serialize, Serializer};
use std::fmt;
use std::ops::{Add, Div, Mul};

/// USD total. Display and serde use scale 2.
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub struct Usd(Decimal);

/// USD per GPU-hour. Display and serde use scale 4.
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub struct UsdPerGpuHour(Decimal);

/// USD per kilowatt-hour.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct UsdPerKwh(Decimal);

impl Usd {
    /// Preferred constructor for a declared purchase (integer cents).
    pub fn from_cents(cents: i64) -> Self {
        Self(Decimal::new(cents, 2))
    }

    pub fn amount(self) -> Decimal {
        self.0
    }

    pub(crate) fn from_decimal(value: Decimal) -> Self {
        Self(value)
    }
}

impl TryFrom<Decimal> for Usd {
    type Error = DomainError;

    fn try_from(value: Decimal) -> Result<Self, Self::Error> {
        Ok(Self(value))
    }
}

impl UsdPerGpuHour {
    pub fn from_cents(cents: i64) -> Self {
        Self(Decimal::new(cents, 2))
    }

    pub fn amount(self) -> Decimal {
        self.0
    }

    pub(crate) fn from_decimal(value: Decimal) -> Self {
        Self(value)
    }
}

impl TryFrom<Decimal> for UsdPerGpuHour {
    type Error = DomainError;

    fn try_from(value: Decimal) -> Result<Self, Self::Error> {
        Ok(Self(value))
    }
}

impl UsdPerKwh {
    pub fn from_cents(cents: i64) -> Self {
        Self(Decimal::new(cents, 2))
    }

    pub fn amount(self) -> Decimal {
        self.0
    }
}

impl TryFrom<Decimal> for UsdPerKwh {
    type Error = DomainError;

    fn try_from(value: Decimal) -> Result<Self, Self::Error> {
        Ok(Self(value))
    }
}

impl Add for Usd {
    type Output = Self;

    fn add(self, rhs: Self) -> Self::Output {
        Self(self.0 + rhs.0)
    }
}

impl Mul<GpuHour> for UsdPerGpuHour {
    type Output = Usd;

    fn mul(self, rhs: GpuHour) -> Self::Output {
        Usd(self.0 * rhs.amount())
    }
}

impl Div<GpuHour> for Usd {
    type Output = UsdPerGpuHour;

    fn div(self, rhs: GpuHour) -> Self::Output {
        UsdPerGpuHour(self.0 / rhs.amount())
    }
}

impl fmt::Display for Usd {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(&format_decimal(self.0, 2))
    }
}

impl fmt::Display for UsdPerGpuHour {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(&format_decimal(self.0, 4))
    }
}

impl fmt::Display for UsdPerKwh {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(&self.0.to_string())
    }
}

/// Display scale, not a lossless OCPI token. Ingest must not serialize money this way.
impl Serialize for Usd {
    fn serialize<S: Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        serializer.serialize_str(&format_decimal(self.0, 2))
    }
}

impl Serialize for UsdPerGpuHour {
    fn serialize<S: Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        serializer.serialize_str(&format_decimal(self.0, 4))
    }
}

impl Serialize for UsdPerKwh {
    fn serialize<S: Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        serializer.serialize_str(&self.0.to_string())
    }
}

impl<'de> Deserialize<'de> for Usd {
    fn deserialize<D: Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error> {
        Ok(Self(deserialize_decimal_string(deserializer)?))
    }
}

impl<'de> Deserialize<'de> for UsdPerGpuHour {
    fn deserialize<D: Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error> {
        Ok(Self(deserialize_decimal_string(deserializer)?))
    }
}

impl<'de> Deserialize<'de> for UsdPerKwh {
    fn deserialize<D: Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error> {
        Ok(Self(deserialize_decimal_string(deserializer)?))
    }
}

fn format_decimal(value: Decimal, scale: u32) -> String {
    value.round_dp(scale).to_string()
}

fn deserialize_decimal_string<'de, D: Deserializer<'de>>(
    deserializer: D,
) -> Result<Decimal, D::Error> {
    struct DecimalStringVisitor;

    impl Visitor<'_> for DecimalStringVisitor {
        type Value = Decimal;

        fn expecting(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
            f.write_str("a decimal string")
        }

        fn visit_str<E: de::Error>(self, v: &str) -> Result<Self::Value, E> {
            Decimal::from_str_exact(v).map_err(E::custom)
        }
    }

    deserializer.deserialize_str(DecimalStringVisitor)
}
