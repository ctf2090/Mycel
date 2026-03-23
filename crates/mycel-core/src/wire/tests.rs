use std::fs;
use std::path::PathBuf;

use base64::Engine;
use ed25519_dalek::{Signer, SigningKey};
use serde_json::{json, Value};

use crate::canonical::{signed_payload_bytes, wire_envelope_signed_payload_bytes};
use crate::protocol::{recompute_declared_object_identity, recompute_object_id};
use crate::replay::{compute_state_hash, DocumentState};
use crate::store::write_object_value_to_store;

use super::{
    derive_wire_object_payload_identity, parse_wire_envelope, validate_wire_envelope,
    validate_wire_object_payload_behavior, validate_wire_payload, verify_wire_envelope_signature,
    WireMessageType, WirePeerDirectory, WireSession,
};

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

#[test]
fn wire_session_verifies_incoming_hello_from_registered_peer() {
    let signing_key = signing_key();
    let sender_key = sender_public_key(&signing_key);
    let mut session = WireSession::default();
    session
        .register_known_peer("node:alpha", &sender_key)
        .expect("known peer should register");
    let value = signed_hello_message(&signing_key, "node:alpha", "node:alpha");

    let envelope = session
        .verify_incoming(&value)
        .expect("registered sender should verify");

    assert_eq!(envelope.from(), "node:alpha");
    assert_eq!(envelope.message_type(), WireMessageType::Hello);
}

#[test]
fn wire_session_rejects_unknown_sender() {
    let signing_key = signing_key();
    let value = signed_hello_message(&signing_key, "node:alpha", "node:alpha");

    let error = WireSession::default().verify_incoming(&value).unwrap_err();

    assert_eq!(error, "unknown wire sender 'node:alpha'");
}

#[test]
fn wire_session_rejects_hello_node_id_mismatch() {
    let signing_key = signing_key();
    let sender_key = sender_public_key(&signing_key);
    let mut session = WireSession::new(WirePeerDirectory::new());
    session
        .register_known_peer("node:alpha", &sender_key)
        .expect("known peer should register");
    let value = signed_hello_message(&signing_key, "node:alpha", "node:beta");

    let error = session.verify_incoming(&value).unwrap_err();

    assert_eq!(
        error,
        "wire HELLO payload 'node_id' must equal envelope 'from'"
    );
}

#[test]
fn wire_session_rejects_manifest_before_hello() {
    let signing_key = signing_key();
    let sender_key = sender_public_key(&signing_key);
    let mut session = WireSession::default();
    session
        .register_known_peer("node:alpha", &sender_key)
        .expect("known peer should register");
    let value = signed_manifest_message(&signing_key, "node:alpha", "node:alpha");

    let error = session.verify_incoming(&value).unwrap_err();

    assert_eq!(
        error,
        "wire MANIFEST requires prior HELLO from 'node:alpha'"
    );
}

#[test]
fn wire_session_records_manifest_heads() {
    let signing_key = signing_key();
    let sender_key = sender_public_key(&signing_key);
    let mut session = WireSession::default();
    session
        .register_known_peer("node:alpha", &sender_key)
        .expect("known peer should register");
    let hello = signed_hello_message(&signing_key, "node:alpha", "node:alpha");
    let manifest = signed_manifest_message(&signing_key, "node:alpha", "node:alpha");

    session
        .verify_incoming(&hello)
        .expect("HELLO should verify");
    session
        .verify_incoming(&manifest)
        .expect("MANIFEST should verify");

    let state = session
        .peer_session("node:alpha")
        .expect("peer session should exist");
    assert_eq!(
        state
            .advertised_document_heads
            .get("doc:test")
            .map(|revisions| revisions.len()),
        Some(1)
    );
    assert!(state
        .advertised_document_heads
        .get("doc:test")
        .is_some_and(|revisions| revisions.contains("rev:test")));
}

