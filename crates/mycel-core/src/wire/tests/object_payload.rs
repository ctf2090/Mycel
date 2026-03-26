use serde_json::json;

use super::*;

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
