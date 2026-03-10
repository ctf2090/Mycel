use std::fs;
use std::path::PathBuf;

use base64::Engine;
use ed25519_dalek::{Signer, SigningKey};
use serde_json::{json, Value};
use sha2::{Digest, Sha256};

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

fn signed_object(mut value: Value, signer_field: &str, id_field: &str, id_prefix: &str) -> Value {
    let signing_key = signing_key();
    value[signer_field] = Value::String(signer_id(&signing_key));
    let id = recompute_id(&value, id_field, id_prefix);
    value[id_field] = Value::String(id);
    let signature = sign_value(&signing_key, &value);
    value["signature"] = Value::String(signature);
    value
}

fn profile_id(policy: &Value) -> String {
    let mut hasher = Sha256::new();
    hasher.update(canonical_json(policy).as_bytes());
    format!("hash:{:x}", hasher.finalize())
}

struct StoreFixtureInfo {
    source_dir: common::TempDir,
    store_dir: common::TempDir,
    signer: String,
    revision_id: String,
    view_id: String,
    profile_id: String,
}

fn build_store_with_view() -> StoreFixtureInfo {
    let source_dir = create_temp_dir("store-index-source");
    let store_dir = create_temp_dir("store-index-root");
    let patch_path = source_dir.path().join("patch.json");
    let revision_path = source_dir.path().join("revision.json");
    let view_path = source_dir.path().join("view.json");

    let patch = signed_object(
        json!({
            "type": "patch",
            "version": "mycel/0.1",
            "doc_id": "doc:index",
            "base_revision": "rev:genesis-null",
            "timestamp": 1u64,
            "ops": []
        }),
        "author",
        "patch_id",
        "patch",
    );
    fs::write(
        &patch_path,
        serde_json::to_string_pretty(&patch).expect("patch should serialize"),
    )
    .expect("patch should write");

    let state_hash = {
        let mut hasher = Sha256::new();
        hasher.update(canonical_json(&json!({"doc_id": "doc:index", "blocks": []})).as_bytes());
        format!("hash:{:x}", hasher.finalize())
    };
    let revision = signed_object(
        json!({
            "type": "revision",
            "version": "mycel/0.1",
            "doc_id": "doc:index",
            "parents": [],
            "patches": [patch["patch_id"].as_str().expect("patch id should exist")],
            "state_hash": state_hash,
            "timestamp": 2u64
        }),
        "author",
        "revision_id",
        "rev",
    );
    let revision_id = revision["revision_id"]
        .as_str()
        .expect("revision id should exist")
        .to_string();
    fs::write(
        &revision_path,
        serde_json::to_string_pretty(&revision).expect("revision should serialize"),
    )
    .expect("revision should write");

    let policy = json!({
        "accept_keys": [signer_id(&signing_key())],
        "merge_rule": "manual-reviewed",
        "preferred_branches": ["main"]
    });
    let profile_id = profile_id(&policy);
    let view = signed_object(
        json!({
            "type": "view",
            "version": "mycel/0.1",
            "documents": {
                "doc:index": revision["revision_id"].as_str().expect("revision id should exist")
            },
            "policy": policy,
            "timestamp": 3u64
        }),
        "maintainer",
        "view_id",
        "view",
    );
    let view_id = view["view_id"]
        .as_str()
        .expect("view id should exist")
        .to_string();
    fs::write(
        &view_path,
        serde_json::to_string_pretty(&view).expect("view should serialize"),
    )
    .expect("view should write");

    let ingest = run_mycel(&[
        "store",
        "ingest",
        &path_arg(&source_dir.path().to_path_buf()),
        "--into",
        &path_arg(&store_dir.path().to_path_buf()),
    ]);
    assert_success(&ingest);

    StoreFixtureInfo {
        source_dir,
        store_dir,
        signer: signer_id(&signing_key()),
        revision_id,
        view_id,
        profile_id,
    }
}