#[test]
fn wire_session_merges_incremental_heads_updates() {
    let signing_key = signing_key();
    let sender_key = sender_public_key(&signing_key);
    let mut session = WireSession::default();
    session
        .register_known_peer("node:alpha", &sender_key)
        .expect("known peer should register");
    let hello = signed_hello_message(&signing_key, "node:alpha", "node:alpha");
    let manifest = signed_manifest_message(&signing_key, "node:alpha", "node:alpha");
    let heads = signed_heads_message(
        &signing_key,
        "node:alpha",
        json!({
            "doc:test": ["rev:next"],
            "doc:extra": ["rev:extra"]
        }),
        false,
    );

    session
        .verify_incoming(&hello)
        .expect("HELLO should verify");
    session
        .verify_incoming(&manifest)
        .expect("MANIFEST should verify");
    session
        .verify_incoming(&heads)
        .expect("HEADS should verify");

    let state = session
        .peer_session("node:alpha")
        .expect("peer session should exist");
    assert!(state
        .advertised_document_heads
        .get("doc:test")
        .is_some_and(|revisions| {
            revisions.contains("rev:test") && revisions.contains("rev:next")
        }));
    assert!(state
        .advertised_document_heads
        .get("doc:extra")
        .is_some_and(|revisions| revisions.contains("rev:extra")));
}

#[test]
fn wire_session_replaces_heads_when_replace_is_true() {
    let signing_key = signing_key();
    let sender_key = sender_public_key(&signing_key);
    let mut session = WireSession::default();
    session
        .register_known_peer("node:alpha", &sender_key)
        .expect("known peer should register");
    let hello = signed_hello_message(&signing_key, "node:alpha", "node:alpha");
    let manifest = signed_manifest_message(&signing_key, "node:alpha", "node:alpha");
    let heads = signed_heads_message(
        &signing_key,
        "node:alpha",
        json!({
            "doc:replacement": ["rev:replacement"]
        }),
        true,
    );

    session
        .verify_incoming(&hello)
        .expect("HELLO should verify");
    session
        .verify_incoming(&manifest)
        .expect("MANIFEST should verify");
    session
        .verify_incoming(&heads)
        .expect("HEADS should verify");

    let state = session
        .peer_session("node:alpha")
        .expect("peer session should exist");
    assert!(!state.advertised_document_heads.contains_key("doc:test"));
    assert!(state
        .advertised_document_heads
        .get("doc:replacement")
        .is_some_and(|revisions| revisions.contains("rev:replacement")));
}

#[test]
fn wire_session_rejects_snapshot_offer_without_snapshot_capability() {
    let signing_key = signing_key();
    let sender_key = sender_public_key(&signing_key);
    let mut session = WireSession::default();
    session
        .register_known_peer("node:alpha", &sender_key)
        .expect("known peer should register");
    let hello = signed_hello_message(&signing_key, "node:alpha", "node:alpha");
    let snapshot_offer =
        signed_snapshot_offer_message(&signing_key, "node:alpha", "snap:test-offer");

    session
        .verify_incoming(&hello)
        .expect("HELLO should verify");
    let error = session.verify_incoming(&snapshot_offer).unwrap_err();

    assert_eq!(
        error,
        "wire SNAPSHOT_OFFER requires advertised capability 'snapshot-sync' from 'node:alpha'"
    );
}

#[test]
fn wire_session_accepts_snapshot_offer_with_snapshot_capability_and_unlocks_want() {
    let signing_key = signing_key();
    let sender_key = sender_public_key(&signing_key);
    let mut session = WireSession::default();
    session
        .register_known_peer("node:alpha", &sender_key)
        .expect("known peer should register");
    let hello = signed_hello_message_with_capabilities(
        &signing_key,
        "node:alpha",
        "node:alpha",
        json!(["patch-sync", "snapshot-sync"]),
    );
    let manifest = signed_manifest_message_with_capabilities(
        &signing_key,
        "node:alpha",
        "node:alpha",
        json!(["patch-sync", "snapshot-sync"]),
    );
    let snapshot_offer =
        signed_snapshot_offer_message(&signing_key, "node:alpha", "snap:test-offer");
    let want = signed_want_message(&signing_key, "node:alpha", &["snap:test-offer"]);

    session
        .verify_incoming(&hello)
        .expect("HELLO should verify");
    session
        .verify_incoming(&manifest)
        .expect("MANIFEST should verify");
    session
        .verify_incoming(&snapshot_offer)
        .expect("SNAPSHOT_OFFER should verify");
    session
        .verify_incoming(&want)
        .expect("snapshot WANT should verify after offer");

    let state = session
        .peer_session("node:alpha")
        .expect("peer session should exist");
    assert!(state.reachable_object_ids.contains("snap:test-offer"));
    assert!(state.pending_object_ids.contains("snap:test-offer"));
}

