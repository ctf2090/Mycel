use std::fs;
use std::path::PathBuf;

use base64::Engine;
use ed25519_dalek::{Signer, SigningKey};
use proptest::prelude::*;
use serde_json::{json, Value};

use crate::canonical::{signed_payload_bytes, wire_envelope_signed_payload_bytes};
use crate::protocol::{recompute_declared_object_identity, recompute_object_id};
use crate::replay::{compute_state_hash, DocumentState};
use crate::store::write_object_value_to_store;

use super::{
    derive_wire_object_payload_identity, parse_wire_envelope, validate_wire_envelope,
    validate_wire_object_payload_behavior, validate_wire_payload, validate_wire_timestamp,
    verify_wire_envelope_signature, WireMessageType, WirePeerDirectory, WireSession,
};

fn hello_envelope_with(timestamp: &str) -> Value {
    json!({
        "type": "HELLO",
        "version": "mycel-wire/0.1",
        "msg_id": "msg:hello-proptest-001",
        "timestamp": timestamp,
        "from": "node:alpha",
        "payload": {
            "node_id": "node:alpha",
            "capabilities": ["patch-sync"],
            "nonce": "n:test"
        },
        "sig": "sig:placeholder"
    })
}

fn signed_patch_body_for_wire_tests() -> Value {
    sign_object_value(
        &signing_key(),
        json!({
            "type": "patch",
            "version": "mycel/0.1",
            "patch_id": "patch:placeholder",
            "doc_id": "doc:test",
            "base_revision": "rev:genesis-null",
            "author": "pk:ed25519:placeholder",
            "timestamp": 1u64,
            "ops": [],
            "signature": "sig:placeholder"
        }),
        "author",
        "patch_id",
        "patch",
    )
}

fn valid_object_payload_for_proptests() -> Value {
    let body = signed_patch_body_for_wire_tests();
    let identity = recompute_declared_object_identity(&body)
        .expect("wire proptest patch body identity should recompute");
    json!({
        "object_id": identity.object_id,
        "object_type": "patch",
        "encoding": "json",
        "hash_alg": "sha256",
        "hash": identity.hash,
        "body": body
    })
}

fn valid_wire_timestamp_strategy() -> impl Strategy<Value = String> {
    (
        0u16..=9999,
        0u8..=99,
        0u8..=99,
        0u8..=99,
        0u8..=99,
        0u8..=99,
        any::<bool>(),
        prop_oneof![Just('+'), Just('-')],
        0u8..=99,
        0u8..=99,
    )
        .prop_map(
            |(year, month, day, hour, minute, second, use_z, offset_sign, offset_hour, offset_minute)| {
                if use_z {
                    format!("{year:04}-{month:02}-{day:02}T{hour:02}:{minute:02}:{second:02}Z")
                } else {
                    format!(
                        "{year:04}-{month:02}-{day:02}T{hour:02}:{minute:02}:{second:02}{offset_sign}{offset_hour:02}:{offset_minute:02}"
                    )
                }
            },
        )
}

fn invalid_wire_timestamp_strategy() -> impl Strategy<Value = String> {
    (
        0u16..=9999,
        0u8..=99,
        0u8..=99,
        0u8..=99,
        0u8..=99,
        0u8..=99,
        prop_oneof![Just('+'), Just('-')],
        0u8..=99,
        0u8..=99,
    )
        .prop_flat_map(
            |(year, month, day, hour, minute, second, offset_sign, offset_hour, offset_minute)| {
                let no_t = format!(
                    "{year:04}-{month:02}-{day:02} {hour:02}:{minute:02}:{second:02}Z"
                );
                let slash_date = format!(
                    "{year:04}/{month:02}/{day:02}T{hour:02}:{minute:02}:{second:02}Z"
                );
                let no_offset_colon = format!(
                    "{year:04}-{month:02}-{day:02}T{hour:02}:{minute:02}:{second:02}{offset_sign}{offset_hour:02}{offset_minute:02}"
                );
                let missing_offset =
                    format!("{year:04}-{month:02}-{day:02}T{hour:02}:{minute:02}:{second:02}");
                let short_time = format!("{year:04}-{month:02}-{day:02}T{hour:02}:{minute:02}Z");
                prop_oneof![
                    Just(no_t),
                    Just(slash_date),
                    Just(no_offset_colon),
                    Just(missing_offset),
                    Just(short_time),
                ]
            },
        )
}