#[test]
fn store_index_json_reads_persisted_manifest() {
    let fixture = build_store_with_view();

    let output = run_mycel(&[
        "store",
        "index",
        &path_arg(&fixture.store_dir.path().to_path_buf()),
        "--json",
    ]);

    assert_success(&output);
    let json = assert_json_status(&output, "ok");
    assert_eq!(json["stored_object_count"], 3);
    assert!(
        json["object_ids_by_type"]["patch"]
            .as_array()
            .is_some_and(|values| values.len() == 1),
        "expected patch index, stdout: {}",
        stdout_text(&output)
    );
    assert!(
        json["doc_revisions"]["doc:index"]
            .as_array()
            .is_some_and(|values| values.len() == 1),
        "expected doc revision index, stdout: {}",
        stdout_text(&output)
    );
    assert!(
        json["author_patches"][fixture.signer]
            .as_array()
            .is_some_and(|values| values.len() == 1),
        "expected author patch index, stdout: {}",
        stdout_text(&output)
    );
    assert!(
        json["profile_heads"][fixture.profile_id]["doc:index"]
            .as_array()
            .is_some_and(|values| values.len() == 1),
        "expected profile head index, stdout: {}",
        stdout_text(&output)
    );
}

#[test]
fn store_index_json_filters_common_indexes() {
    let fixture = build_store_with_view();

    let output = run_mycel(&[
        "store",
        "index",
        &path_arg(&fixture.store_dir.path().to_path_buf()),
        "--doc-id",
        "doc:index",
        "--author",
        &fixture.signer,
        "--profile-id",
        &fixture.profile_id,
        "--object-type",
        "patch",
        "--json",
    ]);

    assert_success(&output);
    let json = assert_json_status(&output, "ok");
    assert_eq!(json["filters"]["doc_id"], "doc:index");
    assert_eq!(json["filters"]["author"], fixture.signer);
    assert_eq!(json["filters"]["profile_id"], fixture.profile_id);
    assert_eq!(json["filters"]["object_type"], "patch");
    assert_eq!(
        json["object_ids_by_type"]
            .as_object()
            .map(|values| values.len()),
        Some(1)
    );
    assert!(
        json["object_ids_by_type"]["patch"]
            .as_array()
            .is_some_and(|values| values.len() == 1),
        "expected filtered patch object index, stdout: {}",
        stdout_text(&output)
    );
    assert_eq!(
        json["doc_revisions"].as_object().map(|values| values.len()),
        Some(1)
    );
    assert_eq!(
        json["author_patches"]
            .as_object()
            .map(|values| values.len()),
        Some(1)
    );
    assert_eq!(
        json["profile_heads"].as_object().map(|values| values.len()),
        Some(1)
    );
    assert_eq!(
        json["view_governance"]
            .as_array()
            .map(|values| values.len()),
        Some(1)
    );
}

#[test]
fn store_index_json_filters_by_revision_and_view() {
    let fixture = build_store_with_view();

    let output = run_mycel(&[
        "store",
        "index",
        &path_arg(&fixture.store_dir.path().to_path_buf()),
        "--revision-id",
        &fixture.revision_id,
        "--view-id",
        &fixture.view_id,
        "--json",
    ]);

    assert_success(&output);
    let json = assert_json_status(&output, "ok");
    assert_eq!(json["filters"]["revision_id"], fixture.revision_id);
    assert_eq!(json["filters"]["view_id"], fixture.view_id);
    assert!(
        json["revision_parents"][fixture.revision_id]
            .as_array()
            .is_some_and(|values| values.is_empty()),
        "expected revision parent entry, stdout: {}",
        stdout_text(&output)
    );
    assert_eq!(
        json["view_governance"]
            .as_array()
            .map(|values| values.len()),
        Some(1)
    );
    assert_eq!(json["view_governance"][0]["view_id"], fixture.view_id);
    assert!(
        json["profile_heads"][fixture.profile_id]["doc:index"]
            .as_array()
            .is_some_and(|values| values.len() == 1),
        "expected filtered profile head index, stdout: {}",
        stdout_text(&output)
    );
}

#[test]
fn store_index_text_reports_summary() {
    let fixture = build_store_with_view();

    let output = run_mycel(&[
        "store",
        "index",
        &path_arg(&fixture.store_dir.path().to_path_buf()),
        "--doc-id",
        "doc:index",
    ]);

    assert_success(&output);
    assert_empty_stderr(&output);
    let stdout = stdout_text(&output);
    assert!(
        stdout.contains("document revision indexes: 1"),
        "stdout: {stdout}"
    );
    assert!(
        stdout.contains("filter doc_id: doc:index"),
        "stdout: {stdout}"
    );
    assert!(stdout.contains("store index: ok"), "stdout: {stdout}");
}

