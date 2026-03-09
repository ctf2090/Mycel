use std::fs;
use std::path::PathBuf;

use serde_json::{json, Value};

mod common;

use common::{
    assert_empty_stderr, assert_exit_code, assert_json_status, assert_stderr_contains,
    assert_stdout_contains, assert_success, create_temp_dir, parse_json_stdout, run_mycel,
    stdout_text,
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

#[test]
fn object_verify_json_reports_ok_for_valid_patch() {
    let object = write_object_file(
        "object-verify-patch",
        "patch.json",
        json!({
            "type": "patch",
            "version": "mycel/0.1",
            "doc_id": "doc:test",
            "base_revision": "rev:genesis-null",
            "author": "pk:authorA",
            "timestamp": 1777778888u64,
            "ops": [],
            "patch_id": "patch:76d519509ad9f7b9c2bf4a7a4def39ff5f9c5e4fb4d798e9c8cfdfa2cb48bc43",
            "signature": "sig:test"
        }),
    );
    let path = path_arg(&object.path);
    let output = run_mycel(&["object", "verify", &path, "--json"]);

    assert_success(&output);
    let json = assert_json_status(&output, "ok");
    assert_eq!(json["object_type"], "patch");
    assert_eq!(json["signature_rule"], "required");
    assert_eq!(json["signer_field"], "author");
    assert_eq!(json["signer"], "pk:authorA");
    assert_eq!(
        json["declared_id"],
        "patch:76d519509ad9f7b9c2bf4a7a4def39ff5f9c5e4fb4d798e9c8cfdfa2cb48bc43"
    );
    assert_eq!(
        json["recomputed_id"],
        "patch:76d519509ad9f7b9c2bf4a7a4def39ff5f9c5e4fb4d798e9c8cfdfa2cb48bc43"
    );
    assert!(
        json["notes"]
            .as_array()
            .is_some_and(|notes| notes.iter().any(|entry| entry
                .as_str()
                .is_some_and(|message| message.contains("not implemented yet")))),
        "expected crypto verification note, stdout: {}",
        stdout_text(&output)
    );
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
fn object_verify_json_fails_for_mismatched_revision_id() {
    let object = write_object_file(
        "object-verify-revision-mismatch",
        "revision.json",
        json!({
            "type": "revision",
            "version": "mycel/0.1",
            "doc_id": "doc:test",
            "parents": [],
            "patches": [],
            "state_hash": "hash:test-state",
            "author": "pk:authorA",
            "timestamp": 1777778890u64,
            "revision_id": "rev:wrong",
            "signature": "sig:test"
        }),
    );
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
            "maintainer": "pk:maintainerA",
            "documents": {
                "doc:test": "rev:test"
            },
            "policy": {
                "accept_keys": ["pk:maintainerA"],
                "merge_rule": "manual-reviewed",
                "preferred_branches": ["main"]
            },
            "timestamp": 1777778891u64,
            "view_id": "view:c2623b62880fab0e836335e5fcfd5be45856e188f6bb63e7f1195c38258a580a"
        }),
    );
    let path = path_arg(&object.path);
    let output = run_mycel(&["object", "verify", &path]);

    assert_exit_code(&output, 1);
    assert_stdout_contains(&output, "verification: failed");
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
    assert_stderr_contains(&output, "missing object verify target");
    assert_stdout_contains(&output, "Object options:");
}

#[test]
fn object_verify_unknown_subcommand_fails_cleanly() {
    let output = run_mycel(&["object", "bogus"]);

    assert_exit_code(&output, 2);
    assert_stderr_contains(&output, "unknown object subcommand: bogus");
    assert_stdout_contains(&output, "Object options:");
}
