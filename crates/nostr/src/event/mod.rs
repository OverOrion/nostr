// Copyright (c) 2021 Paul Miller
// Copyright (c) 2022-2023 Yuki Kishimoto
// Distributed under the MIT software license

//! Event

#[cfg(feature = "std")]
use std::str::FromStr;

#[cfg(all(feature = "alloc", not(feature = "std")))]
use alloc::{
    str::FromStr,
    string::{String, ToString},
    vec::Vec,
};

use secp256k1::schnorr::Signature;
use secp256k1::Message;

use secp256k1::XOnlyPublicKey;
use secp256k1::{Secp256k1, Verification};
use serde::{Deserialize, Serialize};
use serde_json::Value;

pub mod builder;
pub mod id;
pub mod kind;
pub mod tag;
pub mod unsigned;

pub use self::builder::EventBuilder;
pub use self::id::EventId;
pub use self::kind::Kind;
pub use self::tag::{Marker, Tag, TagKind};
pub use self::unsigned::UnsignedEvent;
use crate::Timestamp;
#[cfg(feature = "std")]
use crate::SECP256K1;

/// [`Event`] error
#[derive(Debug, thiserror::Error)]
pub enum Error {
    /// Invalid signature
    #[error("invalid signature")]
    InvalidSignature,
    /// Error serializing or deserializing JSON data
    #[error("Serde json Error: {0}")]
    Json(serde_json::Error),
    /// Secp256k1 error
    #[error("Secp256k1 Error: {0}")]
    Secp256k1(secp256k1::Error),
    /// Hex decoding error
    #[error("Hex Error: {0}")]
    Hex(bitcoin_hashes::hex::Error),
    /// OpenTimestamps error
    #[cfg(feature = "nip03")]
    #[error(transparent)]
    OpenTimestamps(#[from] nostr_ots::Error),
}

impl From<serde_json::Error> for Error {
    fn from(error: serde_json::Error) -> Self {
        Self::Json(error)
    }
}

impl From<secp256k1::Error> for Error {
    fn from(error: secp256k1::Error) -> Self {
        Self::Secp256k1(error)
    }
}

impl From<bitcoin_hashes::hex::Error> for Error {
    fn from(error: bitcoin_hashes::hex::Error) -> Self {
        Self::Hex(error)
    }
}

/// [`Event`] struct
#[derive(Debug, Clone, Eq, PartialEq, Ord, PartialOrd, Serialize, Deserialize)]
pub struct Event {
    /// Id
    pub id: EventId,
    /// Author
    pub pubkey: XOnlyPublicKey,
    /// Timestamp (seconds)
    pub created_at: Timestamp,
    /// Kind
    pub kind: Kind,
    /// Vector of [`Tag`]
    pub tags: Vec<Tag>,
    /// Content
    pub content: String,
    /// Signature
    pub sig: Signature,
    /// OpenTimestamps Attestations
    #[cfg(feature = "nip03")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub ots: Option<String>,
}

impl Event {
    /// Verify Event
    #[cfg(feature = "std")]
    pub fn verify(&self) -> Result<(), Error> {
        self.verify_with_context(SECP256K1)
    }

    /// Verify Event
    pub fn verify_with_context<C: Verification>(&self, secp: &Secp256k1<C>) -> Result<(), Error> {
        let id = EventId::new(
            &self.pubkey,
            self.created_at,
            &self.kind,
            &self.tags,
            &self.content,
        );
        let message = Message::from_slice(id.as_bytes())?;
        secp.verify_schnorr(&self.sig, &message, &self.pubkey)
            .map_err(|_| Error::InvalidSignature)
    }

    /// New event from [`Value`]
    pub fn from_value(value: Value) -> Result<Self, Error> {
        let event: Self = serde_json::from_value(value)?;
        Ok(event)
    }

    /// New event from json string
    pub fn from_json<S>(json: S) -> Result<Self, Error>
    where
        S: Into<String>,
    {
        let event: Self = serde_json::from_str(&json.into())?;
        Ok(event)
    }

    /// Get event as json string
    pub fn as_json(&self) -> String {
        serde_json::json!(self).to_string()
    }

    /// Timestamp this event with OpenTimestamps, according to NIP-03
    #[cfg(feature = "nip03")]
    pub fn timestamp(&mut self) -> Result<(), Error> {
        let ots = nostr_ots::timestamp_event(&self.id.to_hex())?;
        self.ots = Some(ots);
        Ok(())
    }
}

impl Event {
    /// This is just for serde sanity checking
    #[allow(dead_code)]
    pub(crate) fn new_dummy(
        id: &str,
        pubkey: &str,
        created_at: Timestamp,
        kind: u8,
        tags: Vec<Tag>,
        content: &str,
        sig: &str,
    ) -> Result<Self, Error> {
        let id = EventId::from_hex(id).unwrap();
        let pubkey = XOnlyPublicKey::from_str(pubkey)?;
        let kind = serde_json::from_str(&kind.to_string())?;
        let sig = Signature::from_str(sig)?;

        let event = Event {
            id,
            pubkey,
            created_at,
            kind,
            tags,
            content: content.to_string(),
            sig,
            #[cfg(feature = "nip03")]
            ots: None,
        };

        Ok(event)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    use crate::Keys;

    #[test]
    fn test_tags_deser_without_recommended_relay() {
        //The TAG array has dynamic length because the third element(Recommended relay url) is optional
        let sample_event = r#"{"content":"uRuvYr585B80L6rSJiHocw==?iv=oh6LVqdsYYol3JfFnXTbPA==","created_at":1640839235,"id":"2be17aa3031bdcb006f0fce80c146dea9c1c0268b0af2398bb673365c6444d45","kind":4,"pubkey":"f86c44a2de95d9149b51c6a29afeabba264c18e2fa7c49de93424a0c56947785","sig":"a5d9290ef9659083c490b303eb7ee41356d8778ff19f2f91776c8dc4443388a64ffcf336e61af4c25c05ac3ae952d1ced889ed655b67790891222aaa15b99fdd","tags":[["p","13adc511de7e1cfcf1c6b7f6365fb5a03442d7bcacf565ea57fa7770912c023d"]]}"#;
        let ev_ser = Event::from_json(sample_event).unwrap();
        assert_eq!(ev_ser.as_json(), sample_event);
    }

    #[test]
    fn test_custom_kind() {
        let keys = Keys::generate();
        let e: Event = EventBuilder::new(Kind::Custom(123), "my content", &[])
            .to_event(&keys)
            .unwrap();

        let serialized = e.as_json();
        let deserialized = Event::from_json(serialized).unwrap();

        assert_eq!(e, deserialized);
        assert_eq!(Kind::Custom(123), e.kind);
        assert_eq!(Kind::Custom(123), deserialized.kind);
    }
}