#[test]
fn wire_session_rejects_view_announce_without_view_capability() {
    let signing_key = signing_key();
    let sender_key = sender_public_key(&signing_key);
    let mut session = WireSession::default();
    session
        .register_known_peer("node:alpha", &sender_key)
        .expect("known peer should register");
    let hello = signed_hello_message(&signing_key, "node:alpha", "node:alpha");
    let view_announce =
        signed_view_announce_message(&signing_key, "node:alpha", "view:test-announce");

    session
        .verify_incoming(&hello)
        .expect("HELLO should verify");
    let error = session.verify_incoming(&view_announce).unwrap_err();

    assert_eq!(
        error,
        "wire VIEW_ANNOUNCE requires advertised capability 'view-sync' from 'node:alpha'"
    );
}

#[test]
fn wire_session_accepts_view_announce_with_view_capability_and_unlocks_want() {
    let signing_key = signing_key();
    let sender_key = sender_public_key(&signing_key);
    let mut session = WireSession::default();
    session
        .register_known_peer("node:alpha", &sender_key)
        .expect("known peer should register");
    let hello = signed_hello_message_with_capabilities(
        &signing_key,
        "node:alpha",
        "node:alpha",
        json!(["patch-sync", "view-sync"]),
    );
    let manifest = signed_manifest_message_with_capabilities(
        &signing_key,
        "node:alpha",
        "node:alpha",
        json!(["patch-sync", "view-sync"]),
    );
    let view_announce =
        signed_view_announce_message(&signing_key, "node:alpha", "view:test-announce");
    let want = signed_want_message(&signing_key, "node:alpha", &["view:test-announce"]);

    session
        .verify_incoming(&hello)
        .expect("HELLO should verify");
    session
        .verify_incoming(&manifest)
        .expect("MANIFEST should verify");
    session
        .verify_incoming(&view_announce)
        .expect("VIEW_ANNOUNCE should verify");
    session
        .verify_incoming(&want)
        .expect("view WANT should verify after announcement");

    let state = session
        .peer_session("node:alpha")
        .expect("peer session should exist");
    assert!(state.reachable_object_ids.contains("view:test-announce"));
    assert!(state.pending_object_ids.contains("view:test-announce"));
}

#[test]
fn wire_session_rejects_want_before_head_context() {
    let signing_key = signing_key();
    let sender_key = sender_public_key(&signing_key);
    let mut session = WireSession::default();
    session
        .register_known_peer("node:alpha", &sender_key)
        .expect("known peer should register");
    let hello = signed_hello_message(&signing_key, "node:alpha", "node:alpha");
    let want = signed_want_message(&signing_key, "node:alpha", &["patch:test"]);

    session
        .verify_incoming(&hello)
        .expect("HELLO should verify");
    let error = session.verify_incoming(&want).unwrap_err();

    assert_eq!(
        error,
        "wire WANT requires prior MANIFEST or HEADS from 'node:alpha'"
    );
}

#[test]
fn wire_session_rejects_unadvertised_revision_want() {
    let signing_key = signing_key();
    let sender_key = sender_public_key(&signing_key);
    let mut session = WireSession::default();
    session
        .register_known_peer("node:alpha", &sender_key)
        .expect("known peer should register");
    let hello = signed_hello_message(&signing_key, "node:alpha", "node:alpha");
    let manifest = signed_manifest_message(&signing_key, "node:alpha", "node:alpha");
    let want = signed_want_message(&signing_key, "node:alpha", &["rev:missing"]);

    session
        .verify_incoming(&hello)
        .expect("HELLO should verify");
    session
        .verify_incoming(&manifest)
        .expect("MANIFEST should verify");
    let error = session.verify_incoming(&want).unwrap_err();

    assert_eq!(
        error,
        "wire WANT revision 'rev:missing' is not reachable from accepted sync roots for 'node:alpha'"
    );
}

