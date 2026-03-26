use std::fs;
use std::path::{Path, PathBuf};

use ed25519_dalek::SigningKey;
use insta::assert_json_snapshot;
use mycel_core::author::signer_id;
use serde_json::{json, Value};

use mycel_core::canonical::prefixed_canonical_hash;

mod common;

use common::{
    assert_empty_stderr, assert_exit_code, assert_json_status, assert_stderr_contains,
    assert_success, create_temp_dir, run_mycel, signed_test_object, stdout_text,
};

fn path_arg(path: &Path) -> String {
    path.to_string_lossy().into_owned()
}

fn local_policy_path(store_root: &Path) -> PathBuf {
    store_root.join("local").join("policy.json")
}

fn signing_key() -> SigningKey {
    SigningKey::from_bytes(&[7u8; 32])
}

fn signed_object(value: Value, signer_field: &str, id_field: &str, id_prefix: &str) -> Value {
    let signing_key = signing_key();
    signed_test_object(value, &signing_key, signer_field, id_field, id_prefix)
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
        &path_arg(source_dir.path()),
        "--into",
        &path_arg(store_dir.path()),
        "--json",
    ]);

    assert_success(&output);
    let json = assert_json_status(&output, "ok");
    assert_json_snapshot!(
        "store_ingest_json_writes_verified_objects_into_store_layout",
        json,
        {
            ".index_manifest_path" => "[index_manifest_path]",
            ".source" => "[source]",
            ".store_root" => "[store_root]",
            ".stored_objects[].path" => "[stored_object_path]",
        }
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

    let rebuild = run_mycel(&["store", "rebuild", &path_arg(store_dir.path()), "--json"]);
    assert_success(&rebuild);
    let rebuild_json = assert_json_status(&rebuild, "ok");
    assert_eq!(rebuild_json["stored_object_count"], 2);
}

#[test]
fn store_ingest_preserves_local_policy_file_and_keeps_it_out_of_manifest() {
    let source_dir = create_temp_dir("store-ingest-policy-source");
    let store_dir = create_temp_dir("store-ingest-policy-root");
    let store_root = store_dir.path().to_path_buf();
    let init = run_mycel(&["store", "init", &path_arg(&store_root), "--json"]);
    assert_success(&init);
    let init_json = assert_json_status(&init, "ok");
    assert!(
        init_json["local_policy_path"]
            .as_str()
            .is_some_and(|path| path.ends_with("/local/policy.json")),
        "expected local policy path in store init summary, stdout: {}",
        stdout_text(&init)
    );

    let custom_local_policy = json!({
        "version": "mycel-local-policy/0.1",
        "transport": {
            "preferred_peers": ["node:relay-a"],
            "anonymity_mode": "tor-only"
        },
        "safety": {
            "require_manual_review": true
        }
    });
    let local_policy = local_policy_path(&store_root);
    fs::write(
        &local_policy,
        serde_json::to_string_pretty(&custom_local_policy)
            .expect("custom local policy should serialize"),
    )
    .expect("custom local policy should write");

    let patch = signed_object(
        json!({
            "type": "patch",
            "version": "mycel/0.1",
            "doc_id": "doc:store-ingest-policy",
            "base_revision": "rev:genesis-null",
            "timestamp": 1u64,
            "ops": []
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
        source_dir.path().join("patch.json"),
        serde_json::to_string_pretty(&patch).expect("patch should serialize"),
    )
    .expect("patch should write");

    let expected_state = json!({
        "doc_id": "doc:store-ingest-policy",
        "blocks": []
    });
    let state_hash =
        prefixed_canonical_hash(&expected_state, "hash").expect("state hash should canonicalize");
    let revision = signed_object(
        json!({
            "type": "revision",
            "version": "mycel/0.1",
            "doc_id": "doc:store-ingest-policy",
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
        source_dir.path().join("revision.json"),
        serde_json::to_string_pretty(&revision).expect("revision should serialize"),
    )
    .expect("revision should write");

    let policy = json!({
        "accept_keys": [signer_id(&signing_key())],
        "merge_rule": "manual-reviewed",
        "preferred_branches": ["main"]
    });
    let view = signed_object(
        json!({
            "type": "view",
            "version": "mycel/0.1",
            "documents": {
                "doc:store-ingest-policy": revision["revision_id"].as_str().expect("revision id should exist")
            },
            "policy": policy,
            "timestamp": 3u64
        }),
        "maintainer",
        "view_id",
        "view",
    );
    fs::write(
        source_dir.path().join("view.json"),
        serde_json::to_string_pretty(&view).expect("view should serialize"),
    )
    .expect("view should write");

    let output = run_mycel(&[
        "store",
        "ingest",
        &path_arg(source_dir.path()),
        "--into",
        &path_arg(&store_root),
        "--json",
    ]);

    assert_success(&output);
    let json = assert_json_status(&output, "ok");
    assert_json_snapshot!(
        "store_ingest_preserves_local_policy_file_and_keeps_it_out_of_manifest",
        json,
        {
            ".index_manifest_path" => "[index_manifest_path]",
            ".source" => "[source]",
            ".store_root" => "[store_root]",
            ".stored_objects[].path" => "[stored_object_path]",
        }
    );
    let manifest_path = store_root.join("indexes").join("manifest.json");
    let manifest: Value =
        serde_json::from_str(&fs::read_to_string(&manifest_path).expect("manifest should read"))
            .expect("manifest should parse");
    assert_eq!(
        manifest["view_governance"]
            .as_array()
            .expect("view_governance should be an array")
            .len(),
        1
    );
    assert!(manifest.get("transport").is_none());
    assert!(manifest.get("safety").is_none());

    let persisted_local_policy: Value =
        serde_json::from_str(&fs::read_to_string(&local_policy).expect("local policy should read"))
            .expect("local policy should parse");
    assert_eq!(persisted_local_policy, custom_local_policy);
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
        &path_arg(source_dir.path()),
        "--into",
        &path_arg(store_dir.path()),
    ]);
    assert_success(&first);
    assert_empty_stderr(&first);

    let second = run_mycel(&[
        "store",
        "ingest",
        &path_arg(source_dir.path()),
        "--into",
        &path_arg(store_dir.path()),
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
        &path_arg(store_dir.path()),
    ]);

    assert_exit_code(&output, 2);
    assert_stderr_contains(&output, "ingest source does not exist");
}
