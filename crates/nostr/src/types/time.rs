// Copyright (c) 2022-2023 Yuki Kishimoto
// Distributed under the MIT software license

//! Time

use core::time::Duration;
use core::ops::{Add, Sub};
use core::str::FromStr;

#[cfg(feature = "std")]
use std::{fmt, num, time::{SystemTime, UNIX_EPOCH}};

#[cfg(feature = "alloc")]
use alloc::fmt;
#[cfg(feature = "alloc")]
use core::num;

#[cfg(target_arch = "wasm32")]
use instant::SystemTime;

#[cfg(target_arch = "wasm32")]
const UNIX_EPOCH: SystemTime = SystemTime::UNIX_EPOCH;


/// Helper trait for acquiring time in `no_std` environments.
#[cfg(not(feature = "std"))]
pub trait TimeSupplier {
    type Now;

    fn now(&self) -> Self::Now;
    fn elapsed_since(&self, since: Self::Now) -> Duration;

    fn as_i64(&self) -> i64;
}

/// Unix timestamp in seconds
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
pub struct Timestamp(i64);

impl Timestamp {
    /// Get UNIX timestamp
    #[cfg(feature = "std")]
    pub fn now() -> Self {
        let ts: u64 = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap_or_default()
            .as_secs();
        Self(ts as i64)
    }

    #[cfg(not(feature = "std"))]
    pub fn now_no_std(time_supplier: &impl TimeSupplier) -> Self {
        Self(time_supplier.as_i64())
    }

    /// Get timestamp as [`u64`]
    pub fn as_u64(&self) -> u64 {
        if self.0 >= 0 {
            self.0 as u64
        } else {
            0
        }
    }

    /// Get timestamp as [`i64`]
    pub fn as_i64(&self) -> i64 {
        self.0
    }
}

impl From<u64> for Timestamp {
    fn from(timestamp: u64) -> Self {
        Self(timestamp as i64)
    }
}

impl FromStr for Timestamp {
    type Err = num::ParseIntError;
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        Ok(Self(s.parse::<i64>()?))
    }
}

impl fmt::Display for Timestamp {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.0)
    }
}

impl Add<Duration> for Timestamp {
    type Output = Self;
    fn add(self, rhs: Duration) -> Self::Output {
        Self(self.0.saturating_add(rhs.as_secs() as i64))
    }
}

impl Sub<Duration> for Timestamp {
    type Output = Self;
    fn sub(self, rhs: Duration) -> Self::Output {
        Self(self.0.saturating_sub(rhs.as_secs() as i64))
    }
}

impl Add<u64> for Timestamp {
    type Output = Self;
    fn add(self, rhs: u64) -> Self::Output {
        self.add(rhs as i64)
    }
}

impl Sub<u64> for Timestamp {
    type Output = Self;
    fn sub(self, rhs: u64) -> Self::Output {
        self.sub(rhs as i64)
    }
}

impl Add<i64> for Timestamp {
    type Output = Self;
    fn add(self, rhs: i64) -> Self::Output {
        Self(self.0.saturating_add(rhs))
    }
}

impl Sub<i64> for Timestamp {
    type Output = Self;
    fn sub(self, rhs: i64) -> Self::Output {
        Self(self.0.saturating_sub(rhs))
    }
}
