// Copyright (c) 2022-2023 Yuki Kishimoto
// Distributed under the MIT software license

//! Contact

use secp256k1::XOnlyPublicKey;
use serde::{Deserialize, Serialize};

use crate::event::tag::UncheckedUrl;

/// Contact
#[derive(Debug, Clone, Eq, PartialEq, Ord, PartialOrd, Deserialize, Serialize)]
pub struct Contact {
    /// Public key
    pub pk: XOnlyPublicKey,
    /// Relay url
    pub relay_url: Option<UncheckedUrl>,
    /// Alias
    pub alias: Option<String>,
}

impl Contact {
    /// Create new [`Contact`]
    pub fn new<S>(pk: XOnlyPublicKey, relay_url: Option<UncheckedUrl>, alias: Option<S>) -> Self
    where
        S: Into<String>,
    {
        Self {
            pk,
            relay_url,
            alias: alias.map(|a| a.into()),
        }
    }
}
