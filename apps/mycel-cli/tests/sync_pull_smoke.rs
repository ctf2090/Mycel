use std::fs;
use std::path::Path;
use std::process::{Command, Stdio};

use base64::Engine;
use ed25519_dalek::{Signer, SigningKey};
use serde_json::{json, Value};

use mycel_core::canonical::{signed_payload_bytes, wire_envelope_signed_payload_bytes};
use mycel_core::protocol::recompute_object_id;
use mycel_core::replay::{compute_state_hash, DocumentState};
use mycel_core::store::write_object_value_to_store;

mod common;

use common::{
    assert_json_status, assert_stderr_contains, assert_success, create_temp_dir, mycel_bin,
    run_mycel, stdout_text,
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
    signed_patch_object_message_for_doc(signing_key, sender, "doc:test", base_revision)
}

fn signed_patch_object_message_for_doc(
    signing_key: &SigningKey,
    sender: &str,
    doc_id: &str,
    base_revision: &str,
) -> Value {
    let body = sign_object_value(
        signing_key,
        json!({
            "type": "patch",
            "version": "mycel/0.1",
            "patch_id": "patch:placeholder",
            "doc_id": doc_id,
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
    signed_revision_object_message_for_doc(signing_key, sender, "doc:test", parents, patches)
}

fn signed_revision_object_message_for_doc(
    signing_key: &SigningKey,
    sender: &str,
    doc_id: &str,
    parents: &[&str],
    patches: &[&str],
) -> Value {
    let body = sign_object_value(
        signing_key,
        json!({
            "type": "revision",
            "version": "mycel/0.1",
            "revision_id": "rev:placeholder",
            "doc_id": doc_id,
            "parents": parents,
            "patches": patches,
            "state_hash": empty_state_hash(doc_id),
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

#[path = "sync_pull_smoke/peer_store.rs"]
mod peer_store;

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
fn sync_pull_json_recovers_missing_dependency_via_want_cycle() {
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

    let transcript_dir = create_temp_dir("sync-pull-partial-recovery-source");
    let transcript_path = transcript_dir
        .path()
        .join("pull-partial-recovery-transcript.json");
    let store_root = create_temp_dir("sync-pull-partial-recovery-store");

    write_object_value_to_store(store_root.path(), &patch_object["payload"]["body"])
        .expect("partial local patch should write to store");

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
    assert_eq!(json["message_count"], 5);
    assert_eq!(json["verified_message_count"], 5);
    assert_eq!(json["object_message_count"], 1);
    assert_eq!(json["verified_object_count"], 1);
    assert_eq!(json["written_object_count"], 1);
    assert_eq!(json["existing_object_count"], 0);

    let manifest_path = store_root.path().join("indexes").join("manifest.json");
    let manifest: Value =
        serde_json::from_str(&fs::read_to_string(&manifest_path).expect("manifest should read"))
            .expect("manifest should parse");
    assert_eq!(manifest["stored_object_count"], 2);
    let revisions = manifest["doc_revisions"]["doc:test"]
        .as_array()
        .expect("expected recovered revision index array");
    assert_eq!(revisions.len(), 1);
    assert_eq!(revisions[0].as_str(), Some(revision_id.as_str()));
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
fn sync_pull_json_accepts_view_announce_when_capability_is_advertised() {
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
    let view_object = signed_view_object_message(&signing_key, sender, &revision_id);
    let view_id = view_object["payload"]["object_id"]
        .as_str()
        .expect("view object id should exist")
        .to_string();
    let transcript_dir = create_temp_dir("sync-pull-view-announce-source");
    let transcript_path = transcript_dir
        .path()
        .join("pull-view-announce-transcript.json");
    let store_root = create_temp_dir("sync-pull-view-announce-store");
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
                signed_manifest_message_with_capabilities(
                    &signing_key,
                    sender,
                    &revision_id,
                    json!(["patch-sync", "view-sync"])
                ),
                signed_view_announce_message(&signing_key, sender, &view_id),
                signed_want_message(&signing_key, sender, &[&view_id]),
                view_object,
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
    assert_eq!(
        manifest["view_governance"].as_array().map(Vec::len),
        Some(1)
    );
    assert_eq!(manifest["view_governance"][0]["view_id"], view_id);
    assert_eq!(manifest["document_views"]["doc:test"][0], view_id);
}

#[path = "sync_pull_smoke/positive.rs"]
mod positive;

#[path = "sync_pull_smoke/negative.rs"]
mod negative;
