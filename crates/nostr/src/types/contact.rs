// Copyright (c) 2022-2023 Yuki Kishimoto
// Distributed under the MIT software license

//! Contact

#[cfg(all(not(feature = "std"), feature = "sgx"))]
use crate::sgx_reexport_prelude::*;

use std::string::String;

use secp256k1::XOnlyPublicKey;

/// Contact
#[derive(Debug, Clone, Eq, PartialEq, Ord, PartialOrd, Deserialize, Serialize)]
#[serde(crate = "self::serde")]
pub struct Contact {
    /// Public key
    pub pk: XOnlyPublicKey,
    /// Relay url
    pub relay_url: Option<String>,
    /// Alias
    pub alias: Option<String>,
}

impl Contact {
    /// Create new [`Contact`]
    pub fn new<S>(pk: XOnlyPublicKey, relay_url: Option<S>, alias: Option<S>) -> Self
    where
        S: Into<String>,
    {
        Self {
            pk,
            relay_url: relay_url.map(|a| a.into()),
            alias: alias.map(|a| a.into()),
        }
    }
}