#[test]
fn store_index_path_only_prints_manifest_path() {
    let fixture = build_store_with_view();

    let output = run_mycel(&[
        "store",
        "index",
        &path_arg(&fixture.store_dir.path().to_path_buf()),
        "--path-only",
    ]);

    assert_success(&output);
    assert_empty_stderr(&output);
    assert_eq!(
        stdout_text(&output).trim(),
        fixture
            .store_dir
            .path()
            .join("indexes")
            .join("manifest.json")
            .to_string_lossy()
    );
    let _ = fixture.source_dir.path();
}

#[test]
fn store_index_path_only_rejects_json() {
    let fixture = build_store_with_view();
    let output = run_mycel(&[
        "store",
        "index",
        &path_arg(&fixture.store_dir.path().to_path_buf()),
        "--path-only",
        "--json",
    ]);

    assert_exit_code(&output, 2);
    assert_stderr_contains(
        &output,
        "store index --path-only cannot be used with --json",
    );
}

#[test]
fn store_index_doc_only_json_prunes_other_sections() {
    let fixture = build_store_with_view();
    let output = run_mycel(&[
        "store",
        "index",
        &path_arg(&fixture.store_dir.path().to_path_buf()),
        "--doc-only",
        "--json",
    ]);

    assert_success(&output);
    let json = assert_json_status(&output, "ok");
    assert_eq!(json["projection"], "doc-only");
    assert_eq!(
        json["doc_revisions"].as_object().map(|values| values.len()),
        Some(1)
    );
    assert_eq!(
        json["revision_parents"]
            .as_object()
            .map(|values| values.len()),
        Some(0)
    );
    assert_eq!(
        json["view_governance"]
            .as_array()
            .map(|values| values.len()),
        Some(0)
    );
    assert_eq!(
        json["profile_heads"].as_object().map(|values| values.len()),
        Some(0)
    );
}

#[test]
fn store_index_governance_only_json_prunes_non_governance_sections() {
    let fixture = build_store_with_view();
    let output = run_mycel(&[
        "store",
        "index",
        &path_arg(&fixture.store_dir.path().to_path_buf()),
        "--governance-only",
        "--json",
    ]);

    assert_success(&output);
    let json = assert_json_status(&output, "ok");
    assert_eq!(json["projection"], "governance-only");
    assert_eq!(
        json["view_governance"]
            .as_array()
            .map(|values| values.len()),
        Some(1)
    );
    assert_eq!(
        json["profile_heads"].as_object().map(|values| values.len()),
        Some(1)
    );
    assert_eq!(
        json["doc_revisions"].as_object().map(|values| values.len()),
        Some(0)
    );
    assert_eq!(
        json["revision_parents"]
            .as_object()
            .map(|values| values.len()),
        Some(0)
    );
}

#[test]
fn store_index_parents_only_text_reports_projection() {
    let fixture = build_store_with_view();
    let output = run_mycel(&[
        "store",
        "index",
        &path_arg(&fixture.store_dir.path().to_path_buf()),
        "--parents-only",
    ]);

    assert_success(&output);
    assert_empty_stderr(&output);
    let stdout = stdout_text(&output);
    assert!(
        stdout.contains("projection: parents-only"),
        "stdout: {stdout}"
    );
    assert!(
        stdout.contains("revision parent indexes: 1"),
        "stdout: {stdout}"
    );
    assert!(
        stdout.contains("document revision indexes: 0"),
        "stdout: {stdout}"
    );
    assert!(
        stdout.contains("view governance records: 0"),
        "stdout: {stdout}"
    );
}

#[test]
fn store_index_rejects_multiple_projection_flags() {
    let fixture = build_store_with_view();
    let output = run_mycel(&[
        "store",
        "index",
        &path_arg(&fixture.store_dir.path().to_path_buf()),
        "--doc-only",
        "--governance-only",
    ]);

    assert_exit_code(&output, 2);
    assert_stderr_contains(
        &output,
        "store index projection flags are mutually exclusive",
    );
}

#[test]
fn store_index_missing_manifest_fails_cleanly() {
    let store_dir = create_temp_dir("store-index-missing");
    let output = run_mycel(&["store", "index", &path_arg(&store_dir.path().to_path_buf())]);

    assert_exit_code(&output, 2);
    assert_stderr_contains(&output, "failed to read store index manifest");
}