fn invalid_object_type_strategy() -> impl Strategy<Value = String> {
    ".*".prop_filter("object_type must be unsupported", |value| {
        !matches!(
            value.as_str(),
            "document" | "block" | "patch" | "revision" | "view" | "snapshot"
        )
    })
}

fn invalid_canonical_object_id_strategy() -> impl Strategy<Value = String> {
    ".*".prop_filter("object_id must violate canonical prefix rules", |value| {
        !["patch:", "rev:", "view:", "snap:"]
            .iter()
            .any(|prefix| value.starts_with(prefix) && value.len() > prefix.len())
    })
}

#[path = "tests/property.rs"]
mod property;
#[path = "tests/session.rs"]
mod session;

fn signing_key() -> SigningKey {
    SigningKey::from_bytes(&[9u8; 32])
}

fn temp_dir(prefix: &str) -> PathBuf {
    let unique = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .expect("system time should be after unix epoch")
        .as_nanos();
    let path = std::env::temp_dir().join(format!("mycel-wire-{prefix}-{unique}"));
    fs::create_dir_all(&path).expect("temp dir should be created");
    path
}

fn sender_public_key(signing_key: &SigningKey) -> String {
    format!(
        "pk:ed25519:{}",
        base64::engine::general_purpose::STANDARD.encode(signing_key.verifying_key().as_bytes())
    )
}

fn sign_wire_value(signing_key: &SigningKey, value: &Value) -> String {
    let payload =
        wire_envelope_signed_payload_bytes(value).expect("wire payload should canonicalize");
    let signature = signing_key.sign(&payload);
    format!(
        "sig:ed25519:{}",
        base64::engine::general_purpose::STANDARD.encode(signature.to_bytes())
    )
}

fn sign_object_value(
    signing_key: &SigningKey,
    mut value: Value,
    signer_field: &str,
    id_field: &str,
    prefix: &str,
) -> Value {
    value[signer_field] = Value::String(sender_public_key(signing_key));
    let object_id =
        recompute_object_id(&value, id_field, prefix).expect("test object ID should recompute");
    value[id_field] = Value::String(object_id);
    let payload = signed_payload_bytes(&value).expect("object payload should canonicalize");
    let signature = signing_key.sign(&payload);
    value["signature"] = Value::String(format!(
        "sig:ed25519:{}",
        base64::engine::general_purpose::STANDARD.encode(signature.to_bytes())
    ));
    value
}

fn empty_state_hash(doc_id: &str) -> String {
    compute_state_hash(&DocumentState {
        doc_id: doc_id.to_string(),
        blocks: Vec::new(),
        metadata: serde_json::Map::new(),
    })
    .expect("empty state hash should compute")
}

fn signed_hello_message(signing_key: &SigningKey, sender: &str, payload_node_id: &str) -> Value {
    signed_hello_message_with_capabilities(
        signing_key,
        sender,
        payload_node_id,
        json!(["patch-sync"]),
    )
}

fn signed_hello_message_with_capabilities(
    signing_key: &SigningKey,
    sender: &str,
    payload_node_id: &str,
    capabilities: Value,
) -> Value {
    let mut value = json!({
        "type": "HELLO",
        "version": "mycel-wire/0.1",
        "msg_id": "msg:hello-signed-001",
        "timestamp": "2026-03-08T20:00:00+08:00",
        "from": sender,
        "payload": {
            "node_id": payload_node_id,
            "capabilities": capabilities,
            "nonce": "n:test"
        },
        "sig": "sig:placeholder"
    });
    value["sig"] = Value::String(sign_wire_value(signing_key, &value));
    value
}

fn signed_manifest_message(signing_key: &SigningKey, sender: &str, payload_node_id: &str) -> Value {
    signed_manifest_message_with_capabilities(
        signing_key,
        sender,
        payload_node_id,
        json!(["patch-sync"]),
    )
}

