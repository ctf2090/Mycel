use std::fs;
use std::path::{Path, PathBuf};

use ed25519_dalek::SigningKey;
use mycel_core::author::signer_id;
use serde_json::{json, Value};

mod common;

use common::{
    assert_empty_stderr, assert_exit_code, assert_json_status, assert_stderr_contains,
    assert_success, create_temp_dir, prefixed_hash_for_test, run_mycel, signed_test_object,
    stdout_text,
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
    let state_hash = prefixed_hash_for_test(&expected_state, "hash");
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

    let output = run_mycel(&["store", "rebuild", &path_arg(dir.path()), "--json"]);

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
fn store_rebuild_leaves_local_policy_file_intact() {
    let source_dir = create_temp_dir("store-rebuild-policy-source");
    let store_dir = create_temp_dir("store-rebuild-policy-root");
    let store_root = store_dir.path().to_path_buf();
    let init = run_mycel(&["store", "init", &path_arg(&store_root), "--json"]);
    assert_success(&init);

    let patch = signed_object(
        json!({
            "type": "patch",
            "version": "mycel/0.1",
            "doc_id": "doc:store-rebuild-policy",
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

    let expected_state = json!({
        "doc_id": "doc:store-rebuild-policy",
        "blocks": []
    });
    let state_hash = prefixed_hash_for_test(&expected_state, "hash");
    let revision = signed_object(
        json!({
            "type": "revision",
            "version": "mycel/0.1",
            "doc_id": "doc:store-rebuild-policy",
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
                "doc:store-rebuild-policy": revision["revision_id"].as_str().expect("revision id should exist")
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

    let ingest = run_mycel(&[
        "store",
        "ingest",
        &path_arg(source_dir.path()),
        "--into",
        &path_arg(&store_root),
        "--json",
    ]);
    assert_success(&ingest);

    let custom_local_policy = json!({
        "version": "mycel-local-policy/0.1",
        "transport": {
            "preferred_peers": ["node:relay-b"]
        },
        "safety": {
            "warn_only": true
        }
    });
    let local_policy = local_policy_path(&store_root);
    fs::write(
        &local_policy,
        serde_json::to_string_pretty(&custom_local_policy)
            .expect("custom local policy should serialize"),
    )
    .expect("custom local policy should write");

    let output = run_mycel(&["store", "rebuild", &path_arg(&store_root), "--json"]);

    assert_success(&output);
    let json = assert_json_status(&output, "ok");
    assert_eq!(json["stored_object_count"], 3);
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

    let output = run_mycel(&["store", "rebuild", &path_arg(dir.path())]);

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
        &path_arg(source_dir.path()),
        "--into",
        &path_arg(store_dir.path()),
    ]);
    assert_success(&ingest);

    let output = run_mycel(&["store", "rebuild", &path_arg(store_dir.path()), "--json"]);

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
fn store_rebuild_store_root_recovers_multi_document_indexes_after_index_loss() {
    let source_dir = create_temp_dir("store-rebuild-multi-doc-source");
    let store_dir = create_temp_dir("store-rebuild-multi-doc-root");

    let patch_a = signed_object(
        json!({
            "type": "patch",
            "version": "mycel/0.1",
            "doc_id": "doc:multi-a",
            "base_revision": "rev:genesis-null",
            "timestamp": 1u64,
            "ops": [
                {
                    "op": "insert_block",
                    "new_block": {
                        "block_id": "blk:multi-a-001",
                        "block_type": "paragraph",
                        "content": "Doc A",
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
    let patch_a_id = patch_a["patch_id"]
        .as_str()
        .expect("patch A id should exist")
        .to_string();
    fs::write(
        source_dir.path().join("patch-a.json"),
        serde_json::to_string_pretty(&patch_a).expect("patch A should serialize"),
    )
    .expect("patch A should write");

    let patch_b = signed_object(
        json!({
            "type": "patch",
            "version": "mycel/0.1",
            "doc_id": "doc:multi-b",
            "base_revision": "rev:genesis-null",
            "timestamp": 2u64,
            "ops": [
                {
                    "op": "insert_block",
                    "new_block": {
                        "block_id": "blk:multi-b-001",
                        "block_type": "paragraph",
                        "content": "Doc B",
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
    let patch_b_id = patch_b["patch_id"]
        .as_str()
        .expect("patch B id should exist")
        .to_string();
    fs::write(
        source_dir.path().join("patch-b.json"),
        serde_json::to_string_pretty(&patch_b).expect("patch B should serialize"),
    )
    .expect("patch B should write");

    let state_a = json!({
        "doc_id": "doc:multi-a",
        "blocks": [
            {
                "block_id": "blk:multi-a-001",
                "block_type": "paragraph",
                "content": "Doc A",
                "attrs": {},
                "children": []
            }
        ]
    });
    let revision_a = signed_object(
        json!({
            "type": "revision",
            "version": "mycel/0.1",
            "doc_id": "doc:multi-a",
            "parents": [],
            "patches": [patch_a_id],
            "state_hash": prefixed_hash_for_test(&state_a, "hash"),
            "timestamp": 3u64
        }),
        "author",
        "revision_id",
        "rev",
    );
    let revision_a_id = revision_a["revision_id"]
        .as_str()
        .expect("revision A id should exist")
        .to_string();
    fs::write(
        source_dir.path().join("revision-a.json"),
        serde_json::to_string_pretty(&revision_a).expect("revision A should serialize"),
    )
    .expect("revision A should write");

    let state_b = json!({
        "doc_id": "doc:multi-b",
        "blocks": [
            {
                "block_id": "blk:multi-b-001",
                "block_type": "paragraph",
                "content": "Doc B",
                "attrs": {},
                "children": []
            }
        ]
    });
    let revision_b = signed_object(
        json!({
            "type": "revision",
            "version": "mycel/0.1",
            "doc_id": "doc:multi-b",
            "parents": [],
            "patches": [patch_b_id],
            "state_hash": prefixed_hash_for_test(&state_b, "hash"),
            "timestamp": 4u64
        }),
        "author",
        "revision_id",
        "rev",
    );
    let revision_b_id = revision_b["revision_id"]
        .as_str()
        .expect("revision B id should exist")
        .to_string();
    fs::write(
        source_dir.path().join("revision-b.json"),
        serde_json::to_string_pretty(&revision_b).expect("revision B should serialize"),
    )
    .expect("revision B should write");

    let ingest = run_mycel(&[
        "store",
        "ingest",
        &path_arg(source_dir.path()),
        "--into",
        &path_arg(store_dir.path()),
        "--json",
    ]);
    assert_success(&ingest);

    let indexes_dir = store_dir.path().join("indexes");
    fs::remove_dir_all(&indexes_dir).expect("indexes directory should be removable");
    assert!(
        !indexes_dir.exists(),
        "expected indexes directory to be removed before rebuild"
    );

    let rebuild = run_mycel(&["store", "rebuild", &path_arg(store_dir.path()), "--json"]);
    assert_success(&rebuild);
    let rebuild_json = assert_json_status(&rebuild, "ok");
    assert_eq!(rebuild_json["stored_object_count"], 4);
    assert!(
        rebuild_json["doc_revisions"]["doc:multi-a"]
            .as_array()
            .is_some_and(|values| values.iter().any(|value| value == &json!(revision_a_id))),
        "expected doc:multi-a revision index after rebuild, stdout: {}",
        stdout_text(&rebuild)
    );
    assert!(
        rebuild_json["doc_revisions"]["doc:multi-b"]
            .as_array()
            .is_some_and(|values| values.iter().any(|value| value == &json!(revision_b_id))),
        "expected doc:multi-b revision index after rebuild, stdout: {}",
        stdout_text(&rebuild)
    );

    let manifest_path = indexes_dir.join("manifest.json");
    assert!(
        manifest_path.exists(),
        "expected rebuild to recreate persisted manifest, stdout: {}",
        stdout_text(&rebuild)
    );
    let manifest: Value =
        serde_json::from_str(&fs::read_to_string(&manifest_path).expect("manifest should read"))
            .expect("manifest should parse");
    assert_eq!(manifest["stored_object_count"], 4);
    assert!(
        manifest["doc_revisions"]["doc:multi-a"]
            .as_array()
            .is_some_and(|values| values.iter().any(|value| value == &json!(revision_a_id))),
        "expected manifest doc:multi-a revision index"
    );
    assert!(
        manifest["doc_revisions"]["doc:multi-b"]
            .as_array()
            .is_some_and(|values| values.iter().any(|value| value == &json!(revision_b_id))),
        "expected manifest doc:multi-b revision index"
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

    let output = run_mycel(&["store", "rebuild", &path_arg(dir.path()), "--json"]);

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

    let output = run_mycel(&["store", "rebuild", &path_arg(dir.path()), "--json"]);

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

    let output = run_mycel(&["store", "rebuild", &path_arg(dir.path()), "--json"]);

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
    let empty_state = prefixed_hash_for_test(&empty_state_value, "hash");
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

    let output = run_mycel(&["store", "rebuild", &path_arg(dir.path()), "--json"]);

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
    let parent_state_hash = prefixed_hash_for_test(&parent_state, "hash");

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

    let output = run_mycel(&["store", "rebuild", &path_arg(dir.path()), "--json"]);

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