#[test]
fn wire_session_rejects_non_revision_want_without_sync_root() {
    let signing_key = signing_key();
    let sender_key = sender_public_key(&signing_key);
    let mut session = WireSession::default();
    session
        .register_known_peer("node:alpha", &sender_key)
        .expect("known peer should register");
    let hello = signed_hello_message(&signing_key, "node:alpha", "node:alpha");
    let manifest = signed_manifest_message(&signing_key, "node:alpha", "node:alpha");
    let want = signed_want_message(&signing_key, "node:alpha", &["patch:test"]);

    session
        .verify_incoming(&hello)
        .expect("HELLO should verify");
    session
        .verify_incoming(&manifest)
        .expect("MANIFEST should verify");
    let error = session.verify_incoming(&want).unwrap_err();

    assert_eq!(
        error,
        "wire WANT object 'patch:test' is not reachable from accepted sync roots for 'node:alpha'"
    );
}

#[test]
fn wire_session_rejects_follow_on_object_before_root_object_arrives() {
    let signing_key = signing_key();
    let sender_key = sender_public_key(&signing_key);
    let mut session = WireSession::default();
    session
        .register_known_peer("node:alpha", &sender_key)
        .expect("known peer should register");
    let patch_object = signed_patch_object_message(&signing_key, "node:alpha", "rev:genesis-null");
    let patch_id = patch_object["payload"]["object_id"]
        .as_str()
        .expect("signed patch OBJECT should include object_id")
        .to_owned();
    let revision_object =
        signed_revision_object_message(&signing_key, "node:alpha", &[], &[patch_id.as_str()]);
    let revision_id = revision_object["payload"]["object_id"]
        .as_str()
        .expect("signed revision OBJECT should include object_id")
        .to_owned();
    let hello = signed_hello_message(&signing_key, "node:alpha", "node:alpha");
    let manifest = signed_manifest_message_with_heads(
        &signing_key,
        "node:alpha",
        "node:alpha",
        json!({
            "doc:test": [revision_id.clone()]
        }),
    );
    let want = signed_want_message(
        &signing_key,
        "node:alpha",
        &[revision_id.as_str(), patch_id.as_str()],
    );

    session
        .verify_incoming(&hello)
        .expect("HELLO should verify");
    session
        .verify_incoming(&manifest)
        .expect("MANIFEST should verify");
    let error = session.verify_incoming(&want).unwrap_err();

    assert_eq!(
        error,
        format!(
            "wire WANT object '{}' is not reachable from accepted sync roots for 'node:alpha'",
            patch_id
        )
    );
}

#[test]
fn wire_session_accepts_follow_on_patch_after_reachable_revision_object() {
    let signing_key = signing_key();
    let sender_key = sender_public_key(&signing_key);
    let mut session = WireSession::default();
    session
        .register_known_peer("node:alpha", &sender_key)
        .expect("known peer should register");
    let patch_object = signed_patch_object_message(&signing_key, "node:alpha", "rev:genesis-null");
    let patch_id = patch_object["payload"]["object_id"]
        .as_str()
        .expect("signed patch OBJECT should include object_id")
        .to_owned();
    let revision_object =
        signed_revision_object_message(&signing_key, "node:alpha", &[], &[patch_id.as_str()]);
    let revision_id = revision_object["payload"]["object_id"]
        .as_str()
        .expect("signed revision OBJECT should include object_id")
        .to_owned();
    let hello = signed_hello_message(&signing_key, "node:alpha", "node:alpha");
    let manifest = signed_manifest_message_with_heads(
        &signing_key,
        "node:alpha",
        "node:alpha",
        json!({
            "doc:test": [revision_id.clone()]
        }),
    );
    let root_want = signed_want_message(&signing_key, "node:alpha", &[revision_id.as_str()]);
    let follow_on_want = signed_want_message(&signing_key, "node:alpha", &[patch_id.as_str()]);

    session
        .verify_incoming(&hello)
        .expect("HELLO should verify");
    session
        .verify_incoming(&manifest)
        .expect("MANIFEST should verify");
    session
        .verify_incoming(&root_want)
        .expect("root WANT should verify");
    let envelope = session
        .verify_incoming(&revision_object)
        .expect("reachable revision OBJECT should verify");

    assert_eq!(envelope.message_type(), WireMessageType::Object);
    assert!(session
        .peer_session("node:alpha")
        .is_some_and(|state| state.reachable_object_ids.contains(&patch_id)));

    session
        .verify_incoming(&follow_on_want)
        .expect("follow-on patch WANT should verify");
    let patch_envelope = session
        .verify_incoming(&patch_object)
        .expect("reachable patch OBJECT should verify");

    assert_eq!(patch_envelope.message_type(), WireMessageType::Object);
    assert_eq!(
        session
            .peer_session("node:alpha")
            .map(|state| state.pending_object_ids.len()),
        Some(0)
    );
    assert!(session
        .peer_session("node:alpha")
        .is_some_and(|state| state.accepted_sync_roots.contains(&revision_id)));
}