fn signed_manifest_message_with_capabilities(
    signing_key: &SigningKey,
    sender: &str,
    payload_node_id: &str,
    capabilities: Value,
) -> Value {
    let mut value = json!({
        "type": "MANIFEST",
        "version": "mycel-wire/0.1",
        "msg_id": "msg:manifest-signed-001",
        "timestamp": "2026-03-08T20:00:10+08:00",
        "from": sender,
        "payload": {
            "node_id": payload_node_id,
            "capabilities": capabilities,
            "heads": {
                "doc:test": ["rev:test"]
            }
        },
        "sig": "sig:placeholder"
    });
    value["sig"] = Value::String(sign_wire_value(signing_key, &value));
    value
}

fn signed_snapshot_offer_message(
    signing_key: &SigningKey,
    sender: &str,
    snapshot_id: &str,
) -> Value {
    let mut value = json!({
        "type": "SNAPSHOT_OFFER",
        "version": "mycel-wire/0.1",
        "msg_id": "msg:snapshot-offer-signed-001",
        "timestamp": "2026-03-08T20:00:40+08:00",
        "from": sender,
        "payload": {
            "snapshot_id": snapshot_id,
            "root_hash": "hash:snapshot-root",
            "documents": ["doc:test"]
        },
        "sig": "sig:placeholder"
    });
    value["sig"] = Value::String(sign_wire_value(signing_key, &value));
    value
}

fn signed_view_announce_message(signing_key: &SigningKey, sender: &str, view_id: &str) -> Value {
    let mut value = json!({
        "type": "VIEW_ANNOUNCE",
        "version": "mycel-wire/0.1",
        "msg_id": "msg:view-announce-signed-001",
        "timestamp": "2026-03-08T20:00:45+08:00",
        "from": sender,
        "payload": {
            "view_id": view_id,
            "maintainer": sender_public_key(signing_key),
            "documents": {
                "doc:test": "rev:test"
            }
        },
        "sig": "sig:placeholder"
    });
    value["sig"] = Value::String(sign_wire_value(signing_key, &value));
    value
}

fn signed_manifest_message_with_heads(
    signing_key: &SigningKey,
    sender: &str,
    payload_node_id: &str,
    heads: Value,
) -> Value {
    let mut value = json!({
        "type": "MANIFEST",
        "version": "mycel-wire/0.1",
        "msg_id": "msg:manifest-signed-001",
        "timestamp": "2026-03-08T20:00:10+08:00",
        "from": sender,
        "payload": {
            "node_id": payload_node_id,
            "capabilities": ["patch-sync"],
            "heads": heads
        },
        "sig": "sig:placeholder"
    });
    value["sig"] = Value::String(sign_wire_value(signing_key, &value));
    value
}

fn signed_want_message(signing_key: &SigningKey, sender: &str, object_ids: &[&str]) -> Value {
    let mut value = json!({
        "type": "WANT",
        "version": "mycel-wire/0.1",
        "msg_id": "msg:want-signed-001",
        "timestamp": "2026-03-08T20:01:00+08:00",
        "from": sender,
        "payload": {
            "objects": object_ids
        },
        "sig": "sig:placeholder"
    });
    value["sig"] = Value::String(sign_wire_value(signing_key, &value));
    value
}

fn signed_heads_message(
    signing_key: &SigningKey,
    sender: &str,
    documents: Value,
    replace: bool,
) -> Value {
    let mut value = json!({
        "type": "HEADS",
        "version": "mycel-wire/0.1",
        "msg_id": "msg:heads-signed-001",
        "timestamp": "2026-03-08T20:00:30+08:00",
        "from": sender,
        "payload": {
            "documents": documents,
            "replace": replace
        },
        "sig": "sig:placeholder"
    });
    value["sig"] = Value::String(sign_wire_value(signing_key, &value));
    value
}

fn signed_object_message(signing_key: &SigningKey, sender: &str) -> Value {
    signed_patch_object_message(signing_key, sender, "rev:genesis-null")
}

