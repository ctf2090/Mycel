use std::fs;
use std::path::PathBuf;

use base64::Engine;
use ed25519_dalek::{Signer, SigningKey};
use serde_json::{json, Value};

use mycel_core::canonical::{prefixed_canonical_hash, signed_payload_bytes};

mod common;

use common::{
    assert_empty_stderr, assert_exit_code, assert_json_status, assert_stderr_contains,
    assert_success, create_temp_dir, run_mycel, stdout_text,
};

fn path_arg(path: &PathBuf) -> String {
    path.to_string_lossy().into_owned()
}

fn signing_key() -> SigningKey {
    SigningKey::from_bytes(&[7u8; 32])
}

fn signer_id(signing_key: &SigningKey) -> String {
    format!(
        "pk:ed25519:{}",
        base64::engine::general_purpose::STANDARD.encode(signing_key.verifying_key().as_bytes())
    )
}

fn recompute_id(value: &Value, id_field: &str, prefix: &str) -> String {
    let mut object = value
        .as_object()
        .cloned()
        .expect("test object should be JSON object");
    object.remove(id_field);
    object.remove("signature");
    prefixed_canonical_hash(&Value::Object(object), prefix).expect("object id should canonicalize")
}

fn sign_value(signing_key: &SigningKey, value: &Value) -> String {
    let payload = signed_payload_bytes(value).expect("payload should canonicalize");
    let signature = signing_key.sign(&payload);
    format!(
        "sig:ed25519:{}",
        base64::engine::general_purpose::STANDARD.encode(signature.to_bytes())
    )
}

fn signed_object(mut value: Value, signer_field: &str, id_field: &str, id_prefix: &str) -> Value {
    let signing_key = signing_key();
    value[signer_field] = Value::String(signer_id(&signing_key));
    let id = recompute_id(&value, id_field, id_prefix);
    value[id_field] = Value::String(id);
    let signature = sign_value(&signing_key, &value);
    value["signature"] = Value::String(signature);
    value
}

#[test]
fn store_ingest_json_writes_verified_objects_into_store_layout() {
    let source_dir = create_temp_dir("store-ingest-source");
    let store_dir = create_temp_dir("store-ingest-root");
    let patch_path = source_dir.path().join("patch.json");
    let revision_path = source_dir.path().join("revision.json");

    let patch = signed_object(
        json!({
            "type": "patch",
            "version": "mycel/0.1",
            "doc_id": "doc:store-ingest",
            "base_revision": "rev:genesis-null",
            "timestamp": 1u64,
            "ops": [
                {
                    "op": "insert_block",
                    "new_block": {
                        "block_id": "blk:store-ingest-001",
                        "block_type": "paragraph",
                        "content": "Stored by CLI",
                        "attrs": {},
                        "children": []
                    }
                }
            ]
        }),
        "author",
        "patch_id",
        "patch",
    );
    let patch_id = patch["patch_id"]
        .as_str()
        .expect("patch id should exist")
        .to_string();
    fs::write(
        &patch_path,
        serde_json::to_string_pretty(&patch).expect("patch should serialize"),
    )
    .expect("patch should write");

    let expected_state = json!({
        "doc_id": "doc:store-ingest",
        "blocks": [
            {
                "block_id": "blk:store-ingest-001",
                "block_type": "paragraph",
                "content": "Stored by CLI",
                "attrs": {},
                "children": []
            }
        ]
    });
    let state_hash =
        prefixed_canonical_hash(&expected_state, "hash").expect("state hash should canonicalize");

    let revision = signed_object(
        json!({
            "type": "revision",
            "version": "mycel/0.1",
            "doc_id": "doc:store-ingest",
            "parents": [],
            "patches": [patch_id],
            "state_hash": state_hash,
            "timestamp": 2u64
        }),
        "author",
        "revision_id",
        "rev",
    );
    fs::write(
        &revision_path,
        serde_json::to_string_pretty(&revision).expect("revision should serialize"),
    )
    .expect("revision should write");

    let output = run_mycel(&[
        "store",
        "ingest",
        &path_arg(&source_dir.path().to_path_buf()),
        "--into",
        &path_arg(&store_dir.path().to_path_buf()),
        "--json",
    ]);

    assert_success(&output);
    let json = assert_json_status(&output, "ok");
    assert_eq!(json["verified_object_count"], 2);
    assert_eq!(json["written_object_count"], 2);
    assert_eq!(json["existing_object_count"], 0);
    assert_eq!(json["skipped_object_count"], 0);
    assert_eq!(json["indexed_object_count"], 2);
    assert!(
        json["index_manifest_path"]
            .as_str()
            .is_some_and(|path| path.ends_with("/indexes/manifest.json")),
        "expected persisted manifest path, stdout: {}",
        stdout_text(&output)
    );

    let patch_hash = patch["patch_id"]
        .as_str()
        .and_then(|value| value.split_once(':'))
        .map(|(_, hash)| hash)
        .expect("patch id should include hash");
    let revision_hash = revision["revision_id"]
        .as_str()
        .and_then(|value| value.split_once(':'))
        .map(|(_, hash)| hash)
        .expect("revision id should include hash");

    assert!(
        store_dir
            .path()
            .join("objects")
            .join("patch")
            .join(format!("{patch_hash}.json"))
            .exists(),
        "expected stored patch, stdout: {}",
        stdout_text(&output)
    );
    assert!(
        store_dir
            .path()
            .join("objects")
            .join("revision")
            .join(format!("{revision_hash}.json"))
            .exists(),
        "expected stored revision, stdout: {}",
        stdout_text(&output)
    );
    let manifest_path = store_dir.path().join("indexes").join("manifest.json");
    assert!(
        manifest_path.exists(),
        "expected persisted index manifest, stdout: {}",
        stdout_text(&output)
    );
    let manifest: Value =
        serde_json::from_str(&fs::read_to_string(&manifest_path).expect("manifest should read"))
            .expect("manifest should parse");
    assert_eq!(manifest["stored_object_count"], 2);
    assert!(
        manifest["doc_revisions"]["doc:store-ingest"]
            .as_array()
            .is_some_and(|values| values.len() == 1),
        "expected manifest doc revision index"
    );

    let rebuild = run_mycel(&[
        "store",
        "rebuild",
        &path_arg(&store_dir.path().to_path_buf()),
        "--json",
    ]);
    assert_success(&rebuild);
    let rebuild_json = assert_json_status(&rebuild, "ok");
    assert_eq!(rebuild_json["stored_object_count"], 2);
}