#[test]
fn wire_session_expands_reachability_from_known_object_index() {
    let signing_key = signing_key();
    let sender_key = sender_public_key(&signing_key);
    let mut session = WireSession::default();
    session
        .register_known_peer("node:alpha", &sender_key)
        .expect("known peer should register");
    let base_revision_object = signed_revision_object_message(&signing_key, "node:alpha", &[], &[]);
    let base_revision_id = base_revision_object["payload"]["object_id"]
        .as_str()
        .expect("signed base revision OBJECT should include object_id")
        .to_owned();
    let patch_object = signed_patch_object_message(&signing_key, "node:alpha", &base_revision_id);
    let patch_id = patch_object["payload"]["object_id"]
        .as_str()
        .expect("signed patch OBJECT should include object_id")
        .to_owned();
    let root_revision_object = signed_revision_object_message(
        &signing_key,
        "node:alpha",
        &[base_revision_id.as_str()],
        &[patch_id.as_str()],
    );
    let root_revision_id = root_revision_object["payload"]["object_id"]
        .as_str()
        .expect("signed root revision OBJECT should include object_id")
        .to_owned();
    session.set_known_verified_object_index(std::collections::BTreeMap::from([
        (
            root_revision_id.clone(),
            root_revision_object["payload"]["body"].clone(),
        ),
        (patch_id.clone(), patch_object["payload"]["body"].clone()),
        (
            base_revision_id.clone(),
            base_revision_object["payload"]["body"].clone(),
        ),
    ]));

    let hello = signed_hello_message(&signing_key, "node:alpha", "node:alpha");
    let manifest = signed_manifest_message_with_heads(
        &signing_key,
        "node:alpha",
        "node:alpha",
        json!({
            "doc:test": [root_revision_id.clone()]
        }),
    );
    let root_want = signed_want_message(&signing_key, "node:alpha", &[root_revision_id.as_str()]);
    let follow_on_want = signed_want_message(
        &signing_key,
        "node:alpha",
        &[patch_id.as_str(), base_revision_id.as_str()],
    );

    session
        .verify_incoming(&hello)
        .expect("HELLO should verify");
    session
        .verify_incoming(&manifest)
        .expect("MANIFEST should verify");
    session
        .verify_incoming(&root_want)
        .expect("root WANT should verify");

    assert!(session.peer_session("node:alpha").is_some_and(|state| {
        state.reachable_object_ids.contains(&patch_id)
            && state.reachable_object_ids.contains(&base_revision_id)
    }));

    session
        .verify_incoming(&follow_on_want)
        .expect("known-index-expanded WANT should verify");
}

#[test]
fn wire_session_loads_known_verified_object_index_from_store() {
    let store_root = temp_dir("known-index");
    let signing_key = signing_key();
    let sender_key = sender_public_key(&signing_key);
    let base_revision_object = signed_revision_object_message(&signing_key, "node:alpha", &[], &[]);
    let base_revision_id = base_revision_object["payload"]["object_id"]
        .as_str()
        .expect("signed base revision OBJECT should include object_id")
        .to_owned();
    let patch_object = signed_patch_object_message(&signing_key, "node:alpha", &base_revision_id);
    let patch_id = patch_object["payload"]["object_id"]
        .as_str()
        .expect("signed patch OBJECT should include object_id")
        .to_owned();
    let root_revision_object = signed_revision_object_message(
        &signing_key,
        "node:alpha",
        &[base_revision_id.as_str()],
        &[patch_id.as_str()],
    );
    let root_revision_id = root_revision_object["payload"]["object_id"]
        .as_str()
        .expect("signed root revision OBJECT should include object_id")
        .to_owned();

    write_object_value_to_store(&store_root, &base_revision_object["payload"]["body"])
        .expect("base revision should write to store");
    write_object_value_to_store(&store_root, &patch_object["payload"]["body"])
        .expect("patch should write to store");
    write_object_value_to_store(&store_root, &root_revision_object["payload"]["body"])
        .expect("root revision should write to store");

    let mut known_peers = WirePeerDirectory::new();
    known_peers
        .register_known_peer("node:alpha", &sender_key)
        .expect("known peer should register");
    let mut session = WireSession::from_store_root(known_peers, &store_root)
        .expect("session should bootstrap from store root");

    let hello = signed_hello_message(&signing_key, "node:alpha", "node:alpha");
    let manifest = signed_manifest_message_with_heads(
        &signing_key,
        "node:alpha",
        "node:alpha",
        json!({
            "doc:test": [root_revision_id.clone()]
        }),
    );
    let root_want = signed_want_message(&signing_key, "node:alpha", &[root_revision_id.as_str()]);
    let follow_on_want = signed_want_message(
        &signing_key,
        "node:alpha",
        &[patch_id.as_str(), base_revision_id.as_str()],
    );

    session
        .verify_incoming(&hello)
        .expect("HELLO should verify");
    session
        .verify_incoming(&manifest)
        .expect("MANIFEST should verify");
    session
        .verify_incoming(&root_want)
        .expect("root WANT should verify");
    session
        .verify_incoming(&follow_on_want)
        .expect("store-backed reachable WANT should verify");

    let _ = fs::remove_dir_all(store_root);
}