fn signed_patch_object_message(
    signing_key: &SigningKey,
    sender: &str,
    base_revision: &str,
) -> Value {
    let body = sign_object_value(
        signing_key,
        json!({
            "type": "patch",
            "version": "mycel/0.1",
            "patch_id": "patch:placeholder",
            "doc_id": "doc:test",
            "base_revision": base_revision,
            "author": "pk:ed25519:placeholder",
            "timestamp": 1u64,
            "ops": [],
            "signature": "sig:placeholder"
        }),
        "author",
        "patch_id",
        "patch",
    );
    let object_id = body["patch_id"]
        .as_str()
        .expect("signed patch body should include patch_id")
        .to_owned();
    let object_hash = object_id
        .split_once(':')
        .map(|(_, hash)| hash.to_string())
        .expect("wire object ID should contain hash");

    let mut value = json!({
        "type": "OBJECT",
        "version": "mycel-wire/0.1",
        "msg_id": "msg:object-signed-001",
        "timestamp": "2026-03-08T20:01:02+08:00",
        "from": sender,
        "payload": {
            "object_id": object_id,
            "object_type": "patch",
            "encoding": "json",
            "hash_alg": "sha256",
            "hash": format!("hash:{object_hash}"),
            "body": body
        },
        "sig": "sig:placeholder"
    });
    value["sig"] = Value::String(sign_wire_value(signing_key, &value));
    value
}

fn signed_revision_object_message(
    signing_key: &SigningKey,
    sender: &str,
    parents: &[&str],
    patches: &[&str],
) -> Value {
    let body = sign_object_value(
        signing_key,
        json!({
            "type": "revision",
            "version": "mycel/0.1",
            "revision_id": "rev:placeholder",
            "doc_id": "doc:test",
            "parents": parents,
            "patches": patches,
            "state_hash": empty_state_hash("doc:test"),
            "author": "pk:ed25519:placeholder",
            "timestamp": 1u64,
            "signature": "sig:placeholder"
        }),
        "author",
        "revision_id",
        "rev",
    );
    let object_id = body["revision_id"]
        .as_str()
        .expect("signed revision body should include revision_id")
        .to_owned();
    let object_hash = object_id
        .split_once(':')
        .map(|(_, hash)| hash.to_string())
        .expect("wire revision ID should contain hash");

    let mut value = json!({
        "type": "OBJECT",
        "version": "mycel-wire/0.1",
        "msg_id": "msg:revision-object-signed-001",
        "timestamp": "2026-03-08T20:01:02+08:00",
        "from": sender,
        "payload": {
            "object_id": object_id,
            "object_type": "revision",
            "encoding": "json",
            "hash_alg": "sha256",
            "hash": format!("hash:{object_hash}"),
            "body": body
        },
        "sig": "sig:placeholder"
    });
    value["sig"] = Value::String(sign_wire_value(signing_key, &value));
    value
}

fn signed_error_message(signing_key: &SigningKey, sender: &str, in_reply_to: &str) -> Value {
    let mut value = json!({
        "type": "ERROR",
        "version": "mycel-wire/0.1",
        "msg_id": "msg:error-signed-001",
        "timestamp": "2026-03-08T20:02:00+08:00",
        "from": sender,
        "payload": {
            "in_reply_to": in_reply_to,
            "code": "ERR_UNKNOWN",
            "detail": "test error"
        },
        "sig": "sig:placeholder"
    });
    value["sig"] = Value::String(sign_wire_value(signing_key, &value));
    value
}

fn signed_bye_message(signing_key: &SigningKey, sender: &str) -> Value {
    let mut value = json!({
        "type": "BYE",
        "version": "mycel-wire/0.1",
        "msg_id": "msg:bye-signed-001",
        "timestamp": "2026-03-08T20:02:00+08:00",
        "from": sender,
        "payload": {
            "reason": "done"
        },
        "sig": "sig:placeholder"
    });
    value["sig"] = Value::String(sign_wire_value(signing_key, &value));
    value
}

#[test]
fn parse_wire_envelope_accepts_minimal_hello() {
    let value = json!({
        "type": "HELLO",
        "version": "mycel-wire/0.1",
        "msg_id": "msg:hello-001",
        "timestamp": "2026-03-08T20:00:00+08:00",
        "from": "node:alpha",
        "payload": {
            "node_id": "node:alpha",
            "capabilities": ["patch-sync"],
            "nonce": "n:test"
        },
        "sig": "sig:placeholder"
    });

    let envelope = parse_wire_envelope(&value).expect("wire envelope should parse");

    assert_eq!(envelope.message_type(), WireMessageType::Hello);
    assert_eq!(envelope.from(), "node:alpha");
    assert_eq!(
        envelope.payload().get("node_id").and_then(Value::as_str),
        Some("node:alpha")
    );
}

