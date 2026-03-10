use std::fs;
use std::path::PathBuf;

use base64::Engine;
use ed25519_dalek::{Signer, SigningKey};
use serde_json::{json, Value};
use sha2::{Digest, Sha256};

mod common;

use common::{
    assert_empty_stderr, assert_exit_code, assert_json_status, assert_stderr_contains,
    assert_stderr_starts_with, assert_stdout_contains, assert_success, assert_top_level_help,
    create_temp_dir, parse_json_stdout, run_mycel, stdout_text,
};

struct TempObjectFile {
    _dir: common::TempDir,
    path: PathBuf,
}

fn write_object_file(prefix: &str, name: &str, value: Value) -> TempObjectFile {
    let dir = create_temp_dir(prefix);
    let path = dir.path().join(name);
    let content = serde_json::to_string_pretty(&value).expect("object JSON should serialize");
    fs::write(&path, content).expect("object JSON should be written");
    TempObjectFile { _dir: dir, path }
}

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

fn canonical_json(value: &Value) -> String {
    match value {
        Value::Null => panic!("test objects should not use null"),
        Value::Bool(boolean) => boolean.to_string(),
        Value::Number(number) => number.to_string(),
        Value::String(string) => serde_json::to_string(string).expect("string should encode"),
        Value::Array(values) => format!(
            "[{}]",
            values
                .iter()
                .map(canonical_json)
                .collect::<Vec<_>>()
                .join(",")
        ),
        Value::Object(entries) => {
            let mut keys: Vec<&String> = entries.keys().collect();
            keys.sort_unstable();
            let parts = keys
                .into_iter()
                .map(|key| {
                    format!(
                        "{}:{}",
                        serde_json::to_string(key).expect("key should encode"),
                        canonical_json(&entries[key])
                    )
                })
                .collect::<Vec<_>>();
            format!("{{{}}}", parts.join(","))
        }
    }
}

fn recompute_id(value: &Value, id_field: &str, prefix: &str) -> String {
    let mut object = value
        .as_object()
        .cloned()
        .expect("test object should be JSON object");
    object.remove(id_field);
    object.remove("signature");
    let canonical = canonical_json(&Value::Object(object));
    let mut hasher = Sha256::new();
    hasher.update(canonical.as_bytes());
    format!("{prefix}:{:x}", hasher.finalize())
}

fn sign_value(signing_key: &SigningKey, value: &Value) -> String {
    let mut object = value
        .as_object()
        .cloned()
        .expect("test object should be JSON object");
    object.remove("signature");
    let canonical = canonical_json(&Value::Object(object));
    let signature = signing_key.sign(canonical.as_bytes());
    format!(
        "sig:ed25519:{}",
        base64::engine::general_purpose::STANDARD.encode(signature.to_bytes())
    )
}

