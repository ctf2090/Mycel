use serde_json::{json, Value};

use super::*;
use crate::protocol::recompute_declared_object_identity;

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
