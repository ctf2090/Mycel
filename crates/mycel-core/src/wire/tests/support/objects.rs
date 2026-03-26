use super::*;

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

pub(crate) fn valid_object_payload_for_proptests() -> Value {
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

pub(crate) fn signed_object_message(signing_key: &SigningKey, sender: &str) -> Value {
    signed_patch_object_message(signing_key, sender, "rev:genesis-null")
}

pub(crate) fn signed_patch_object_message(
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

pub(crate) fn signed_revision_object_message(
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
