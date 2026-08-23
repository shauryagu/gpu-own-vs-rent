//! Bitemporal wrappers. `AsOf` exists for Gate 6 and is unused here.

use time::{Date, OffsetDateTime};

/// Transaction time: when we learned the observation.
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub struct FetchedAt(OffsetDateTime);

/// Market / settlement date (UTC calendar date of the source timestamp).
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub struct ValidOn(Date);

/// Query time. Unused until Gate 6.
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub struct AsOf(Date);

impl FetchedAt {
    pub fn new(when: OffsetDateTime) -> Self {
        Self(when)
    }

    pub fn get(self) -> OffsetDateTime {
        self.0
    }
}

impl ValidOn {
    pub fn new(day: Date) -> Self {
        Self(day)
    }

    pub fn get(self) -> Date {
        self.0
    }
}

impl AsOf {
    pub fn new(day: Date) -> Self {
        Self(day)
    }

    pub fn get(self) -> Date {
        self.0
    }
}