#[test]
fn parse_wire_envelope_rejects_wrong_version() {
    let value = json!({
        "type": "HELLO",
        "version": "mycel-wire/9.9",
        "msg_id": "msg:hello-001",
        "timestamp": "2026-03-08T20:00:00+08:00",
        "from": "node:alpha",
        "payload": {
            "node_id": "node:alpha",
            "capabilities": ["patch-sync"],
            "nonce": "n:test"
        },
        "sig": "sig:placeholder"
    });

    let error = parse_wire_envelope(&value).unwrap_err();

    assert_eq!(error, "wire envelope 'version' must equal 'mycel-wire/0.1'");
}

#[test]
fn validate_wire_payload_rejects_object_body_type_mismatch() {
    let payload = json!({
        "object_id": "patch:test",
        "object_type": "patch",
        "encoding": "json",
        "hash_alg": "sha256",
        "hash": "hash:test",
        "body": {
            "type": "revision",
            "version": "mycel/0.1",
            "revision_id": "rev:test",
            "doc_id": "doc:test",
            "parents": [],
            "patches": [],
            "state_hash": "hash:test",
            "author": "pk:ed25519:test",
            "timestamp": 1u64
        }
    });

    validate_wire_payload(
        WireMessageType::Object,
        payload.as_object().expect("payload should be object"),
    )
    .expect("OBJECT payload shape should validate before behavior checks");
    let error = validate_wire_object_payload_behavior(
        payload.as_object().expect("payload should be object"),
    )
    .unwrap_err();

    assert!(error.contains("OBJECT body type 'revision' does not match object_type 'patch'"));
}

#[test]
fn validate_wire_payload_rejects_non_sha256_object_hash_algorithm() {
    let payload = json!({
        "object_id": "patch:test",
        "object_type": "patch",
        "encoding": "json",
        "hash_alg": "blake3",
        "hash": "hash:test",
        "body": {
            "type": "patch",
            "version": "mycel/0.1",
            "patch_id": "patch:test",
            "doc_id": "doc:test",
            "base_revision": "rev:genesis-null",
            "author": "pk:ed25519:test",
            "timestamp": 1u64,
            "ops": []
        }
    });

    let error = validate_wire_payload(
        WireMessageType::Object,
        payload.as_object().expect("payload should be object"),
    )
    .unwrap_err();

    assert_eq!(error, "OBJECT payload 'hash_alg' must equal 'sha256'");
}

#[test]
fn validate_wire_payload_rejects_unknown_hello_payload_field() {
    let payload = json!({
        "node_id": "node:alpha",
        "capabilities": ["patch-sync"],
        "nonce": "n:test",
        "unexpected": true
    });

    let error = validate_wire_payload(
        WireMessageType::Hello,
        payload.as_object().expect("payload should be object"),
    )
    .unwrap_err();

    assert_eq!(error, "top-level contains unexpected field 'unexpected'");
}

#[test]
fn validate_wire_payload_rejects_non_array_hello_topics() {
    let payload = json!({
        "node_id": "node:alpha",
        "capabilities": ["patch-sync"],
        "topics": "text/core",
        "nonce": "n:test"
    });

    let error = validate_wire_payload(
        WireMessageType::Hello,
        payload.as_object().expect("payload should be object"),
    )
    .unwrap_err();

    assert_eq!(error, "top-level 'topics' must be an array");
}

#[test]
fn validate_wire_payload_rejects_negative_snapshot_offer_size_bytes() {
    let payload = json!({
        "snapshot_id": "snap:test",
        "root_hash": "hash:test",
        "documents": ["doc:test"],
        "size_bytes": -1
    });

    let error = validate_wire_payload(
        WireMessageType::SnapshotOffer,
        payload.as_object().expect("payload should be object"),
    )
    .unwrap_err();

    assert_eq!(
        error,
        "wire payload field 'size_bytes' must be a non-negative integer"
    );
}

#[test]
fn validate_wire_payload_rejects_unknown_snapshot_offer_payload_field() {
    let payload = json!({
        "snapshot_id": "snap:test",
        "root_hash": "hash:test",
        "documents": ["doc:test"],
        "unknown_count": 7u64
    });

    let error = validate_wire_payload(
        WireMessageType::SnapshotOffer,
        payload.as_object().expect("payload should be object"),
    )
    .unwrap_err();

    assert_eq!(error, "top-level contains unexpected field 'unknown_count'");
}

