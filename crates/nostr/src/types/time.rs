// Copyright (c) 2022-2023 Yuki Kishimoto
// Distributed under the MIT software license

//! Time

use core::ops::{Add, Sub};
use core::str::FromStr;
use core::time::Duration;

#[cfg(feature = "std")]
use std::{
    fmt, num,
    time::{SystemTime, UNIX_EPOCH},
};

#[cfg(all(feature = "alloc", not(feature = "std")))]
use alloc::fmt;
#[cfg(all(feature = "alloc", not(feature = "std")))]
use core::num;

#[cfg(target_arch = "wasm32")]
use instant::SystemTime;

#[cfg(target_arch = "wasm32")]
const UNIX_EPOCH: SystemTime = SystemTime::UNIX_EPOCH;

/// Helper trait for acquiring time in `no_std` environments.
pub trait TimeSupplier {
    /// The current time from the specified `TimeSupplier`
    type Now: Clone;
    /// The starting point for the specified `TimeSupplier`
    type StartingPoint: Clone;

    /// Get the current time as the associated `Now` type
    fn instant_now(&self) -> Self::Now;
    /// Get the current time as the associated `StartingPoint` type
    fn now(&self) -> Self::StartingPoint;
    /// Get a duration since the StartingPoint.
    fn duration_since_starting_point(&self, now: Self::StartingPoint) -> Duration;
    /// Get the starting point from the specified `TimeSupplier`
    fn starting_point(&self) -> Self::StartingPoint;
    /// Get the elapsed time as `Duration` starting from `since` to `now`
    fn elapsed_instant_since(&self, now: Self::Now, since: Self::Now) -> Duration;
    /// Get the elapsed time as `Duration` starting from `since` to `now`
    fn elapsed_since(&self, now: Self::StartingPoint, since: Self::StartingPoint) -> Duration;

    //  /// Get the elapsed time as `Duration` starting from `since` to `now`
    //  /// This is the specialised case for handling the `StartingPoint` in case its type is different
    //  /// than the `Now` type.
    //  fn elapsed_duration(&self, now: Self::Now, since: Self::StartingPoint) -> Duration;

    /// Convert the specified `Duration` to `i64`
    fn as_i64(&self, duration: Duration) -> i64;
    /// Convert the specified `Duration` to `Timestamp`
    fn to_timestamp(&self, duration: Duration) -> Timestamp;
}

#[cfg(target_arch = "wasm32")]
use instant::Instant as InstantWasm32;
#[cfg(target_arch = "wasm32")]
impl TimeSupplier for InstantWasm32 {
    type Now = InstantWasm32;
    type StartingPoint = std::time::SystemTime;

    fn instant_now(&self) -> Self::Now {
        InstantWasm32::now()
    }

    fn starting_point(&self) -> Self::Now {
        std::time::UNIX_EPOCH
    }

    fn elapsed_since(&self, now: Self::Now, since: Self::Now) -> Duration {
        now - since
    }

    fn as_i64(&self, duration: Duration) -> i64 {
        duration.as_millis() as i64
    }

    fn to_timestamp(&self, duration: Duration) -> Timestamp {
        Timestamp(duration.as_millis() as i64)
    }
}

#[cfg(all(not(target_arch = "wasm32"), feature = "std"))]
use std::time::Instant;
#[cfg(all(not(target_arch = "wasm32"), feature = "std"))]
impl TimeSupplier for Instant {
    type Now = Instant;
    type StartingPoint = std::time::SystemTime;

    fn now(&self) -> Self::StartingPoint {
        SystemTime::now()
    }

    fn instant_now(&self) -> Self::Now {
        Instant::now()
    }

    fn duration_since_starting_point(&self, now: Self::StartingPoint) -> Duration {
        now.duration_since(self.starting_point()).expect("duration_since panicked")
    }

    fn starting_point(&self) -> Self::StartingPoint {
        std::time::UNIX_EPOCH
    }

    fn elapsed_instant_since(&self, now: Self::Now, since: Self::Now) -> Duration {
        now - since
    }

    fn elapsed_since(&self, now: Self::StartingPoint, since: Self::StartingPoint) -> Duration {
        now.duration_since(since).expect("duration_since panicked")
    }

//     fn elapsed_duration(&self, now: Self::Now, since: Self::StartingPoint) -> Duration {
//         let dur = since.duration_since(self.starting_point).expect("Clock may have gone backwards");
//         now - since
//     }
// 
    fn as_i64(&self, duration: Duration) -> i64 {
        duration.as_millis() as i64
    }

    fn to_timestamp(&self, duration: Duration) -> Timestamp {
        Timestamp(self.as_i64(duration))
    }
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

    /// Get UNIX timestamp from the specified `TimeSupplier`
    #[cfg(not(feature = "std"))]
    pub fn now_nostd<T>(time_supplier: &T) -> Self
    where
        T: TimeSupplier,
    {
        let now = time_supplier.now();
        let starting_point = time_supplier.starting_point();
        let duration = time_supplier.elapsed_duration(now, starting_point);

        time_supplier.to_timestamp(duration)
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
