// Copyright (c) 2022-2023 Yuki Kishimoto
// Distributed under the MIT software license

#![warn(missing_docs)]
#![warn(rustdoc::bare_urls)]

//! Rust implementation of the Nostr protocol.

#![cfg_attr(
    feature = "default",
    doc = include_str!("../README.md")
)]


#![cfg_attr(not(feature = "std"), no_std)]

#[cfg(all(feature = "std", feature = "sgx"))]
compile_error!("feature \"std\" and feature \"sgx\" cannot be enabled at the same time");

#[cfg(all(not(feature = "std"), feature = "sgx"))]
extern crate sgx_tstd as std;

// re-export module to properly feature gate sgx and regular std environment
#[cfg(all(not(feature = "std"), feature = "sgx"))]
pub mod sgx_reexport_prelude {
	pub use thiserror_sgx as thiserror;
}

use std::boxed::Box;


#[macro_use]
pub extern crate serde;

#[cfg(feature = "nip19")]
pub use bech32;
#[cfg(feature = "nip06")]
pub use bip39;
#[cfg(feature = "nip06")]
pub use bitcoin;
pub use bitcoin_hashes as hashes;
pub use secp256k1::{self, SECP256K1};
pub use serde_json;
pub use url::{self, Url};

pub mod event;
pub mod key;
pub mod message;
pub mod nips;
pub mod prelude;
pub mod types;

pub use self::event::{Event, EventBuilder, EventId, Kind, Tag, UnsignedEvent};
pub use self::key::Keys;
pub use self::message::{ClientMessage, Filter, RelayMessage, SubscriptionId};
pub use self::types::{ChannelId, Contact, Entity, Metadata, Profile, Timestamp};

/// Result
pub type Result<T, E = Box<dyn std::error::Error>> = std::result::Result<T, E>;