#[test]
fn validate_wire_payload_rejects_non_string_error_detail() {
    let payload = json!({
        "in_reply_to": "msg:test",
        "code": "INVALID_HASH",
        "detail": 7
    });

    let error = validate_wire_payload(
        WireMessageType::Error,
        payload.as_object().expect("payload should be object"),
    )
    .unwrap_err();

    assert_eq!(error, "wire payload field 'detail' must be a string");
}

#[test]
fn derive_wire_object_payload_identity_matches_signed_patch_body() {
    let signing_key = signing_key();
    let body = sign_object_value(
        &signing_key,
        json!({
            "type": "patch",
            "version": "mycel/0.1",
            "patch_id": "patch:placeholder",
            "doc_id": "doc:test",
            "base_revision": "rev:genesis-null",
            "author": "pk:ed25519:placeholder",
            "timestamp": 1u64,
            "ops": [],
            "signature": "sig:placeholder"
        }),
        "author",
        "patch_id",
        "patch",
    );

    let identity = derive_wire_object_payload_identity(&body)
        .expect("wire object payload identity should derive");

    assert_eq!(identity.object_type, "patch");
    assert_eq!(
        identity.object_id,
        body["patch_id"]
            .as_str()
            .expect("signed patch body should include patch_id")
    );
    assert_eq!(
        identity.hash,
        format!(
            "hash:{}",
            identity
                .object_id
                .split_once(':')
                .map(|(_, digest)| digest)
                .expect("object ID should include digest")
        )
    );
}

#[test]
fn validate_wire_object_payload_behavior_accepts_payload_built_from_shared_identity_helper() {
    let signing_key = signing_key();
    let body = sign_object_value(
        &signing_key,
        json!({
            "type": "revision",
            "version": "mycel/0.1",
            "revision_id": "rev:placeholder",
            "doc_id": "doc:test",
            "parents": [],
            "patches": [],
            "state_hash": empty_state_hash("doc:test"),
            "author": "pk:ed25519:placeholder",
            "timestamp": 1u64,
            "signature": "sig:placeholder"
        }),
        "author",
        "revision_id",
        "rev",
    );
    let identity = derive_wire_object_payload_identity(&body)
        .expect("wire object payload identity should derive");
    let payload = json!({
        "object_id": identity.object_id,
        "object_type": identity.object_type,
        "encoding": "json",
        "hash_alg": "sha256",
        "hash": identity.hash,
        "body": body
    });

    validate_wire_payload(
        WireMessageType::Object,
        payload.as_object().expect("payload should be object"),
    )
    .expect("OBJECT payload shape should validate");
    validate_wire_object_payload_behavior(payload.as_object().expect("payload should be object"))
        .expect("OBJECT payload should match shared identity helper output");
}

#[test]
fn validate_wire_object_payload_behavior_rejects_missing_required_body_signature() {
    let body = json!({
        "type": "patch",
        "version": "mycel/0.1",
        "patch_id": "patch:8d13c0b560f101a83ed57f4ab84f5a39a214ba53cc4bfe4f4f6de643eb447c0a",
        "doc_id": "doc:test",
        "base_revision": "rev:genesis-null",
        "author": "pk:ed25519:test",
        "timestamp": 1u64,
        "ops": []
    });
    let payload = json!({
        "object_id": body["patch_id"],
        "object_type": "patch",
        "encoding": "json",
        "hash_alg": "sha256",
        "hash": "hash:8d13c0b560f101a83ed57f4ab84f5a39a214ba53cc4bfe4f4f6de643eb447c0a",
        "body": body
    });

    let error = validate_wire_object_payload_behavior(
        payload.as_object().expect("payload should be object"),
    )
    .unwrap_err();

    assert!(
        error.contains("OBJECT body failed shared verification"),
        "expected shared verification prefix, got {error}"
    );
    assert!(
        error.contains("patch object is missing required top-level 'signature'"),
        "expected missing signature error, got {error}"
    );
}

