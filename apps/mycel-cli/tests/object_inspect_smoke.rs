use std::fs;
use std::path::PathBuf;

use serde_json::{json, Value};

mod common;

use common::{
    assert_empty_stderr, assert_exit_code, assert_json_status, assert_stderr_contains,
    assert_stdout_contains, assert_success, create_temp_dir, run_mycel, stdout_text,
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

#[test]
fn object_inspect_json_reports_ok_for_document() {
    let object = write_object_file(
        "object-inspect-document",
        "document.json",
        json!({
            "type": "document",
            "version": "mycel/0.1",
            "doc_id": "doc:test",
            "title": "Plain document"
        }),
    );
    let path = path_arg(&object.path);
    let output = run_mycel(&["object", "inspect", &path, "--json"]);

    assert_success(&output);
    let json = assert_json_status(&output, "ok");
    assert_eq!(json["object_type"], "document");
    assert_eq!(json["version"], "mycel/0.1");
    assert_eq!(json["signature_rule"], "forbidden");
    assert_eq!(json["has_signature"], false);
    assert_eq!(
        json["top_level_keys"],
        json!(["doc_id", "title", "type", "version"])
    );
}

#[test]
fn object_inspect_text_warns_for_patch_missing_signature() {
    let object = write_object_file(
        "object-inspect-patch-warning",
        "patch.json",
        json!({
            "type": "patch",
            "version": "mycel/0.1",
            "doc_id": "doc:test",
            "base_revision": "rev:genesis-null",
            "timestamp": 1777778888u64,
            "ops": []
        }),
    );
    let path = path_arg(&object.path);
    let output = run_mycel(&["object", "inspect", &path]);

    assert_success(&output);
    assert_empty_stderr(&output);
    assert_stdout_contains(&output, "object type: patch");
    assert_stdout_contains(&output, "signature rule: required");
    assert_stdout_contains(&output, "has signature: no");
    assert_stdout_contains(&output, "inspection: warning");
    assert_stdout_contains(
        &output,
        "note: patch object is missing string signer field 'author'",
    );
    assert_stdout_contains(
        &output,
        "note: patch object is missing top-level 'signature'",
    );
}

#[test]
fn object_inspect_json_warns_for_unsupported_type() {
    let object = write_object_file(
        "object-inspect-unsupported",
        "custom.json",
        json!({
            "type": "custom-object",
            "version": "mycel/0.1",
            "title": "Experimental object"
        }),
    );
    let path = path_arg(&object.path);
    let output = run_mycel(&["object", "inspect", &path, "--json"]);

    assert_success(&output);
    let json = assert_json_status(&output, "warning");
    assert_eq!(json["object_type"], "custom-object");
    assert!(
        json["notes"]
            .as_array()
            .is_some_and(|notes| notes
                .iter()
                .any(|entry| entry.as_str().is_some_and(
                    |message| message.contains("unsupported object type 'custom-object'")
                ))),
        "expected unsupported-type note, stdout: {}",
        stdout_text(&output)
    );
}

#[test]
fn object_inspect_json_warns_for_document_with_non_string_doc_id() {
    let object = write_object_file(
        "object-inspect-document-wrong-id-type",
        "document.json",
        json!({
            "type": "document",
            "version": "mycel/0.1",
            "doc_id": 7,
            "title": "Hello"
        }),
    );
    let path = path_arg(&object.path);
    let output = run_mycel(&["object", "inspect", &path, "--json"]);

    assert_success(&output);
    let json = assert_json_status(&output, "warning");
    assert_eq!(json["object_type"], "document");
    assert!(
        json["notes"]
            .as_array()
            .is_some_and(|notes| notes.iter().any(|entry| entry
                .as_str()
                .is_some_and(|message| message.contains("top-level 'doc_id' should be a string")))),
        "expected doc_id warning, stdout: {}",
        stdout_text(&output)
    );
}

#[test]
fn object_inspect_json_warns_for_block_with_non_string_block_id() {
    let object = write_object_file(
        "object-inspect-block-wrong-id-type",
        "block.json",
        json!({
            "type": "block",
            "version": "mycel/0.1",
            "block_id": 7,
            "text": "Hello"
        }),
    );
    let path = path_arg(&object.path);
    let output = run_mycel(&["object", "inspect", &path, "--json"]);

    assert_success(&output);
    let json = assert_json_status(&output, "warning");
    assert_eq!(json["object_type"], "block");
    assert!(
        json["notes"]
            .as_array()
            .is_some_and(|notes| notes
                .iter()
                .any(|entry| entry.as_str().is_some_and(
                    |message| message.contains("top-level 'block_id' should be a string")
                ))),
        "expected block_id warning, stdout: {}",
        stdout_text(&output)
    );
}

#[test]
fn object_inspect_json_warns_for_view_with_wrong_document_key_prefix() {
    let object = write_object_file(
        "object-inspect-view-wrong-document-key-prefix",
        "view.json",
        json!({
            "type": "view",
            "version": "mycel/0.1",
            "view_id": "view:test",
            "maintainer": "pk:ed25519:test",
            "documents": {
                "patch:test": "rev:test"
            },
            "policy": {
                "merge_rule": "manual-reviewed"
            },
            "timestamp": 12u64
        }),
    );
    let path = path_arg(&object.path);
    let output = run_mycel(&["object", "inspect", &path, "--json"]);

    assert_success(&output);
    let json = assert_json_status(&output, "warning");
    assert_eq!(json["object_type"], "view");
    assert!(
        json["notes"].as_array().is_some_and(|notes| notes
            .iter()
            .any(|entry| entry.as_str().is_some_and(|message| message
                .contains("top-level 'documents.patch:test key' must use 'doc:' prefix")))),
        "expected view documents key-prefix warning, stdout: {}",
        stdout_text(&output)
    );
}

#[test]
fn object_inspect_json_warns_for_snapshot_with_wrong_document_value_prefix() {
    let object = write_object_file(
        "object-inspect-snapshot-wrong-document-value-prefix",
        "snapshot.json",
        json!({
            "type": "snapshot",
            "version": "mycel/0.1",
            "snapshot_id": "snap:test",
            "documents": {
                "doc:test": "patch:test"
            },
            "included_objects": ["patch:test"],
            "root_hash": "hash:test",
            "created_by": "pk:ed25519:test",
            "timestamp": 9u64
        }),
    );
    let path = path_arg(&object.path);
    let output = run_mycel(&["object", "inspect", &path, "--json"]);

    assert_success(&output);
    let json = assert_json_status(&output, "warning");
    assert_eq!(json["object_type"], "snapshot");
    assert!(
        json["notes"]
            .as_array()
            .is_some_and(|notes| notes
                .iter()
                .any(|entry| entry.as_str().is_some_and(|message| message
                    .contains("top-level 'documents.doc:test' must use 'rev:' prefix")))),
        "expected snapshot documents value-prefix warning, stdout: {}",
        stdout_text(&output)
    );
}

#[test]
fn object_inspect_json_warns_for_patch_with_wrong_base_revision_prefix() {
    let object = write_object_file(
        "object-inspect-patch-wrong-base-revision-prefix",
        "patch.json",
        json!({
            "type": "patch",
            "version": "mycel/0.1",
            "patch_id": "patch:test",
            "doc_id": "doc:test",
            "base_revision": "hash:base",
            "author": "pk:ed25519:test",
            "timestamp": 1u64,
            "ops": [],
            "signature": "sig:ed25519:test"
        }),
    );
    let path = path_arg(&object.path);
    let output = run_mycel(&["object", "inspect", &path, "--json"]);

    assert_success(&output);
    let json = assert_json_status(&output, "warning");
    assert_eq!(json["object_type"], "patch");
    assert!(
        json["notes"]
            .as_array()
            .is_some_and(|notes| notes
                .iter()
                .any(|entry| entry
                    .as_str()
                    .is_some_and(|message| message
                        .contains("top-level 'base_revision' must use 'rev:' prefix")))),
        "expected patch base_revision prefix warning, stdout: {}",
        stdout_text(&output)
    );
}

#[test]
fn object_inspect_json_warns_for_patch_with_wrong_block_reference_prefix() {
    let object = write_object_file(
        "object-inspect-patch-wrong-block-reference-prefix",
        "patch.json",
        json!({
            "type": "patch",
            "version": "mycel/0.1",
            "patch_id": "patch:test",
            "doc_id": "doc:test",
            "base_revision": "rev:genesis-null",
            "author": "pk:ed25519:test",
            "timestamp": 1u64,
            "ops": [
                {
                    "op": "replace_block",
                    "block_id": "paragraph-1",
                    "new_content": "Hello"
                }
            ],
            "signature": "sig:ed25519:test"
        }),
    );
    let path = path_arg(&object.path);
    let output = run_mycel(&["object", "inspect", &path, "--json"]);

    assert_success(&output);
    let json = assert_json_status(&output, "warning");
    assert_eq!(json["object_type"], "patch");
    assert!(
        json["notes"].as_array().is_some_and(|notes| notes
            .iter()
            .any(|entry| entry.as_str().is_some_and(|message| message
                .contains("top-level 'ops[0]': top-level 'block_id' must use 'blk:' prefix")))),
        "expected patch block reference prefix warning, stdout: {}",
        stdout_text(&output)
    );
}

#[test]
fn object_inspect_json_warns_for_revision_with_wrong_state_hash_prefix() {
    let object = write_object_file(
        "object-inspect-revision-wrong-state-hash-prefix",
        "revision.json",
        json!({
            "type": "revision",
            "version": "mycel/0.1",
            "revision_id": "rev:test",
            "doc_id": "doc:test",
            "parents": ["rev:base"],
            "patches": [],
            "state_hash": "rev:test",
            "author": "pk:ed25519:test",
            "timestamp": 1u64,
            "signature": "sig:ed25519:test"
        }),
    );
    let path = path_arg(&object.path);
    let output = run_mycel(&["object", "inspect", &path, "--json"]);

    assert_success(&output);
    let json = assert_json_status(&output, "warning");
    assert_eq!(json["object_type"], "revision");
    assert!(
        json["notes"]
            .as_array()
            .is_some_and(|notes| notes
                .iter()
                .any(|entry| entry
                    .as_str()
                    .is_some_and(|message| message
                        .contains("top-level 'state_hash' must use 'hash:' prefix")))),
        "expected revision state_hash prefix warning, stdout: {}",
        stdout_text(&output)
    );
}

#[test]
fn object_inspect_json_warns_for_revision_with_wrong_author_prefix() {
    let object = write_object_file(
        "object-inspect-revision-wrong-author-prefix",
        "revision.json",
        json!({
            "type": "revision",
            "version": "mycel/0.1",
            "revision_id": "rev:test",
            "doc_id": "doc:test",
            "parents": ["rev:base"],
            "patches": [],
            "state_hash": "hash:test",
            "author": "author:test",
            "timestamp": 1u64,
            "signature": "sig:ed25519:test"
        }),
    );
    let path = path_arg(&object.path);
    let output = run_mycel(&["object", "inspect", &path, "--json"]);

    assert_success(&output);
    let json = assert_json_status(&output, "warning");
    assert_eq!(json["object_type"], "revision");
    assert!(
        json["notes"]
            .as_array()
            .is_some_and(|notes| notes
                .iter()
                .any(|entry| entry.as_str().is_some_and(
                    |message| message.contains("top-level 'author' must use 'pk:' prefix")
                ))),
        "expected revision author prefix warning, stdout: {}",
        stdout_text(&output)
    );
}

#[test]
fn object_inspect_json_warns_for_view_with_non_object_policy() {
    let object = write_object_file(
        "object-inspect-view-non-object-policy",
        "view.json",
        json!({
            "type": "view",
            "version": "mycel/0.1",
            "view_id": "view:test",
            "maintainer": "pk:ed25519:test",
            "documents": {
                "doc:test": "rev:test"
            },
            "policy": "manual-reviewed",
            "timestamp": 12u64,
            "signature": "sig:ed25519:test"
        }),
    );
    let path = path_arg(&object.path);
    let output = run_mycel(&["object", "inspect", &path, "--json"]);

    assert_success(&output);
    let json = assert_json_status(&output, "warning");
    assert_eq!(json["object_type"], "view");
    assert!(
        json["notes"]
            .as_array()
            .is_some_and(|notes| notes.iter().any(|entry| entry
                .as_str()
                .is_some_and(|message| message.contains("top-level 'policy' must be an object")))),
        "expected non-object policy warning, stdout: {}",
        stdout_text(&output)
    );
}

#[test]
fn object_inspect_json_warns_for_view_with_wrong_maintainer_prefix() {
    let object = write_object_file(
        "object-inspect-view-wrong-maintainer-prefix",
        "view.json",
        json!({
            "type": "view",
            "version": "mycel/0.1",
            "view_id": "view:test",
            "maintainer": "maintainer:test",
            "documents": {
                "doc:test": "rev:test"
            },
            "policy": {
                "merge_rule": "manual-reviewed"
            },
            "timestamp": 12u64,
            "signature": "sig:ed25519:test"
        }),
    );
    let path = path_arg(&object.path);
    let output = run_mycel(&["object", "inspect", &path, "--json"]);

    assert_success(&output);
    let json = assert_json_status(&output, "warning");
    assert_eq!(json["object_type"], "view");
    assert!(
        json["notes"]
            .as_array()
            .is_some_and(|notes| notes
                .iter()
                .any(|entry| entry.as_str().is_some_and(
                    |message| message.contains("top-level 'maintainer' must use 'pk:' prefix")
                ))),
        "expected maintainer prefix warning, stdout: {}",
        stdout_text(&output)
    );
}

#[test]
fn object_inspect_json_warns_for_view_with_empty_documents() {
    let object = write_object_file(
        "object-inspect-view-empty-documents",
        "view.json",
        json!({
            "type": "view",
            "version": "mycel/0.1",
            "view_id": "view:test",
            "maintainer": "pk:ed25519:test",
            "documents": {},
            "policy": {
                "merge_rule": "manual-reviewed"
            },
            "timestamp": 12u64,
            "signature": "sig:ed25519:test"
        }),
    );
    let path = path_arg(&object.path);
    let output = run_mycel(&["object", "inspect", &path, "--json"]);

    assert_success(&output);
    let json = assert_json_status(&output, "warning");
    assert_eq!(json["object_type"], "view");
    assert!(
        json["notes"]
            .as_array()
            .is_some_and(|notes| notes
                .iter()
                .any(|entry| entry.as_str().is_some_and(
                    |message| message.contains("top-level 'documents' must not be empty")
                ))),
        "expected empty documents warning, stdout: {}",
        stdout_text(&output)
    );
}

#[test]
fn object_inspect_json_warns_for_snapshot_missing_declared_revision() {
    let object = write_object_file(
        "object-inspect-snapshot-missing-declared-revision",
        "snapshot.json",
        json!({
            "type": "snapshot",
            "version": "mycel/0.1",
            "snapshot_id": "snap:test",
            "documents": {
                "doc:test": "rev:test"
            },
            "included_objects": ["patch:test"],
            "root_hash": "hash:test",
            "created_by": "pk:ed25519:test",
            "timestamp": 9u64,
            "signature": "sig:ed25519:test"
        }),
    );
    let path = path_arg(&object.path);
    let output = run_mycel(&["object", "inspect", &path, "--json"]);

    assert_success(&output);
    let json = assert_json_status(&output, "warning");
    assert_eq!(json["object_type"], "snapshot");
    assert!(
        json["notes"].as_array().is_some_and(|notes| notes.iter().any(|entry| {
            entry.as_str().is_some_and(|message| {
                message.contains(
                    "top-level 'included_objects' must include revision 'rev:test' declared by 'documents.doc:test'",
                )
            })
        })),
        "expected missing declared revision warning, stdout: {}",
        stdout_text(&output)
    );
}

#[test]
fn object_inspect_json_warns_for_snapshot_missing_documents() {
    let object = write_object_file(
        "object-inspect-snapshot-missing-documents",
        "snapshot.json",
        json!({
            "type": "snapshot",
            "version": "mycel/0.1",
            "snapshot_id": "snap:test",
            "included_objects": ["rev:test"],
            "root_hash": "hash:test",
            "created_by": "pk:ed25519:test",
            "timestamp": 9u64,
            "signature": "sig:ed25519:test"
        }),
    );
    let path = path_arg(&object.path);
    let output = run_mycel(&["object", "inspect", &path, "--json"]);

    assert_success(&output);
    let json = assert_json_status(&output, "warning");
    assert_eq!(json["object_type"], "snapshot");
    assert!(
        json["notes"]
            .as_array()
            .is_some_and(|notes| notes.iter().any(|entry| {
                entry
                    .as_str()
                    .is_some_and(|message| message.contains("missing object field 'documents'"))
            })),
        "expected missing documents warning, stdout: {}",
        stdout_text(&output)
    );
}

#[test]
fn object_inspect_json_warns_for_snapshot_with_duplicate_included_objects() {
    let object = write_object_file(
        "object-inspect-snapshot-duplicate-included-objects",
        "snapshot.json",
        json!({
            "type": "snapshot",
            "version": "mycel/0.1",
            "snapshot_id": "snap:test",
            "documents": {
                "doc:test": "rev:test"
            },
            "included_objects": ["rev:test", "rev:test"],
            "root_hash": "hash:test",
            "created_by": "pk:ed25519:test",
            "timestamp": 9u64,
            "signature": "sig:ed25519:test"
        }),
    );
    let path = path_arg(&object.path);
    let output = run_mycel(&["object", "inspect", &path, "--json"]);

    assert_success(&output);
    let json = assert_json_status(&output, "warning");
    assert_eq!(json["object_type"], "snapshot");
    assert!(
        json["notes"]
            .as_array()
            .is_some_and(|notes| notes.iter().any(|entry| {
                entry.as_str().is_some_and(|message| {
                    message.contains(
                        "top-level 'included_objects[1]' duplicates 'included_objects[0]'",
                    )
                })
            })),
        "expected duplicate included_objects warning, stdout: {}",
        stdout_text(&output)
    );
}

#[test]
fn object_inspect_json_warns_for_snapshot_with_empty_included_object_entry() {
    let object = write_object_file(
        "object-inspect-snapshot-empty-included-object-entry",
        "snapshot.json",
        json!({
            "type": "snapshot",
            "version": "mycel/0.1",
            "snapshot_id": "snap:test",
            "documents": {
                "doc:test": "rev:test"
            },
            "included_objects": ["rev:test", ""],
            "root_hash": "hash:test",
            "created_by": "pk:ed25519:test",
            "timestamp": 9u64,
            "signature": "sig:ed25519:test"
        }),
    );
    let path = path_arg(&object.path);
    let output = run_mycel(&["object", "inspect", &path, "--json"]);

    assert_success(&output);
    let json = assert_json_status(&output, "warning");
    assert_eq!(json["object_type"], "snapshot");
    assert!(
        json["notes"]
            .as_array()
            .is_some_and(|notes| notes.iter().any(|entry| {
                entry.as_str().is_some_and(|message| {
                    message.contains("top-level 'included_objects[1]' must not be an empty string")
                })
            })),
        "expected empty included_objects entry warning, stdout: {}",
        stdout_text(&output)
    );
}

#[test]
fn object_inspect_json_warns_for_snapshot_with_non_canonical_included_object_id() {
    let object = write_object_file(
        "object-inspect-snapshot-non-canonical-included-object-id",
        "snapshot.json",
        json!({
            "type": "snapshot",
            "version": "mycel/0.1",
            "snapshot_id": "snap:test",
            "documents": {
                "doc:test": "rev:test"
            },
            "included_objects": ["doc:test"],
            "root_hash": "hash:test",
            "created_by": "pk:ed25519:test",
            "timestamp": 9u64,
            "signature": "sig:ed25519:test"
        }),
    );
    let path = path_arg(&object.path);
    let output = run_mycel(&["object", "inspect", &path, "--json"]);

    assert_success(&output);
    let json = assert_json_status(&output, "warning");
    assert_eq!(json["object_type"], "snapshot");
    assert!(
        json["notes"]
            .as_array()
            .is_some_and(|notes| notes.iter().any(|entry| {
                entry.as_str().is_some_and(|message| {
                    message.contains(
                        "top-level 'included_objects[0]' must use a canonical object ID prefix",
                    )
                })
            })),
        "expected canonical included_objects warning, stdout: {}",
        stdout_text(&output)
    );
}

#[test]
fn object_inspect_json_warns_for_snapshot_with_wrong_root_hash_prefix() {
    let object = write_object_file(
        "object-inspect-snapshot-wrong-root-hash-prefix",
        "snapshot.json",
        json!({
            "type": "snapshot",
            "version": "mycel/0.1",
            "snapshot_id": "snap:test",
            "documents": {
                "doc:test": "rev:test"
            },
            "included_objects": ["rev:test", "patch:test"],
            "root_hash": "rev:test",
            "created_by": "pk:ed25519:test",
            "timestamp": 9u64,
            "signature": "sig:ed25519:test"
        }),
    );
    let path = path_arg(&object.path);
    let output = run_mycel(&["object", "inspect", &path, "--json"]);

    assert_success(&output);
    let json = assert_json_status(&output, "warning");
    assert_eq!(json["object_type"], "snapshot");
    assert!(
        json["notes"]
            .as_array()
            .is_some_and(|notes| notes
                .iter()
                .any(|entry| entry.as_str().is_some_and(
                    |message| message.contains("top-level 'root_hash' must use 'hash:' prefix")
                ))),
        "expected root_hash prefix warning, stdout: {}",
        stdout_text(&output)
    );
}

#[test]
fn object_inspect_json_warns_for_snapshot_with_wrong_created_by_prefix() {
    let object = write_object_file(
        "object-inspect-snapshot-wrong-created-by-prefix",
        "snapshot.json",
        json!({
            "type": "snapshot",
            "version": "mycel/0.1",
            "snapshot_id": "snap:test",
            "documents": {
                "doc:test": "rev:test"
            },
            "included_objects": ["rev:test", "patch:test"],
            "root_hash": "hash:test",
            "created_by": "creator:test",
            "timestamp": 9u64,
            "signature": "sig:ed25519:test"
        }),
    );
    let path = path_arg(&object.path);
    let output = run_mycel(&["object", "inspect", &path, "--json"]);

    assert_success(&output);
    let json = assert_json_status(&output, "warning");
    assert_eq!(json["object_type"], "snapshot");
    assert!(
        json["notes"]
            .as_array()
            .is_some_and(|notes| notes
                .iter()
                .any(|entry| entry.as_str().is_some_and(
                    |message| message.contains("top-level 'created_by' must use 'pk:' prefix")
                ))),
        "expected created_by prefix warning, stdout: {}",
        stdout_text(&output)
    );
}

#[test]
fn object_inspect_json_fails_for_non_object_top_level_value() {
    let object = write_raw_object_file("object-inspect-non-object", "array.json", "[1,2,3]");
    let path = path_arg(&object.path);
    let output = run_mycel(&["object", "inspect", &path, "--json"]);

    assert_exit_code(&output, 1);
    let json = assert_json_status(&output, "failed");
    assert!(
        json["errors"]
            .as_array()
            .is_some_and(|errors| errors.iter().any(|entry| {
                entry.as_str().is_some_and(|message| {
                    message.contains("top-level JSON value must be an object")
                })
            })),
        "expected top-level object error, stdout: {}",
        stdout_text(&output)
    );
}

#[test]
fn object_inspect_missing_target_fails_cleanly() {
    let output = run_mycel(&["object", "inspect"]);

    assert_exit_code(&output, 2);
    assert_stderr_contains(&output, "required arguments were not provided");
    assert_stderr_contains(&output, "<PATH>");
}
