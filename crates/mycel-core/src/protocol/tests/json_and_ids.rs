use super::*;

#[test]
fn parse_json_value_strict_rejects_duplicate_top_level_keys() {
    let error = parse_json_value_strict(r#"{"type":"document","doc_id":"doc:a","doc_id":"doc:b"}"#)
        .unwrap_err();

    assert!(error.contains("duplicate object key 'doc_id'"));
}

#[test]
fn parse_json_value_strict_rejects_duplicate_nested_keys() {
    let error = parse_json_value_strict(
        r#"{"type":"snapshot","documents":{"doc:a":"rev:a","doc:a":"rev:b"}}"#,
    )
    .unwrap_err();

    assert!(error.contains("duplicate object key 'doc:a'"));
}

#[test]
fn ensure_supported_json_values_rejects_null_with_path() {
    let error = ensure_supported_json_values(&json!({
        "type": "document",
        "title": null
    }))
    .unwrap_err();

    assert_eq!(error, "$.title: null is not allowed");
}

#[test]
fn parse_json_strict_rejects_floating_point_numbers() {
    let error = parse_json_strict::<Value>(r#"{"timeout":1.5}"#).unwrap_err();

    assert_eq!(
        error,
        "$.timeout: floating-point numbers are not allowed in canonical objects"
    );
}

#[test]
fn canonical_json_is_sorted_and_compact() {
    let canonical = canonical_json(&json!({
        "z": 2,
        "a": [true, {"b": "x", "a": 1}]
    }))
    .expect("canonical JSON should render");

    assert_eq!(canonical, "{\"a\":[true,{\"a\":1,\"b\":\"x\"}],\"z\":2}");
}

#[test]
fn canonical_object_json_excluding_fields_omits_requested_top_level_keys() {
    let canonical = canonical_object_json_excluding_fields(
        &json!({
            "type": "patch",
            "patch_id": "patch:declared",
            "doc_id": "doc:test",
            "nested": {
                "signature": "keep-nested"
            },
            "signature": "sig:omit-me"
        }),
        &["patch_id", "signature"],
    )
    .expect("canonical object JSON should render");

    assert_eq!(
        canonical,
        "{\"doc_id\":\"doc:test\",\"nested\":{\"signature\":\"keep-nested\"},\"type\":\"patch\"}"
    );
}

#[test]
fn canonical_object_json_excluding_fields_requires_object_input() {
    let error = canonical_object_json_excluding_fields(&json!(["not-an-object"]), &["signature"])
        .unwrap_err();

    assert_eq!(error, "top-level JSON value must be an object");
}

#[test]
fn canonical_json_round_trips_through_strict_parse() {
    let value = json!({
        "type": "revision",
        "version": "mycel/0.1",
        "doc_id": "doc:test",
        "parents": ["rev:base", "rev:side"],
        "patches": ["patch:test"],
        "merge_strategy": "semantic-block-merge",
        "state_hash": "hash:test",
        "author": "pk:ed25519:test",
        "timestamp": 7u64
    });
    let canonical = canonical_json(&value).expect("canonical JSON should render");
    let reparsed = parse_json_strict::<Value>(&canonical).expect("canonical JSON should parse");
    let canonical_after_reparse =
        canonical_json(&reparsed).expect("reparsed canonical JSON should render");

    assert_eq!(reparsed, value);
    assert_eq!(canonical_after_reparse, canonical);
}

#[test]
fn canonical_sha256_hex_is_reproducible_across_object_key_order() {
    let left = json!({
        "z": 2,
        "a": [true, {"b": "x", "a": 1}]
    });
    let right = json!({
        "a": [true, {"a": 1, "b": "x"}],
        "z": 2
    });

    let left_hash = canonical_sha256_hex(&left).expect("left hash should compute");
    let right_hash = canonical_sha256_hex(&right).expect("right hash should compute");

    assert_eq!(left_hash, right_hash);
}

#[test]
fn prefixed_canonical_hash_adds_requested_prefix() {
    let value = json!({
        "doc_id": "doc:test",
        "blocks": [],
        "metadata": {}
    });

    let prefixed = prefixed_canonical_hash(&value, "hash").expect("hash should compute");

    assert!(prefixed.starts_with("hash:"));
    assert_eq!(
        prefixed,
        format!(
            "hash:{}",
            canonical_sha256_hex(&value).expect("digest should compute")
        )
    );
}

#[test]
fn prefixed_canonical_object_hash_excluding_fields_matches_manual_reduction() {
    let value = json!({
        "type": "patch",
        "patch_id": "patch:declared",
        "doc_id": "doc:test",
        "base_revision": "rev:genesis-null",
        "author": "pk:ed25519:test",
        "timestamp": 1u64,
        "ops": [],
        "signature": "sig:ed25519:test"
    });

    let helper_hash = prefixed_canonical_object_hash_excluding_fields(
        &value,
        "patch",
        &["patch_id", "signature"],
    )
    .expect("helper hash should compute");
    let manual_hash = prefixed_canonical_hash(
        &json!({
            "type": "patch",
            "doc_id": "doc:test",
            "base_revision": "rev:genesis-null",
            "author": "pk:ed25519:test",
            "timestamp": 1u64,
            "ops": []
        }),
        "patch",
    )
    .expect("manual hash should compute");

    assert_eq!(helper_hash, manual_hash);
}

#[test]
fn recompute_object_id_omits_signature_and_derived_id_field() {
    let value = json!({
        "type": "patch",
        "version": "mycel/0.1",
        "patch_id": "patch:declared",
        "doc_id": "doc:test",
        "base_revision": "rev:genesis-null",
        "author": "pk:ed25519:test",
        "timestamp": 1u64,
        "ops": [],
        "signature": "sig:ed25519:test"
    });

    let recomputed =
        recompute_object_id(&value, "patch_id", "patch").expect("patch ID should recompute");
    assert!(recomputed.starts_with("patch:"));
    assert_ne!(recomputed, "patch:declared");
}

#[test]
fn recompute_object_id_is_reproducible_across_object_key_order() {
    let left = json!({
        "type": "patch",
        "version": "mycel/0.1",
        "doc_id": "doc:test",
        "base_revision": "rev:genesis-null",
        "author": "pk:ed25519:test",
        "timestamp": 1u64,
        "ops": []
    });
    let right = json!({
        "timestamp": 1u64,
        "ops": [],
        "author": "pk:ed25519:test",
        "base_revision": "rev:genesis-null",
        "doc_id": "doc:test",
        "version": "mycel/0.1",
        "type": "patch"
    });

    let left_id =
        recompute_object_id(&left, "patch_id", "patch").expect("left patch ID should compute");
    let right_id =
        recompute_object_id(&right, "patch_id", "patch").expect("right patch ID should compute");

    assert_eq!(left_id, right_id);
}

#[test]
fn signed_payload_bytes_are_reproducible_across_object_key_order() {
    let left = json!({
        "type": "patch",
        "version": "mycel/0.1",
        "patch_id": "patch:test",
        "doc_id": "doc:test",
        "base_revision": "rev:genesis-null",
        "author": "pk:ed25519:test",
        "timestamp": 1u64,
        "ops": [],
        "signature": "sig:placeholder-left"
    });
    let right = json!({
        "signature": "sig:placeholder-right",
        "ops": [],
        "timestamp": 1u64,
        "author": "pk:ed25519:test",
        "base_revision": "rev:genesis-null",
        "doc_id": "doc:test",
        "patch_id": "patch:test",
        "version": "mycel/0.1",
        "type": "patch"
    });

    let left_payload = signed_payload_bytes(&left).expect("left payload should canonicalize");
    let right_payload = signed_payload_bytes(&right).expect("right payload should canonicalize");

    assert_eq!(left_payload, right_payload);
}

#[test]
fn wire_envelope_signed_payload_bytes_are_reproducible_across_key_order() {
    let left = json!({
        "type": "HELLO",
        "version": "mycel-wire/0.1",
        "msg_id": "msg:hello-001",
        "timestamp": "2026-03-08T20:00:00+08:00",
        "from": "node:alpha",
        "payload": {
            "node_id": "node:alpha",
            "agent": "mycel-node/0.1"
        },
        "sig": "sig:left"
    });
    let right = json!({
        "sig": "sig:right",
        "payload": {
            "agent": "mycel-node/0.1",
            "node_id": "node:alpha"
        },
        "from": "node:alpha",
        "timestamp": "2026-03-08T20:00:00+08:00",
        "msg_id": "msg:hello-001",
        "version": "mycel-wire/0.1",
        "type": "HELLO"
    });

    let left_payload = wire_envelope_signed_payload_bytes(&left)
        .expect("left wire envelope payload should canonicalize");
    let right_payload = wire_envelope_signed_payload_bytes(&right)
        .expect("right wire envelope payload should canonicalize");

    assert_eq!(left_payload, right_payload);
}
