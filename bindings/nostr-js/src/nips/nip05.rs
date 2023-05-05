// Copyright (c) 2022-2023 Yuki Kishimoto
// Distributed under the MIT software license

use js_sys::Promise;
use nostr::nips::nip05;
use wasm_bindgen::prelude::*;

use crate::future::future_to_promise;
use crate::key::JsPublicKey;

/// Verify NIP05
#[wasm_bindgen(js_name = verifyNip05)]
pub fn verify_nip05(public_key: JsPublicKey, nip05: String) -> Promise {
    future_to_promise(async move {
        Ok(nip05::verify(public_key.into(), nip05.as_str(), None)
            .await
            .is_ok())
    })
}