#[test]
fn store_ingest_text_reports_existing_objects_on_repeat_ingest() {
    let source_dir = create_temp_dir("store-ingest-repeat-source");
    let store_dir = create_temp_dir("store-ingest-repeat-root");
    let patch = signed_object(
        json!({
            "type": "patch",
            "version": "mycel/0.1",
            "doc_id": "doc:repeat",
            "base_revision": "rev:genesis-null",
            "timestamp": 1u64,
            "ops": []
        }),
        "author",
        "patch_id",
        "patch",
    );
    fs::write(
        source_dir.path().join("patch.json"),
        serde_json::to_string_pretty(&patch).expect("patch should serialize"),
    )
    .expect("patch should write");

    let first = run_mycel(&[
        "store",
        "ingest",
        &path_arg(&source_dir.path().to_path_buf()),
        "--into",
        &path_arg(&store_dir.path().to_path_buf()),
    ]);
    assert_success(&first);
    assert_empty_stderr(&first);

    let second = run_mycel(&[
        "store",
        "ingest",
        &path_arg(&source_dir.path().to_path_buf()),
        "--into",
        &path_arg(&store_dir.path().to_path_buf()),
    ]);
    assert_success(&second);
    assert_empty_stderr(&second);
    let stdout = stdout_text(&second);
    assert!(stdout.contains("existing objects: 1"), "stdout: {stdout}");
    assert!(stdout.contains("written objects: 0"), "stdout: {stdout}");
    assert!(stdout.contains("indexed objects: 1"), "stdout: {stdout}");
    assert!(stdout.contains("index manifest:"), "stdout: {stdout}");
    assert!(stdout.contains("store ingest: ok"), "stdout: {stdout}");
}

#[test]
fn store_ingest_missing_source_fails_cleanly() {
    let store_dir = create_temp_dir("store-ingest-missing-root");
    let output = run_mycel(&[
        "store",
        "ingest",
        "missing-store-ingest-source",
        "--into",
        &path_arg(&store_dir.path().to_path_buf()),
    ]);

    assert_exit_code(&output, 2);
    assert_stderr_contains(&output, "ingest source does not exist");
}