#[test]
fn wire_session_rejects_unrequested_object() {
    let signing_key = signing_key();
    let sender_key = sender_public_key(&signing_key);
    let mut session = WireSession::default();
    session
        .register_known_peer("node:alpha", &sender_key)
        .expect("known peer should register");
    let hello = signed_hello_message(&signing_key, "node:alpha", "node:alpha");
    let manifest = signed_manifest_message(&signing_key, "node:alpha", "node:alpha");
    let object = signed_object_message(&signing_key, "node:alpha");
    let object_id = object["payload"]["object_id"]
        .as_str()
        .expect("signed OBJECT payload should include object_id")
        .to_owned();

    session
        .verify_incoming(&hello)
        .expect("HELLO should verify");
    session
        .verify_incoming(&manifest)
        .expect("MANIFEST should verify");
    let error = session.verify_incoming(&object).unwrap_err();

    assert_eq!(
        error,
        format!("wire OBJECT '{object_id}' was not requested from 'node:alpha'")
    );
}

#[test]
fn wire_session_rejects_messages_after_bye() {
    let signing_key = signing_key();
    let sender_key = sender_public_key(&signing_key);
    let mut session = WireSession::default();
    session
        .register_known_peer("node:alpha", &sender_key)
        .expect("known peer should register");
    let hello = signed_hello_message(&signing_key, "node:alpha", "node:alpha");
    let bye = signed_bye_message(&signing_key, "node:alpha");
    let want = signed_want_message(&signing_key, "node:alpha", &["patch:test"]);

    session
        .verify_incoming(&hello)
        .expect("HELLO should verify");
    session.verify_incoming(&bye).expect("BYE should verify");
    let error = session.verify_incoming(&want).unwrap_err();

    assert_eq!(error, "wire session for 'node:alpha' is already closed");
}

#[test]
fn wire_session_rejects_duplicate_hello() {
    let signing_key = signing_key();
    let sender_key = sender_public_key(&signing_key);
    let mut session = WireSession::default();
    session
        .register_known_peer("node:alpha", &sender_key)
        .expect("known peer should register");
    let hello = signed_hello_message(&signing_key, "node:alpha", "node:alpha");

    session
        .verify_incoming(&hello)
        .expect("first HELLO should verify");
    let error = session.verify_incoming(&hello).unwrap_err();

    assert_eq!(
        error,
        "wire session already received HELLO from 'node:alpha'"
    );
}

#[test]
fn wire_session_accepts_error_before_hello() {
    let signing_key = signing_key();
    let sender_key = sender_public_key(&signing_key);
    let mut session = WireSession::default();
    session
        .register_known_peer("node:alpha", &sender_key)
        .expect("known peer should register");
    let error_msg = signed_error_message(&signing_key, "node:alpha", "msg:some-prior-msg");

    // ERROR must be accepted even before HELLO has been received,
    // because it carries no sequencing restriction.
    session
        .verify_incoming(&error_msg)
        .expect("ERROR should be accepted before HELLO");

    let state = session
        .peer_session("node:alpha")
        .expect("peer session should exist");
    assert!(
        !state.hello_received(),
        "hello_received must remain false after an ERROR-only exchange"
    );
}
