// Copyright (c) 2022-2023 Yuki Kishimoto
// Distributed under the MIT software license

//! Entity

#[cfg(all(not(feature = "std"), feature = "sgx"))]
use crate::sgx_reexport_prelude::*;

/// Nostr [`Entity`]
#[derive(Debug, Clone, Eq, PartialEq, Ord, PartialOrd, Serialize, Deserialize)]
#[serde(crate = "self::serde")]
pub enum Entity {
    /// Account
    Account,
    /// Channel
    Channel,
    /// Unknown
    Unknown,
}