#[test]
fn validate_wire_object_payload_behavior_rejects_shared_semantic_edge_failure() {
    let signing_key = signing_key();
    let body = sign_object_value(
        &signing_key,
        json!({
            "type": "view",
            "version": "mycel/0.1",
            "view_id": "view:placeholder",
            "maintainer": "pk:ed25519:placeholder",
            "documents": {
                "doc:test": "rev:test"
            },
            "policy": {
                "accept_keys": [""],
                "merge_rule": "manual-reviewed"
            },
            "timestamp": 12u64,
            "signature": "sig:placeholder"
        }),
        "maintainer",
        "view_id",
        "view",
    );
    let identity = derive_wire_object_payload_identity(&body)
        .expect("wire object payload identity should derive");
    let payload = json!({
        "object_id": identity.object_id,
        "object_type": identity.object_type,
        "encoding": "json",
        "hash_alg": "sha256",
        "hash": identity.hash,
        "body": body
    });

    let error = validate_wire_object_payload_behavior(
        payload.as_object().expect("payload should be object"),
    )
    .unwrap_err();

    assert!(
        error.contains("OBJECT body failed shared verification"),
        "expected shared verification prefix, got {error}"
    );
    assert!(
        error.contains("top-level 'policy.accept_keys[0]' must not be an empty string"),
        "expected view semantic-edge error, got {error}"
    );
}

#[test]
fn validate_wire_envelope_accepts_concrete_object_payload() {
    let signing_key = signing_key();
    let body = sign_object_value(
        &signing_key,
        json!({
            "type": "patch",
            "version": "mycel/0.1",
            "patch_id": "patch:placeholder",
            "doc_id": "doc:test",
            "base_revision": "rev:genesis-null",
            "author": "pk:ed25519:placeholder",
            "timestamp": 1u64,
            "ops": [],
            "signature": "sig:placeholder"
        }),
        "author",
        "patch_id",
        "patch",
    );
    let identity = recompute_declared_object_identity(&body)
        .expect("concrete wire object identity should recompute");

    let value = json!({
        "type": "OBJECT",
        "version": "mycel-wire/0.1",
        "msg_id": "msg:obj-concrete-001",
        "timestamp": "2026-03-08T20:01:02+08:00",
        "from": "node:alpha",
        "payload": {
            "object_id": identity.object_id,
            "object_type": "patch",
            "encoding": "json",
            "hash_alg": "sha256",
            "hash": identity.hash,
            "body": body
        },
        "sig": "sig:..."
    });

    let envelope = validate_wire_envelope(&value).expect("wire envelope should validate");
    validate_wire_object_payload_behavior(envelope.payload())
        .expect("concrete OBJECT payload should match recomputed ID and hash");
}

#[test]
fn verify_wire_envelope_signature_accepts_valid_signed_hello() {
    let signing_key = signing_key();
    let sender_key = sender_public_key(&signing_key);
    let value = signed_hello_message(&signing_key, "node:alpha", "node:alpha");

    let envelope =
        verify_wire_envelope_signature(&value, &sender_key).expect("wire signature should verify");

    assert_eq!(envelope.message_type(), WireMessageType::Hello);
}

#[test]
fn verify_wire_envelope_signature_rejects_invalid_signature() {
    let signing_key = signing_key();
    let sender_key = sender_public_key(&signing_key);
    let value = json!({
        "type": "HELLO",
        "version": "mycel-wire/0.1",
        "msg_id": "msg:hello-signed-001",
        "timestamp": "2026-03-08T20:00:00+08:00",
        "from": "node:alpha",
        "payload": {
            "node_id": "node:alpha",
            "capabilities": ["patch-sync"],
            "nonce": "n:test"
        },
        "sig": "sig:ed25519:AAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAA=="
    });

    let error = verify_wire_envelope_signature(&value, &sender_key).unwrap_err();

    assert!(error.contains("Ed25519 signature verification failed"));
}

#[test]
fn verify_wire_envelope_signature_rejects_malformed_sender_public_key() {
    let signing_key = signing_key();
    let value = signed_hello_message(&signing_key, "node:alpha", "node:alpha");

    let error = verify_wire_envelope_signature(&value, "node:alpha").unwrap_err();

    assert_eq!(
        error,
        "sender public key must use format 'pk:ed25519:<base64>'"
    );
}
