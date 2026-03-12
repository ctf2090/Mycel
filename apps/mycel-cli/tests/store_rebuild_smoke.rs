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

#[test]
fn store_rebuild_json_indexes_verified_object_graph() {
    let dir = create_temp_dir("store-rebuild");
    let patch_path = dir.path().join("patch.json");
    let revision_path = dir.path().join("revision.json");
    let view_path = dir.path().join("view.json");

    let patch = signed_object(
        json!({
            "type": "patch",
            "version": "mycel/0.1",
            "doc_id": "doc:test",
            "base_revision": "rev:genesis-null",
            "timestamp": 1u64,
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
        serde_json::to_string_pretty(&patch).expect("patch should serialize"),
    )
    .expect("patch should write");

    let expected_state = json!({
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
    });
    let mut hasher = Sha256::new();
    hasher.update(canonical_json(&expected_state).as_bytes());
    let state_hash = format!("hash:{:x}", hasher.finalize());
    let revision = signed_object(
        json!({
            "type": "revision",
            "version": "mycel/0.1",
            "doc_id": "doc:test",
            "parents": [],
            "patches": [patch["patch_id"].as_str().expect("patch id should exist")],
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
                "doc:test": revision["revision_id"].as_str().expect("revision id should exist")
            },
            "policy": policy,
            "timestamp": 3u64
        }),
        "maintainer",
        "view_id",
        "view",
    );
    fs::write(
        &view_path,
        serde_json::to_string_pretty(&view).expect("view should serialize"),
    )
    .expect("view should write");

    let output = run_mycel(&[
        "store",
        "rebuild",
        &path_arg(&dir.path().to_path_buf()),
        "--json",
    ]);

    assert_success(&output);
    let json = assert_json_status(&output, "ok");
    assert_eq!(json["verified_object_count"], 3);
    assert_eq!(json["stored_object_count"], 3);
    assert!(
        json["doc_revisions"]["doc:test"]
            .as_array()
            .is_some_and(|values| values.len() == 1),
        "expected doc revision index, stdout: {}",
        stdout_text(&output)
    );
    assert!(
        json["author_patches"][signer_id(&signing_key())]
            .as_array()
            .is_some_and(|values| values.len() == 1),
        "expected author patch index, stdout: {}",
        stdout_text(&output)
    );
    assert!(
        json["profile_heads"]
            .as_object()
            .is_some_and(|profiles| profiles.len() == 1),
        "expected profile heads index, stdout: {}",
        stdout_text(&output)
    );
}

#[test]
fn store_rebuild_text_reports_summary() {
    let dir = create_temp_dir("store-rebuild-text");
    let patch = signed_object(
        json!({
            "type": "patch",
            "version": "mycel/0.1",
            "doc_id": "doc:text",
            "base_revision": "rev:genesis-null",
            "timestamp": 1u64,
            "ops": []
        }),
        "author",
        "patch_id",
        "patch",
    );
    fs::write(
        dir.path().join("patch.json"),
        serde_json::to_string_pretty(&patch).expect("patch should serialize"),
    )
    .expect("patch should write");

    let output = run_mycel(&["store", "rebuild", &path_arg(&dir.path().to_path_buf())]);

    assert_success(&output);
    assert_empty_stderr(&output);
    assert!(stdout_text(&output).contains("store rebuild: ok"));
}

#[test]
fn store_rebuild_store_root_persists_index_manifest() {
    let source_dir = create_temp_dir("store-rebuild-source");
    let store_dir = create_temp_dir("store-rebuild-root");
    let patch = signed_object(
        json!({
            "type": "patch",
            "version": "mycel/0.1",
            "doc_id": "doc:store-root",
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

    let ingest = run_mycel(&[
        "store",
        "ingest",
        &path_arg(&source_dir.path().to_path_buf()),
        "--into",
        &path_arg(&store_dir.path().to_path_buf()),
    ]);
    assert_success(&ingest);

    let output = run_mycel(&[
        "store",
        "rebuild",
        &path_arg(&store_dir.path().to_path_buf()),
        "--json",
    ]);

    assert_success(&output);
    let json = assert_json_status(&output, "ok");
    assert_eq!(json["stored_object_count"], 1);
    assert!(
        json["index_manifest_path"]
            .as_str()
            .is_some_and(|path| path.ends_with("/indexes/manifest.json")),
        "expected persisted manifest path, stdout: {}",
        stdout_text(&output)
    );

    let manifest_path = store_dir.path().join("indexes").join("manifest.json");
    assert!(manifest_path.exists(), "expected persisted manifest");
    let manifest: Value =
        serde_json::from_str(&fs::read_to_string(&manifest_path).expect("manifest should read"))
            .expect("manifest should parse");
    assert_eq!(manifest["stored_object_count"], 1);
    assert!(
        manifest["object_ids_by_type"]["patch"]
            .as_array()
            .is_some_and(|values| values.len() == 1),
        "expected patch object index"
    );
}

#[test]
fn store_rebuild_json_fails_for_duplicate_declared_object_ids() {
    let dir = create_temp_dir("store-rebuild-duplicate-ids");
    let patch = signed_object(
        json!({
            "type": "patch",
            "version": "mycel/0.1",
            "doc_id": "doc:duplicate",
            "base_revision": "rev:genesis-null",
            "timestamp": 1u64,
            "ops": []
        }),
        "author",
        "patch_id",
        "patch",
    );
    fs::write(
        dir.path().join("patch-a.json"),
        serde_json::to_string_pretty(&patch).expect("patch should serialize"),
    )
    .expect("first patch should write");
    fs::write(
        dir.path().join("patch-b.json"),
        serde_json::to_string_pretty(&patch).expect("patch should serialize"),
    )
    .expect("second patch should write");

    let output = run_mycel(&[
        "store",
        "rebuild",
        &path_arg(&dir.path().to_path_buf()),
        "--json",
    ]);

    assert_exit_code(&output, 1);
    let json = assert_json_status(&output, "failed");
    assert!(
        json["errors"]
            .as_array()
            .is_some_and(|errors| errors.iter().any(|entry| {
                entry.as_str().is_some_and(|message| {
                    message.contains("duplicate declared object ID")
                        && message.contains(patch["patch_id"].as_str().unwrap())
                })
            })),
        "expected duplicate object ID error, stdout: {}",
        stdout_text(&output)
    );
}

#[test]
fn store_rebuild_json_fails_for_missing_revision_patch_dependency() {
    let dir = create_temp_dir("store-rebuild-missing-patch");
    let revision = signed_object(
        json!({
            "type": "revision",
            "version": "mycel/0.1",
            "doc_id": "doc:missing-patch",
            "parents": [],
            "patches": ["patch:missing"],
            "state_hash": "hash:missing",
            "timestamp": 2u64
        }),
        "author",
        "revision_id",
        "rev",
    );
    fs::write(
        dir.path().join("revision.json"),
        serde_json::to_string_pretty(&revision).expect("revision should serialize"),
    )
    .expect("revision should write");

    let output = run_mycel(&[
        "store",
        "rebuild",
        &path_arg(&dir.path().to_path_buf()),
        "--json",
    ]);

    assert_exit_code(&output, 1);
    let json = assert_json_status(&output, "failed");
    assert_eq!(json["verified_object_count"], 0);
    assert_eq!(json["stored_object_count"], 0);
    assert!(
        json["errors"]
            .as_array()
            .is_some_and(|errors| errors.iter().any(|entry| {
                entry.as_str().is_some_and(|message| {
                    message.contains(
                        "revision replay failed: missing patch 'patch:missing' for replay",
                    )
                })
            })),
        "expected missing patch replay error, stdout: {}",
        stdout_text(&output)
    );
}

#[test]
fn store_rebuild_json_fails_for_missing_parent_revision_dependency() {
    let dir = create_temp_dir("store-rebuild-missing-parent");
    let revision = signed_object(
        json!({
            "type": "revision",
            "version": "mycel/0.1",
            "doc_id": "doc:missing-parent",
            "parents": ["rev:missing-parent"],
            "patches": [],
            "state_hash": "hash:missing-parent",
            "timestamp": 2u64
        }),
        "author",
        "revision_id",
        "rev",
    );
    fs::write(
        dir.path().join("revision.json"),
        serde_json::to_string_pretty(&revision).expect("revision should serialize"),
    )
    .expect("revision should write");

    let output = run_mycel(&[
        "store",
        "rebuild",
        &path_arg(&dir.path().to_path_buf()),
        "--json",
    ]);

    assert_exit_code(&output, 1);
    let json = assert_json_status(&output, "failed");
    assert_eq!(json["verified_object_count"], 0);
    assert_eq!(json["stored_object_count"], 0);
    assert!(
        json["errors"].as_array().is_some_and(|errors| errors.iter().any(|entry| {
            entry.as_str().is_some_and(|message| {
                message.contains(
                    "revision replay failed: missing parent revision 'rev:missing-parent' for replay",
                )
            })
        })),
        "expected missing parent replay error, stdout: {}",
        stdout_text(&output)
    );
}

#[test]
fn store_rebuild_json_reports_multi_hop_ancestry_context_for_parent_missing_patch_dependency() {
    let dir = create_temp_dir("store-rebuild-ancestry-missing-patch");
    let doc_id = "doc:ancestor-missing-patch";
    let empty_state_value = json!({
        "doc_id": doc_id,
        "blocks": []
    });
    let mut empty_state_hasher = Sha256::new();
    empty_state_hasher.update(canonical_json(&empty_state_value).as_bytes());
    let empty_state = format!("hash:{:x}", empty_state_hasher.finalize());
    let parent_revision = signed_object(
        json!({
            "type": "revision",
            "version": "mycel/0.1",
            "doc_id": doc_id,
            "parents": [],
            "patches": ["patch:missing-ancestor"],
            "state_hash": empty_state,
            "timestamp": 1u64
        }),
        "author",
        "revision_id",
        "rev",
    );
    let parent_revision_id = parent_revision["revision_id"]
        .as_str()
        .expect("parent revision id should exist")
        .to_string();
    let revision = signed_object(
        json!({
            "type": "revision",
            "version": "mycel/0.1",
            "doc_id": doc_id,
            "parents": [parent_revision_id.clone()],
            "patches": [],
            "state_hash": empty_state,
            "timestamp": 2u64
        }),
        "author",
        "revision_id",
        "rev",
    );
    fs::write(
        dir.path().join("parent.json"),
        serde_json::to_string_pretty(&parent_revision).expect("parent revision should serialize"),
    )
    .expect("parent revision should write");
    fs::write(
        dir.path().join("revision.json"),
        serde_json::to_string_pretty(&revision).expect("revision should serialize"),
    )
    .expect("revision should write");

    let output = run_mycel(&[
        "store",
        "rebuild",
        &path_arg(&dir.path().to_path_buf()),
        "--json",
    ]);

    assert_exit_code(&output, 1);
    let json = assert_json_status(&output, "failed");
    assert!(
        json["errors"]
            .as_array()
            .is_some_and(|errors| errors.iter().any(|entry| {
                entry.as_str().is_some_and(|message| {
                    message.contains(&format!(
                        "while verifying ancestry through parent revision '{parent_revision_id}'"
                    )) && message.contains("missing patch 'patch:missing-ancestor' for replay")
                })
            })),
        "expected nested ancestry-context replay error, stdout: {}",
        stdout_text(&output)
    );
}

#[test]
fn store_rebuild_json_fails_for_cross_document_parent_revision_dependency() {
    let dir = create_temp_dir("store-rebuild-cross-doc-parent");
    let parent_state = json!({
        "doc_id": "doc:parent",
        "blocks": []
    });
    let mut hasher = Sha256::new();
    hasher.update(canonical_json(&parent_state).as_bytes());
    let parent_state_hash = format!("hash:{:x}", hasher.finalize());

    let parent_revision = signed_object(
        json!({
            "type": "revision",
            "version": "mycel/0.1",
            "doc_id": "doc:parent",
            "parents": [],
            "patches": [],
            "state_hash": parent_state_hash,
            "timestamp": 1u64
        }),
        "author",
        "revision_id",
        "rev",
    );
    let parent_revision_id = parent_revision["revision_id"]
        .as_str()
        .expect("parent revision id should exist")
        .to_string();
    fs::write(
        dir.path().join("parent-revision.json"),
        serde_json::to_string_pretty(&parent_revision).expect("parent revision should serialize"),
    )
    .expect("parent revision should write");

    let child_revision = signed_object(
        json!({
            "type": "revision",
            "version": "mycel/0.1",
            "doc_id": "doc:child",
            "parents": [parent_revision_id],
            "patches": [],
            "state_hash": "hash:child",
            "timestamp": 2u64
        }),
        "author",
        "revision_id",
        "rev",
    );
    fs::write(
        dir.path().join("child-revision.json"),
        serde_json::to_string_pretty(&child_revision).expect("child revision should serialize"),
    )
    .expect("child revision should write");

    let output = run_mycel(&[
        "store",
        "rebuild",
        &path_arg(&dir.path().to_path_buf()),
        "--json",
    ]);

    assert_exit_code(&output, 1);
    let json = assert_json_status(&output, "failed");
    assert_eq!(json["verified_object_count"], 1);
    assert_eq!(json["stored_object_count"], 1);
    assert!(
        json["errors"]
            .as_array()
            .is_some_and(|errors| errors.iter().any(|entry| {
                entry.as_str().is_some_and(|message| {
                    message.contains("revision replay failed: parent revision")
                        && message.contains("belongs to 'doc:parent' instead of 'doc:child'")
                })
            })),
        "expected cross-document parent replay error, stdout: {}",
        stdout_text(&output)
    );
}

#[test]
fn store_rebuild_missing_target_fails_cleanly() {
    let output = run_mycel(&["store", "rebuild"]);

    assert_exit_code(&output, 2);
    assert_stderr_contains(&output, "required arguments were not provided");
    assert_stderr_contains(&output, "<PATH>");
}

#[test]
fn store_rebuild_json_fails_for_missing_target() {
    let output = run_mycel(&["store", "rebuild", "missing-store-target", "--json"]);

    assert_exit_code(&output, 2);
    assert_stderr_contains(&output, "store target does not exist");
}
