use std::fs;
use std::path::Path;

use base64::Engine;
use ed25519_dalek::{Signer, SigningKey};
use serde_json::{json, Value};

use mycel_core::canonical::{signed_payload_bytes, wire_envelope_signed_payload_bytes};
use mycel_core::protocol::recompute_object_id;
use mycel_core::replay::{compute_state_hash, DocumentState};
use mycel_core::store::write_object_value_to_store;

mod common;

use common::{
    assert_json_status, assert_stderr_contains, assert_success, create_temp_dir, run_mycel,
    stdout_text,
};

fn path_arg(path: &Path) -> String {
    path.to_string_lossy().into_owned()
}

fn signing_key() -> SigningKey {
    SigningKey::from_bytes(&[11u8; 32])
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

fn signed_hello_message(signing_key: &SigningKey, sender: &str) -> Value {
    signed_hello_message_with_capabilities(signing_key, sender, json!(["patch-sync"]))
}

fn signed_hello_message_with_capabilities(
    signing_key: &SigningKey,
    sender: &str,
    capabilities: Value,
) -> Value {
    let mut value = json!({
        "type": "HELLO",
        "version": "mycel-wire/0.1",
        "msg_id": "msg:hello-cli-sync-001",
        "timestamp": "2026-03-08T20:00:00+08:00",
        "from": sender,
        "payload": {
            "node_id": sender,
            "capabilities": capabilities,
            "nonce": "n:cli-sync"
        },
        "sig": "sig:placeholder"
    });
    value["sig"] = Value::String(sign_wire_value(signing_key, &value));
    value
}

fn signed_hello_message_with_node_id(
    signing_key: &SigningKey,
    sender: &str,
    payload_node_id: &str,
) -> Value {
    let mut value = signed_hello_message(signing_key, sender);
    value["payload"]["node_id"] = Value::String(payload_node_id.to_string());
    value["sig"] = Value::String(sign_wire_value(signing_key, &value));
    value
}

fn signed_manifest_message(signing_key: &SigningKey, sender: &str, revision_id: &str) -> Value {
    signed_manifest_message_with_capabilities(
        signing_key,
        sender,
        revision_id,
        json!(["patch-sync"]),
    )
}

fn signed_manifest_message_with_capabilities(
    signing_key: &SigningKey,
    sender: &str,
    revision_id: &str,
    capabilities: Value,
) -> Value {
    let mut value = json!({
        "type": "MANIFEST",
        "version": "mycel-wire/0.1",
        "msg_id": "msg:manifest-cli-sync-001",
        "timestamp": "2026-03-08T20:00:10+08:00",
        "from": sender,
        "payload": {
            "node_id": sender,
            "capabilities": capabilities,
            "heads": {
                "doc:test": [revision_id]
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
        "msg_id": "msg:snapshot-offer-cli-sync-001",
        "timestamp": "2026-03-08T20:00:30+08:00",
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
        "msg_id": "msg:view-announce-cli-sync-001",
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

fn signed_heads_message(
    signing_key: &SigningKey,
    sender: &str,
    revision_id: &str,
    replace: bool,
) -> Value {
    let mut value = json!({
        "type": "HEADS",
        "version": "mycel-wire/0.1",
        "msg_id": "msg:heads-cli-sync-001",
        "timestamp": "2026-03-08T20:00:20+08:00",
        "from": sender,
        "payload": {
            "documents": {
                "doc:test": [revision_id]
            },
            "replace": replace
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
        "msg_id": "msg:want-cli-sync-001",
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
        "msg_id": "msg:object-cli-sync-patch-001",
        "timestamp": "2026-03-08T20:01:10+08:00",
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
            "timestamp": 2u64,
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
        "msg_id": "msg:object-cli-sync-rev-001",
        "timestamp": "2026-03-08T20:01:12+08:00",
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

fn signed_snapshot_object_message(
    signing_key: &SigningKey,
    sender: &str,
    revision_id: &str,
) -> Value {
    let body = sign_object_value(
        signing_key,
        json!({
            "type": "snapshot",
            "version": "mycel/0.1",
            "snapshot_id": "snap:placeholder",
            "documents": {
                "doc:test": revision_id
            },
            "included_objects": [revision_id],
            "root_hash": "hash:snapshot-root",
            "created_by": "pk:ed25519:placeholder",
            "timestamp": 3u64,
            "signature": "sig:placeholder"
        }),
        "created_by",
        "snapshot_id",
        "snap",
    );
    let object_id = body["snapshot_id"]
        .as_str()
        .expect("signed snapshot body should include snapshot_id")
        .to_owned();
    let object_hash = object_id
        .split_once(':')
        .map(|(_, hash)| hash.to_string())
        .expect("wire snapshot ID should contain hash");

    let mut value = json!({
        "type": "OBJECT",
        "version": "mycel-wire/0.1",
        "msg_id": "msg:object-cli-sync-snapshot-001",
        "timestamp": "2026-03-08T20:01:14+08:00",
        "from": sender,
        "payload": {
            "object_id": object_id,
            "object_type": "snapshot",
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

fn signed_view_object_message(signing_key: &SigningKey, sender: &str, revision_id: &str) -> Value {
    let body = sign_object_value(
        signing_key,
        json!({
            "type": "view",
            "version": "mycel/0.1",
            "view_id": "view:placeholder",
            "maintainer": "pk:ed25519:placeholder",
            "documents": {
                "doc:test": revision_id
            },
            "policy": {
                "accept_keys": [sender_public_key(signing_key)],
                "merge_rule": "manual-reviewed",
                "preferred_branches": ["main"]
            },
            "timestamp": 4u64,
            "signature": "sig:placeholder"
        }),
        "maintainer",
        "view_id",
        "view",
    );
    let object_id = body["view_id"]
        .as_str()
        .expect("signed view body should include view_id")
        .to_owned();
    let object_hash = object_id
        .split_once(':')
        .map(|(_, hash)| hash.to_string())
        .expect("wire view ID should contain hash");

    let mut value = json!({
        "type": "OBJECT",
        "version": "mycel-wire/0.1",
        "msg_id": "msg:object-cli-sync-view-001",
        "timestamp": "2026-03-08T20:01:16+08:00",
        "from": sender,
        "payload": {
            "object_id": object_id,
            "object_type": "view",
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

fn signed_bye_message(signing_key: &SigningKey, sender: &str) -> Value {
    let mut value = json!({
        "type": "BYE",
        "version": "mycel-wire/0.1",
        "msg_id": "msg:bye-cli-sync-001",
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

fn signed_error_message(signing_key: &SigningKey, sender: &str, in_reply_to: &str) -> Value {
    let mut value = json!({
        "type": "ERROR",
        "version": "mycel-wire/0.1",
        "msg_id": "msg:error-cli-sync-001",
        "timestamp": "2026-03-08T20:02:10+08:00",
        "from": sender,
        "payload": {
            "in_reply_to": in_reply_to,
            "code": "ERR_UNKNOWN",
            "detail": "simulated error"
        },
        "sig": "sig:placeholder"
    });
    value["sig"] = Value::String(sign_wire_value(signing_key, &value));
    value
}

fn write_transcript(path: &Path, transcript: &Value) {
    fs::write(
        path,
        serde_json::to_string_pretty(transcript).expect("transcript should serialize"),
    )
    .expect("transcript should write");
}

fn write_signing_key(path: &Path, signing_key: &SigningKey) {
    fs::write(
        path,
        base64::engine::general_purpose::STANDARD.encode(signing_key.to_bytes()),
    )
    .expect("signing key should write");
}

#[test]
fn sync_pull_json_replays_verified_wire_transcript_into_store() {
    let signing_key = signing_key();
    let sender = "node:alpha";
    let patch_object = signed_patch_object_message(&signing_key, sender, "rev:genesis-null");
    let patch_id = patch_object["payload"]["object_id"]
        .as_str()
        .expect("patch object id should exist")
        .to_string();
    let revision_object = signed_revision_object_message(&signing_key, sender, &[], &[&patch_id]);
    let revision_id = revision_object["payload"]["object_id"]
        .as_str()
        .expect("revision object id should exist")
        .to_string();
    let transcript_dir = create_temp_dir("sync-pull-source");
    let transcript_path = transcript_dir.path().join("pull-transcript.json");
    let store_root = create_temp_dir("sync-pull-store");
    write_transcript(
        &transcript_path,
        &json!({
            "peer": {
                "node_id": sender,
                "public_key": sender_public_key(&signing_key)
            },
            "messages": [
                signed_hello_message(&signing_key, sender),
                signed_manifest_message(&signing_key, sender, &revision_id),
                signed_want_message(&signing_key, sender, &[&revision_id]),
                revision_object,
                signed_want_message(&signing_key, sender, &[&patch_id]),
                patch_object,
                signed_bye_message(&signing_key, sender)
            ]
        }),
    );

    let output = run_mycel(&[
        "sync",
        "pull",
        &path_arg(&transcript_path),
        "--into",
        &path_arg(store_root.path()),
        "--json",
    ]);

    assert_success(&output);
    let json = assert_json_status(&output, "ok");
    assert_eq!(json["peer_node_id"], sender);
    assert_eq!(json["message_count"], 7);
    assert_eq!(json["verified_message_count"], 7);
    assert_eq!(json["object_message_count"], 2);
    assert_eq!(json["verified_object_count"], 2);
    assert_eq!(json["written_object_count"], 2);
    assert_eq!(json["existing_object_count"], 0);
    assert_eq!(json["stored_objects"].as_array().map(Vec::len), Some(2));
    assert!(
        json["index_manifest_path"]
            .as_str()
            .is_some_and(|path| path.ends_with("/indexes/manifest.json")),
        "expected manifest path, stdout: {}",
        stdout_text(&output)
    );

    let manifest_path = store_root.path().join("indexes").join("manifest.json");
    let manifest: Value =
        serde_json::from_str(&fs::read_to_string(&manifest_path).expect("manifest should read"))
            .expect("manifest should parse");
    assert_eq!(manifest["stored_object_count"], 2);
    assert_eq!(
        manifest["doc_revisions"]["doc:test"]
            .as_array()
            .map(Vec::len),
        Some(1),
        "expected synced revision to be indexed"
    );
}

#[test]
fn sync_pull_json_replays_first_time_heads_transcript_into_store() {
    let signing_key = signing_key();
    let sender = "node:alpha";
    let patch_object = signed_patch_object_message(&signing_key, sender, "rev:genesis-null");
    let patch_id = patch_object["payload"]["object_id"]
        .as_str()
        .expect("patch object id should exist")
        .to_string();
    let revision_object = signed_revision_object_message(&signing_key, sender, &[], &[&patch_id]);
    let revision_id = revision_object["payload"]["object_id"]
        .as_str()
        .expect("revision object id should exist")
        .to_string();
    let transcript_dir = create_temp_dir("sync-pull-heads-source");
    let transcript_path = transcript_dir.path().join("pull-heads-transcript.json");
    let store_root = create_temp_dir("sync-pull-heads-store");
    write_transcript(
        &transcript_path,
        &json!({
            "peer": {
                "node_id": sender,
                "public_key": sender_public_key(&signing_key)
            },
            "messages": [
                signed_hello_message(&signing_key, sender),
                signed_heads_message(&signing_key, sender, &revision_id, true),
                signed_want_message(&signing_key, sender, &[&revision_id]),
                revision_object,
                signed_want_message(&signing_key, sender, &[&patch_id]),
                patch_object,
                signed_bye_message(&signing_key, sender)
            ]
        }),
    );

    let output = run_mycel(&[
        "sync",
        "pull",
        &path_arg(&transcript_path),
        "--into",
        &path_arg(store_root.path()),
        "--json",
    ]);

    assert_success(&output);
    let json = assert_json_status(&output, "ok");
    assert_eq!(json["peer_node_id"], sender);
    assert_eq!(json["message_count"], 7);
    assert_eq!(json["verified_message_count"], 7);
    assert_eq!(json["object_message_count"], 2);
    assert_eq!(json["verified_object_count"], 2);
    assert_eq!(json["written_object_count"], 2);
    assert_eq!(json["existing_object_count"], 0);
    assert!(
        json["notes"]
            .as_array()
            .is_some_and(|notes| notes.is_empty()),
        "expected no first-time sync warnings, stdout: {}",
        stdout_text(&output)
    );

    let manifest_path = store_root.path().join("indexes").join("manifest.json");
    let manifest: Value =
        serde_json::from_str(&fs::read_to_string(&manifest_path).expect("manifest should read"))
            .expect("manifest should parse");
    assert_eq!(manifest["stored_object_count"], 2);
    assert_eq!(
        manifest["doc_revisions"]["doc:test"]
            .as_array()
            .map(Vec::len),
        Some(1),
        "expected synced revision to be indexed"
    );
}

#[test]
fn sync_pull_json_replays_incremental_transcript_into_existing_store() {
    let signing_key = signing_key();
    let sender = "node:alpha";

    let base_patch_object = signed_patch_object_message(&signing_key, sender, "rev:genesis-null");
    let base_patch_id = base_patch_object["payload"]["object_id"]
        .as_str()
        .expect("base patch object id should exist")
        .to_string();
    let base_revision_object =
        signed_revision_object_message(&signing_key, sender, &[], &[&base_patch_id]);
    let base_revision_id = base_revision_object["payload"]["object_id"]
        .as_str()
        .expect("base revision object id should exist")
        .to_string();

    let follow_patch_object = signed_patch_object_message(&signing_key, sender, &base_revision_id);
    let follow_patch_id = follow_patch_object["payload"]["object_id"]
        .as_str()
        .expect("follow patch object id should exist")
        .to_string();
    let follow_revision_object = signed_revision_object_message(
        &signing_key,
        sender,
        &[&base_revision_id],
        &[&follow_patch_id],
    );
    let follow_revision_id = follow_revision_object["payload"]["object_id"]
        .as_str()
        .expect("follow revision object id should exist")
        .to_string();

    let transcript_dir = create_temp_dir("sync-pull-incremental-source");
    let transcript_path = transcript_dir
        .path()
        .join("pull-incremental-transcript.json");
    let store_root = create_temp_dir("sync-pull-incremental-store");
    write_object_value_to_store(store_root.path(), &base_patch_object["payload"]["body"])
        .expect("base patch should write to store");
    write_object_value_to_store(store_root.path(), &base_revision_object["payload"]["body"])
        .expect("base revision should write to store");

    write_transcript(
        &transcript_path,
        &json!({
            "peer": {
                "node_id": sender,
                "public_key": sender_public_key(&signing_key)
            },
            "messages": [
                signed_hello_message(&signing_key, sender),
                signed_manifest_message(&signing_key, sender, &follow_revision_id),
                signed_want_message(&signing_key, sender, &[&follow_revision_id]),
                follow_revision_object,
                signed_want_message(&signing_key, sender, &[&follow_patch_id]),
                follow_patch_object,
                signed_bye_message(&signing_key, sender)
            ]
        }),
    );

    let output = run_mycel(&[
        "sync",
        "pull",
        &path_arg(&transcript_path),
        "--into",
        &path_arg(store_root.path()),
        "--json",
    ]);

    assert_success(&output);
    let json = assert_json_status(&output, "ok");
    assert_eq!(json["peer_node_id"], sender);
    assert_eq!(json["message_count"], 7);
    assert_eq!(json["verified_message_count"], 7);
    assert_eq!(json["object_message_count"], 2);
    assert_eq!(json["verified_object_count"], 2);
    assert_eq!(json["written_object_count"], 2);
    assert_eq!(json["existing_object_count"], 0);
    assert!(
        json["notes"]
            .as_array()
            .is_some_and(|notes| notes.is_empty()),
        "expected no incremental sync warnings, stdout: {}",
        stdout_text(&output)
    );

    let manifest_path = store_root.path().join("indexes").join("manifest.json");
    let manifest: Value =
        serde_json::from_str(&fs::read_to_string(&manifest_path).expect("manifest should read"))
            .expect("manifest should parse");
    assert_eq!(manifest["stored_object_count"], 4);
    let revisions = manifest["doc_revisions"]["doc:test"]
        .as_array()
        .expect("expected synced revision index array");
    assert_eq!(revisions.len(), 2);
    assert!(revisions
        .iter()
        .any(|value| value.as_str() == Some(base_revision_id.as_str())));
    assert!(revisions
        .iter()
        .any(|value| value.as_str() == Some(follow_revision_id.as_str())));
}

#[test]
fn sync_pull_json_reports_missing_bye_as_session_note() {
    let signing_key = signing_key();
    let sender = "node:alpha";
    let patch_object = signed_patch_object_message(&signing_key, sender, "rev:genesis-null");
    let patch_id = patch_object["payload"]["object_id"]
        .as_str()
        .expect("patch object id should exist")
        .to_string();
    let revision_object = signed_revision_object_message(&signing_key, sender, &[], &[&patch_id]);
    let revision_id = revision_object["payload"]["object_id"]
        .as_str()
        .expect("revision object id should exist")
        .to_string();
    let transcript_dir = create_temp_dir("sync-pull-missing-bye");
    let transcript_path = transcript_dir.path().join("missing-bye-transcript.json");
    let store_root = create_temp_dir("sync-pull-missing-bye-store");
    write_transcript(
        &transcript_path,
        &json!({
            "peer": {
                "node_id": sender,
                "public_key": sender_public_key(&signing_key)
            },
            "messages": [
                signed_hello_message(&signing_key, sender),
                signed_manifest_message(&signing_key, sender, &revision_id),
                signed_want_message(&signing_key, sender, &[&revision_id]),
                revision_object,
                signed_want_message(&signing_key, sender, &[&patch_id]),
                patch_object
            ]
        }),
    );

    let output = run_mycel(&[
        "sync",
        "pull",
        &path_arg(&transcript_path),
        "--into",
        &path_arg(store_root.path()),
        "--json",
    ]);

    assert_success(&output);
    let json = assert_json_status(&output, "warning");
    assert_eq!(json["peer_node_id"], sender);
    assert_eq!(json["message_count"], 6);
    assert_eq!(json["verified_message_count"], 6);
    assert_eq!(json["object_message_count"], 2);
    assert_eq!(json["verified_object_count"], 2);
    assert_eq!(json["written_object_count"], 2);
    assert_eq!(json["existing_object_count"], 0);
    assert!(
        json["notes"]
            .as_array()
            .is_some_and(|notes| notes
                .iter()
                .any(|note| note.as_str().is_some_and(|message| message
                    .contains("sync transcript ended without BYE from 'node:alpha'")))),
        "expected missing-BYE session note, stdout: {}",
        stdout_text(&output)
    );

    let manifest_path = store_root.path().join("indexes").join("manifest.json");
    let manifest: Value =
        serde_json::from_str(&fs::read_to_string(&manifest_path).expect("manifest should read"))
            .expect("manifest should parse");
    assert_eq!(manifest["stored_object_count"], 2);
}

#[test]
fn sync_pull_json_accepts_snapshot_offer_when_capability_is_advertised() {
    let signing_key = signing_key();
    let sender = "node:alpha";
    let snapshot_object = signed_snapshot_object_message(&signing_key, sender, "rev:test");
    let snapshot_id = snapshot_object["payload"]["object_id"]
        .as_str()
        .expect("snapshot object id should exist")
        .to_string();
    let transcript_dir = create_temp_dir("sync-pull-snapshot-offer-source");
    let transcript_path = transcript_dir
        .path()
        .join("pull-snapshot-offer-transcript.json");
    let store_root = create_temp_dir("sync-pull-snapshot-offer-store");
    write_transcript(
        &transcript_path,
        &json!({
            "peer": {
                "node_id": sender,
                "public_key": sender_public_key(&signing_key)
            },
            "messages": [
                signed_hello_message_with_capabilities(
                    &signing_key,
                    sender,
                    json!(["patch-sync", "snapshot-sync"])
                ),
                signed_manifest_message_with_capabilities(
                    &signing_key,
                    sender,
                    "rev:test",
                    json!(["patch-sync", "snapshot-sync"])
                ),
                signed_snapshot_offer_message(&signing_key, sender, &snapshot_id),
                signed_want_message(&signing_key, sender, &[&snapshot_id]),
                snapshot_object,
                signed_bye_message(&signing_key, sender)
            ]
        }),
    );

    let output = run_mycel(&[
        "sync",
        "pull",
        &path_arg(&transcript_path),
        "--into",
        &path_arg(store_root.path()),
        "--json",
    ]);

    assert_success(&output);
    let json = assert_json_status(&output, "ok");
    assert_eq!(json["peer_node_id"], sender);
    assert_eq!(json["message_count"], 6);
    assert_eq!(json["verified_message_count"], 6);
    assert_eq!(json["object_message_count"], 1);
    assert_eq!(json["verified_object_count"], 1);
    assert_eq!(json["written_object_count"], 1);

    let manifest_path = store_root.path().join("indexes").join("manifest.json");
    let manifest: Value =
        serde_json::from_str(&fs::read_to_string(&manifest_path).expect("manifest should read"))
            .expect("manifest should parse");
    assert_eq!(manifest["stored_object_count"], 1);
}

#[test]
fn sync_peer_store_json_fetches_offered_snapshots_into_local_store() {
    let signing_key = signing_key();
    let sender = "node:alpha";
    let remote_store = create_temp_dir("sync-peer-store-snapshot-remote");
    let local_store = create_temp_dir("sync-peer-store-snapshot-local");
    let signing_key_path = remote_store.path().join("peer.key");

    let patch_object = signed_patch_object_message(&signing_key, sender, "rev:genesis-null");
    let patch_id = patch_object["payload"]["object_id"]
        .as_str()
        .expect("patch object id should exist")
        .to_string();
    let revision_object = signed_revision_object_message(&signing_key, sender, &[], &[&patch_id]);
    let revision_id = revision_object["payload"]["object_id"]
        .as_str()
        .expect("revision object id should exist")
        .to_string();
    let snapshot_object = signed_snapshot_object_message(&signing_key, sender, &revision_id);
    let snapshot_id = snapshot_object["payload"]["object_id"]
        .as_str()
        .expect("snapshot object id should exist")
        .to_string();

    for body in [
        &patch_object["payload"]["body"],
        &revision_object["payload"]["body"],
        &snapshot_object["payload"]["body"],
    ] {
        write_object_value_to_store(remote_store.path(), body)
            .expect("object should write to remote store");
    }
    write_signing_key(&signing_key_path, &signing_key);

    let output = run_mycel(&[
        "sync",
        "peer-store",
        "--from",
        &path_arg(remote_store.path()),
        "--into",
        &path_arg(local_store.path()),
        "--peer-node-id",
        sender,
        "--signing-key",
        &path_arg(&signing_key_path),
        "--json",
    ]);

    assert_success(&output);
    let json = assert_json_status(&output, "ok");
    assert_eq!(json["peer_node_id"], sender);
    assert_eq!(
        json["object_message_count"], 3,
        "expected revision, patch, and snapshot transfer"
    );
    assert_eq!(json["written_object_count"], 3);

    let manifest_path = local_store.path().join("indexes").join("manifest.json");
    let manifest: Value =
        serde_json::from_str(&fs::read_to_string(&manifest_path).expect("manifest should read"))
            .expect("manifest should parse");
    assert_eq!(manifest["stored_object_count"], 3);
    assert_eq!(manifest["object_ids_by_type"]["snapshot"][0], snapshot_id);
}

#[test]
fn sync_peer_store_json_runs_first_time_sync_into_local_store() {
    let signing_key = signing_key();
    let sender = "node:alpha";
    let remote_store = create_temp_dir("sync-peer-store-remote");
    let local_store = create_temp_dir("sync-peer-store-local");
    let signing_key_path = remote_store.path().join("peer.key");

    let patch_object = signed_patch_object_message(&signing_key, sender, "rev:genesis-null");
    let patch_id = patch_object["payload"]["object_id"]
        .as_str()
        .expect("patch object id should exist")
        .to_string();
    let revision_object = signed_revision_object_message(&signing_key, sender, &[], &[&patch_id]);

    write_object_value_to_store(remote_store.path(), &patch_object["payload"]["body"])
        .expect("patch should write to remote store");
    write_object_value_to_store(remote_store.path(), &revision_object["payload"]["body"])
        .expect("revision should write to remote store");
    write_signing_key(&signing_key_path, &signing_key);

    let output = run_mycel(&[
        "sync",
        "peer-store",
        "--from",
        &path_arg(remote_store.path()),
        "--into",
        &path_arg(local_store.path()),
        "--peer-node-id",
        sender,
        "--signing-key",
        &path_arg(&signing_key_path),
        "--json",
    ]);

    assert_success(&output);
    let json = assert_json_status(&output, "ok");
    assert_eq!(json["peer_node_id"], sender);
    assert_eq!(json["source_store"], path_arg(remote_store.path()));
    assert_eq!(json["message_count"], 7);
    assert_eq!(json["object_message_count"], 2);
    assert_eq!(json["written_object_count"], 2);
    assert_eq!(json["existing_object_count"], 0);

    let manifest_path = local_store.path().join("indexes").join("manifest.json");
    let manifest: Value =
        serde_json::from_str(&fs::read_to_string(&manifest_path).expect("manifest should read"))
            .expect("manifest should parse");
    assert_eq!(manifest["stored_object_count"], 2);
}

#[test]
fn sync_peer_store_json_fetches_announced_views_into_governance_indexes() {
    let signing_key = signing_key();
    let sender = "node:alpha";
    let remote_store = create_temp_dir("sync-peer-store-view-remote");
    let local_store = create_temp_dir("sync-peer-store-view-local");
    let signing_key_path = remote_store.path().join("peer.key");

    let patch_object = signed_patch_object_message(&signing_key, sender, "rev:genesis-null");
    let patch_id = patch_object["payload"]["object_id"]
        .as_str()
        .expect("patch object id should exist")
        .to_string();
    let revision_object = signed_revision_object_message(&signing_key, sender, &[], &[&patch_id]);
    let revision_id = revision_object["payload"]["object_id"]
        .as_str()
        .expect("revision object id should exist")
        .to_string();
    let view_object = signed_view_object_message(&signing_key, sender, &revision_id);
    let view_id = view_object["payload"]["object_id"]
        .as_str()
        .expect("view object id should exist")
        .to_string();

    for body in [
        &patch_object["payload"]["body"],
        &revision_object["payload"]["body"],
        &view_object["payload"]["body"],
    ] {
        write_object_value_to_store(remote_store.path(), body)
            .expect("object should write to remote store");
    }
    write_signing_key(&signing_key_path, &signing_key);

    let output = run_mycel(&[
        "sync",
        "peer-store",
        "--from",
        &path_arg(remote_store.path()),
        "--into",
        &path_arg(local_store.path()),
        "--peer-node-id",
        sender,
        "--signing-key",
        &path_arg(&signing_key_path),
        "--json",
    ]);

    assert_success(&output);
    let json = assert_json_status(&output, "ok");
    assert_eq!(json["peer_node_id"], sender);
    assert_eq!(json["object_message_count"], 3);
    assert_eq!(json["written_object_count"], 3);

    let manifest_path = local_store.path().join("indexes").join("manifest.json");
    let manifest: Value =
        serde_json::from_str(&fs::read_to_string(&manifest_path).expect("manifest should read"))
            .expect("manifest should parse");
    assert_eq!(manifest["stored_object_count"], 3);
    assert_eq!(
        manifest["view_governance"].as_array().map(Vec::len),
        Some(1)
    );
    assert_eq!(manifest["view_governance"][0]["view_id"], view_id);
    assert_eq!(manifest["document_views"]["doc:test"][0], view_id);
}

#[test]
fn sync_peer_store_json_reports_noop_when_local_store_is_current() {
    let signing_key = signing_key();
    let sender = "node:alpha";
    let remote_store = create_temp_dir("sync-peer-store-noop-remote");
    let local_store = create_temp_dir("sync-peer-store-noop-local");
    let signing_key_path = remote_store.path().join("peer.key");

    let patch_object = signed_patch_object_message(&signing_key, sender, "rev:genesis-null");
    let patch_id = patch_object["payload"]["object_id"]
        .as_str()
        .expect("patch object id should exist")
        .to_string();
    let revision_object = signed_revision_object_message(&signing_key, sender, &[], &[&patch_id]);

    for store_root in [remote_store.path(), local_store.path()] {
        write_object_value_to_store(store_root, &patch_object["payload"]["body"])
            .expect("patch should write to store");
        write_object_value_to_store(store_root, &revision_object["payload"]["body"])
            .expect("revision should write to store");
    }
    write_signing_key(&signing_key_path, &signing_key);

    let output = run_mycel(&[
        "sync",
        "peer-store",
        "--from",
        &path_arg(remote_store.path()),
        "--into",
        &path_arg(local_store.path()),
        "--peer-node-id",
        sender,
        "--signing-key",
        &path_arg(&signing_key_path),
        "--json",
    ]);

    assert_success(&output);
    let json = assert_json_status(&output, "ok");
    assert_eq!(json["peer_node_id"], sender);
    assert_eq!(json["object_message_count"], 0);
    assert_eq!(json["written_object_count"], 0);
    assert!(
        json["notes"]
            .as_array()
            .is_some_and(|notes| notes.iter().any(|note| {
                note.as_str()
                    .is_some_and(|value| value.contains("no WANT messages"))
            })),
        "expected no-op note, stdout: {}",
        stdout_text(&output)
    );
}

#[test]
fn sync_pull_text_reports_verification_failure_without_storing_objects() {
    let signing_key = signing_key();
    let sender = "node:alpha";
    let patch_object = signed_patch_object_message(&signing_key, sender, "rev:genesis-null");
    let patch_id = patch_object["payload"]["object_id"]
        .as_str()
        .expect("patch object id should exist")
        .to_string();
    let revision_object = signed_revision_object_message(&signing_key, sender, &[], &[&patch_id]);
    let revision_id = revision_object["payload"]["object_id"]
        .as_str()
        .expect("revision object id should exist")
        .to_string();
    let mut invalid_object = revision_object.clone();
    invalid_object["payload"]["hash"] = Value::String("hash:tampered".to_string());
    let transcript_dir = create_temp_dir("sync-pull-invalid");
    let transcript_path = transcript_dir.path().join("invalid-transcript.json");
    let store_root = create_temp_dir("sync-pull-invalid-store");
    write_transcript(
        &transcript_path,
        &json!({
            "peer": {
                "node_id": sender,
                "public_key": sender_public_key(&signing_key)
            },
            "messages": [
                signed_hello_message(&signing_key, sender),
                signed_manifest_message(&signing_key, sender, &revision_id),
                signed_want_message(&signing_key, sender, &[&revision_id]),
                invalid_object
            ]
        }),
    );

    let output = run_mycel(&[
        "sync",
        "pull",
        &path_arg(&transcript_path),
        "--into",
        &path_arg(store_root.path()),
    ]);

    assert!(
        !output.status.success(),
        "expected failure, stdout: {}, stderr: {}",
        stdout_text(&output),
        String::from_utf8_lossy(&output.stderr)
    );
    assert_stderr_contains(&output, "message 4 failed verification");
    let stdout = stdout_text(&output);
    assert!(stdout.contains("sync pull: failed"), "stdout: {stdout}");
    assert!(stdout.contains("verified messages: 3"), "stdout: {stdout}");
    assert!(!store_root
        .path()
        .join("indexes")
        .join("manifest.json")
        .exists());
}

#[test]
fn sync_pull_json_reports_object_id_mismatch_without_storing_objects() {
    let signing_key = signing_key();
    let sender = "node:alpha";
    let patch_object = signed_patch_object_message(&signing_key, sender, "rev:genesis-null");
    let patch_id = patch_object["payload"]["object_id"]
        .as_str()
        .expect("patch object id should exist")
        .to_string();
    let revision_object = signed_revision_object_message(&signing_key, sender, &[], &[&patch_id]);
    let mut invalid_object = revision_object.clone();
    invalid_object["payload"]["object_id"] = Value::String("rev:mismatch".to_string());
    let transcript_dir = create_temp_dir("sync-pull-object-id-mismatch");
    let transcript_path = transcript_dir
        .path()
        .join("object-id-mismatch-transcript.json");
    let store_root = create_temp_dir("sync-pull-object-id-mismatch-store");
    write_transcript(
        &transcript_path,
        &json!({
            "peer": {
                "node_id": sender,
                "public_key": sender_public_key(&signing_key)
            },
            "messages": [
                signed_hello_message(&signing_key, sender),
                signed_manifest_message(&signing_key, sender, "rev:mismatch"),
                signed_want_message(&signing_key, sender, &["rev:mismatch"]),
                invalid_object
            ]
        }),
    );

    let output = run_mycel(&[
        "sync",
        "pull",
        &path_arg(&transcript_path),
        "--into",
        &path_arg(store_root.path()),
        "--json",
    ]);

    assert!(
        !output.status.success(),
        "expected failure, stdout: {}, stderr: {}",
        stdout_text(&output),
        String::from_utf8_lossy(&output.stderr)
    );
    let json = assert_json_status(&output, "failed");
    assert_eq!(json["verified_message_count"], 3);
    assert_eq!(json["object_message_count"], 0);
    assert_eq!(json["written_object_count"], 0);
    assert!(
        json["errors"]
            .as_array()
            .is_some_and(|errors| errors.iter().any(|error| error
                .as_str()
                .is_some_and(|message| message.contains("OBJECT payload object_id")))),
        "expected object-id mismatch error, stdout: {}",
        stdout_text(&output)
    );
    assert!(!store_root
        .path()
        .join("indexes")
        .join("manifest.json")
        .exists());
}

#[test]
fn sync_pull_json_rejects_messages_after_bye() {
    let signing_key = signing_key();
    let sender = "node:alpha";
    let transcript_dir = create_temp_dir("sync-pull-after-bye");
    let transcript_path = transcript_dir.path().join("after-bye-transcript.json");
    let store_root = create_temp_dir("sync-pull-after-bye-store");
    write_transcript(
        &transcript_path,
        &json!({
            "peer": {
                "node_id": sender,
                "public_key": sender_public_key(&signing_key)
            },
            "messages": [
                signed_hello_message(&signing_key, sender),
                signed_bye_message(&signing_key, sender),
                signed_want_message(&signing_key, sender, &["patch:test"])
            ]
        }),
    );

    let output = run_mycel(&[
        "sync",
        "pull",
        &path_arg(&transcript_path),
        "--into",
        &path_arg(store_root.path()),
        "--json",
    ]);

    assert!(
        !output.status.success(),
        "expected failure, stdout: {}, stderr: {}",
        stdout_text(&output),
        String::from_utf8_lossy(&output.stderr)
    );
    let json = assert_json_status(&output, "failed");
    assert_eq!(json["verified_message_count"], 2);
    assert_eq!(json["object_message_count"], 0);
    assert_eq!(json["written_object_count"], 0);
    assert!(
        json["errors"]
            .as_array()
            .is_some_and(|errors| errors.iter().any(|error| {
                error.as_str().is_some_and(|message| {
                    message.contains("wire session for 'node:alpha' is already closed")
                })
            })),
        "expected already-closed error, stdout: {}",
        stdout_text(&output)
    );
    assert!(!store_root
        .path()
        .join("indexes")
        .join("manifest.json")
        .exists());
}

#[test]
fn sync_pull_json_rejects_duplicate_hello() {
    let signing_key = signing_key();
    let sender = "node:alpha";
    let transcript_dir = create_temp_dir("sync-pull-duplicate-hello");
    let transcript_path = transcript_dir
        .path()
        .join("duplicate-hello-transcript.json");
    let store_root = create_temp_dir("sync-pull-duplicate-hello-store");
    write_transcript(
        &transcript_path,
        &json!({
            "peer": {
                "node_id": sender,
                "public_key": sender_public_key(&signing_key)
            },
            "messages": [
                signed_hello_message(&signing_key, sender),
                signed_hello_message(&signing_key, sender)
            ]
        }),
    );

    let output = run_mycel(&[
        "sync",
        "pull",
        &path_arg(&transcript_path),
        "--into",
        &path_arg(store_root.path()),
        "--json",
    ]);

    assert!(
        !output.status.success(),
        "expected failure, stdout: {}, stderr: {}",
        stdout_text(&output),
        String::from_utf8_lossy(&output.stderr)
    );
    let json = assert_json_status(&output, "failed");
    assert_eq!(json["verified_message_count"], 1);
    assert_eq!(json["object_message_count"], 0);
    assert_eq!(json["written_object_count"], 0);
    assert!(
        json["errors"]
            .as_array()
            .is_some_and(|errors| errors.iter().any(|error| {
                error
                    .as_str()
                    .is_some_and(|message| message.contains("wire session already received HELLO"))
            })),
        "expected duplicate-HELLO error, stdout: {}",
        stdout_text(&output)
    );
    assert!(!store_root
        .path()
        .join("indexes")
        .join("manifest.json")
        .exists());
}

#[test]
fn sync_pull_json_rejects_unknown_sender_hello() {
    let expected_peer = "node:alpha";
    let unexpected_sender = "node:beta";
    let expected_signing_key = signing_key();
    let unexpected_signing_key = SigningKey::from_bytes(&[19u8; 32]);
    let transcript_dir = create_temp_dir("sync-pull-unknown-sender-hello");
    let transcript_path = transcript_dir
        .path()
        .join("unknown-sender-hello-transcript.json");
    let store_root = create_temp_dir("sync-pull-unknown-sender-hello-store");
    write_transcript(
        &transcript_path,
        &json!({
            "peer": {
                "node_id": expected_peer,
                "public_key": sender_public_key(&expected_signing_key)
            },
            "messages": [
                signed_hello_message(&unexpected_signing_key, unexpected_sender)
            ]
        }),
    );

    let output = run_mycel(&[
        "sync",
        "pull",
        &path_arg(&transcript_path),
        "--into",
        &path_arg(store_root.path()),
        "--json",
    ]);

    assert!(
        !output.status.success(),
        "expected failure, stdout: {}, stderr: {}",
        stdout_text(&output),
        String::from_utf8_lossy(&output.stderr)
    );
    let json = assert_json_status(&output, "failed");
    assert_eq!(json["peer_node_id"], expected_peer);
    assert_eq!(json["message_count"], 1);
    assert_eq!(json["verified_message_count"], 0);
    assert_eq!(json["object_message_count"], 0);
    assert_eq!(json["written_object_count"], 0);
    assert!(
        json["errors"]
            .as_array()
            .is_some_and(|errors| errors.iter().any(|error| {
                error
                    .as_str()
                    .is_some_and(|message| message.contains("unknown wire sender 'node:beta'"))
            })),
        "expected unknown-sender error, stdout: {}",
        stdout_text(&output)
    );
    assert!(!store_root
        .path()
        .join("indexes")
        .join("manifest.json")
        .exists());
}

#[test]
fn sync_pull_json_rejects_hello_sender_identity_mismatch() {
    let sender = "node:alpha";
    let mismatched_node_id = "node:beta";
    let signing_key = signing_key();
    let transcript_dir = create_temp_dir("sync-pull-hello-node-id-mismatch");
    let transcript_path = transcript_dir
        .path()
        .join("hello-node-id-mismatch-transcript.json");
    let store_root = create_temp_dir("sync-pull-hello-node-id-mismatch-store");
    write_transcript(
        &transcript_path,
        &json!({
            "peer": {
                "node_id": sender,
                "public_key": sender_public_key(&signing_key)
            },
            "messages": [
                signed_hello_message_with_node_id(&signing_key, sender, mismatched_node_id)
            ]
        }),
    );

    let output = run_mycel(&[
        "sync",
        "pull",
        &path_arg(&transcript_path),
        "--into",
        &path_arg(store_root.path()),
        "--json",
    ]);

    assert!(
        !output.status.success(),
        "expected failure, stdout: {}, stderr: {}",
        stdout_text(&output),
        String::from_utf8_lossy(&output.stderr)
    );
    let json = assert_json_status(&output, "failed");
    assert_eq!(json["peer_node_id"], sender);
    assert_eq!(json["message_count"], 1);
    assert_eq!(json["verified_message_count"], 0);
    assert_eq!(json["object_message_count"], 0);
    assert_eq!(json["written_object_count"], 0);
    assert!(
        json["errors"]
            .as_array()
            .is_some_and(|errors| errors.iter().any(|error| {
                error.as_str().is_some_and(|message| {
                    message.contains("wire HELLO payload 'node_id' must equal envelope 'from'")
                })
            })),
        "expected HELLO sender-identity mismatch error, stdout: {}",
        stdout_text(&output)
    );
    assert!(!store_root
        .path()
        .join("indexes")
        .join("manifest.json")
        .exists());
}

#[test]
fn sync_pull_json_rejects_want_before_manifest_or_heads() {
    let signing_key = signing_key();
    let sender = "node:alpha";
    let transcript_dir = create_temp_dir("sync-pull-want-before-manifest");
    let transcript_path = transcript_dir
        .path()
        .join("want-before-manifest-transcript.json");
    let store_root = create_temp_dir("sync-pull-want-before-manifest-store");
    write_transcript(
        &transcript_path,
        &json!({
            "peer": {
                "node_id": sender,
                "public_key": sender_public_key(&signing_key)
            },
            "messages": [
                signed_hello_message(&signing_key, sender),
                signed_want_message(&signing_key, sender, &["patch:test"])
            ]
        }),
    );

    let output = run_mycel(&[
        "sync",
        "pull",
        &path_arg(&transcript_path),
        "--into",
        &path_arg(store_root.path()),
        "--json",
    ]);

    assert!(
        !output.status.success(),
        "expected failure, stdout: {}, stderr: {}",
        stdout_text(&output),
        String::from_utf8_lossy(&output.stderr)
    );
    let json = assert_json_status(&output, "failed");
    assert_eq!(json["verified_message_count"], 1);
    assert_eq!(json["object_message_count"], 0);
    assert_eq!(json["written_object_count"], 0);
    assert!(
        json["errors"]
            .as_array()
            .is_some_and(|errors| errors.iter().any(|error| {
                error.as_str().is_some_and(|message| {
                    message.contains("wire WANT requires prior MANIFEST or HEADS")
                })
            })),
        "expected WANT-before-head-context error, stdout: {}",
        stdout_text(&output)
    );
    assert!(!store_root
        .path()
        .join("indexes")
        .join("manifest.json")
        .exists());
}

#[test]
fn sync_pull_json_rejects_unreachable_want_revision() {
    let signing_key = signing_key();
    let sender = "node:alpha";
    let transcript_dir = create_temp_dir("sync-pull-unreachable-want-revision");
    let transcript_path = transcript_dir
        .path()
        .join("unreachable-want-revision-transcript.json");
    let store_root = create_temp_dir("sync-pull-unreachable-want-revision-store");
    write_transcript(
        &transcript_path,
        &json!({
            "peer": {
                "node_id": sender,
                "public_key": sender_public_key(&signing_key)
            },
            "messages": [
                signed_hello_message(&signing_key, sender),
                signed_manifest_message(&signing_key, sender, "rev:test"),
                signed_want_message(&signing_key, sender, &["rev:missing"])
            ]
        }),
    );

    let output = run_mycel(&[
        "sync",
        "pull",
        &path_arg(&transcript_path),
        "--into",
        &path_arg(store_root.path()),
        "--json",
    ]);

    assert!(
        !output.status.success(),
        "expected failure, stdout: {}, stderr: {}",
        stdout_text(&output),
        String::from_utf8_lossy(&output.stderr)
    );
    let json = assert_json_status(&output, "failed");
    assert_eq!(json["verified_message_count"], 2);
    assert_eq!(json["object_message_count"], 0);
    assert_eq!(json["written_object_count"], 0);
    assert!(
        json["errors"]
            .as_array()
            .is_some_and(|errors| errors.iter().any(|error| {
                error.as_str().is_some_and(|message| {
                    message.contains(
                        "wire WANT revision 'rev:missing' is not reachable from accepted sync roots for 'node:alpha'",
                    )
                })
            })),
        "expected unreachable-WANT-revision error, stdout: {}",
        stdout_text(&output)
    );
    assert!(!store_root
        .path()
        .join("indexes")
        .join("manifest.json")
        .exists());
}

#[test]
fn sync_pull_json_rejects_unreachable_want_object() {
    let signing_key = signing_key();
    let sender = "node:alpha";
    let transcript_dir = create_temp_dir("sync-pull-unreachable-want-object");
    let transcript_path = transcript_dir
        .path()
        .join("unreachable-want-object-transcript.json");
    let store_root = create_temp_dir("sync-pull-unreachable-want-object-store");
    write_transcript(
        &transcript_path,
        &json!({
            "peer": {
                "node_id": sender,
                "public_key": sender_public_key(&signing_key)
            },
            "messages": [
                signed_hello_message(&signing_key, sender),
                signed_manifest_message(&signing_key, sender, "rev:test"),
                signed_want_message(&signing_key, sender, &["patch:test"])
            ]
        }),
    );

    let output = run_mycel(&[
        "sync",
        "pull",
        &path_arg(&transcript_path),
        "--into",
        &path_arg(store_root.path()),
        "--json",
    ]);

    assert!(
        !output.status.success(),
        "expected failure, stdout: {}, stderr: {}",
        stdout_text(&output),
        String::from_utf8_lossy(&output.stderr)
    );
    let json = assert_json_status(&output, "failed");
    assert_eq!(json["verified_message_count"], 2);
    assert_eq!(json["object_message_count"], 0);
    assert_eq!(json["written_object_count"], 0);
    assert!(
        json["errors"]
            .as_array()
            .is_some_and(|errors| errors.iter().any(|error| {
                error.as_str().is_some_and(|message| {
                    message.contains(
                        "wire WANT object 'patch:test' is not reachable from accepted sync roots for 'node:alpha'",
                    )
                })
            })),
        "expected unreachable-WANT-object error, stdout: {}",
        stdout_text(&output)
    );
    assert!(!store_root
        .path()
        .join("indexes")
        .join("manifest.json")
        .exists());
}

#[test]
fn sync_pull_json_rejects_unrequested_object_before_manifest_or_heads() {
    let signing_key = signing_key();
    let sender = "node:alpha";
    let object = signed_patch_object_message(&signing_key, sender, "rev:genesis-null");
    let object_id = object["payload"]["object_id"]
        .as_str()
        .expect("signed OBJECT payload should include object_id")
        .to_string();
    let transcript_dir = create_temp_dir("sync-pull-unrequested-object-before-manifest");
    let transcript_path = transcript_dir
        .path()
        .join("unrequested-object-before-manifest-transcript.json");
    let store_root = create_temp_dir("sync-pull-unrequested-object-before-manifest-store");
    write_transcript(
        &transcript_path,
        &json!({
            "peer": {
                "node_id": sender,
                "public_key": sender_public_key(&signing_key)
            },
            "messages": [
                signed_hello_message(&signing_key, sender),
                object
            ]
        }),
    );

    let output = run_mycel(&[
        "sync",
        "pull",
        &path_arg(&transcript_path),
        "--into",
        &path_arg(store_root.path()),
        "--json",
    ]);

    assert!(
        !output.status.success(),
        "expected failure, stdout: {}, stderr: {}",
        stdout_text(&output),
        String::from_utf8_lossy(&output.stderr)
    );
    let json = assert_json_status(&output, "failed");
    assert_eq!(json["verified_message_count"], 1);
    assert_eq!(json["object_message_count"], 0);
    assert_eq!(json["written_object_count"], 0);
    assert!(
        json["errors"]
            .as_array()
            .is_some_and(|errors| errors.iter().any(|error| {
                error.as_str().is_some_and(|message| {
                    message.contains(&format!(
                        "wire OBJECT '{object_id}' was not requested from '{sender}'"
                    ))
                })
            })),
        "expected unrequested-object error, stdout: {}",
        stdout_text(&output)
    );
    assert!(!store_root
        .path()
        .join("indexes")
        .join("manifest.json")
        .exists());
}

#[test]
fn sync_pull_json_allows_error_before_hello_but_still_requires_sync_messages() {
    let signing_key = signing_key();
    let sender = "node:alpha";
    let transcript_dir = create_temp_dir("sync-pull-error-before-hello-then-hello");
    let transcript_path = transcript_dir
        .path()
        .join("error-before-hello-then-hello-transcript.json");
    let store_root = create_temp_dir("sync-pull-error-before-hello-then-hello-store");
    write_transcript(
        &transcript_path,
        &json!({
            "peer": {
                "node_id": sender,
                "public_key": sender_public_key(&signing_key)
            },
            "messages": [
                signed_error_message(&signing_key, sender, "msg:missing-hello"),
                signed_hello_message(&signing_key, sender),
                signed_bye_message(&signing_key, sender)
            ]
        }),
    );

    let output = run_mycel(&[
        "sync",
        "pull",
        &path_arg(&transcript_path),
        "--into",
        &path_arg(store_root.path()),
        "--json",
    ]);

    assert!(
        !output.status.success(),
        "expected failure, stdout: {}, stderr: {}",
        stdout_text(&output),
        String::from_utf8_lossy(&output.stderr)
    );
    let json = assert_json_status(&output, "failed");
    assert_eq!(json["peer_node_id"], sender);
    assert_eq!(json["message_count"], 3);
    assert_eq!(json["verified_message_count"], 3);
    assert_eq!(json["object_message_count"], 0);
    assert_eq!(json["written_object_count"], 0);
    assert!(
        json["errors"]
            .as_array()
            .is_some_and(|errors| errors.iter().any(|error| {
                error
                    .as_str()
                    .is_some_and(|message| message.contains("did not include MANIFEST or HEADS"))
            })),
        "expected missing MANIFEST/HEADS error, stdout: {}",
        stdout_text(&output)
    );
    assert!(
        json["errors"]
            .as_array()
            .is_some_and(|errors| errors.iter().any(|error| {
                error
                    .as_str()
                    .is_some_and(|message| message.contains("did not include any OBJECT messages"))
            })),
        "expected missing OBJECT error, stdout: {}",
        stdout_text(&output)
    );
    assert!(!store_root
        .path()
        .join("indexes")
        .join("manifest.json")
        .exists());
}

#[test]
fn sync_pull_json_reports_explicit_error_only_transcript_as_failed_sync() {
    let signing_key = signing_key();
    let sender = "node:alpha";
    let transcript_dir = create_temp_dir("sync-pull-error-only");
    let transcript_path = transcript_dir.path().join("error-only-transcript.json");
    let store_root = create_temp_dir("sync-pull-error-only-store");
    write_transcript(
        &transcript_path,
        &json!({
            "peer": {
                "node_id": sender,
                "public_key": sender_public_key(&signing_key)
            },
            "messages": [
                signed_error_message(&signing_key, sender, "msg:missing-hello")
            ]
        }),
    );

    let output = run_mycel(&[
        "sync",
        "pull",
        &path_arg(&transcript_path),
        "--into",
        &path_arg(store_root.path()),
        "--json",
    ]);

    assert!(
        !output.status.success(),
        "expected failure, stdout: {}, stderr: {}",
        stdout_text(&output),
        String::from_utf8_lossy(&output.stderr)
    );
    let json = assert_json_status(&output, "failed");
    assert_eq!(json["peer_node_id"], sender);
    assert_eq!(json["message_count"], 1);
    assert_eq!(json["verified_message_count"], 1);
    assert_eq!(json["object_message_count"], 0);
    assert_eq!(json["written_object_count"], 0);
    assert!(
        json["errors"]
            .as_array()
            .is_some_and(|errors| errors.iter().any(|error| {
                error.as_str().is_some_and(|message| {
                    message.contains("did not include HELLO from 'node:alpha'")
                })
            })),
        "expected missing-HELLO error after ERROR-only transcript, stdout: {}",
        stdout_text(&output)
    );
    assert!(
        json["errors"]
            .as_array()
            .is_some_and(|errors| errors.iter().any(|error| {
                error
                    .as_str()
                    .is_some_and(|message| message.contains("did not include MANIFEST or HEADS"))
            })),
        "expected missing-MANIFEST/HEADS error after ERROR-only transcript, stdout: {}",
        stdout_text(&output)
    );
    assert!(
        json["errors"]
            .as_array()
            .is_some_and(|errors| errors.iter().any(|error| {
                error
                    .as_str()
                    .is_some_and(|message| message.contains("did not include any OBJECT messages"))
            })),
        "expected missing-OBJECT error after ERROR-only transcript, stdout: {}",
        stdout_text(&output)
    );
    assert!(
        json["notes"]
            .as_array()
            .is_some_and(|notes| notes
                .iter()
                .any(|note| note.as_str().is_some_and(|message| message
                    .contains("sync transcript ended without BYE from 'node:alpha'")))),
        "expected missing-BYE note after ERROR-only transcript, stdout: {}",
        stdout_text(&output)
    );
    assert!(!store_root
        .path()
        .join("indexes")
        .join("manifest.json")
        .exists());
}

#[test]
fn sync_pull_json_rejects_unrequested_object_message() {
    let signing_key = signing_key();
    let sender = "node:alpha";
    let patch_object = signed_patch_object_message(&signing_key, sender, "rev:genesis-null");
    let patch_id = patch_object["payload"]["object_id"]
        .as_str()
        .expect("patch object id should exist")
        .to_string();
    let transcript_dir = create_temp_dir("sync-pull-unrequested-object");
    let transcript_path = transcript_dir
        .path()
        .join("unrequested-object-transcript.json");
    let store_root = create_temp_dir("sync-pull-unrequested-object-store");
    write_transcript(
        &transcript_path,
        &json!({
            "peer": {
                "node_id": sender,
                "public_key": sender_public_key(&signing_key)
            },
            "messages": [
                signed_hello_message(&signing_key, sender),
                signed_manifest_message(&signing_key, sender, "rev:test"),
                patch_object
            ]
        }),
    );

    let output = run_mycel(&[
        "sync",
        "pull",
        &path_arg(&transcript_path),
        "--into",
        &path_arg(store_root.path()),
        "--json",
    ]);

    assert!(
        !output.status.success(),
        "expected failure, stdout: {}, stderr: {}",
        stdout_text(&output),
        String::from_utf8_lossy(&output.stderr)
    );
    let json = assert_json_status(&output, "failed");
    assert_eq!(json["verified_message_count"], 2);
    assert_eq!(json["object_message_count"], 0);
    assert_eq!(json["written_object_count"], 0);
    assert!(
        json["errors"]
            .as_array()
            .is_some_and(|errors| errors.iter().any(|error| {
                error.as_str().is_some_and(|message| {
                    message.contains(&format!(
                        "wire OBJECT '{patch_id}' was not requested from '{sender}'"
                    ))
                })
            })),
        "expected unrequested-object error, stdout: {}",
        stdout_text(&output)
    );
    assert!(!store_root
        .path()
        .join("indexes")
        .join("manifest.json")
        .exists());
}

#[test]
fn sync_pull_json_rejects_snapshot_offer_without_advertised_capability() {
    let signing_key = signing_key();
    let sender = "node:alpha";
    let transcript_dir = create_temp_dir("sync-pull-snapshot-offer-without-capability");
    let transcript_path = transcript_dir
        .path()
        .join("snapshot-offer-without-capability-transcript.json");
    let store_root = create_temp_dir("sync-pull-snapshot-offer-without-capability-store");
    write_transcript(
        &transcript_path,
        &json!({
            "peer": {
                "node_id": sender,
                "public_key": sender_public_key(&signing_key)
            },
            "messages": [
                signed_hello_message(&signing_key, sender),
                signed_manifest_message(&signing_key, sender, "rev:test"),
                signed_snapshot_offer_message(&signing_key, sender, "snap:test-offer")
            ]
        }),
    );

    let output = run_mycel(&[
        "sync",
        "pull",
        &path_arg(&transcript_path),
        "--into",
        &path_arg(store_root.path()),
        "--json",
    ]);

    assert!(
        !output.status.success(),
        "expected failure, stdout: {}, stderr: {}",
        stdout_text(&output),
        String::from_utf8_lossy(&output.stderr)
    );
    let json = assert_json_status(&output, "failed");
    assert_eq!(json["verified_message_count"], 2);
    assert_eq!(json["object_message_count"], 0);
    assert_eq!(json["written_object_count"], 0);
    assert!(
        json["errors"]
            .as_array()
            .is_some_and(|errors| errors.iter().any(|error| {
                error.as_str().is_some_and(|message| {
                    message.contains(
                        "wire SNAPSHOT_OFFER requires advertised capability 'snapshot-sync'",
                    )
                })
            })),
        "expected snapshot capability error, stdout: {}",
        stdout_text(&output)
    );
    assert!(!store_root
        .path()
        .join("indexes")
        .join("manifest.json")
        .exists());
}

#[test]
fn sync_pull_json_rejects_view_announce_without_advertised_capability() {
    let signing_key = signing_key();
    let sender = "node:alpha";
    let transcript_dir = create_temp_dir("sync-pull-view-announce-without-capability");
    let transcript_path = transcript_dir
        .path()
        .join("view-announce-without-capability-transcript.json");
    let store_root = create_temp_dir("sync-pull-view-announce-without-capability-store");
    write_transcript(
        &transcript_path,
        &json!({
            "peer": {
                "node_id": sender,
                "public_key": sender_public_key(&signing_key)
            },
            "messages": [
                signed_hello_message(&signing_key, sender),
                signed_manifest_message(&signing_key, sender, "rev:test"),
                signed_view_announce_message(&signing_key, sender, "view:test-announce")
            ]
        }),
    );

    let output = run_mycel(&[
        "sync",
        "pull",
        &path_arg(&transcript_path),
        "--into",
        &path_arg(store_root.path()),
        "--json",
    ]);

    assert!(
        !output.status.success(),
        "expected failure, stdout: {}, stderr: {}",
        stdout_text(&output),
        String::from_utf8_lossy(&output.stderr)
    );
    let json = assert_json_status(&output, "failed");
    assert_eq!(json["verified_message_count"], 2);
    assert_eq!(json["object_message_count"], 0);
    assert_eq!(json["written_object_count"], 0);
    assert!(
        json["errors"]
            .as_array()
            .is_some_and(|errors| errors.iter().any(|error| {
                error.as_str().is_some_and(|message| {
                    message
                        .contains("wire VIEW_ANNOUNCE requires advertised capability 'view-sync'")
                })
            })),
        "expected view capability error, stdout: {}",
        stdout_text(&output)
    );
    assert!(!store_root
        .path()
        .join("indexes")
        .join("manifest.json")
        .exists());
}

#[test]
fn sync_pull_json_snapshot_offer_before_manifest_does_not_unlock_want() {
    let signing_key = signing_key();
    let sender = "node:alpha";
    let transcript_dir = create_temp_dir("sync-pull-snapshot-offer-before-manifest");
    let transcript_path = transcript_dir
        .path()
        .join("snapshot-offer-before-manifest-transcript.json");
    let store_root = create_temp_dir("sync-pull-snapshot-offer-before-manifest-store");
    write_transcript(
        &transcript_path,
        &json!({
            "peer": {
                "node_id": sender,
                "public_key": sender_public_key(&signing_key)
            },
            "messages": [
                signed_hello_message_with_capabilities(
                    &signing_key,
                    sender,
                    json!(["patch-sync", "snapshot-sync"])
                ),
                signed_snapshot_offer_message(&signing_key, sender, "snap:test-offer"),
                signed_want_message(&signing_key, sender, &["snap:test-offer"])
            ]
        }),
    );

    let output = run_mycel(&[
        "sync",
        "pull",
        &path_arg(&transcript_path),
        "--into",
        &path_arg(store_root.path()),
        "--json",
    ]);

    assert!(
        !output.status.success(),
        "expected failure, stdout: {}, stderr: {}",
        stdout_text(&output),
        String::from_utf8_lossy(&output.stderr)
    );
    let json = assert_json_status(&output, "failed");
    assert_eq!(json["verified_message_count"], 2);
    assert_eq!(json["object_message_count"], 0);
    assert_eq!(json["written_object_count"], 0);
    assert!(
        json["errors"]
            .as_array()
            .is_some_and(|errors| errors.iter().any(|error| {
                error.as_str().is_some_and(|message| {
                    message.contains("wire WANT requires prior MANIFEST or HEADS")
                })
            })),
        "expected WANT-before-manifest error after SNAPSHOT_OFFER, stdout: {}",
        stdout_text(&output)
    );
    assert!(!store_root
        .path()
        .join("indexes")
        .join("manifest.json")
        .exists());
}

#[test]
fn sync_pull_json_view_announce_before_manifest_does_not_unlock_want() {
    let signing_key = signing_key();
    let sender = "node:alpha";
    let transcript_dir = create_temp_dir("sync-pull-view-announce-before-manifest");
    let transcript_path = transcript_dir
        .path()
        .join("view-announce-before-manifest-transcript.json");
    let store_root = create_temp_dir("sync-pull-view-announce-before-manifest-store");
    write_transcript(
        &transcript_path,
        &json!({
            "peer": {
                "node_id": sender,
                "public_key": sender_public_key(&signing_key)
            },
            "messages": [
                signed_hello_message_with_capabilities(
                    &signing_key,
                    sender,
                    json!(["patch-sync", "view-sync"])
                ),
                signed_view_announce_message(&signing_key, sender, "view:test-announce"),
                signed_want_message(&signing_key, sender, &["view:test-announce"])
            ]
        }),
    );

    let output = run_mycel(&[
        "sync",
        "pull",
        &path_arg(&transcript_path),
        "--into",
        &path_arg(store_root.path()),
        "--json",
    ]);

    assert!(
        !output.status.success(),
        "expected failure, stdout: {}, stderr: {}",
        stdout_text(&output),
        String::from_utf8_lossy(&output.stderr)
    );
    let json = assert_json_status(&output, "failed");
    assert_eq!(json["verified_message_count"], 2);
    assert_eq!(json["object_message_count"], 0);
    assert_eq!(json["written_object_count"], 0);
    assert!(
        json["errors"]
            .as_array()
            .is_some_and(|errors| errors.iter().any(|error| {
                error.as_str().is_some_and(|message| {
                    message.contains("wire WANT requires prior MANIFEST or HEADS")
                })
            })),
        "expected WANT-before-manifest error after VIEW_ANNOUNCE, stdout: {}",
        stdout_text(&output)
    );
    assert!(!store_root
        .path()
        .join("indexes")
        .join("manifest.json")
        .exists());
}

#[test]
fn sync_pull_text_reports_pending_requested_object_failure() {
    let signing_key = signing_key();
    let sender = "node:alpha";
    let patch_object = signed_patch_object_message(&signing_key, sender, "rev:genesis-null");
    let patch_id = patch_object["payload"]["object_id"]
        .as_str()
        .expect("patch object id should exist")
        .to_string();
    let revision_object = signed_revision_object_message(&signing_key, sender, &[], &[&patch_id]);
    let revision_id = revision_object["payload"]["object_id"]
        .as_str()
        .expect("revision object id should exist")
        .to_string();
    let transcript_dir = create_temp_dir("sync-pull-pending");
    let transcript_path = transcript_dir.path().join("pending-transcript.json");
    let store_root = create_temp_dir("sync-pull-pending-store");
    write_transcript(
        &transcript_path,
        &json!({
            "peer": {
                "node_id": sender,
                "public_key": sender_public_key(&signing_key)
            },
            "messages": [
                signed_hello_message(&signing_key, sender),
                signed_manifest_message(&signing_key, sender, &revision_id),
                signed_want_message(&signing_key, sender, &[&revision_id]),
                signed_bye_message(&signing_key, sender)
            ]
        }),
    );

    let output = run_mycel(&[
        "sync",
        "pull",
        &path_arg(&transcript_path),
        "--into",
        &path_arg(store_root.path()),
    ]);

    assert!(
        !output.status.success(),
        "expected failure, stdout: {}, stderr: {}",
        stdout_text(&output),
        String::from_utf8_lossy(&output.stderr)
    );
    assert_stderr_contains(
        &output,
        "sync transcript did not include any OBJECT messages",
    );
    assert_stderr_contains(
        &output,
        "sync transcript ended with 1 pending requested object(s)",
    );
    let stdout = stdout_text(&output);
    assert!(stdout.contains("sync pull: failed"), "stdout: {stdout}");
    assert!(stdout.contains("verified messages: 4"), "stdout: {stdout}");
    assert!(stdout.contains("object messages: 0"), "stdout: {stdout}");
    assert!(!store_root
        .path()
        .join("indexes")
        .join("manifest.json")
        .exists());
}
