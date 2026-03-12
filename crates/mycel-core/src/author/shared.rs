use std::path::Path;

use base64::Engine;
use ed25519_dalek::{Signer, SigningKey};
use serde_json::Value;

use crate::canonical::signed_payload_bytes;
use crate::store::{load_stored_object_value, StoreRebuildError};

pub fn parse_signing_key_seed(seed: &str) -> Result<SigningKey, String> {
    let decoded = base64::engine::general_purpose::STANDARD
        .decode(seed.trim())
        .map_err(|error| format!("failed to decode base64 signing key seed: {error}"))?;
    let bytes: [u8; 32] = decoded
        .try_into()
        .map_err(|_| "signing key seed must decode to exactly 32 bytes".to_string())?;
    Ok(SigningKey::from_bytes(&bytes))
}

pub fn signer_id(signing_key: &SigningKey) -> String {
    format!(
        "pk:ed25519:{}",
        base64::engine::general_purpose::STANDARD.encode(signing_key.verifying_key().as_bytes())
    )
}

pub(crate) fn ensure_document_exists(
    store_root: &Path,
    doc_id: &str,
) -> Result<(), StoreRebuildError> {
    ensure_object_exists(store_root, doc_id, "document")
}

pub(crate) fn ensure_object_exists(
    store_root: &Path,
    object_id: &str,
    label: &str,
) -> Result<(), StoreRebuildError> {
    load_stored_object_value(store_root, object_id)
        .map(|_| ())
        .map_err(|_| {
            StoreRebuildError::new(format!(
                "{label} '{}' was not found in the store",
                object_id
            ))
        })
}

pub(crate) fn sign_object_value(
    signing_key: &SigningKey,
    value: &Value,
) -> Result<String, StoreRebuildError> {
    let payload = signed_payload_bytes(value).map_err(|error| {
        StoreRebuildError::new(format!("failed to compute signed payload: {error}"))
    })?;
    let signature = signing_key.sign(&payload);
    Ok(format!(
        "sig:ed25519:{}",
        base64::engine::general_purpose::STANDARD.encode(signature.to_bytes())
    ))
}
