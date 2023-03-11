// Copyright (c) 2022-2023 Yuki Kishimoto
// Distributed under the MIT software license

#![warn(missing_docs)]
#![warn(rustdoc::bare_urls)]

//! High level Nostr client library.

#![cfg_attr(
    feature = "all-nips",
    doc = include_str!("../README.md")
)]

#[cfg(feature = "blocking")]
use once_cell::sync::Lazy;
#[cfg(feature = "blocking")]
use tokio::runtime::Runtime;

pub use nostr::{self, *};

pub mod client;
pub mod prelude;
pub mod relay;
pub mod subscription;

/* #[cfg(feature = "blocking")]
pub use self::client::blocking;
pub use self::client::{Client, Options}; */
pub use self::client::Client;
pub use self::relay::pool::RelayPoolNotification;
pub use self::relay::{Relay, RelayStatus};

#[cfg(feature = "blocking")]
static RUNTIME: Lazy<Runtime> = Lazy::new(|| Runtime::new().expect("Can't start Tokio runtime"));