fn state_hash(value: &Value) -> String {
    let canonical = canonical_json(value);
    let mut hasher = Sha256::new();
    hasher.update(canonical.as_bytes());
    format!("hash:{:x}", hasher.finalize())
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
fn object_verify_json_reports_ok_for_valid_patch() {
    let object = write_object_file(
        "object-verify-patch",
        "patch.json",
        signed_object(
            json!({
                "type": "patch",
                "version": "mycel/0.1",
                "doc_id": "doc:test",
                "base_revision": "rev:genesis-null",
                "timestamp": 1777778888u64,
                "ops": []
            }),
            "author",
            "patch_id",
            "patch",
        ),
    );
    let path = path_arg(&object.path);
    let output = run_mycel(&["object", "verify", &path, "--json"]);

    assert_success(&output);
    let json = assert_json_status(&output, "ok");
    assert_eq!(json["object_type"], "patch");
    assert_eq!(json["signature_rule"], "required");
    assert_eq!(json["signer_field"], "author");
    assert_eq!(json["signature_verification"], "verified");
    assert_eq!(json["signer"], signer_id(&signing_key()));
    assert_eq!(json["declared_id"], json["recomputed_id"]);
    assert_eq!(json["notes"], Value::Array(Vec::new()));
}

#[test]
fn object_verify_text_reports_ok_for_document_without_signature() {
    let object = write_object_file(
        "object-verify-document",
        "document.json",
        json!({
            "type": "document",
            "version": "mycel/0.1",
            "doc_id": "doc:test",
            "title": "Plain document"
        }),
    );
    let path = path_arg(&object.path);
    let output = run_mycel(&["object", "verify", &path]);

    assert_success(&output);
    assert_empty_stderr(&output);
    assert_stdout_contains(&output, "object type: document");
    assert_stdout_contains(&output, "signature rule: forbidden");
    assert_stdout_contains(&output, "verification: ok");
}

#[test]
fn object_verify_json_fails_for_document_missing_doc_id() {
    let object = write_object_file(
        "object-verify-document-missing-doc-id",
        "document.json",
        json!({
            "type": "document",
            "version": "mycel/0.1",
            "title": "Plain document"
        }),
    );
    let path = path_arg(&object.path);
    let output = run_mycel(&["object", "verify", &path, "--json"]);

    assert_exit_code(&output, 1);
    let json = assert_json_status(&output, "failed");
    assert_eq!(json["object_type"], "document");
    assert!(
        json["errors"]
            .as_array()
            .is_some_and(
                |errors| errors
                    .iter()
                    .any(|entry| entry.as_str().is_some_and(|message| message
                        .contains("document object is missing string field 'doc_id'")))
            ),
        "expected missing doc_id error, stdout: {}",
        stdout_text(&output)
    );
}

#[test]
fn object_verify_json_fails_for_mismatched_revision_id() {
    let mut revision = signed_object(
        json!({
            "type": "revision",
            "version": "mycel/0.1",
            "doc_id": "doc:test",
            "parents": [],
            "patches": [],
            "state_hash": "hash:test-state",
            "timestamp": 1777778890u64
        }),
        "author",
        "revision_id",
        "rev",
    );
    revision["revision_id"] = Value::String("rev:wrong".to_string());
    revision["signature"] = Value::String(sign_value(&signing_key(), &revision));
    let object = write_object_file("object-verify-revision-mismatch", "revision.json", revision);
    let path = path_arg(&object.path);
    let output = run_mycel(&["object", "verify", &path, "--json"]);

    assert_exit_code(&output, 1);
    let json = assert_json_status(&output, "failed");
    assert_eq!(json["object_type"], "revision");
    assert!(
        json["errors"]
            .as_array()
            .is_some_and(|errors| errors.iter().any(|entry| entry
                .as_str()
                .is_some_and(|message| message.contains("declared revision_id does not match")))),
        "expected derived ID mismatch error, stdout: {}",
        stdout_text(&output)
    );
}

#[test]
fn object_verify_text_fails_when_signed_object_is_missing_signature() {
    let object = write_object_file(
        "object-verify-view-missing-signature",
        "view.json",
        json!({
            "type": "view",
            "version": "mycel/0.1",
            "maintainer": signer_id(&signing_key()),
            "documents": {
                "doc:test": "rev:test"
            },
            "policy": {
                "accept_keys": ["pk:maintainerA"],
                "merge_rule": "manual-reviewed",
                "preferred_branches": ["main"]
            },
            "timestamp": 1777778891u64,
            "view_id": "view:placeholder"
        }),
    );
    let path = path_arg(&object.path);
    let output = run_mycel(&["object", "verify", &path]);

    assert_exit_code(&output, 1);
    assert_stdout_contains(&output, "verification: failed");
    assert_stderr_starts_with(&output, "error: ");
    assert_stderr_contains(
        &output,
        "view object is missing required top-level 'signature'",
    );
}

#[test]
fn object_verify_json_fails_when_document_has_forbidden_signature() {
    let object = write_object_file(
        "object-verify-document-signature",
        "document.json",
        json!({
            "type": "document",
            "version": "mycel/0.1",
            "doc_id": "doc:test",
            "title": "Plain document",
            "signature": "sig:not-allowed"
        }),
    );
    let path = path_arg(&object.path);
    let output = run_mycel(&["object", "verify", &path, "--json"]);

    assert_exit_code(&output, 1);
    let json = parse_json_stdout(&output);
    assert_eq!(json["status"], "failed");
    assert_eq!(json["object_type"], "document");
    assert!(
        json["errors"]
            .as_array()
            .is_some_and(|errors| errors.iter().any(|entry| {
                entry.as_str().is_some_and(|message| {
                    message.contains("must not include top-level 'signature'")
                })
            })),
        "expected forbidden signature error, stdout: {}",
        stdout_text(&output)
    );
}

#[test]
fn object_verify_missing_target_fails_cleanly() {
    let output = run_mycel(&["object", "verify"]);

    assert_exit_code(&output, 2);
    assert_stderr_contains(&output, "required arguments were not provided");
    assert_stderr_contains(&output, "<PATH>");
}

#[test]
fn object_verify_unknown_subcommand_fails_cleanly() {
    let output = run_mycel(&["object", "bogus"]);

    assert_exit_code(&output, 2);
    assert_stderr_contains(&output, "unknown object subcommand: bogus");
    assert_top_level_help(&stdout_text(&output));
}

#[test]
fn object_verify_json_fails_for_invalid_patch_signature() {
    let mut patch = signed_object(
        json!({
            "type": "patch",
            "version": "mycel/0.1",
            "doc_id": "doc:test",
            "base_revision": "rev:genesis-null",
            "timestamp": 1777778888u64,
            "ops": []
        }),
        "author",
        "patch_id",
        "patch",
    );
    patch["signature"] = Value::String(
        "sig:ed25519:AAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAA=="
            .to_string(),
    );
    let object = write_object_file("object-verify-patch-bad-signature", "patch.json", patch);
    let path = path_arg(&object.path);
    let output = run_mycel(&["object", "verify", &path, "--json"]);

    assert_exit_code(&output, 1);
    let json = parse_json_stdout(&output);
    assert_eq!(json["status"], "failed");
    assert_eq!(json["signature_verification"], "failed");
    assert!(
        json["errors"]
            .as_array()
            .is_some_and(|errors| errors.iter().any(|entry| entry
                .as_str()
                .is_some_and(|message| message.contains("Ed25519 signature verification failed")))),
        "expected signature failure, stdout: {}",
        stdout_text(&output)
    );
}

#[test]
fn object_verify_json_reports_ok_for_revision_with_replayed_state_hash() {
    let dir = create_temp_dir("object-verify-revision-replay");
    let patch_path = dir.path().join("patch.json");
    let revision_path = dir.path().join("revision.json");
    let patch = signed_object(
        json!({
            "type": "patch",
            "version": "mycel/0.1",
            "doc_id": "doc:test",
            "base_revision": "rev:genesis-null",
            "timestamp": 1777778888u64,
            "ops": [
                {
                    "op": "insert_block",
                    "new_block": {
                        "block_id": "blk:001",
                        "block_type": "paragraph",
                        "content": "Hello",
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
    fs::write(
        &patch_path,
        serde_json::to_string_pretty(&patch).expect("patch JSON should serialize"),
    )
    .expect("patch JSON should be written");
    let expected_state_hash = state_hash(&json!({
        "doc_id": "doc:test",
        "blocks": [
            {
                "block_id": "blk:001",
                "block_type": "paragraph",
                "content": "Hello",
                "attrs": {},
                "children": []
            }
        ]
    }));
    let revision = signed_object(
        json!({
            "type": "revision",
            "version": "mycel/0.1",
            "doc_id": "doc:test",
            "parents": [],
            "patches": [patch["patch_id"].as_str().expect("patch id should exist")],
            "state_hash": expected_state_hash,
            "timestamp": 1777778890u64
        }),
        "author",
        "revision_id",
        "rev",
    );
    fs::write(
        &revision_path,
        serde_json::to_string_pretty(&revision).expect("revision JSON should serialize"),
    )
    .expect("revision JSON should be written");

    let output = run_mycel(&["object", "verify", &path_arg(&revision_path), "--json"]);

    assert_success(&output);
    let json = assert_json_status(&output, "ok");
    assert_eq!(json["object_type"], "revision");
    assert_eq!(json["state_hash_verification"], "verified");
    assert_eq!(json["declared_state_hash"], json["recomputed_state_hash"]);
}

#[test]
fn object_verify_json_fails_for_revision_state_hash_mismatch() {
    let dir = create_temp_dir("object-verify-revision-state-hash-mismatch");
    let patch_path = dir.path().join("patch.json");
    let revision_path = dir.path().join("revision.json");
    let patch = signed_object(
        json!({
            "type": "patch",
            "version": "mycel/0.1",
            "doc_id": "doc:test",
            "base_revision": "rev:genesis-null",
            "timestamp": 1777778888u64,
            "ops": [
                {
                    "op": "insert_block",
                    "new_block": {
                        "block_id": "blk:001",
                        "block_type": "paragraph",
                        "content": "Hello",
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
    fs::write(
        &patch_path,
        serde_json::to_string_pretty(&patch).expect("patch JSON should serialize"),
    )
    .expect("patch JSON should be written");
    let revision = signed_object(
        json!({
            "type": "revision",
            "version": "mycel/0.1",
            "doc_id": "doc:test",
            "parents": [],
            "patches": [patch["patch_id"].as_str().expect("patch id should exist")],
            "state_hash": "hash:wrong",
            "timestamp": 1777778890u64
        }),
        "author",
        "revision_id",
        "rev",
    );
    fs::write(
        &revision_path,
        serde_json::to_string_pretty(&revision).expect("revision JSON should serialize"),
    )
    .expect("revision JSON should be written");

    let output = run_mycel(&["object", "verify", &path_arg(&revision_path), "--json"]);

    assert_exit_code(&output, 1);
    let json = assert_json_status(&output, "failed");
    assert_eq!(json["object_type"], "revision");
    assert_eq!(json["state_hash_verification"], "failed");
    assert!(
        json["errors"]
            .as_array()
            .is_some_and(
                |errors| errors
                    .iter()
                    .any(|entry| entry.as_str().is_some_and(|message| {
                        message.contains("declared state_hash does not match replayed state hash")
                    }))
            ),
        "expected state-hash mismatch error, stdout: {}",
        stdout_text(&output)
    );
}
