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

fn write_raw_object_file(prefix: &str, name: &str, content: &str) -> TempObjectFile {
    let dir = create_temp_dir(prefix);
    let path = dir.path().join(name);
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
            "title": "Plain document",
            "language": "en",
            "content_model": "block-tree",
            "created_at": 1777777777u64,
            "created_by": "pk:authorA",
            "genesis_revision": "rev:genesis"
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
fn object_verify_json_fails_for_document_with_non_string_doc_id() {
    let object = write_object_file(
        "object-verify-document-non-string-doc-id",
        "document.json",
        json!({
            "type": "document",
            "version": "mycel/0.1",
            "doc_id": 7,
            "title": "Plain document",
            "language": "zh-Hant",
            "content_model": "block-tree",
            "created_at": 1u64,
            "created_by": "pk:ed25519:test",
            "genesis_revision": "rev:test"
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
            .is_some_and(|errors| errors.iter().any(|entry| {
                entry
                    .as_str()
                    .is_some_and(|message| message.contains("top-level 'doc_id' must be a string"))
            })),
        "expected doc_id type error, stdout: {}",
        stdout_text(&output)
    );
}

#[test]
fn object_verify_json_fails_for_document_with_wrong_doc_id_prefix() {
    let object = write_object_file(
        "object-verify-document-wrong-doc-id-prefix",
        "document.json",
        json!({
            "type": "document",
            "version": "mycel/0.1",
            "doc_id": "revision:test",
            "title": "Plain document",
            "language": "zh-Hant",
            "content_model": "block-tree",
            "created_at": 1u64,
            "created_by": "pk:ed25519:test",
            "genesis_revision": "rev:test"
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
                    .any(|entry| entry.as_str().is_some_and(|message| {
                        message.contains("top-level 'doc_id' must use 'doc:' prefix")
                    }))
            ),
        "expected wrong doc_id prefix error, stdout: {}",
        stdout_text(&output)
    );
}

#[test]
fn object_verify_json_fails_for_document_with_unknown_top_level_field() {
    let object = write_object_file(
        "object-verify-document-unknown-field",
        "document.json",
        json!({
            "type": "document",
            "version": "mycel/0.1",
            "doc_id": "doc:test",
            "title": "Plain document",
            "language": "zh-Hant",
            "content_model": "block-tree",
            "created_at": 1u64,
            "created_by": "pk:ed25519:test",
            "genesis_revision": "rev:test",
            "unexpected": true
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
            .is_some_and(|errors| errors.iter().any(|entry| {
                entry.as_str().is_some_and(|message| {
                    message.contains("top-level contains unexpected field 'unexpected'")
                })
            })),
        "expected unknown-field validation error, stdout: {}",
        stdout_text(&output)
    );
}

#[test]
fn object_verify_json_fails_for_document_with_wrong_content_model() {
    let object = write_object_file(
        "object-verify-document-wrong-content-model",
        "document.json",
        json!({
            "type": "document",
            "version": "mycel/0.1",
            "doc_id": "doc:test",
            "title": "Plain document",
            "language": "zh-Hant",
            "content_model": "markdown",
            "created_at": 1u64,
            "created_by": "pk:ed25519:test",
            "genesis_revision": "rev:test"
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
            .is_some_and(|errors| errors.iter().any(|entry| {
                entry.as_str().is_some_and(|message| {
                    message.contains("top-level 'content_model' must equal 'block-tree'")
                })
            })),
        "expected content_model validation error, stdout: {}",
        stdout_text(&output)
    );
}

#[test]
fn object_verify_json_fails_for_document_with_wrong_created_by_prefix() {
    let object = write_object_file(
        "object-verify-document-wrong-created-by-prefix",
        "document.json",
        json!({
            "type": "document",
            "version": "mycel/0.1",
            "doc_id": "doc:test",
            "title": "Plain document",
            "language": "zh-Hant",
            "content_model": "block-tree",
            "created_at": 1u64,
            "created_by": "sig:bad",
            "genesis_revision": "rev:test"
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
            .is_some_and(|errors| errors.iter().any(|entry| {
                entry.as_str().is_some_and(|message| {
                    message.contains("top-level 'created_by' must use 'pk:' prefix")
                })
            })),
        "expected created_by prefix error, stdout: {}",
        stdout_text(&output)
    );
}

#[test]
fn object_verify_json_fails_for_document_with_wrong_genesis_revision_prefix() {
    let object = write_object_file(
        "object-verify-document-wrong-genesis-revision-prefix",
        "document.json",
        json!({
            "type": "document",
            "version": "mycel/0.1",
            "doc_id": "doc:test",
            "title": "Plain document",
            "language": "zh-Hant",
            "content_model": "block-tree",
            "created_at": 1u64,
            "created_by": "pk:ed25519:test",
            "genesis_revision": "hash:test"
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
            .is_some_and(|errors| errors.iter().any(|entry| {
                entry.as_str().is_some_and(|message| {
                    message.contains("top-level 'genesis_revision' must use 'rev:' prefix")
                })
            })),
        "expected genesis_revision prefix error, stdout: {}",
        stdout_text(&output)
    );
}

#[test]
fn object_verify_json_fails_for_duplicate_object_keys() {
    let object = write_raw_object_file(
        "object-verify-duplicate-keys",
        "document.json",
        r#"{
  "type": "document",
  "version": "mycel/0.1",
  "doc_id": "doc:first",
  "doc_id": "doc:second",
  "title": "Duplicate key object"
}"#,
    );
    let path = path_arg(&object.path);
    let output = run_mycel(&["object", "verify", &path, "--json"]);

    assert_exit_code(&output, 1);
    let json = assert_json_status(&output, "failed");
    assert!(
        json["errors"]
            .as_array()
            .is_some_and(|errors| errors.iter().any(|entry| {
                entry.as_str().is_some_and(|message| {
                    message.contains("failed to parse JSON: duplicate object key 'doc_id'")
                })
            })),
        "expected duplicate-key parse error, stdout: {}",
        stdout_text(&output)
    );
}

#[test]
fn object_verify_json_fails_for_document_missing_title() {
    let object = write_object_file(
        "object-verify-document-missing-title",
        "document.json",
        json!({
            "type": "document",
            "version": "mycel/0.1",
            "doc_id": "doc:test",
            "language": "zh-Hant",
            "content_model": "block-tree",
            "created_at": 1u64,
            "created_by": "pk:ed25519:test",
            "genesis_revision": "rev:test"
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
            .is_some_and(|errors| errors.iter().any(|entry| {
                entry
                    .as_str()
                    .is_some_and(|message| message.contains("missing string field 'title'"))
            })),
        "expected missing title error, stdout: {}",
        stdout_text(&output)
    );
}

#[test]
fn object_verify_json_fails_for_block_missing_block_id() {
    let object = write_object_file(
        "object-verify-block-missing-block-id",
        "block.json",
        json!({
            "type": "block",
            "version": "mycel/0.1",
            "block_type": "paragraph",
            "content": "Hello",
            "attrs": {},
            "children": []
        }),
    );
    let path = path_arg(&object.path);
    let output = run_mycel(&["object", "verify", &path, "--json"]);

    assert_exit_code(&output, 1);
    let json = assert_json_status(&output, "failed");
    assert_eq!(json["object_type"], "block");
    assert!(
        json["errors"]
            .as_array()
            .is_some_and(
                |errors| errors
                    .iter()
                    .any(|entry| entry.as_str().is_some_and(|message| message
                        .contains("block object is missing string field 'block_id'")))
            ),
        "expected missing block_id error, stdout: {}",
        stdout_text(&output)
    );
}

#[test]
fn object_verify_json_fails_for_block_with_non_string_block_id() {
    let object = write_object_file(
        "object-verify-block-non-string-block-id",
        "block.json",
        json!({
            "type": "block",
            "version": "mycel/0.1",
            "block_id": 7,
            "block_type": "paragraph",
            "content": "Hello",
            "attrs": {},
            "children": []
        }),
    );
    let path = path_arg(&object.path);
    let output = run_mycel(&["object", "verify", &path, "--json"]);

    assert_exit_code(&output, 1);
    let json = assert_json_status(&output, "failed");
    assert_eq!(json["object_type"], "block");
    assert!(
        json["errors"]
            .as_array()
            .is_some_and(|errors| errors.iter().any(|entry| {
                entry.as_str().is_some_and(|message| {
                    message.contains("top-level 'block_id' must be a string")
                })
            })),
        "expected block_id type error, stdout: {}",
        stdout_text(&output)
    );
}

#[test]
fn object_verify_json_fails_for_block_missing_attrs() {
    let object = write_object_file(
        "object-verify-block-missing-attrs",
        "block.json",
        json!({
            "type": "block",
            "version": "mycel/0.1",
            "block_id": "blk:001",
            "block_type": "paragraph",
            "content": "Hello",
            "children": []
        }),
    );
    let path = path_arg(&object.path);
    let output = run_mycel(&["object", "verify", &path, "--json"]);

    assert_exit_code(&output, 1);
    let json = assert_json_status(&output, "failed");
    assert_eq!(json["object_type"], "block");
    assert!(
        json["errors"]
            .as_array()
            .is_some_and(|errors| errors.iter().any(|entry| {
                entry
                    .as_str()
                    .is_some_and(|message| message.contains("missing object field 'attrs'"))
            })),
        "expected missing attrs error, stdout: {}",
        stdout_text(&output)
    );
}

#[test]
fn object_verify_json_fails_for_block_with_wrong_block_id_prefix() {
    let object = write_object_file(
        "object-verify-block-wrong-block-id-prefix",
        "block.json",
        json!({
            "type": "block",
            "version": "mycel/0.1",
            "block_id": "paragraph-1",
            "block_type": "paragraph",
            "content": "Hello",
            "attrs": {},
            "children": []
        }),
    );
    let path = path_arg(&object.path);
    let output = run_mycel(&["object", "verify", &path, "--json"]);

    assert_exit_code(&output, 1);
    let json = assert_json_status(&output, "failed");
    assert_eq!(json["object_type"], "block");
    assert!(
        json["errors"]
            .as_array()
            .is_some_and(|errors| errors.iter().any(|entry| {
                entry.as_str().is_some_and(|message| {
                    message.contains("top-level 'block_id' must use 'blk:' prefix")
                })
            })),
        "expected block_id prefix error, stdout: {}",
        stdout_text(&output)
    );
}

#[test]
fn object_verify_json_fails_for_block_with_unknown_top_level_field() {
    let object = write_object_file(
        "object-verify-block-unknown-top-level-field",
        "block.json",
        json!({
            "type": "block",
            "version": "mycel/0.1",
            "block_id": "blk:001",
            "block_type": "paragraph",
            "content": "Hello",
            "attrs": {},
            "children": [],
            "unexpected": true
        }),
    );
    let path = path_arg(&object.path);
    let output = run_mycel(&["object", "verify", &path, "--json"]);

    assert_exit_code(&output, 1);
    let json = assert_json_status(&output, "failed");
    assert_eq!(json["object_type"], "block");
    assert!(
        json["errors"]
            .as_array()
            .is_some_and(|errors| errors.iter().any(|entry| {
                entry.as_str().is_some_and(|message| {
                    message.contains("top-level contains unexpected field 'unexpected'")
                })
            })),
        "expected unknown-field error, stdout: {}",
        stdout_text(&output)
    );
}

#[test]
fn object_verify_json_fails_for_block_with_unknown_nested_child_field() {
    let object = write_object_file(
        "object-verify-block-unknown-nested-child-field",
        "block.json",
        json!({
            "type": "block",
            "version": "mycel/0.1",
            "block_id": "blk:001",
            "block_type": "paragraph",
            "content": "Hello",
            "attrs": {},
            "children": [
                {
                    "block_id": "blk:002",
                    "block_type": "paragraph",
                    "content": "Child",
                    "attrs": {},
                    "children": [],
                    "unexpected": true
                }
            ]
        }),
    );
    let path = path_arg(&object.path);
    let output = run_mycel(&["object", "verify", &path, "--json"]);

    assert_exit_code(&output, 1);
    let json = assert_json_status(&output, "failed");
    assert_eq!(json["object_type"], "block");
    assert!(
        json["errors"]
            .as_array()
            .is_some_and(|errors| errors.iter().any(|entry| {
                entry.as_str().is_some_and(|message| {
                    message
                        .contains("top-level 'children[0]' contains unexpected field 'unexpected'")
                })
            })),
        "expected nested child unknown-field error, stdout: {}",
        stdout_text(&output)
    );
}

#[test]
fn object_verify_json_fails_for_floating_point_values() {
    let object = write_raw_object_file(
        "object-verify-float-value",
        "document.json",
        r#"{
  "type": "document",
  "version": "mycel/0.1",
  "doc_id": "doc:test",
  "priority": 1.5
}"#,
    );
    let path = path_arg(&object.path);
    let output = run_mycel(&["object", "verify", &path, "--json"]);

    assert_exit_code(&output, 1);
    let json = assert_json_status(&output, "failed");
    assert!(
        json["errors"]
            .as_array()
            .is_some_and(
                |errors| errors
                    .iter()
                    .any(|entry| entry.as_str().is_some_and(|message| {
                        message.contains("$.priority: floating-point numbers are not allowed")
                    }))
            ),
        "expected floating-point validation error, stdout: {}",
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
fn object_verify_json_fails_for_revision_with_wrong_revision_id_prefix() {
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
    revision["revision_id"] = Value::String(
        revision["revision_id"]
            .as_str()
            .expect("revision_id should exist")
            .replacen("rev:", "patch:", 1),
    );
    revision["signature"] = Value::String(sign_value(&signing_key(), &revision));
    let object = write_object_file(
        "object-verify-revision-wrong-derived-id-prefix",
        "revision.json",
        revision,
    );
    let path = path_arg(&object.path);
    let output = run_mycel(&["object", "verify", &path, "--json"]);

    assert_exit_code(&output, 1);
    let json = assert_json_status(&output, "failed");
    assert_eq!(json["object_type"], "revision");
    assert!(
        json["errors"]
            .as_array()
            .is_some_and(|errors| errors.iter().any(|entry| {
                entry.as_str().is_some_and(|message| {
                    message.contains("top-level 'revision_id' must use 'rev:' prefix")
                })
            })),
        "expected revision_id prefix error, stdout: {}",
        stdout_text(&output)
    );
}

#[test]
fn object_verify_json_fails_for_revision_with_non_string_revision_id() {
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
    revision["revision_id"] = json!(7);
    let object = write_object_file(
        "object-verify-revision-non-string-derived-id",
        "revision.json",
        revision,
    );
    let path = path_arg(&object.path);
    let output = run_mycel(&["object", "verify", &path, "--json"]);

    assert_exit_code(&output, 1);
    let json = assert_json_status(&output, "failed");
    assert_eq!(json["object_type"], "revision");
    assert!(
        json["errors"]
            .as_array()
            .is_some_and(|errors| errors.iter().any(|entry| {
                entry.as_str().is_some_and(|message| {
                    message.contains("top-level 'revision_id' must be a string")
                })
            })),
        "expected revision_id type error, stdout: {}",
        stdout_text(&output)
    );
}

#[test]
fn object_verify_json_fails_for_duplicate_revision_parent_ids() {
    let revision = signed_object(
        json!({
            "type": "revision",
            "version": "mycel/0.1",
            "doc_id": "doc:test",
            "parents": ["rev:base", "rev:base"],
            "patches": [],
            "state_hash": "hash:test-state",
            "timestamp": 1777778890u64
        }),
        "author",
        "revision_id",
        "rev",
    );
    let object = write_object_file(
        "object-verify-revision-duplicate-parents",
        "revision.json",
        revision,
    );
    let path = path_arg(&object.path);
    let output = run_mycel(&["object", "verify", &path, "--json"]);

    assert_exit_code(&output, 1);
    let json = assert_json_status(&output, "failed");
    assert_eq!(json["object_type"], "revision");
    assert!(
        json["errors"]
            .as_array()
            .is_some_and(|errors| errors.iter().any(|entry| {
                entry.as_str().is_some_and(|message| {
                    message.contains("top-level 'parents[1]' duplicates 'parents[0]'")
                })
            })),
        "expected duplicate parent error, stdout: {}",
        stdout_text(&output)
    );
}

#[test]
fn object_verify_json_fails_for_duplicate_revision_patch_ids() {
    let revision = signed_object(
        json!({
            "type": "revision",
            "version": "mycel/0.1",
            "doc_id": "doc:test",
            "parents": ["rev:base"],
            "patches": ["patch:test", "patch:test"],
            "state_hash": "hash:test-state",
            "timestamp": 1777778890u64
        }),
        "author",
        "revision_id",
        "rev",
    );
    let object = write_object_file(
        "object-verify-revision-duplicate-patches",
        "revision.json",
        revision,
    );
    let path = path_arg(&object.path);
    let output = run_mycel(&["object", "verify", &path, "--json"]);

    assert_exit_code(&output, 1);
    let json = assert_json_status(&output, "failed");
    assert_eq!(json["object_type"], "revision");
    assert!(
        json["errors"]
            .as_array()
            .is_some_and(|errors| errors.iter().any(|entry| {
                entry.as_str().is_some_and(|message| {
                    message.contains("top-level 'patches[1]' duplicates 'patches[0]'")
                })
            })),
        "expected duplicate patch error, stdout: {}",
        stdout_text(&output)
    );
}

#[test]
fn object_verify_json_fails_for_revision_with_wrong_parent_prefix() {
    let revision = signed_object(
        json!({
            "type": "revision",
            "version": "mycel/0.1",
            "doc_id": "doc:test",
            "parents": ["hash:base"],
            "patches": [],
            "state_hash": "hash:test-state",
            "timestamp": 1777778890u64
        }),
        "author",
        "revision_id",
        "rev",
    );
    let object = write_object_file(
        "object-verify-revision-wrong-parent-prefix",
        "revision.json",
        revision,
    );
    let path = path_arg(&object.path);
    let output = run_mycel(&["object", "verify", &path, "--json"]);

    assert_exit_code(&output, 1);
    let json = assert_json_status(&output, "failed");
    assert_eq!(json["object_type"], "revision");
    assert!(
        json["errors"]
            .as_array()
            .is_some_and(|errors| errors.iter().any(|entry| {
                entry.as_str().is_some_and(|message| {
                    message.contains("top-level 'parents[0]' must use 'rev:' prefix")
                })
            })),
        "expected parent prefix error, stdout: {}",
        stdout_text(&output)
    );
}

#[test]
fn object_verify_json_fails_for_genesis_revision_with_merge_strategy() {
    let revision = signed_object(
        json!({
            "type": "revision",
            "version": "mycel/0.1",
            "doc_id": "doc:test",
            "parents": [],
            "patches": [],
            "merge_strategy": "semantic-block-merge",
            "state_hash": "hash:test-state",
            "timestamp": 1777778890u64
        }),
        "author",
        "revision_id",
        "rev",
    );
    let object = write_object_file(
        "object-verify-revision-genesis-merge-strategy",
        "revision.json",
        revision,
    );
    let path = path_arg(&object.path);
    let output = run_mycel(&["object", "verify", &path, "--json"]);

    assert_exit_code(&output, 1);
    let json = assert_json_status(&output, "failed");
    assert_eq!(json["object_type"], "revision");
    assert!(
        json["errors"]
            .as_array()
            .is_some_and(|errors| errors.iter().any(|entry| {
                entry.as_str().is_some_and(|message| {
                    message.contains(
                        "top-level 'merge_strategy' is not allowed when 'parents' is empty",
                    )
                })
            })),
        "expected genesis merge_strategy error, stdout: {}",
        stdout_text(&output)
    );
}

#[test]
fn object_verify_json_fails_for_mismatched_snapshot_id() {
    let mut snapshot = signed_object(
        json!({
            "type": "snapshot",
            "version": "mycel/0.1",
            "documents": {
                "doc:test": "rev:test"
            },
            "included_objects": ["rev:test", "patch:test"],
            "root_hash": "hash:test",
            "timestamp": 9u64
        }),
        "created_by",
        "snapshot_id",
        "snap",
    );
    snapshot["snapshot_id"] = Value::String("snap:wrong".to_string());
    snapshot["signature"] = Value::String(sign_value(&signing_key(), &snapshot));
    let object = write_object_file("object-verify-snapshot-mismatch", "snapshot.json", snapshot);
    let path = path_arg(&object.path);
    let output = run_mycel(&["object", "verify", &path, "--json"]);

    assert_exit_code(&output, 1);
    let json = assert_json_status(&output, "failed");
    assert_eq!(json["object_type"], "snapshot");
    assert!(
        json["errors"]
            .as_array()
            .is_some_and(|errors| errors.iter().any(|entry| entry
                .as_str()
                .is_some_and(|message| message.contains("declared snapshot_id does not match")))),
        "expected snapshot derived ID mismatch error, stdout: {}",
        stdout_text(&output)
    );
}

#[test]
fn object_verify_json_fails_for_snapshot_with_wrong_snapshot_id_prefix() {
    let mut snapshot = signed_object(
        json!({
            "type": "snapshot",
            "version": "mycel/0.1",
            "documents": {
                "doc:test": "rev:test"
            },
            "included_objects": ["rev:test", "patch:test"],
            "root_hash": "hash:test",
            "timestamp": 9u64
        }),
        "created_by",
        "snapshot_id",
        "snap",
    );
    snapshot["snapshot_id"] = Value::String(
        snapshot["snapshot_id"]
            .as_str()
            .expect("snapshot_id should exist")
            .replacen("snap:", "view:", 1),
    );
    snapshot["signature"] = Value::String(sign_value(&signing_key(), &snapshot));
    let object = write_object_file(
        "object-verify-snapshot-wrong-derived-id-prefix",
        "snapshot.json",
        snapshot,
    );
    let path = path_arg(&object.path);
    let output = run_mycel(&["object", "verify", &path, "--json"]);

    assert_exit_code(&output, 1);
    let json = assert_json_status(&output, "failed");
    assert_eq!(json["object_type"], "snapshot");
    assert!(
        json["errors"]
            .as_array()
            .is_some_and(|errors| errors.iter().any(|entry| {
                entry.as_str().is_some_and(|message| {
                    message.contains("top-level 'snapshot_id' must use 'snap:' prefix")
                })
            })),
        "expected snapshot_id prefix error, stdout: {}",
        stdout_text(&output)
    );
}

#[test]
fn object_verify_json_fails_for_duplicate_snapshot_included_objects() {
    let snapshot = signed_object(
        json!({
            "type": "snapshot",
            "version": "mycel/0.1",
            "documents": {
                "doc:test": "rev:test"
            },
            "included_objects": ["rev:test", "rev:test"],
            "root_hash": "hash:test",
            "timestamp": 9u64
        }),
        "created_by",
        "snapshot_id",
        "snap",
    );
    let object = write_object_file(
        "object-verify-snapshot-duplicate-included-objects",
        "snapshot.json",
        snapshot,
    );
    let path = path_arg(&object.path);
    let output = run_mycel(&["object", "verify", &path, "--json"]);

    assert_exit_code(&output, 1);
    let json = assert_json_status(&output, "failed");
    assert_eq!(json["object_type"], "snapshot");
    assert!(
        json["errors"]
            .as_array()
            .is_some_and(|errors| errors.iter().any(|entry| {
                entry.as_str().is_some_and(|message| {
                    message.contains(
                        "top-level 'included_objects[1]' duplicates 'included_objects[0]'",
                    )
                })
            })),
        "expected duplicate included_objects error, stdout: {}",
        stdout_text(&output)
    );
}

#[test]
fn object_verify_json_fails_for_snapshot_with_empty_included_object_entry() {
    let snapshot = signed_object(
        json!({
            "type": "snapshot",
            "version": "mycel/0.1",
            "documents": {
                "doc:test": "rev:test"
            },
            "included_objects": ["rev:test", ""],
            "root_hash": "hash:test",
            "timestamp": 9u64
        }),
        "created_by",
        "snapshot_id",
        "snap",
    );
    let object = write_object_file(
        "object-verify-snapshot-empty-included-object-entry",
        "snapshot.json",
        snapshot,
    );
    let path = path_arg(&object.path);
    let output = run_mycel(&["object", "verify", &path, "--json"]);

    assert_exit_code(&output, 1);
    let json = assert_json_status(&output, "failed");
    assert_eq!(json["object_type"], "snapshot");
    assert!(
        json["errors"]
            .as_array()
            .is_some_and(|errors| errors.iter().any(|entry| {
                entry.as_str().is_some_and(|message| {
                    message.contains("top-level 'included_objects[1]' must not be an empty string")
                })
            })),
        "expected empty included_objects entry error, stdout: {}",
        stdout_text(&output)
    );
}

#[test]
fn object_verify_json_fails_for_snapshot_with_non_canonical_included_object_id() {
    let snapshot = signed_object(
        json!({
            "type": "snapshot",
            "version": "mycel/0.1",
            "documents": {
                "doc:test": "rev:test"
            },
            "included_objects": ["doc:test"],
            "root_hash": "hash:test",
            "timestamp": 9u64
        }),
        "created_by",
        "snapshot_id",
        "snap",
    );
    let object = write_object_file(
        "object-verify-snapshot-non-canonical-included-object-id",
        "snapshot.json",
        snapshot,
    );
    let path = path_arg(&object.path);
    let output = run_mycel(&["object", "verify", &path, "--json"]);

    assert_exit_code(&output, 1);
    let json = assert_json_status(&output, "failed");
    assert_eq!(json["object_type"], "snapshot");
    assert!(
        json["errors"]
            .as_array()
            .is_some_and(|errors| errors.iter().any(|entry| {
                entry.as_str().is_some_and(|message| {
                    message.contains(
                        "top-level 'included_objects[0]' must use a canonical object ID prefix",
                    )
                })
            })),
        "expected canonical included_objects ID error, stdout: {}",
        stdout_text(&output)
    );
}

#[test]
fn object_verify_json_fails_for_snapshot_missing_documents() {
    let snapshot = signed_object(
        json!({
            "type": "snapshot",
            "version": "mycel/0.1",
            "included_objects": ["rev:test"],
            "root_hash": "hash:test",
            "timestamp": 9u64
        }),
        "created_by",
        "snapshot_id",
        "snap",
    );
    let object = write_object_file(
        "object-verify-snapshot-missing-documents",
        "snapshot.json",
        snapshot,
    );
    let path = path_arg(&object.path);
    let output = run_mycel(&["object", "verify", &path, "--json"]);

    assert_exit_code(&output, 1);
    let json = assert_json_status(&output, "failed");
    assert_eq!(json["object_type"], "snapshot");
    assert!(
        json["errors"]
            .as_array()
            .is_some_and(|errors| errors.iter().any(|entry| {
                entry
                    .as_str()
                    .is_some_and(|message| message.contains("missing object field 'documents'"))
            })),
        "expected missing documents error, stdout: {}",
        stdout_text(&output)
    );
}

#[test]
fn object_verify_json_fails_for_snapshot_with_empty_documents() {
    let snapshot = signed_object(
        json!({
            "type": "snapshot",
            "version": "mycel/0.1",
            "documents": {},
            "included_objects": ["rev:test"],
            "root_hash": "hash:test",
            "timestamp": 9u64
        }),
        "created_by",
        "snapshot_id",
        "snap",
    );
    let object = write_object_file(
        "object-verify-snapshot-empty-documents",
        "snapshot.json",
        snapshot,
    );
    let path = path_arg(&object.path);
    let output = run_mycel(&["object", "verify", &path, "--json"]);

    assert_exit_code(&output, 1);
    let json = assert_json_status(&output, "failed");
    assert_eq!(json["object_type"], "snapshot");
    assert!(
        json["errors"]
            .as_array()
            .is_some_and(|errors| errors.iter().any(|entry| {
                entry.as_str().is_some_and(|message| {
                    message.contains("top-level 'documents' must not be empty")
                })
            })),
        "expected empty documents error, stdout: {}",
        stdout_text(&output)
    );
}

#[test]
fn object_verify_json_fails_for_snapshot_missing_declared_revision_in_included_objects() {
    let snapshot = signed_object(
        json!({
            "type": "snapshot",
            "version": "mycel/0.1",
            "documents": {
                "doc:test": "rev:test"
            },
            "included_objects": ["patch:test"],
            "root_hash": "hash:test",
            "timestamp": 1777778890u64
        }),
        "created_by",
        "snapshot_id",
        "snap",
    );
    let object = write_object_file(
        "object-verify-snapshot-missing-declared-revision",
        "snapshot.json",
        snapshot,
    );
    let path = path_arg(&object.path);
    let output = run_mycel(&["object", "verify", &path, "--json"]);

    assert_exit_code(&output, 1);
    let json = assert_json_status(&output, "failed");
    assert_eq!(json["object_type"], "snapshot");
    assert!(
        json["errors"]
            .as_array()
            .is_some_and(|errors| errors.iter().any(|entry| {
                entry.as_str().is_some_and(|message| {
                    message.contains(
                        "top-level 'included_objects' must include revision 'rev:test' declared by 'documents.doc:test'",
                    )
                })
            })),
        "expected missing declared revision error, stdout: {}",
        stdout_text(&output)
    );
}

#[test]
fn object_verify_json_fails_for_snapshot_with_wrong_root_hash_prefix() {
    let snapshot = signed_object(
        json!({
            "type": "snapshot",
            "version": "mycel/0.1",
            "documents": {
                "doc:test": "rev:test"
            },
            "included_objects": ["rev:test", "patch:test"],
            "root_hash": "rev:test",
            "timestamp": 1777778890u64
        }),
        "created_by",
        "snapshot_id",
        "snap",
    );
    let object = write_object_file(
        "object-verify-snapshot-wrong-root-hash-prefix",
        "snapshot.json",
        snapshot,
    );
    let path = path_arg(&object.path);
    let output = run_mycel(&["object", "verify", &path, "--json"]);

    assert_exit_code(&output, 1);
    let json = assert_json_status(&output, "failed");
    assert_eq!(json["object_type"], "snapshot");
    assert!(
        json["errors"]
            .as_array()
            .is_some_and(|errors| errors.iter().any(|entry| {
                entry.as_str().is_some_and(|message| {
                    message.contains("top-level 'root_hash' must use 'hash:' prefix")
                })
            })),
        "expected root_hash prefix error, stdout: {}",
        stdout_text(&output)
    );
}

#[test]
fn object_verify_json_fails_for_snapshot_with_wrong_document_value_prefix() {
    let snapshot = signed_object(
        json!({
            "type": "snapshot",
            "version": "mycel/0.1",
            "documents": {
                "doc:test": "patch:test"
            },
            "included_objects": ["patch:test"],
            "root_hash": "hash:test",
            "timestamp": 1777778890u64
        }),
        "created_by",
        "snapshot_id",
        "snap",
    );
    let object = write_object_file(
        "object-verify-snapshot-wrong-document-value-prefix",
        "snapshot.json",
        snapshot,
    );
    let path = path_arg(&object.path);
    let output = run_mycel(&["object", "verify", &path, "--json"]);

    assert_exit_code(&output, 1);
    let json = assert_json_status(&output, "failed");
    assert_eq!(json["object_type"], "snapshot");
    assert!(
        json["errors"]
            .as_array()
            .is_some_and(|errors| errors.iter().any(|entry| {
                entry.as_str().is_some_and(|message| {
                    message.contains("top-level 'documents.doc:test' must use 'rev:' prefix")
                })
            })),
        "expected snapshot document revision-prefix error, stdout: {}",
        stdout_text(&output)
    );
}

#[test]
fn object_verify_json_fails_for_snapshot_with_wrong_document_key_prefix() {
    let snapshot = signed_object(
        json!({
            "type": "snapshot",
            "version": "mycel/0.1",
            "documents": {
                "patch:test": "rev:test"
            },
            "included_objects": ["rev:test", "patch:test"],
            "root_hash": "hash:test",
            "timestamp": 1777778890u64
        }),
        "created_by",
        "snapshot_id",
        "snap",
    );
    let object = write_object_file(
        "object-verify-snapshot-wrong-document-key-prefix",
        "snapshot.json",
        snapshot,
    );
    let path = path_arg(&object.path);
    let output = run_mycel(&["object", "verify", &path, "--json"]);

    assert_exit_code(&output, 1);
    let json = assert_json_status(&output, "failed");
    assert_eq!(json["object_type"], "snapshot");
    assert!(
        json["errors"]
            .as_array()
            .is_some_and(|errors| errors.iter().any(|entry| {
                entry.as_str().is_some_and(|message| {
                    message.contains("top-level 'documents.patch:test key' must use 'doc:' prefix")
                })
            })),
        "expected snapshot document key-prefix error, stdout: {}",
        stdout_text(&output)
    );
}

#[test]
fn object_verify_json_fails_for_snapshot_with_wrong_created_by_prefix() {
    let mut snapshot = signed_object(
        json!({
            "type": "snapshot",
            "version": "mycel/0.1",
            "documents": {
                "doc:test": "rev:test"
            },
            "included_objects": ["rev:test", "patch:test"],
            "root_hash": "hash:test",
            "timestamp": 1777778890u64
        }),
        "created_by",
        "snapshot_id",
        "snap",
    );
    snapshot["created_by"] = Value::String("sig:bad".to_string());
    let object = write_object_file(
        "object-verify-snapshot-wrong-created-by-prefix",
        "snapshot.json",
        snapshot,
    );
    let path = path_arg(&object.path);
    let output = run_mycel(&["object", "verify", &path, "--json"]);

    assert_exit_code(&output, 1);
    let json = assert_json_status(&output, "failed");
    assert_eq!(json["object_type"], "snapshot");
    assert!(
        json["errors"]
            .as_array()
            .is_some_and(|errors| errors.iter().any(|entry| {
                entry.as_str().is_some_and(|message| {
                    message.contains("signer field must use format 'pk:ed25519:<base64>'")
                })
            })),
        "expected created_by signer-format error, stdout: {}",
        stdout_text(&output)
    );
}

#[test]
fn object_verify_json_fails_for_snapshot_missing_created_by() {
    let mut snapshot = signed_object(
        json!({
            "type": "snapshot",
            "version": "mycel/0.1",
            "documents": {
                "doc:test": "rev:test"
            },
            "included_objects": ["rev:test", "patch:test"],
            "root_hash": "hash:test",
            "timestamp": 1777778890u64
        }),
        "created_by",
        "snapshot_id",
        "snap",
    );
    snapshot
        .as_object_mut()
        .expect("snapshot should be an object")
        .remove("created_by");
    let object = write_object_file(
        "object-verify-snapshot-missing-created-by",
        "snapshot.json",
        snapshot,
    );
    let path = path_arg(&object.path);
    let output = run_mycel(&["object", "verify", &path, "--json"]);

    assert_exit_code(&output, 1);
    let json = assert_json_status(&output, "failed");
    assert_eq!(json["object_type"], "snapshot");
    assert!(
        json["errors"]
            .as_array()
            .is_some_and(|errors| errors.iter().any(|entry| {
                entry.as_str().is_some_and(|message| {
                    message.contains("snapshot object is missing string signer field 'created_by'")
                })
            })),
        "expected missing created_by signer-field error, stdout: {}",
        stdout_text(&output)
    );
}

#[test]
fn object_verify_json_fails_for_snapshot_with_non_string_snapshot_id() {
    let mut snapshot = json!({
        "type": "snapshot",
        "version": "mycel/0.1",
        "documents": {
            "doc:test": "rev:test"
        },
        "included_objects": ["rev:test", "patch:test"],
        "root_hash": "hash:test",
        "created_by": signer_id(&signing_key()),
        "timestamp": 9u64,
        "snapshot_id": 7
    });
    snapshot["signature"] = Value::String(sign_value(&signing_key(), &snapshot));
    let object = write_object_file(
        "object-verify-snapshot-non-string-id",
        "snapshot.json",
        snapshot,
    );
    let path = path_arg(&object.path);
    let output = run_mycel(&["object", "verify", &path, "--json"]);

    assert_exit_code(&output, 1);
    let json = assert_json_status(&output, "failed");
    assert_eq!(json["object_type"], "snapshot");
    assert!(
        json["errors"]
            .as_array()
            .is_some_and(|errors| errors.iter().any(|entry| {
                entry.as_str().is_some_and(|message| {
                    message.contains("top-level 'snapshot_id' must be a string")
                })
            })),
        "expected snapshot_id type error, stdout: {}",
        stdout_text(&output)
    );
}

#[test]
fn object_verify_json_fails_for_snapshot_with_unknown_top_level_field() {
    let snapshot = signed_object(
        json!({
            "type": "snapshot",
            "version": "mycel/0.1",
            "documents": {
                "doc:test": "rev:test"
            },
            "included_objects": ["rev:test", "patch:test"],
            "root_hash": "hash:test",
            "timestamp": 9u64,
            "unexpected": true
        }),
        "created_by",
        "snapshot_id",
        "snap",
    );
    let object = write_object_file(
        "object-verify-snapshot-unknown-top-level-field",
        "snapshot.json",
        snapshot,
    );
    let path = path_arg(&object.path);
    let output = run_mycel(&["object", "verify", &path, "--json"]);

    assert_exit_code(&output, 1);
    let json = assert_json_status(&output, "failed");
    assert_eq!(json["object_type"], "snapshot");
    assert!(
        json["errors"]
            .as_array()
            .is_some_and(|errors| errors.iter().any(|entry| {
                entry.as_str().is_some_and(|message| {
                    message.contains("top-level contains unexpected field 'unexpected'")
                })
            })),
        "expected unknown-field error, stdout: {}",
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
fn object_verify_json_fails_for_view_with_wrong_maintainer_prefix() {
    let mut view = json!({
        "type": "view",
        "version": "mycel/0.1",
        "maintainer": "sig:bad",
        "documents": {
            "doc:test": "rev:test"
        },
        "policy": {
            "merge_rule": "manual-reviewed"
        },
        "timestamp": 1777778891u64
    });
    let view_id = recompute_id(&view, "view_id", "view");
    view["view_id"] = Value::String(view_id);
    view["signature"] = Value::String(sign_value(&signing_key(), &view));
    let object = write_object_file(
        "object-verify-view-wrong-maintainer-prefix",
        "view.json",
        view,
    );
    let path = path_arg(&object.path);
    let output = run_mycel(&["object", "verify", &path, "--json"]);

    assert_exit_code(&output, 1);
    let json = assert_json_status(&output, "failed");
    assert_eq!(json["object_type"], "view");
    assert!(
        json["errors"]
            .as_array()
            .is_some_and(|errors| errors.iter().any(|entry| {
                entry.as_str().is_some_and(|message| {
                    message.contains("signer field must use format 'pk:ed25519:<base64>'")
                })
            })),
        "expected maintainer signer-format error, stdout: {}",
        stdout_text(&output)
    );
}

#[test]
fn object_verify_json_fails_for_view_missing_maintainer() {
    let mut view = signed_object(
        json!({
            "type": "view",
            "version": "mycel/0.1",
            "documents": {
                "doc:test": "rev:test"
            },
            "policy": {
                "merge_rule": "manual-reviewed"
            },
            "timestamp": 1777778891u64
        }),
        "maintainer",
        "view_id",
        "view",
    );
    view.as_object_mut()
        .expect("view should be an object")
        .remove("maintainer");
    let object = write_object_file("object-verify-view-missing-maintainer", "view.json", view);
    let path = path_arg(&object.path);
    let output = run_mycel(&["object", "verify", &path, "--json"]);

    assert_exit_code(&output, 1);
    let json = assert_json_status(&output, "failed");
    assert_eq!(json["object_type"], "view");
    assert!(
        json["errors"]
            .as_array()
            .is_some_and(|errors| errors.iter().any(|entry| {
                entry.as_str().is_some_and(|message| {
                    message.contains("view object is missing string signer field 'maintainer'")
                })
            })),
        "expected missing maintainer signer-field error, stdout: {}",
        stdout_text(&output)
    );
}

#[test]
fn object_verify_json_fails_for_view_with_wrong_view_id_prefix() {
    let mut view = json!({
        "type": "view",
        "version": "mycel/0.1",
        "maintainer": signer_id(&signing_key()),
        "documents": {
            "doc:test": "rev:test"
        },
        "policy": {
            "merge_rule": "manual-reviewed"
        },
        "timestamp": 1777778891u64
    });
    let view_id = recompute_id(&view, "view_id", "view");
    view["view_id"] = Value::String(view_id.replacen("view:", "snap:", 1));
    view["signature"] = Value::String(sign_value(&signing_key(), &view));
    let object = write_object_file(
        "object-verify-view-wrong-derived-id-prefix",
        "view.json",
        view,
    );
    let path = path_arg(&object.path);
    let output = run_mycel(&["object", "verify", &path, "--json"]);

    assert_exit_code(&output, 1);
    let json = assert_json_status(&output, "failed");
    assert_eq!(json["object_type"], "view");
    assert!(
        json["errors"]
            .as_array()
            .is_some_and(|errors| errors.iter().any(|entry| {
                entry.as_str().is_some_and(|message| {
                    message.contains("top-level 'view_id' must use 'view:' prefix")
                })
            })),
        "expected view_id prefix error, stdout: {}",
        stdout_text(&output)
    );
}

#[test]
fn object_verify_json_fails_for_view_with_non_string_view_id() {
    let mut view = json!({
        "type": "view",
        "version": "mycel/0.1",
        "maintainer": signer_id(&signing_key()),
        "documents": {
            "doc:test": "rev:test"
        },
        "policy": {
            "merge_rule": "manual-reviewed"
        },
        "timestamp": 1777778891u64
    });
    view["view_id"] = Value::String(recompute_id(&view, "view_id", "view"));
    view["signature"] = Value::String(sign_value(&signing_key(), &view));
    view["view_id"] = json!(7);
    let object = write_object_file(
        "object-verify-view-non-string-derived-id",
        "view.json",
        view,
    );
    let path = path_arg(&object.path);
    let output = run_mycel(&["object", "verify", &path, "--json"]);

    assert_exit_code(&output, 1);
    let json = assert_json_status(&output, "failed");
    assert_eq!(json["object_type"], "view");
    assert!(
        json["errors"]
            .as_array()
            .is_some_and(|errors| errors.iter().any(|entry| {
                entry
                    .as_str()
                    .is_some_and(|message| message.contains("top-level 'view_id' must be a string"))
            })),
        "expected view_id type error, stdout: {}",
        stdout_text(&output)
    );
}

#[test]
fn object_verify_json_fails_for_view_with_non_object_policy() {
    let view = signed_object(
        json!({
            "type": "view",
            "version": "mycel/0.1",
            "documents": {
                "doc:test": "rev:test"
            },
            "policy": "manual-reviewed",
            "timestamp": 1777778891u64
        }),
        "maintainer",
        "view_id",
        "view",
    );
    let object = write_object_file("object-verify-view-non-object-policy", "view.json", view);
    let path = path_arg(&object.path);
    let output = run_mycel(&["object", "verify", &path, "--json"]);

    assert_exit_code(&output, 1);
    let json = assert_json_status(&output, "failed");
    assert_eq!(json["object_type"], "view");
    assert!(
        json["errors"]
            .as_array()
            .is_some_and(|errors| errors.iter().any(|entry| {
                entry
                    .as_str()
                    .is_some_and(|message| message.contains("top-level 'policy' must be an object"))
            })),
        "expected non-object policy error, stdout: {}",
        stdout_text(&output)
    );
}

#[test]
fn object_verify_json_fails_for_view_with_empty_documents() {
    let view = signed_object(
        json!({
            "type": "view",
            "version": "mycel/0.1",
            "documents": {},
            "policy": {
                "merge_rule": "manual-reviewed"
            },
            "timestamp": 1777778891u64
        }),
        "maintainer",
        "view_id",
        "view",
    );
    let object = write_object_file("object-verify-view-empty-documents", "view.json", view);
    let path = path_arg(&object.path);
    let output = run_mycel(&["object", "verify", &path, "--json"]);

    assert_exit_code(&output, 1);
    let json = assert_json_status(&output, "failed");
    assert_eq!(json["object_type"], "view");
    assert!(
        json["errors"]
            .as_array()
            .is_some_and(|errors| errors.iter().any(|entry| {
                entry.as_str().is_some_and(|message| {
                    message.contains("top-level 'documents' must not be empty")
                })
            })),
        "expected empty documents error, stdout: {}",
        stdout_text(&output)
    );
}

#[test]
fn object_verify_json_fails_for_view_with_wrong_document_value_prefix() {
    let view = signed_object(
        json!({
            "type": "view",
            "version": "mycel/0.1",
            "documents": {
                "doc:test": "patch:test"
            },
            "policy": {
                "merge_rule": "manual-reviewed"
            },
            "timestamp": 1777778891u64
        }),
        "maintainer",
        "view_id",
        "view",
    );
    let object = write_object_file(
        "object-verify-view-wrong-document-value-prefix",
        "view.json",
        view,
    );
    let path = path_arg(&object.path);
    let output = run_mycel(&["object", "verify", &path, "--json"]);

    assert_exit_code(&output, 1);
    let json = assert_json_status(&output, "failed");
    assert_eq!(json["object_type"], "view");
    assert!(
        json["errors"]
            .as_array()
            .is_some_and(|errors| errors.iter().any(|entry| {
                entry.as_str().is_some_and(|message| {
                    message.contains("top-level 'documents.doc:test' must use 'rev:' prefix")
                })
            })),
        "expected view document revision-prefix error, stdout: {}",
        stdout_text(&output)
    );
}

#[test]
fn object_verify_json_fails_for_view_with_wrong_document_key_prefix() {
    let view = signed_object(
        json!({
            "type": "view",
            "version": "mycel/0.1",
            "documents": {
                "patch:test": "rev:test"
            },
            "policy": {
                "merge_rule": "manual-reviewed"
            },
            "timestamp": 1777778891u64
        }),
        "maintainer",
        "view_id",
        "view",
    );
    let object = write_object_file(
        "object-verify-view-wrong-document-key-prefix",
        "view.json",
        view,
    );
    let path = path_arg(&object.path);
    let output = run_mycel(&["object", "verify", &path, "--json"]);

    assert_exit_code(&output, 1);
    let json = assert_json_status(&output, "failed");
    assert_eq!(json["object_type"], "view");
    assert!(
        json["errors"]
            .as_array()
            .is_some_and(|errors| errors.iter().any(|entry| {
                entry.as_str().is_some_and(|message| {
                    message.contains("top-level 'documents.patch:test key' must use 'doc:' prefix")
                })
            })),
        "expected view document key-prefix error, stdout: {}",
        stdout_text(&output)
    );
}

#[test]
fn object_verify_json_fails_for_view_with_unknown_top_level_field() {
    let view = signed_object(
        json!({
            "type": "view",
            "version": "mycel/0.1",
            "documents": {
                "doc:test": "rev:test"
            },
            "policy": {
                "merge_rule": "manual-reviewed"
            },
            "timestamp": 1777778891u64,
            "unexpected": true
        }),
        "maintainer",
        "view_id",
        "view",
    );
    let object = write_object_file(
        "object-verify-view-unknown-top-level-field",
        "view.json",
        view,
    );
    let path = path_arg(&object.path);
    let output = run_mycel(&["object", "verify", &path, "--json"]);

    assert_exit_code(&output, 1);
    let json = assert_json_status(&output, "failed");
    assert_eq!(json["object_type"], "view");
    assert!(
        json["errors"]
            .as_array()
            .is_some_and(|errors| errors.iter().any(|entry| {
                entry.as_str().is_some_and(|message| {
                    message.contains("top-level contains unexpected field 'unexpected'")
                })
            })),
        "expected unknown-field error, stdout: {}",
        stdout_text(&output)
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
fn object_verify_json_fails_for_patch_op_unknown_field() {
    let patch = signed_object(
        json!({
            "type": "patch",
            "version": "mycel/0.1",
            "doc_id": "doc:test",
            "base_revision": "rev:genesis-null",
            "timestamp": 1777778888u64,
            "ops": [
                {
                    "op": "delete_block",
                    "block_id": "blk:001",
                    "new_content": "unexpected"
                }
            ]
        }),
        "author",
        "patch_id",
        "patch",
    );
    let object = write_object_file("object-verify-patch-op-unknown-field", "patch.json", patch);
    let path = path_arg(&object.path);
    let output = run_mycel(&["object", "verify", &path, "--json"]);

    assert_exit_code(&output, 1);
    let json = assert_json_status(&output, "failed");
    assert_eq!(json["object_type"], "patch");
    assert!(
        json["errors"]
            .as_array()
            .is_some_and(|errors| errors.iter().any(|entry| {
                entry.as_str().is_some_and(|message| {
                    message.contains("patch op contains unexpected field 'new_content'")
                })
            })),
        "expected patch-op unknown-field error, stdout: {}",
        stdout_text(&output)
    );
}

#[test]
fn object_verify_json_fails_for_patch_nested_block_shape_with_path() {
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
                        "content": "Hello"
                    }
                }
            ]
        }),
        "author",
        "patch_id",
        "patch",
    );
    let object = write_object_file(
        "object-verify-patch-nested-block-shape",
        "patch.json",
        patch,
    );
    let path = path_arg(&object.path);
    let output = run_mycel(&["object", "verify", &path, "--json"]);

    assert_exit_code(&output, 1);
    let json = assert_json_status(&output, "failed");
    assert_eq!(json["object_type"], "patch");
    assert!(
        json["errors"]
            .as_array()
            .is_some_and(|errors| errors.iter().any(|entry| {
                entry.as_str().is_some_and(|message| {
                    message.contains("top-level 'ops[0]': top-level 'new_block'")
                        && message.contains("missing object field 'attrs'")
                })
            })),
        "expected nested block shape error with path, stdout: {}",
        stdout_text(&output)
    );
}

#[test]
fn object_verify_json_fails_for_patch_with_wrong_base_revision_prefix() {
    let patch = signed_object(
        json!({
            "type": "patch",
            "version": "mycel/0.1",
            "doc_id": "doc:test",
            "base_revision": "hash:base",
            "timestamp": 1777778888u64,
            "ops": []
        }),
        "author",
        "patch_id",
        "patch",
    );
    let object = write_object_file(
        "object-verify-patch-wrong-base-revision-prefix",
        "patch.json",
        patch,
    );
    let path = path_arg(&object.path);
    let output = run_mycel(&["object", "verify", &path, "--json"]);

    assert_exit_code(&output, 1);
    let json = assert_json_status(&output, "failed");
    assert_eq!(json["object_type"], "patch");
    assert!(
        json["errors"]
            .as_array()
            .is_some_and(|errors| errors.iter().any(|entry| {
                entry.as_str().is_some_and(|message| {
                    message.contains("top-level 'base_revision' must use 'rev:' prefix")
                })
            })),
        "expected base_revision prefix error, stdout: {}",
        stdout_text(&output)
    );
}

#[test]
fn object_verify_json_fails_for_patch_with_wrong_block_reference_prefix() {
    let patch = signed_object(
        json!({
            "type": "patch",
            "version": "mycel/0.1",
            "doc_id": "doc:test",
            "base_revision": "rev:genesis-null",
            "timestamp": 1777778888u64,
            "ops": [
                {
                    "op": "replace_block",
                    "block_id": "paragraph-1",
                    "new_content": "Hello"
                }
            ]
        }),
        "author",
        "patch_id",
        "patch",
    );
    let object = write_object_file(
        "object-verify-patch-wrong-block-reference-prefix",
        "patch.json",
        patch,
    );
    let path = path_arg(&object.path);
    let output = run_mycel(&["object", "verify", &path, "--json"]);

    assert_exit_code(&output, 1);
    let json = assert_json_status(&output, "failed");
    assert_eq!(json["object_type"], "patch");
    assert!(
        json["errors"]
            .as_array()
            .is_some_and(|errors| errors.iter().any(|entry| {
                entry.as_str().is_some_and(|message| {
                    message
                        .contains("top-level 'ops[0]': top-level 'block_id' must use 'blk:' prefix")
                })
            })),
        "expected block reference prefix error, stdout: {}",
        stdout_text(&output)
    );
}

#[test]
fn object_verify_json_fails_for_patch_with_wrong_author_prefix() {
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
    patch["author"] = Value::String("author:test".to_string());
    let object = write_object_file(
        "object-verify-patch-wrong-author-prefix",
        "patch.json",
        patch,
    );
    let path = path_arg(&object.path);
    let output = run_mycel(&["object", "verify", &path, "--json"]);

    assert_exit_code(&output, 1);
    let json = assert_json_status(&output, "failed");
    assert_eq!(json["object_type"], "patch");
    assert!(
        json["errors"]
            .as_array()
            .is_some_and(|errors| errors.iter().any(|entry| {
                entry.as_str().is_some_and(|message| {
                    message.contains("signer field must use format 'pk:ed25519:<base64>'")
                })
            })),
        "expected signer-format error, stdout: {}",
        stdout_text(&output)
    );
}

#[test]
fn object_verify_json_fails_for_patch_missing_author() {
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
    patch
        .as_object_mut()
        .expect("patch should be an object")
        .remove("author");
    let object = write_object_file("object-verify-patch-missing-author", "patch.json", patch);
    let path = path_arg(&object.path);
    let output = run_mycel(&["object", "verify", &path, "--json"]);

    assert_exit_code(&output, 1);
    let json = assert_json_status(&output, "failed");
    assert_eq!(json["object_type"], "patch");
    assert!(
        json["errors"]
            .as_array()
            .is_some_and(|errors| errors.iter().any(|entry| {
                entry.as_str().is_some_and(|message| {
                    message.contains("patch object is missing string signer field 'author'")
                })
            })),
        "expected missing author signer-field error, stdout: {}",
        stdout_text(&output)
    );
}

#[test]
fn object_verify_json_fails_for_patch_with_wrong_patch_id_prefix() {
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
    patch["patch_id"] = Value::String(
        patch["patch_id"]
            .as_str()
            .expect("patch_id should exist")
            .replacen("patch:", "rev:", 1),
    );
    patch["signature"] = Value::String(sign_value(&signing_key(), &patch));
    let object = write_object_file(
        "object-verify-patch-wrong-derived-id-prefix",
        "patch.json",
        patch,
    );
    let path = path_arg(&object.path);
    let output = run_mycel(&["object", "verify", &path, "--json"]);

    assert_exit_code(&output, 1);
    let json = assert_json_status(&output, "failed");
    assert_eq!(json["object_type"], "patch");
    assert!(
        json["errors"]
            .as_array()
            .is_some_and(|errors| errors.iter().any(|entry| {
                entry.as_str().is_some_and(|message| {
                    message.contains("top-level 'patch_id' must use 'patch:' prefix")
                })
            })),
        "expected patch_id prefix error, stdout: {}",
        stdout_text(&output)
    );
}

#[test]
fn object_verify_json_fails_for_patch_with_non_string_patch_id() {
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
    patch["patch_id"] = json!(7);
    let object = write_object_file(
        "object-verify-patch-non-string-derived-id",
        "patch.json",
        patch,
    );
    let path = path_arg(&object.path);
    let output = run_mycel(&["object", "verify", &path, "--json"]);

    assert_exit_code(&output, 1);
    let json = assert_json_status(&output, "failed");
    assert_eq!(json["object_type"], "patch");
    assert!(
        json["errors"]
            .as_array()
            .is_some_and(|errors| errors.iter().any(|entry| {
                entry.as_str().is_some_and(|message| {
                    message.contains("top-level 'patch_id' must be a string")
                })
            })),
        "expected patch_id type error, stdout: {}",
        stdout_text(&output)
    );
}

#[test]
fn object_verify_json_fails_for_patch_with_unknown_top_level_field() {
    let patch = signed_object(
        json!({
            "type": "patch",
            "version": "mycel/0.1",
            "doc_id": "doc:test",
            "base_revision": "rev:genesis-null",
            "timestamp": 1777778888u64,
            "ops": [],
            "unexpected": true
        }),
        "author",
        "patch_id",
        "patch",
    );
    let object = write_object_file(
        "object-verify-patch-unknown-top-level-field",
        "patch.json",
        patch,
    );
    let path = path_arg(&object.path);
    let output = run_mycel(&["object", "verify", &path, "--json"]);

    assert_exit_code(&output, 1);
    let json = assert_json_status(&output, "failed");
    assert_eq!(json["object_type"], "patch");
    assert!(
        json["errors"]
            .as_array()
            .is_some_and(|errors| errors.iter().any(|entry| {
                entry.as_str().is_some_and(|message| {
                    message.contains("top-level contains unexpected field 'unexpected'")
                })
            })),
        "expected unknown top-level field error, stdout: {}",
        stdout_text(&output)
    );
}

#[test]
fn object_verify_json_fails_for_patch_move_without_destination() {
    let patch = signed_object(
        json!({
            "type": "patch",
            "version": "mycel/0.1",
            "doc_id": "doc:test",
            "base_revision": "rev:genesis-null",
            "timestamp": 1777778888u64,
            "ops": [
                {
                    "op": "move_block",
                    "block_id": "blk:001"
                }
            ]
        }),
        "author",
        "patch_id",
        "patch",
    );
    let object = write_object_file(
        "object-verify-patch-move-without-destination",
        "patch.json",
        patch,
    );
    let path = path_arg(&object.path);
    let output = run_mycel(&["object", "verify", &path, "--json"]);

    assert_exit_code(&output, 1);
    let json = assert_json_status(&output, "failed");
    assert_eq!(json["object_type"], "patch");
    assert!(
        json["errors"]
            .as_array()
            .is_some_and(|errors| errors.iter().any(|entry| {
                entry.as_str().is_some_and(|message| {
                    message.contains(
                        "top-level 'ops[0]': move_block requires at least one destination reference",
                    )
                })
            })),
        "expected move_block destination error, stdout: {}",
        stdout_text(&output)
    );
}

#[test]
fn object_verify_json_fails_for_patch_with_mixed_set_metadata_forms() {
    let patch = signed_object(
        json!({
            "type": "patch",
            "version": "mycel/0.1",
            "doc_id": "doc:test",
            "base_revision": "rev:genesis-null",
            "timestamp": 1777778888u64,
            "ops": [
                {
                    "op": "set_metadata",
                    "metadata": {
                        "title": "Hello"
                    },
                    "key": "extra"
                }
            ]
        }),
        "author",
        "patch_id",
        "patch",
    );
    let object = write_object_file(
        "object-verify-patch-mixed-set-metadata-forms",
        "patch.json",
        patch,
    );
    let path = path_arg(&object.path);
    let output = run_mycel(&["object", "verify", &path, "--json"]);

    assert_exit_code(&output, 1);
    let json = assert_json_status(&output, "failed");
    assert_eq!(json["object_type"], "patch");
    assert!(
        json["errors"]
            .as_array()
            .is_some_and(|errors| errors.iter().any(|entry| {
                entry.as_str().is_some_and(|message| {
                    message.contains("top-level 'ops[0]': patch op contains unexpected field 'key'")
                })
            })),
        "expected mixed set_metadata forms error, stdout: {}",
        stdout_text(&output)
    );
}

#[test]
fn object_verify_json_fails_for_patch_with_empty_metadata_entries() {
    let patch = signed_object(
        json!({
            "type": "patch",
            "version": "mycel/0.1",
            "doc_id": "doc:test",
            "base_revision": "rev:genesis-null",
            "timestamp": 1777778888u64,
            "ops": [
                {
                    "op": "set_metadata",
                    "metadata": {}
                }
            ]
        }),
        "author",
        "patch_id",
        "patch",
    );
    let object = write_object_file(
        "object-verify-patch-empty-metadata-entries",
        "patch.json",
        patch,
    );
    let path = path_arg(&object.path);
    let output = run_mycel(&["object", "verify", &path, "--json"]);

    assert_exit_code(&output, 1);
    let json = assert_json_status(&output, "failed");
    assert_eq!(json["object_type"], "patch");
    assert!(
        json["errors"]
            .as_array()
            .is_some_and(|errors| errors.iter().any(|entry| {
                entry.as_str().is_some_and(|message| {
                    message.contains("top-level 'ops[0]': top-level 'metadata' must not be empty")
                })
            })),
        "expected empty metadata error, stdout: {}",
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

#[test]
fn object_verify_json_fails_for_revision_with_wrong_state_hash_prefix() {
    let revision = signed_object(
        json!({
            "type": "revision",
            "version": "mycel/0.1",
            "doc_id": "doc:test",
            "parents": ["rev:base"],
            "patches": [],
            "state_hash": "rev:test",
            "timestamp": 1777778890u64
        }),
        "author",
        "revision_id",
        "rev",
    );
    let object = write_object_file(
        "object-verify-revision-wrong-state-hash-prefix",
        "revision.json",
        revision,
    );
    let path = path_arg(&object.path);
    let output = run_mycel(&["object", "verify", &path, "--json"]);

    assert_exit_code(&output, 1);
    let json = assert_json_status(&output, "failed");
    assert_eq!(json["object_type"], "revision");
    assert!(
        json["errors"]
            .as_array()
            .is_some_and(|errors| errors.iter().any(|entry| {
                entry.as_str().is_some_and(|message| {
                    message.contains("top-level 'state_hash' must use 'hash:' prefix")
                })
            })),
        "expected state_hash prefix error, stdout: {}",
        stdout_text(&output)
    );
}

#[test]
fn object_verify_json_fails_for_revision_missing_state_hash() {
    let revision = signed_object(
        json!({
            "type": "revision",
            "version": "mycel/0.1",
            "doc_id": "doc:test",
            "parents": ["rev:base"],
            "patches": [],
            "timestamp": 1777778890u64
        }),
        "author",
        "revision_id",
        "rev",
    );
    let object = write_object_file(
        "object-verify-revision-missing-state-hash",
        "revision.json",
        revision,
    );
    let path = path_arg(&object.path);
    let output = run_mycel(&["object", "verify", &path, "--json"]);

    assert_exit_code(&output, 1);
    let json = assert_json_status(&output, "failed");
    assert_eq!(json["object_type"], "revision");
    assert!(
        json["errors"]
            .as_array()
            .is_some_and(|errors| errors.iter().any(|entry| {
                entry.as_str().is_some_and(|message| {
                    message.contains("revision object is missing string field 'state_hash'")
                })
            })),
        "expected missing state_hash error, stdout: {}",
        stdout_text(&output)
    );
}

#[test]
fn object_verify_json_fails_for_revision_with_wrong_author_prefix() {
    let mut revision = signed_object(
        json!({
            "type": "revision",
            "version": "mycel/0.1",
            "doc_id": "doc:test",
            "parents": ["rev:base"],
            "patches": [],
            "state_hash": "hash:test-state",
            "timestamp": 1777778890u64
        }),
        "author",
        "revision_id",
        "rev",
    );
    revision["author"] = Value::String("author:test".to_string());
    let object = write_object_file(
        "object-verify-revision-wrong-author-prefix",
        "revision.json",
        revision,
    );
    let path = path_arg(&object.path);
    let output = run_mycel(&["object", "verify", &path, "--json"]);

    assert_exit_code(&output, 1);
    let json = assert_json_status(&output, "failed");
    assert_eq!(json["object_type"], "revision");
    assert!(
        json["errors"]
            .as_array()
            .is_some_and(|errors| errors.iter().any(|entry| {
                entry.as_str().is_some_and(|message| {
                    message.contains("signer field must use format 'pk:ed25519:<base64>'")
                })
            })),
        "expected signer-format error, stdout: {}",
        stdout_text(&output)
    );
}

#[test]
fn object_verify_json_fails_for_revision_missing_author() {
    let mut revision = signed_object(
        json!({
            "type": "revision",
            "version": "mycel/0.1",
            "doc_id": "doc:test",
            "parents": ["rev:base"],
            "patches": [],
            "state_hash": "hash:test-state",
            "timestamp": 1777778890u64
        }),
        "author",
        "revision_id",
        "rev",
    );
    revision
        .as_object_mut()
        .expect("revision should be an object")
        .remove("author");
    let object = write_object_file(
        "object-verify-revision-missing-author",
        "revision.json",
        revision,
    );
    let path = path_arg(&object.path);
    let output = run_mycel(&["object", "verify", &path, "--json"]);

    assert_exit_code(&output, 1);
    let json = assert_json_status(&output, "failed");
    assert_eq!(json["object_type"], "revision");
    assert!(
        json["errors"]
            .as_array()
            .is_some_and(|errors| errors.iter().any(|entry| {
                entry.as_str().is_some_and(|message| {
                    message.contains("revision object is missing string signer field 'author'")
                })
            })),
        "expected missing author signer-field error, stdout: {}",
        stdout_text(&output)
    );
}

#[test]
fn object_verify_json_fails_for_genesis_revision_with_wrong_patch_base_revision() {
    let dir = create_temp_dir("object-verify-revision-genesis-base-mismatch");
    let patch_path = dir.path().join("patch.json");
    let revision_path = dir.path().join("revision.json");
    let patch = signed_object(
        json!({
            "type": "patch",
            "version": "mycel/0.1",
            "doc_id": "doc:test",
            "base_revision": "rev:wrong-base",
            "timestamp": 1777778888u64,
            "ops": []
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
            "state_hash": "hash:test",
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
            .is_some_and(|errors| errors.iter().any(|entry| {
                entry.as_str().is_some_and(|message| {
                    message.contains("base_revision 'rev:wrong-base'")
                        && message.contains("expected 'rev:genesis-null'")
                })
            })),
        "expected genesis base_revision mismatch error, stdout: {}",
        stdout_text(&output)
    );
}

#[test]
fn object_verify_json_fails_for_non_genesis_revision_with_wrong_patch_base_revision() {
    let dir = create_temp_dir("object-verify-revision-parent-base-mismatch");
    let base_revision_path = dir.path().join("revision-base.json");
    let patch_path = dir.path().join("patch.json");
    let revision_path = dir.path().join("revision.json");
    let base_revision = signed_object(
        json!({
            "type": "revision",
            "version": "mycel/0.1",
            "doc_id": "doc:test",
            "parents": [],
            "patches": [],
            "state_hash": state_hash(&json!({
                "doc_id": "doc:test",
                "blocks": []
            })),
            "timestamp": 1777778887u64
        }),
        "author",
        "revision_id",
        "rev",
    );
    fs::write(
        &base_revision_path,
        serde_json::to_string_pretty(&base_revision).expect("base revision JSON should serialize"),
    )
    .expect("base revision JSON should be written");
    let patch = signed_object(
        json!({
            "type": "patch",
            "version": "mycel/0.1",
            "doc_id": "doc:test",
            "base_revision": "rev:wrong-base",
            "timestamp": 1777778888u64,
            "ops": []
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
            "parents": [base_revision["revision_id"].as_str().expect("base revision id should exist")],
            "patches": [patch["patch_id"].as_str().expect("patch id should exist")],
            "state_hash": state_hash(&json!({
                "doc_id": "doc:test",
                "blocks": []
            })),
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
            .is_some_and(|errors| errors.iter().any(|entry| {
                entry.as_str().is_some_and(|message| {
                    message.contains("base_revision 'rev:wrong-base'")
                        && message.contains("expected '")
                        && message.contains("rev:")
                })
            })),
        "expected non-genesis base_revision mismatch error, stdout: {}",
        stdout_text(&output)
    );
}

#[test]
fn object_verify_json_fails_for_single_parent_revision_with_merge_strategy() {
    let dir = create_temp_dir("object-verify-revision-single-parent-merge-strategy");
    let revision_path = dir.path().join("revision.json");
    let revision = signed_object(
        json!({
            "type": "revision",
            "version": "mycel/0.1",
            "doc_id": "doc:test",
            "parents": ["rev:base"],
            "patches": [],
            "merge_strategy": "semantic-block-merge",
            "state_hash": "hash:test",
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
    assert!(
        json["errors"]
            .as_array()
            .is_some_and(|errors| errors.iter().any(|entry| {
                entry.as_str().is_some_and(|message| {
                    message.contains("top-level 'merge_strategy' requires multiple parents")
                })
            })),
        "expected merge_strategy parent-count error, stdout: {}",
        stdout_text(&output)
    );
}

#[test]
fn object_verify_json_fails_for_multi_parent_revision_without_merge_strategy() {
    let dir = create_temp_dir("object-verify-revision-missing-merge-strategy");
    let revision_path = dir.path().join("revision.json");
    let revision = signed_object(
        json!({
            "type": "revision",
            "version": "mycel/0.1",
            "doc_id": "doc:test",
            "parents": ["rev:base", "rev:side"],
            "patches": [],
            "state_hash": "hash:test",
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
    assert!(
        json["errors"]
            .as_array()
            .is_some_and(|errors| errors.iter().any(|entry| {
                entry.as_str().is_some_and(|message| {
                    message.contains(
                        "top-level 'merge_strategy' is required when 'parents' has multiple entries",
                    )
                })
            })),
        "expected missing merge_strategy error, stdout: {}",
        stdout_text(&output)
    );
}

#[test]
fn object_verify_json_reports_ok_for_valid_merge_revision() {
    let dir = create_temp_dir("object-verify-valid-merge-revision");
    let base_patch_path = dir.path().join("patch-base.json");
    let base_revision_path = dir.path().join("revision-base.json");
    let side_patch_path = dir.path().join("patch-side.json");
    let side_revision_path = dir.path().join("revision-side.json");
    let merge_patch_path = dir.path().join("patch-merge.json");
    let merge_revision_path = dir.path().join("revision-merge.json");

    let base_patch = signed_object(
        json!({
            "type": "patch",
            "version": "mycel/0.1",
            "doc_id": "doc:test",
            "base_revision": "rev:genesis-null",
            "timestamp": 1777778887u64,
            "ops": [
                {
                    "op": "insert_block",
                    "new_block": {
                        "block_id": "blk:001",
                        "block_type": "paragraph",
                        "content": "Base",
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
        &base_patch_path,
        serde_json::to_string_pretty(&base_patch).expect("base patch JSON should serialize"),
    )
    .expect("base patch JSON should be written");

    let base_revision = signed_object(
        json!({
            "type": "revision",
            "version": "mycel/0.1",
            "doc_id": "doc:test",
            "parents": [],
            "patches": [base_patch["patch_id"].as_str().expect("base patch id should exist")],
            "state_hash": state_hash(&json!({
                "doc_id": "doc:test",
                "blocks": [
                    {
                        "block_id": "blk:001",
                        "block_type": "paragraph",
                        "content": "Base",
                        "attrs": {},
                        "children": []
                    }
                ]
            })),
            "timestamp": 1777778888u64
        }),
        "author",
        "revision_id",
        "rev",
    );
    fs::write(
        &base_revision_path,
        serde_json::to_string_pretty(&base_revision).expect("base revision JSON should serialize"),
    )
    .expect("base revision JSON should be written");

    let side_patch = signed_object(
        json!({
            "type": "patch",
            "version": "mycel/0.1",
            "doc_id": "doc:test",
            "base_revision": "rev:genesis-null",
            "timestamp": 1777778889u64,
            "ops": [
                {
                    "op": "insert_block",
                    "new_block": {
                        "block_id": "blk:002",
                        "block_type": "paragraph",
                        "content": "Side",
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
        &side_patch_path,
        serde_json::to_string_pretty(&side_patch).expect("side patch JSON should serialize"),
    )
    .expect("side patch JSON should be written");

    let side_revision = signed_object(
        json!({
            "type": "revision",
            "version": "mycel/0.1",
            "doc_id": "doc:test",
            "parents": [],
            "patches": [side_patch["patch_id"].as_str().expect("side patch id should exist")],
            "state_hash": state_hash(&json!({
                "doc_id": "doc:test",
                "blocks": [
                    {
                        "block_id": "blk:002",
                        "block_type": "paragraph",
                        "content": "Side",
                        "attrs": {},
                        "children": []
                    }
                ]
            })),
            "timestamp": 1777778890u64
        }),
        "author",
        "revision_id",
        "rev",
    );
    fs::write(
        &side_revision_path,
        serde_json::to_string_pretty(&side_revision).expect("side revision JSON should serialize"),
    )
    .expect("side revision JSON should be written");

    let merge_patch = signed_object(
        json!({
            "type": "patch",
            "version": "mycel/0.1",
            "doc_id": "doc:test",
            "base_revision": base_revision["revision_id"].as_str().expect("base revision id should exist"),
            "timestamp": 1777778891u64,
            "ops": [
                {
                    "op": "replace_block",
                    "block_id": "blk:001",
                    "new_content": "Merged"
                }
            ]
        }),
        "author",
        "patch_id",
        "patch",
    );
    fs::write(
        &merge_patch_path,
        serde_json::to_string_pretty(&merge_patch).expect("merge patch JSON should serialize"),
    )
    .expect("merge patch JSON should be written");

    let merge_revision = signed_object(
        json!({
            "type": "revision",
            "version": "mycel/0.1",
            "doc_id": "doc:test",
            "parents": [
                base_revision["revision_id"].as_str().expect("base revision id should exist"),
                side_revision["revision_id"].as_str().expect("side revision id should exist")
            ],
            "patches": [merge_patch["patch_id"].as_str().expect("merge patch id should exist")],
            "merge_strategy": "semantic-block-merge",
            "state_hash": state_hash(&json!({
                "doc_id": "doc:test",
                "blocks": [
                    {
                        "block_id": "blk:001",
                        "block_type": "paragraph",
                        "content": "Merged",
                        "attrs": {},
                        "children": []
                    }
                ]
            })),
            "timestamp": 1777778892u64
        }),
        "author",
        "revision_id",
        "rev",
    );
    fs::write(
        &merge_revision_path,
        serde_json::to_string_pretty(&merge_revision)
            .expect("merge revision JSON should serialize"),
    )
    .expect("merge revision JSON should be written");

    let output = run_mycel(&[
        "object",
        "verify",
        &path_arg(&merge_revision_path),
        "--json",
    ]);

    assert_success(&output);
    let json = assert_json_status(&output, "ok");
    assert_eq!(json["object_type"], "revision");
    assert_eq!(json["state_hash_verification"], "verified");
    assert_eq!(json["declared_state_hash"], json["recomputed_state_hash"]);
}

#[test]
fn object_verify_json_fails_when_merge_revision_implicitly_includes_secondary_parent_content() {
    let dir = create_temp_dir("object-verify-merge-secondary-parent-content");
    let base_revision_path = dir.path().join("revision-base.json");
    let side_revision_path = dir.path().join("revision-side.json");
    let merge_revision_path = dir.path().join("revision-merge.json");

    let base_revision = signed_object(
        json!({
            "type": "revision",
            "version": "mycel/0.1",
            "doc_id": "doc:test",
            "parents": [],
            "patches": [],
            "state_hash": state_hash(&json!({
                "doc_id": "doc:test",
                "blocks": [
                    {
                        "block_id": "blk:001",
                        "block_type": "paragraph",
                        "content": "Base",
                        "attrs": {},
                        "children": []
                    }
                ]
            })),
            "timestamp": 1777778888u64
        }),
        "author",
        "revision_id",
        "rev",
    );
    fs::write(
        &base_revision_path,
        serde_json::to_string_pretty(&base_revision).expect("base revision JSON should serialize"),
    )
    .expect("base revision JSON should be written");

    let side_revision = signed_object(
        json!({
            "type": "revision",
            "version": "mycel/0.1",
            "doc_id": "doc:test",
            "parents": [],
            "patches": [],
            "state_hash": state_hash(&json!({
                "doc_id": "doc:test",
                "blocks": [
                    {
                        "block_id": "blk:002",
                        "block_type": "paragraph",
                        "content": "Side",
                        "attrs": {},
                        "children": []
                    }
                ]
            })),
            "timestamp": 1777778889u64
        }),
        "author",
        "revision_id",
        "rev",
    );
    fs::write(
        &side_revision_path,
        serde_json::to_string_pretty(&side_revision).expect("side revision JSON should serialize"),
    )
    .expect("side revision JSON should be written");

    let merge_revision = signed_object(
        json!({
            "type": "revision",
            "version": "mycel/0.1",
            "doc_id": "doc:test",
            "parents": [
                base_revision["revision_id"].as_str().expect("base revision id should exist"),
                side_revision["revision_id"].as_str().expect("side revision id should exist")
            ],
            "patches": [],
            "merge_strategy": "semantic-block-merge",
            "state_hash": state_hash(&json!({
                "doc_id": "doc:test",
                "blocks": [
                    {
                        "block_id": "blk:001",
                        "block_type": "paragraph",
                        "content": "Base",
                        "attrs": {},
                        "children": []
                    },
                    {
                        "block_id": "blk:002",
                        "block_type": "paragraph",
                        "content": "Side",
                        "attrs": {},
                        "children": []
                    }
                ]
            })),
            "timestamp": 1777778890u64
        }),
        "author",
        "revision_id",
        "rev",
    );
    fs::write(
        &merge_revision_path,
        serde_json::to_string_pretty(&merge_revision)
            .expect("merge revision JSON should serialize"),
    )
    .expect("merge revision JSON should be written");

    let output = run_mycel(&[
        "object",
        "verify",
        &path_arg(&merge_revision_path),
        "--json",
    ]);

    assert_exit_code(&output, 1);
    let json = assert_json_status(&output, "failed");
    assert_eq!(json["object_type"], "revision");
    assert_eq!(json["state_hash_verification"], "failed");
    assert!(
        json["errors"]
            .as_array()
            .is_some_and(|errors| errors.iter().any(|entry| {
                entry.as_str().is_some_and(|message| {
                    message.contains("declared state_hash does not match replayed state hash")
                })
            })),
        "expected ancestry-only state-hash mismatch, stdout: {}",
        stdout_text(&output)
    );
}

#[test]
fn object_verify_json_fails_for_revision_with_patch_from_other_document() {
    let dir = create_temp_dir("object-verify-revision-cross-document-patch");
    let patch_path = dir.path().join("patch-other-doc.json");
    let revision_path = dir.path().join("revision.json");

    let patch = signed_object(
        json!({
            "type": "patch",
            "version": "mycel/0.1",
            "doc_id": "doc:other",
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
            "timestamp": 1777778889u64
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
            .is_some_and(|errors| errors.iter().any(|entry| {
                entry.as_str().is_some_and(|message| {
                    message.contains("patch '")
                        && message.contains("belongs to 'doc:other' instead of 'doc:test'")
                })
            })),
        "expected cross-document replay error, stdout: {}",
        stdout_text(&output)
    );
}

#[test]
fn object_verify_json_fails_for_revision_with_missing_parent_revision() {
    let dir = create_temp_dir("object-verify-revision-missing-parent");
    let patch_path = dir.path().join("patch.json");
    let revision_path = dir.path().join("revision.json");

    let patch = signed_object(
        json!({
            "type": "patch",
            "version": "mycel/0.1",
            "doc_id": "doc:test",
            "base_revision": "rev:missing-parent",
            "timestamp": 1777778888u64,
            "ops": []
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
            "parents": ["rev:missing-parent"],
            "patches": [patch["patch_id"].as_str().expect("patch id should exist")],
            "state_hash": "hash:placeholder",
            "timestamp": 1777778889u64
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
            .is_some_and(|errors| errors.iter().any(|entry| {
                entry.as_str().is_some_and(|message| {
                    message.contains("missing parent revision 'rev:missing-parent' for replay")
                })
            })),
        "expected missing parent replay error, stdout: {}",
        stdout_text(&output)
    );
}

#[test]
fn object_verify_json_fails_for_revision_with_missing_patch_object() {
    let dir = create_temp_dir("object-verify-revision-missing-patch");
    let revision_path = dir.path().join("revision.json");

    let revision = signed_object(
        json!({
            "type": "revision",
            "version": "mycel/0.1",
            "doc_id": "doc:test",
            "parents": [],
            "patches": ["patch:missing"],
            "state_hash": "hash:placeholder",
            "timestamp": 1777778889u64
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
            .is_some_and(|errors| errors.iter().any(|entry| {
                entry.as_str().is_some_and(|message| {
                    message.contains("missing patch 'patch:missing' for replay")
                })
            })),
        "expected missing patch replay error, stdout: {}",
        stdout_text(&output)
    );
}

#[test]
fn object_verify_json_fails_for_revision_with_parent_from_other_document() {
    let dir = create_temp_dir("object-verify-revision-cross-document-parent");
    let parent_revision_path = dir.path().join("revision-parent.json");
    let revision_path = dir.path().join("revision.json");

    let parent_state_hash = state_hash(&json!({
        "doc_id": "doc:other",
        "blocks": [],
        "metadata": {}
    }));
    let parent_revision = signed_object(
        json!({
            "type": "revision",
            "version": "mycel/0.1",
            "doc_id": "doc:other",
            "parents": [],
            "patches": [],
            "state_hash": parent_state_hash,
            "timestamp": 1777778888u64
        }),
        "author",
        "revision_id",
        "rev",
    );
    fs::write(
        &parent_revision_path,
        serde_json::to_string_pretty(&parent_revision)
            .expect("parent revision JSON should serialize"),
    )
    .expect("parent revision JSON should be written");

    let revision = signed_object(
        json!({
            "type": "revision",
            "version": "mycel/0.1",
            "doc_id": "doc:test",
            "parents": [parent_revision["revision_id"].as_str().expect("parent revision id should exist")],
            "patches": [],
            "state_hash": "hash:placeholder",
            "timestamp": 1777778889u64
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
            .is_some_and(|errors| errors.iter().any(|entry| {
                entry.as_str().is_some_and(|message| {
                    message.contains("parent revision '")
                        && message.contains("belongs to 'doc:other' instead of 'doc:test'")
                })
            })),
        "expected cross-document parent replay error, stdout: {}",
        stdout_text(&output)
    );
}

#[test]
fn object_verify_json_fails_for_revision_with_unparseable_parent_revision() {
    let dir = create_temp_dir("object-verify-revision-unparseable-parent");
    let parent_revision_path = dir.path().join("revision-parent.json");
    let patch_path = dir.path().join("patch.json");
    let revision_path = dir.path().join("revision.json");

    let malformed_parent = json!({
        "type": "revision",
        "version": "mycel/0.1",
        "revision_id": "rev:bad-parent",
        "doc_id": "doc:test",
        "parents": [],
        "patches": [],
        "author": signer_id(&signing_key()),
        "timestamp": 1777778888u64
    });
    fs::write(
        &parent_revision_path,
        serde_json::to_string_pretty(&malformed_parent)
            .expect("parent revision JSON should serialize"),
    )
    .expect("parent revision JSON should be written");

    let patch = signed_object(
        json!({
            "type": "patch",
            "version": "mycel/0.1",
            "doc_id": "doc:test",
            "base_revision": "rev:bad-parent",
            "timestamp": 1777778889u64,
            "ops": []
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
            "parents": ["rev:bad-parent"],
            "patches": [patch["patch_id"].as_str().expect("patch id should exist")],
            "state_hash": "hash:placeholder",
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
            .is_some_and(|errors| errors.iter().any(|entry| {
                entry.as_str().is_some_and(|message| {
                    message.contains("failed to parse parent revision 'rev:bad-parent'")
                        && message.contains("missing string field 'state_hash'")
                })
            })),
        "expected parent parse replay error, stdout: {}",
        stdout_text(&output)
    );
}

#[test]
fn object_verify_json_fails_for_revision_with_unparseable_patch_dependency() {
    let dir = create_temp_dir("object-verify-revision-unparseable-patch");
    let patch_path = dir.path().join("patch.json");
    let revision_path = dir.path().join("revision.json");

    let malformed_patch = json!({
        "type": "patch",
        "version": "mycel/0.1",
        "patch_id": "patch:bad",
        "doc_id": "doc:test",
        "author": signer_id(&signing_key()),
        "timestamp": 1777778888u64,
        "ops": []
    });
    fs::write(
        &patch_path,
        serde_json::to_string_pretty(&malformed_patch).expect("patch JSON should serialize"),
    )
    .expect("patch JSON should be written");

    let revision = signed_object(
        json!({
            "type": "revision",
            "version": "mycel/0.1",
            "doc_id": "doc:test",
            "parents": [],
            "patches": ["patch:bad"],
            "state_hash": "hash:placeholder",
            "timestamp": 1777778889u64
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
            .is_some_and(|errors| errors.iter().any(|entry| {
                entry
                    .as_str()
                    .is_some_and(|message| message.contains("failed to parse patch 'patch:bad'"))
            })),
        "expected patch parse replay error, stdout: {}",
        stdout_text(&output)
    );
}

#[test]
fn object_verify_json_fails_for_revision_with_invalid_move_cycle() {
    let dir = create_temp_dir("object-verify-revision-move-cycle");
    let parent_patch_path = dir.path().join("patch-parent.json");
    let child_patch_path = dir.path().join("patch-child.json");
    let base_revision_path = dir.path().join("revision-base.json");
    let move_patch_path = dir.path().join("patch-move.json");
    let moved_revision_path = dir.path().join("revision-move.json");

    let parent_patch = signed_object(
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
                        "content": "Parent",
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
        &parent_patch_path,
        serde_json::to_string_pretty(&parent_patch).expect("parent patch JSON should serialize"),
    )
    .expect("parent patch JSON should be written");

    let child_patch = signed_object(
        json!({
            "type": "patch",
            "version": "mycel/0.1",
            "doc_id": "doc:test",
            "base_revision": "rev:genesis-null",
            "timestamp": 1777778889u64,
            "ops": [
                {
                    "op": "insert_block",
                    "parent_block_id": "blk:001",
                    "new_block": {
                        "block_id": "blk:002",
                        "block_type": "paragraph",
                        "content": "Child",
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
        &child_patch_path,
        serde_json::to_string_pretty(&child_patch).expect("child patch JSON should serialize"),
    )
    .expect("child patch JSON should be written");

    let base_state_hash = state_hash(&json!({
        "doc_id": "doc:test",
        "blocks": [
            {
                "block_id": "blk:001",
                "block_type": "paragraph",
                "content": "Parent",
                "attrs": {},
                "children": [
                    {
                        "block_id": "blk:002",
                        "block_type": "paragraph",
                        "content": "Child",
                        "attrs": {},
                        "children": []
                    }
                ]
            }
        ]
    }));
    let base_revision = signed_object(
        json!({
            "type": "revision",
            "version": "mycel/0.1",
            "doc_id": "doc:test",
            "parents": [],
            "patches": [
                parent_patch["patch_id"].as_str().expect("parent patch id should exist"),
                child_patch["patch_id"].as_str().expect("child patch id should exist")
            ],
            "state_hash": base_state_hash,
            "timestamp": 1777778890u64
        }),
        "author",
        "revision_id",
        "rev",
    );
    fs::write(
        &base_revision_path,
        serde_json::to_string_pretty(&base_revision).expect("base revision JSON should serialize"),
    )
    .expect("base revision JSON should be written");

    let move_patch = signed_object(
        json!({
            "type": "patch",
            "version": "mycel/0.1",
            "doc_id": "doc:test",
            "base_revision": base_revision["revision_id"].as_str().expect("base revision id should exist"),
            "timestamp": 1777778891u64,
            "ops": [
                {
                    "op": "move_block",
                    "block_id": "blk:001",
                    "parent_block_id": "blk:002"
                }
            ]
        }),
        "author",
        "patch_id",
        "patch",
    );
    fs::write(
        &move_patch_path,
        serde_json::to_string_pretty(&move_patch).expect("move patch JSON should serialize"),
    )
    .expect("move patch JSON should be written");

    let moved_revision = signed_object(
        json!({
            "type": "revision",
            "version": "mycel/0.1",
            "doc_id": "doc:test",
            "parents": [base_revision["revision_id"].as_str().expect("base revision id should exist")],
            "patches": [move_patch["patch_id"].as_str().expect("move patch id should exist")],
            "state_hash": "hash:placeholder",
            "timestamp": 1777778892u64
        }),
        "author",
        "revision_id",
        "rev",
    );
    fs::write(
        &moved_revision_path,
        serde_json::to_string_pretty(&moved_revision)
            .expect("moved revision JSON should serialize"),
    )
    .expect("moved revision JSON should be written");

    let output = run_mycel(&[
        "object",
        "verify",
        &path_arg(&moved_revision_path),
        "--json",
    ]);

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
                        message.contains("move_block destination parent cannot be the moved block")
                    }))
            ),
        "expected move-cycle replay error, stdout: {}",
        stdout_text(&output)
    );
}
