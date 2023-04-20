// Copyright (c) 2022-2023 Yuki Kishimoto
// Distributed under the MIT software license

//! Prelude

// External crates
#[cfg(feature = "nip19")]
pub use bech32;
#[cfg(feature = "nip06")]
pub use bip39::*;
pub use secp256k1;
pub use url::*;

// Internal modules
pub use crate::Result;

#[cfg(feature = "std")]
pub use crate::SECP256K1;

// NIPs
#[cfg(feature = "nip04")]
pub use crate::nips::nip04::*;
#[cfg(feature = "nip05")]
pub use crate::nips::nip05::*;
#[cfg(feature = "nip06")]
pub use crate::nips::nip06::*;
#[cfg(feature = "nip11")]
pub use crate::nips::nip11::*;
pub use crate::nips::nip13::*;
#[cfg(feature = "nip19")]
pub use crate::nips::nip19;
pub use crate::nips::nip26::*;
#[cfg(feature = "nip46")]
pub use crate::nips::nip46::*;
pub use crate::nips::nip65::*;
