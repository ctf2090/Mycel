use std::fs;
use std::path::PathBuf;

use base64::Engine;
use serde_json::json;

mod common;

use common::{
    assert_json_status, assert_stderr_contains, assert_success, create_temp_dir, run_mycel,
};

fn path_arg(path: &PathBuf) -> String {
    path.to_string_lossy().into_owned()
}

fn write_signing_key_file(prefix: &str) -> (common::TempDir, PathBuf) {
    let dir = create_temp_dir(prefix);
    let path = dir.path().join("signing-key.txt");
    fs::write(
        &path,
        base64::engine::general_purpose::STANDARD.encode([7u8; 32]),
    )
    .expect("signing key should write");
    (dir, path)
}

fn write_ops_file(prefix: &str) -> (common::TempDir, PathBuf) {
    let dir = create_temp_dir(prefix);
    let path = dir.path().join("ops.json");
    fs::write(
        &path,
        serde_json::to_string_pretty(&json!([
            {
                "op": "insert_block",
                "new_block": {
                    "block_id": "blk:author-smoke-001",
                    "block_type": "paragraph",
                    "content": "Hello author smoke",
                    "attrs": {},
                    "children": []
                }
            }
        ]))
        .expect("ops JSON should serialize"),
    )
    .expect("ops JSON should write");
    (dir, path)
}

fn write_resolved_state_file(prefix: &str) -> (common::TempDir, PathBuf) {
    let dir = create_temp_dir(prefix);
    let path = dir.path().join("resolved-state.json");
    fs::write(
        &path,
        serde_json::to_string_pretty(&json!({
            "doc_id": "doc:author-smoke",
            "blocks": [
                {
                    "block_id": "blk:author-smoke-001",
                    "block_type": "paragraph",
                    "content": "Hello author smoke",
                    "attrs": {},
                    "children": []
                },
                {
                    "block_id": "blk:author-smoke-merge-002",
                    "block_type": "paragraph",
                    "content": "Merged side branch",
                    "attrs": {},
                    "children": []
                }
            ]
        }))
        .expect("resolved state JSON should serialize"),
    )
    .expect("resolved state JSON should write");
    (dir, path)
}

fn write_content_variant_ops_file(prefix: &str, content: &str) -> (common::TempDir, PathBuf) {
    let dir = create_temp_dir(prefix);
    let path = dir.path().join("ops.json");
    fs::write(
        &path,
        serde_json::to_string_pretty(&json!([
            {
                "op": "replace_block",
                "block_id": "blk:author-smoke-variant-001",
                "new_content": content
            }
        ]))
        .expect("content variant ops JSON should serialize"),
    )
    .expect("content variant ops JSON should write");
    (dir, path)
}

fn write_content_variant_resolved_state_file(
    prefix: &str,
    content: &str,
) -> (common::TempDir, PathBuf) {
    let dir = create_temp_dir(prefix);
    let path = dir.path().join("resolved-state.json");
    fs::write(
        &path,
        serde_json::to_string_pretty(&json!({
            "doc_id": "doc:author-smoke-content-variant",
            "blocks": [
                {
                    "block_id": "blk:author-smoke-variant-001",
                    "block_type": "paragraph",
                    "content": content,
                    "attrs": {},
                    "children": []
                }
            ]
        }))
        .expect("content variant resolved state JSON should serialize"),
    )
    .expect("content variant resolved state JSON should write");
    (dir, path)
}

fn write_structural_move_ops_file(prefix: &str) -> (common::TempDir, PathBuf) {
    let dir = create_temp_dir(prefix);
    let path = dir.path().join("ops.json");
    fs::write(
        &path,
        serde_json::to_string_pretty(&json!([
            {
                "op": "move_block",
                "block_id": "blk:author-smoke-001",
                "after_block_id": "blk:author-smoke-002"
            }
        ]))
        .expect("move ops JSON should serialize"),
    )
    .expect("move ops JSON should write");
    (dir, path)
}

fn write_structural_insert_ops_file(prefix: &str) -> (common::TempDir, PathBuf) {
    let dir = create_temp_dir(prefix);
    let path = dir.path().join("ops.json");
    fs::write(
        &path,
        serde_json::to_string_pretty(&json!([
            {
                "op": "insert_block",
                "new_block": {
                    "block_id": "blk:author-smoke-003",
                    "block_type": "paragraph",
                    "content": "Structural merge tail",
                    "attrs": {},
                    "children": []
                }
            }
        ]))
        .expect("structural insert ops JSON should serialize"),
    )
    .expect("structural insert ops JSON should write");
    (dir, path)
}

fn write_structural_resolved_state_file(prefix: &str) -> (common::TempDir, PathBuf) {
    let dir = create_temp_dir(prefix);
    let path = dir.path().join("resolved-state.json");
    fs::write(
        &path,
        serde_json::to_string_pretty(&json!({
            "doc_id": "doc:author-smoke-structural",
            "blocks": [
                {
                    "block_id": "blk:author-smoke-002",
                    "block_type": "paragraph",
                    "content": "Second structural block",
                    "attrs": {},
                    "children": []
                },
                {
                    "block_id": "blk:author-smoke-001",
                    "block_type": "paragraph",
                    "content": "Hello author smoke",
                    "attrs": {},
                    "children": []
                },
                {
                    "block_id": "blk:author-smoke-003",
                    "block_type": "paragraph",
                    "content": "Structural merge tail",
                    "attrs": {},
                    "children": []
                }
            ]
        }))
        .expect("structural resolved state JSON should serialize"),
    )
    .expect("structural resolved state JSON should write");
    (dir, path)
}

fn write_nested_parent_choice_resolved_state_file(prefix: &str) -> (common::TempDir, PathBuf) {
    let dir = create_temp_dir(prefix);
    let path = dir.path().join("resolved-state.json");
    fs::write(
        &path,
        serde_json::to_string_pretty(&json!({
            "doc_id": "doc:author-smoke-nested-parent-choice",
            "blocks": [
                {
                    "block_id": "blk:nested-wrapper",
                    "block_type": "paragraph",
                    "content": "Wrapper",
                    "attrs": {},
                    "children": [
                        {
                            "block_id": "blk:nested-left",
                            "block_type": "paragraph",
                            "content": "Left",
                            "attrs": {},
                            "children": [
                                {
                                    "block_id": "blk:nested-leaf",
                                    "block_type": "paragraph",
                                    "content": "Leaf",
                                    "attrs": {},
                                    "children": []
                                }
                            ]
                        },
                        {
                            "block_id": "blk:nested-right",
                            "block_type": "paragraph",
                            "content": "Right",
                            "attrs": {},
                            "children": []
                        }
                    ]
                }
            ]
        }))
        .expect("nested parent choice resolved state JSON should serialize"),
    )
    .expect("nested parent choice resolved state JSON should write");
    (dir, path)
}

fn write_nested_parent_anchor_choice_resolved_state_file(
    prefix: &str,
) -> (common::TempDir, PathBuf) {
    let dir = create_temp_dir(prefix);
    let path = dir.path().join("resolved-state.json");
    fs::write(
        &path,
        serde_json::to_string_pretty(&json!({
            "doc_id": "doc:author-smoke-nested-parent-anchor-choice",
            "blocks": [
                {
                    "block_id": "blk:nested-left",
                    "block_type": "paragraph",
                    "content": "Left",
                    "attrs": {},
                    "children": [
                        {
                            "block_id": "blk:nested-subsection",
                            "block_type": "paragraph",
                            "content": "Subsection",
                            "attrs": {},
                            "children": [
                                {
                                    "block_id": "blk:nested-leaf",
                                    "block_type": "paragraph",
                                    "content": "Leaf",
                                    "attrs": {},
                                    "children": []
                                }
                            ]
                        }
                    ]
                },
                {
                    "block_id": "blk:nested-right",
                    "block_type": "paragraph",
                    "content": "Right",
                    "attrs": {},
                    "children": []
                }
            ]
        }))
        .expect("nested parent anchor choice resolved state JSON should serialize"),
    )
    .expect("nested parent anchor choice resolved state JSON should write");
    (dir, path)
}

fn write_nested_parent_manual_resolved_state_file(prefix: &str) -> (common::TempDir, PathBuf) {
    let dir = create_temp_dir(prefix);
    let path = dir.path().join("resolved-state.json");
    fs::write(
        &path,
        serde_json::to_string_pretty(&json!({
            "doc_id": "doc:author-smoke-nested-parent-manual",
            "blocks": [
                {
                    "block_id": "blk:manual-left",
                    "block_type": "paragraph",
                    "content": "Left",
                    "attrs": {},
                    "children": []
                },
                {
                    "block_id": "blk:manual-right",
                    "block_type": "paragraph",
                    "content": "Right",
                    "attrs": {},
                    "children": [
                        {
                            "block_id": "blk:manual-wrapper",
                            "block_type": "paragraph",
                            "content": "Wrapper",
                            "attrs": {},
                            "children": [
                                {
                                    "block_id": "blk:manual-leaf",
                                    "block_type": "paragraph",
                                    "content": "Leaf",
                                    "attrs": {},
                                    "children": []
                                }
                            ]
                        }
                    ]
                }
            ]
        }))
        .expect("nested parent manual resolved state JSON should serialize"),
    )
    .expect("nested parent manual resolved state JSON should write");
    (dir, path)
}

fn write_nested_sibling_manual_resolved_state_file(prefix: &str) -> (common::TempDir, PathBuf) {
    let dir = create_temp_dir(prefix);
    let path = dir.path().join("resolved-state.json");
    fs::write(
        &path,
        serde_json::to_string_pretty(&json!({
            "doc_id": "doc:author-smoke-nested-sibling-manual",
            "blocks": [
                {
                    "block_id": "blk:nested-parent",
                    "block_type": "paragraph",
                    "content": "Parent",
                    "attrs": {},
                    "children": [
                        {
                            "block_id": "blk:nested-child-b",
                            "block_type": "paragraph",
                            "content": "Child B",
                            "attrs": {},
                            "children": []
                        },
                        {
                            "block_id": "blk:nested-child-d",
                            "block_type": "paragraph",
                            "content": "Child D",
                            "attrs": {},
                            "children": []
                        },
                        {
                            "block_id": "blk:nested-child-a",
                            "block_type": "paragraph",
                            "content": "Child A",
                            "attrs": {},
                            "children": []
                        },
                        {
                            "block_id": "blk:nested-child-c",
                            "block_type": "paragraph",
                            "content": "Child C",
                            "attrs": {},
                            "children": []
                        }
                    ]
                }
            ]
        }))
        .expect("nested sibling manual resolved state JSON should serialize"),
    )
    .expect("nested sibling manual resolved state JSON should write");
    (dir, path)
}

fn write_composed_branch_manual_resolved_state_file(prefix: &str) -> (common::TempDir, PathBuf) {
    let dir = create_temp_dir(prefix);
    let path = dir.path().join("resolved-state.json");
    fs::write(
        &path,
        serde_json::to_string_pretty(&json!({
            "doc_id": "doc:author-smoke-composed-manual",
            "blocks": [
                {
                    "block_id": "blk:cmp-anchor",
                    "block_type": "paragraph",
                    "content": "Anchor",
                    "attrs": {},
                    "children": []
                },
                {
                    "block_id": "blk:cmp-wrapper",
                    "block_type": "paragraph",
                    "content": "Wrapper",
                    "attrs": {},
                    "children": [
                        {
                            "block_id": "blk:cmp-section",
                            "block_type": "paragraph",
                            "content": "Section",
                            "attrs": {},
                            "children": [
                                {
                                    "block_id": "blk:cmp-subsection",
                                    "block_type": "paragraph",
                                    "content": "Subsection",
                                    "attrs": {},
                                    "children": [
                                        {
                                            "block_id": "blk:cmp-leaf-a",
                                            "block_type": "paragraph",
                                            "content": "Leaf A",
                                            "attrs": {},
                                            "children": []
                                        },
                                        {
                                            "block_id": "blk:cmp-leaf-b",
                                            "block_type": "paragraph",
                                            "content": "Leaf B",
                                            "attrs": {},
                                            "children": []
                                        }
                                    ]
                                }
                            ]
                        }
                    ]
                }
            ]
        }))
        .expect("composed branch manual resolved state JSON should serialize"),
    )
    .expect("composed branch manual resolved state JSON should write");
    (dir, path)
}

fn write_attrs_manual_resolved_state_file(prefix: &str) -> (common::TempDir, PathBuf) {
    let dir = create_temp_dir(prefix);
    let path = dir.path().join("resolved-state.json");
    fs::write(
        &path,
        serde_json::to_string_pretty(&json!({
            "doc_id": "doc:author-smoke-attrs-manual",
            "blocks": [
                {
                    "block_id": "blk:merge-attrs",
                    "block_type": "paragraph",
                    "content": "Attrs",
                    "attrs": {
                        "style": "note"
                    },
                    "children": []
                }
            ]
        }))
        .expect("attrs manual resolved state JSON should serialize"),
    )
    .expect("attrs manual resolved state JSON should write");
    (dir, path)
}

#[test]
fn store_authoring_flow_creates_document_patch_and_revision() {
    let store_dir = create_temp_dir("store-author-root");
    let (_key_dir, key_path) = write_signing_key_file("store-author-key");
    let (_ops_dir, ops_path) = write_ops_file("store-author-ops");
    let store_root = path_arg(&store_dir.path().to_path_buf());
    let key_file = path_arg(&key_path);
    let ops_file = path_arg(&ops_path);

    let init = run_mycel(&["store", "init", &store_root, "--json"]);
    assert_success(&init);
    let init_json = assert_json_status(&init, "ok");
    assert_eq!(init_json["store_root"], store_root);

    let document = run_mycel(&[
        "store",
        "create-document",
        &store_root,
        "--doc-id",
        "doc:author-smoke",
        "--title",
        "Author Smoke",
        "--language",
        "en",
        "--signing-key",
        &key_file,
        "--timestamp",
        "10",
        "--json",
    ]);
    assert_success(&document);
    let document_json = assert_json_status(&document, "ok");
    let genesis_revision_id = document_json["genesis_revision_id"]
        .as_str()
        .expect("genesis revision should be string")
        .to_string();
    assert_eq!(document_json["written_object_count"], 2);

    let patch = run_mycel(&[
        "store",
        "create-patch",
        &store_root,
        "--doc-id",
        "doc:author-smoke",
        "--base-revision",
        &genesis_revision_id,
        "--ops",
        &ops_file,
        "--signing-key",
        &key_file,
        "--timestamp",
        "11",
        "--json",
    ]);
    assert_success(&patch);
    let patch_json = assert_json_status(&patch, "ok");
    let patch_id = patch_json["patch_id"]
        .as_str()
        .expect("patch_id should be string")
        .to_string();
    assert_eq!(patch_json["written_object_count"], 1);

    let revision = run_mycel(&[
        "store",
        "commit-revision",
        &store_root,
        "--doc-id",
        "doc:author-smoke",
        "--parent",
        &genesis_revision_id,
        "--patch",
        &patch_id,
        "--signing-key",
        &key_file,
        "--timestamp",
        "12",
        "--json",
    ]);
    assert_success(&revision);
    let revision_json = assert_json_status(&revision, "ok");
    assert_eq!(revision_json["written_object_count"], 1);
    assert!(revision_json["recomputed_state_hash"]
        .as_str()
        .is_some_and(|value| value.starts_with("hash:")));

    let index = run_mycel(&["store", "index", &store_root, "--json"]);
    assert_success(&index);
    let index_json = assert_json_status(&index, "ok");
    assert_eq!(index_json["stored_object_count"], 4);
    assert_eq!(
        index_json["doc_revisions"]["doc:author-smoke"]
            .as_array()
            .map(Vec::len),
        Some(2)
    );
    assert_eq!(
        index_json["object_ids_by_type"]["document"]
            .as_array()
            .map(Vec::len),
        Some(1)
    );
    assert_eq!(
        index_json["object_ids_by_type"]["patch"]
            .as_array()
            .map(Vec::len),
        Some(1)
    );
    assert_eq!(
        index_json["object_ids_by_type"]["revision"]
            .as_array()
            .map(Vec::len),
        Some(2)
    );

    let rebuild = run_mycel(&["store", "rebuild", &store_root, "--json"]);
    assert_success(&rebuild);
    let rebuild_json = assert_json_status(&rebuild, "ok");
    assert_eq!(rebuild_json["stored_object_count"], 4);
    assert_eq!(rebuild_json["verified_object_count"], 4);
}

#[test]
fn store_merge_authoring_flow_creates_merge_patch_and_revision() {
    let store_dir = create_temp_dir("store-merge-root");
    let (_key_dir, key_path) = write_signing_key_file("store-merge-key");
    let (_ops_dir, ops_path) = write_ops_file("store-merge-ops");
    let (_resolved_dir, resolved_state_path) = write_resolved_state_file("store-merge-state");
    let store_root = path_arg(&store_dir.path().to_path_buf());
    let key_file = path_arg(&key_path);
    let ops_file = path_arg(&ops_path);
    let resolved_state_file = path_arg(&resolved_state_path);

    let init = run_mycel(&["store", "init", &store_root, "--json"]);
    assert_success(&init);

    let document = run_mycel(&[
        "store",
        "create-document",
        &store_root,
        "--doc-id",
        "doc:author-smoke",
        "--title",
        "Author Smoke Merge",
        "--language",
        "en",
        "--signing-key",
        &key_file,
        "--timestamp",
        "10",
        "--json",
    ]);
    assert_success(&document);
    let document_json = assert_json_status(&document, "ok");
    let genesis_revision_id = document_json["genesis_revision_id"]
        .as_str()
        .expect("genesis revision should be string")
        .to_string();

    let primary_patch = run_mycel(&[
        "store",
        "create-patch",
        &store_root,
        "--doc-id",
        "doc:author-smoke",
        "--base-revision",
        &genesis_revision_id,
        "--ops",
        &ops_file,
        "--signing-key",
        &key_file,
        "--timestamp",
        "11",
        "--json",
    ]);
    assert_success(&primary_patch);
    let primary_patch_json = assert_json_status(&primary_patch, "ok");
    let primary_patch_id = primary_patch_json["patch_id"]
        .as_str()
        .expect("patch_id should be string")
        .to_string();

    let primary_revision = run_mycel(&[
        "store",
        "commit-revision",
        &store_root,
        "--doc-id",
        "doc:author-smoke",
        "--parent",
        &genesis_revision_id,
        "--patch",
        &primary_patch_id,
        "--signing-key",
        &key_file,
        "--timestamp",
        "12",
        "--json",
    ]);
    assert_success(&primary_revision);
    let primary_revision_json = assert_json_status(&primary_revision, "ok");
    let primary_revision_id = primary_revision_json["revision_id"]
        .as_str()
        .expect("revision_id should be string")
        .to_string();

    let side_ops_dir = create_temp_dir("store-merge-side-ops");
    let side_ops_path = side_ops_dir.path().join("ops.json");
    fs::write(
        &side_ops_path,
        serde_json::to_string_pretty(&json!([
            {
                "op": "insert_block",
                "new_block": {
                    "block_id": "blk:author-smoke-merge-002",
                    "block_type": "paragraph",
                    "content": "Merged side branch",
                    "attrs": {},
                    "children": []
                }
            }
        ]))
        .expect("side ops JSON should serialize"),
    )
    .expect("side ops JSON should write");
    let side_ops_file = path_arg(&side_ops_path);

    let side_patch = run_mycel(&[
        "store",
        "create-patch",
        &store_root,
        "--doc-id",
        "doc:author-smoke",
        "--base-revision",
        &genesis_revision_id,
        "--ops",
        &side_ops_file,
        "--signing-key",
        &key_file,
        "--timestamp",
        "13",
        "--json",
    ]);
    assert_success(&side_patch);
    let side_patch_json = assert_json_status(&side_patch, "ok");
    let side_patch_id = side_patch_json["patch_id"]
        .as_str()
        .expect("side patch_id should be string")
        .to_string();

    let side_revision = run_mycel(&[
        "store",
        "commit-revision",
        &store_root,
        "--doc-id",
        "doc:author-smoke",
        "--parent",
        &genesis_revision_id,
        "--patch",
        &side_patch_id,
        "--signing-key",
        &key_file,
        "--timestamp",
        "14",
        "--json",
    ]);
    assert_success(&side_revision);
    let side_revision_json = assert_json_status(&side_revision, "ok");
    let side_revision_id = side_revision_json["revision_id"]
        .as_str()
        .expect("side revision_id should be string")
        .to_string();

    let merge = run_mycel(&[
        "store",
        "create-merge-revision",
        &store_root,
        "--doc-id",
        "doc:author-smoke",
        "--parent",
        &primary_revision_id,
        "--parent",
        &side_revision_id,
        "--resolved-state",
        &resolved_state_file,
        "--signing-key",
        &key_file,
        "--timestamp",
        "15",
        "--json",
    ]);
    assert_success(&merge);
    let merge_json = assert_json_status(&merge, "ok");
    assert_eq!(merge_json["merge_outcome"], "auto-merged");
    assert_eq!(merge_json["patch_op_count"], 1);
    assert_eq!(
        merge_json["parent_revision_ids"].as_array().map(Vec::len),
        Some(2)
    );
    assert!(merge_json["patch_id"]
        .as_str()
        .is_some_and(|value| value.starts_with("patch:")));
    assert!(merge_json["revision_id"]
        .as_str()
        .is_some_and(|value| value.starts_with("rev:")));

    let index = run_mycel(&["store", "index", &store_root, "--json"]);
    assert_success(&index);
    let index_json = assert_json_status(&index, "ok");
    assert_eq!(index_json["stored_object_count"], 8);
    assert_eq!(
        index_json["doc_revisions"]["doc:author-smoke"]
            .as_array()
            .map(Vec::len),
        Some(4)
    );
    assert_eq!(
        index_json["object_ids_by_type"]["patch"]
            .as_array()
            .map(Vec::len),
        Some(3)
    );
    assert_eq!(
        index_json["object_ids_by_type"]["revision"]
            .as_array()
            .map(Vec::len),
        Some(4)
    );
}

#[test]
fn store_merge_authoring_flow_supports_structural_move_and_insert() {
    let store_dir = create_temp_dir("store-merge-structural-root");
    let (_key_dir, key_path) = write_signing_key_file("store-merge-structural-key");
    let (_resolved_dir, resolved_state_path) =
        write_structural_resolved_state_file("store-merge-structural-state");
    let (_move_ops_dir, move_ops_path) =
        write_structural_move_ops_file("store-merge-structural-move-ops");
    let (_insert_ops_dir, insert_ops_path) =
        write_structural_insert_ops_file("store-merge-structural-insert-ops");
    let store_root = path_arg(&store_dir.path().to_path_buf());
    let key_file = path_arg(&key_path);
    let resolved_state_file = path_arg(&resolved_state_path);
    let move_ops_file = path_arg(&move_ops_path);
    let insert_ops_file = path_arg(&insert_ops_path);

    let init = run_mycel(&["store", "init", &store_root, "--json"]);
    assert_success(&init);

    let document = run_mycel(&[
        "store",
        "create-document",
        &store_root,
        "--doc-id",
        "doc:author-smoke-structural",
        "--title",
        "Author Smoke Structural Merge",
        "--language",
        "en",
        "--signing-key",
        &key_file,
        "--timestamp",
        "20",
        "--json",
    ]);
    assert_success(&document);
    let document_json = assert_json_status(&document, "ok");
    let genesis_revision_id = document_json["genesis_revision_id"]
        .as_str()
        .expect("genesis revision should be string")
        .to_string();

    let base_ops_dir = create_temp_dir("store-merge-structural-base-ops");
    let base_ops_path = base_ops_dir.path().join("ops.json");
    fs::write(
        &base_ops_path,
        serde_json::to_string_pretty(&json!([
            {
                "op": "insert_block",
                "new_block": {
                    "block_id": "blk:author-smoke-001",
                    "block_type": "paragraph",
                    "content": "Hello author smoke",
                    "attrs": {},
                    "children": []
                }
            },
            {
                "op": "insert_block",
                "new_block": {
                    "block_id": "blk:author-smoke-002",
                    "block_type": "paragraph",
                    "content": "Second structural block",
                    "attrs": {},
                    "children": []
                }
            }
        ]))
        .expect("base structural ops JSON should serialize"),
    )
    .expect("base structural ops JSON should write");
    let base_ops_file = path_arg(&base_ops_path);

    let base_patch = run_mycel(&[
        "store",
        "create-patch",
        &store_root,
        "--doc-id",
        "doc:author-smoke-structural",
        "--base-revision",
        &genesis_revision_id,
        "--ops",
        &base_ops_file,
        "--signing-key",
        &key_file,
        "--timestamp",
        "21",
        "--json",
    ]);
    assert_success(&base_patch);
    let base_patch_json = assert_json_status(&base_patch, "ok");
    let base_patch_id = base_patch_json["patch_id"]
        .as_str()
        .expect("base patch_id should be string")
        .to_string();

    let base_revision = run_mycel(&[
        "store",
        "commit-revision",
        &store_root,
        "--doc-id",
        "doc:author-smoke-structural",
        "--parent",
        &genesis_revision_id,
        "--patch",
        &base_patch_id,
        "--signing-key",
        &key_file,
        "--timestamp",
        "22",
        "--json",
    ]);
    assert_success(&base_revision);
    let base_revision_json = assert_json_status(&base_revision, "ok");
    let base_revision_id = base_revision_json["revision_id"]
        .as_str()
        .expect("base revision_id should be string")
        .to_string();

    let move_patch = run_mycel(&[
        "store",
        "create-patch",
        &store_root,
        "--doc-id",
        "doc:author-smoke-structural",
        "--base-revision",
        &base_revision_id,
        "--ops",
        &move_ops_file,
        "--signing-key",
        &key_file,
        "--timestamp",
        "23",
        "--json",
    ]);
    assert_success(&move_patch);
    let move_patch_json = assert_json_status(&move_patch, "ok");
    let move_patch_id = move_patch_json["patch_id"]
        .as_str()
        .expect("move patch_id should be string")
        .to_string();

    let move_revision = run_mycel(&[
        "store",
        "commit-revision",
        &store_root,
        "--doc-id",
        "doc:author-smoke-structural",
        "--parent",
        &base_revision_id,
        "--patch",
        &move_patch_id,
        "--signing-key",
        &key_file,
        "--timestamp",
        "24",
        "--json",
    ]);
    assert_success(&move_revision);
    let move_revision_json = assert_json_status(&move_revision, "ok");
    let move_revision_id = move_revision_json["revision_id"]
        .as_str()
        .expect("move revision_id should be string")
        .to_string();

    let insert_patch = run_mycel(&[
        "store",
        "create-patch",
        &store_root,
        "--doc-id",
        "doc:author-smoke-structural",
        "--base-revision",
        &base_revision_id,
        "--ops",
        &insert_ops_file,
        "--signing-key",
        &key_file,
        "--timestamp",
        "25",
        "--json",
    ]);
    assert_success(&insert_patch);
    let insert_patch_json = assert_json_status(&insert_patch, "ok");
    let insert_patch_id = insert_patch_json["patch_id"]
        .as_str()
        .expect("insert patch_id should be string")
        .to_string();

    let insert_revision = run_mycel(&[
        "store",
        "commit-revision",
        &store_root,
        "--doc-id",
        "doc:author-smoke-structural",
        "--parent",
        &base_revision_id,
        "--patch",
        &insert_patch_id,
        "--signing-key",
        &key_file,
        "--timestamp",
        "26",
        "--json",
    ]);
    assert_success(&insert_revision);
    let insert_revision_json = assert_json_status(&insert_revision, "ok");
    let insert_revision_id = insert_revision_json["revision_id"]
        .as_str()
        .expect("insert revision_id should be string")
        .to_string();

    let merge = run_mycel(&[
        "store",
        "create-merge-revision",
        &store_root,
        "--doc-id",
        "doc:author-smoke-structural",
        "--parent",
        &base_revision_id,
        "--parent",
        &move_revision_id,
        "--parent",
        &insert_revision_id,
        "--resolved-state",
        &resolved_state_file,
        "--signing-key",
        &key_file,
        "--timestamp",
        "27",
        "--json",
    ]);
    assert_success(&merge);
    let merge_json = assert_json_status(&merge, "ok");
    assert_eq!(merge_json["merge_outcome"], "multi-variant");
    assert!(
        merge_json["merge_reasons"]
            .as_array()
            .is_some_and(|reasons| reasons.iter().any(|reason| {
                reason.as_str().is_some_and(|reason| {
                    reason.contains("selected a non-primary sibling placement")
                })
            })),
        "expected structural sibling multi-variant reason, got {merge_json}"
    );
    assert_eq!(merge_json["patch_op_count"], 2);
    assert_eq!(
        merge_json["parent_revision_ids"].as_array().map(Vec::len),
        Some(3)
    );

    let index = run_mycel(&["store", "index", &store_root, "--json"]);
    assert_success(&index);
    let index_json = assert_json_status(&index, "ok");
    assert_eq!(index_json["stored_object_count"], 10);
    assert_eq!(
        index_json["doc_revisions"]["doc:author-smoke-structural"]
            .as_array()
            .map(Vec::len),
        Some(5)
    );
    assert_eq!(
        index_json["object_ids_by_type"]["patch"]
            .as_array()
            .map(Vec::len),
        Some(4)
    );
    assert_eq!(
        index_json["object_ids_by_type"]["revision"]
            .as_array()
            .map(Vec::len),
        Some(5)
    );
}

#[test]
fn store_merge_authoring_flow_reports_content_variant_choice_as_multi_variant() {
    let store_dir = create_temp_dir("store-merge-content-variant-root");
    let (_key_dir, key_path) = write_signing_key_file("store-merge-content-variant-key");
    let (_resolved_dir, resolved_state_path) = write_content_variant_resolved_state_file(
        "store-merge-content-variant-state",
        "Right variant",
    );
    let (_left_ops_dir, left_ops_path) =
        write_content_variant_ops_file("store-merge-content-variant-left-ops", "Left variant");
    let (_right_ops_dir, right_ops_path) =
        write_content_variant_ops_file("store-merge-content-variant-right-ops", "Right variant");
    let (_center_ops_dir, center_ops_path) =
        write_content_variant_ops_file("store-merge-content-variant-center-ops", "Center variant");
    let store_root = path_arg(&store_dir.path().to_path_buf());
    let key_file = path_arg(&key_path);
    let resolved_state_file = path_arg(&resolved_state_path);
    let left_ops_file = path_arg(&left_ops_path);
    let right_ops_file = path_arg(&right_ops_path);
    let center_ops_file = path_arg(&center_ops_path);

    let init = run_mycel(&["store", "init", &store_root, "--json"]);
    assert_success(&init);

    let document = run_mycel(&[
        "store",
        "create-document",
        &store_root,
        "--doc-id",
        "doc:author-smoke-content-variant",
        "--title",
        "Author Smoke Content Variant",
        "--language",
        "en",
        "--signing-key",
        &key_file,
        "--timestamp",
        "30",
        "--json",
    ]);
    assert_success(&document);
    let document_json = assert_json_status(&document, "ok");
    let genesis_revision_id = document_json["genesis_revision_id"]
        .as_str()
        .expect("genesis revision should be string")
        .to_string();

    let base_ops_dir = create_temp_dir("store-merge-content-variant-base-ops");
    let base_ops_path = base_ops_dir.path().join("ops.json");
    fs::write(
        &base_ops_path,
        serde_json::to_string_pretty(&json!([
            {
                "op": "insert_block",
                "new_block": {
                    "block_id": "blk:author-smoke-variant-001",
                    "block_type": "paragraph",
                    "content": "Base",
                    "attrs": {},
                    "children": []
                }
            }
        ]))
        .expect("content variant base ops JSON should serialize"),
    )
    .expect("content variant base ops JSON should write");
    let base_ops_file = path_arg(&base_ops_path);

    let base_patch = run_mycel(&[
        "store",
        "create-patch",
        &store_root,
        "--doc-id",
        "doc:author-smoke-content-variant",
        "--base-revision",
        &genesis_revision_id,
        "--ops",
        &base_ops_file,
        "--signing-key",
        &key_file,
        "--timestamp",
        "31",
        "--json",
    ]);
    assert_success(&base_patch);
    let base_patch_json = assert_json_status(&base_patch, "ok");
    let base_patch_id = base_patch_json["patch_id"]
        .as_str()
        .expect("base patch_id should be string")
        .to_string();

    let base_revision = run_mycel(&[
        "store",
        "commit-revision",
        &store_root,
        "--doc-id",
        "doc:author-smoke-content-variant",
        "--parent",
        &genesis_revision_id,
        "--patch",
        &base_patch_id,
        "--signing-key",
        &key_file,
        "--timestamp",
        "32",
        "--json",
    ]);
    assert_success(&base_revision);
    let base_revision_json = assert_json_status(&base_revision, "ok");
    let base_revision_id = base_revision_json["revision_id"]
        .as_str()
        .expect("base revision_id should be string")
        .to_string();

    let left_patch = run_mycel(&[
        "store",
        "create-patch",
        &store_root,
        "--doc-id",
        "doc:author-smoke-content-variant",
        "--base-revision",
        &base_revision_id,
        "--ops",
        &left_ops_file,
        "--signing-key",
        &key_file,
        "--timestamp",
        "33",
        "--json",
    ]);
    assert_success(&left_patch);
    let left_patch_json = assert_json_status(&left_patch, "ok");
    let left_patch_id = left_patch_json["patch_id"]
        .as_str()
        .expect("left patch_id should be string")
        .to_string();

    let left_revision = run_mycel(&[
        "store",
        "commit-revision",
        &store_root,
        "--doc-id",
        "doc:author-smoke-content-variant",
        "--parent",
        &base_revision_id,
        "--patch",
        &left_patch_id,
        "--signing-key",
        &key_file,
        "--timestamp",
        "34",
        "--json",
    ]);
    assert_success(&left_revision);
    let left_revision_json = assert_json_status(&left_revision, "ok");
    let left_revision_id = left_revision_json["revision_id"]
        .as_str()
        .expect("left revision_id should be string")
        .to_string();

    let right_patch = run_mycel(&[
        "store",
        "create-patch",
        &store_root,
        "--doc-id",
        "doc:author-smoke-content-variant",
        "--base-revision",
        &base_revision_id,
        "--ops",
        &right_ops_file,
        "--signing-key",
        &key_file,
        "--timestamp",
        "35",
        "--json",
    ]);
    assert_success(&right_patch);
    let right_patch_json = assert_json_status(&right_patch, "ok");
    let right_patch_id = right_patch_json["patch_id"]
        .as_str()
        .expect("right patch_id should be string")
        .to_string();

    let right_revision = run_mycel(&[
        "store",
        "commit-revision",
        &store_root,
        "--doc-id",
        "doc:author-smoke-content-variant",
        "--parent",
        &base_revision_id,
        "--patch",
        &right_patch_id,
        "--signing-key",
        &key_file,
        "--timestamp",
        "36",
        "--json",
    ]);
    assert_success(&right_revision);
    let right_revision_json = assert_json_status(&right_revision, "ok");
    let right_revision_id = right_revision_json["revision_id"]
        .as_str()
        .expect("right revision_id should be string")
        .to_string();

    let center_patch = run_mycel(&[
        "store",
        "create-patch",
        &store_root,
        "--doc-id",
        "doc:author-smoke-content-variant",
        "--base-revision",
        &base_revision_id,
        "--ops",
        &center_ops_file,
        "--signing-key",
        &key_file,
        "--timestamp",
        "37",
        "--json",
    ]);
    assert_success(&center_patch);
    let center_patch_json = assert_json_status(&center_patch, "ok");
    let center_patch_id = center_patch_json["patch_id"]
        .as_str()
        .expect("center patch_id should be string")
        .to_string();

    let center_revision = run_mycel(&[
        "store",
        "commit-revision",
        &store_root,
        "--doc-id",
        "doc:author-smoke-content-variant",
        "--parent",
        &base_revision_id,
        "--patch",
        &center_patch_id,
        "--signing-key",
        &key_file,
        "--timestamp",
        "38",
        "--json",
    ]);
    assert_success(&center_revision);
    let center_revision_json = assert_json_status(&center_revision, "ok");
    let center_revision_id = center_revision_json["revision_id"]
        .as_str()
        .expect("center revision_id should be string")
        .to_string();

    let merge = run_mycel(&[
        "store",
        "create-merge-revision",
        &store_root,
        "--doc-id",
        "doc:author-smoke-content-variant",
        "--parent",
        &left_revision_id,
        "--parent",
        &right_revision_id,
        "--parent",
        &center_revision_id,
        "--resolved-state",
        &resolved_state_file,
        "--signing-key",
        &key_file,
        "--timestamp",
        "39",
        "--json",
    ]);
    assert_success(&merge);
    let merge_json = assert_json_status(&merge, "ok");
    assert_eq!(merge_json["merge_outcome"], "multi-variant");
    assert!(
        merge_json["merge_reasons"]
            .as_array()
            .is_some_and(|reasons| reasons.iter().any(|reason| {
                reason
                    .as_str()
                    .is_some_and(|reason| reason.contains("selected a non-primary parent variant"))
            })),
        "expected content variant multi-variant reason, got {merge_json}"
    );
    assert!(
        merge_json["merge_reasons"]
            .as_array()
            .is_some_and(|reasons| reasons.iter().any(|reason| {
                reason
                    .as_str()
                    .is_some_and(|reason| reason.contains("multiple competing parent variants"))
            })),
        "expected competing content variant reason, got {merge_json}"
    );
    assert_eq!(merge_json["patch_op_count"], 1);
    assert_eq!(
        merge_json["parent_revision_ids"].as_array().map(Vec::len),
        Some(3)
    );
}

#[test]
fn store_merge_authoring_flow_reports_nested_parent_choice_as_multi_variant() {
    let store_dir = create_temp_dir("store-merge-nested-parent-root");
    let (_key_dir, key_path) = write_signing_key_file("store-merge-nested-parent-key");
    let (_resolved_dir, resolved_state_path) =
        write_nested_parent_choice_resolved_state_file("store-merge-nested-parent-state");
    let store_root = path_arg(&store_dir.path().to_path_buf());
    let key_file = path_arg(&key_path);
    let resolved_state_file = path_arg(&resolved_state_path);

    let init = run_mycel(&["store", "init", &store_root, "--json"]);
    assert_success(&init);

    let document = run_mycel(&[
        "store",
        "create-document",
        &store_root,
        "--doc-id",
        "doc:author-smoke-nested-parent-choice",
        "--title",
        "Author Smoke Nested Parent Choice",
        "--language",
        "en",
        "--signing-key",
        &key_file,
        "--timestamp",
        "30",
        "--json",
    ]);
    assert_success(&document);
    let document_json = assert_json_status(&document, "ok");
    let genesis_revision_id = document_json["genesis_revision_id"]
        .as_str()
        .expect("genesis revision should be string")
        .to_string();

    let base_ops_dir = create_temp_dir("store-merge-nested-parent-base-ops");
    let base_ops_path = base_ops_dir.path().join("ops.json");
    fs::write(
        &base_ops_path,
        serde_json::to_string_pretty(&json!([
            {
                "op": "insert_block",
                "new_block": {
                    "block_id": "blk:nested-left",
                    "block_type": "paragraph",
                    "content": "Left",
                    "attrs": {},
                    "children": []
                }
            },
            {
                "op": "insert_block",
                "new_block": {
                    "block_id": "blk:nested-right",
                    "block_type": "paragraph",
                    "content": "Right",
                    "attrs": {},
                    "children": []
                }
            },
            {
                "op": "insert_block",
                "new_block": {
                    "block_id": "blk:nested-leaf",
                    "block_type": "paragraph",
                    "content": "Leaf",
                    "attrs": {},
                    "children": []
                }
            }
        ]))
        .expect("nested parent base ops JSON should serialize"),
    )
    .expect("nested parent base ops JSON should write");
    let base_ops_file = path_arg(&base_ops_path);

    let wrapper_ops_dir = create_temp_dir("store-merge-nested-parent-wrapper-ops");
    let wrapper_ops_path = wrapper_ops_dir.path().join("ops.json");
    fs::write(
        &wrapper_ops_path,
        serde_json::to_string_pretty(&json!([
            {
                "op": "insert_block",
                "new_block": {
                    "block_id": "blk:nested-wrapper",
                    "block_type": "paragraph",
                    "content": "Wrapper",
                    "attrs": {},
                    "children": []
                }
            }
        ]))
        .expect("nested parent wrapper ops JSON should serialize"),
    )
    .expect("nested parent wrapper ops JSON should write");
    let wrapper_ops_file = path_arg(&wrapper_ops_path);

    let left_ops_dir = create_temp_dir("store-merge-nested-parent-left-ops");
    let left_ops_path = left_ops_dir.path().join("ops.json");
    fs::write(
        &left_ops_path,
        serde_json::to_string_pretty(&json!([
            {
                "op": "move_block",
                "block_id": "blk:nested-leaf",
                "parent_block_id": "blk:nested-left"
            }
        ]))
        .expect("nested parent left ops JSON should serialize"),
    )
    .expect("nested parent left ops JSON should write");
    let left_ops_file = path_arg(&left_ops_path);

    let right_ops_dir = create_temp_dir("store-merge-nested-parent-right-ops");
    let right_ops_path = right_ops_dir.path().join("ops.json");
    fs::write(
        &right_ops_path,
        serde_json::to_string_pretty(&json!([
            {
                "op": "move_block",
                "block_id": "blk:nested-leaf",
                "parent_block_id": "blk:nested-right"
            }
        ]))
        .expect("nested parent right ops JSON should serialize"),
    )
    .expect("nested parent right ops JSON should write");
    let right_ops_file = path_arg(&right_ops_path);

    let base_patch = run_mycel(&[
        "store",
        "create-patch",
        &store_root,
        "--doc-id",
        "doc:author-smoke-nested-parent-choice",
        "--base-revision",
        &genesis_revision_id,
        "--ops",
        &base_ops_file,
        "--signing-key",
        &key_file,
        "--timestamp",
        "31",
        "--json",
    ]);
    assert_success(&base_patch);
    let base_patch_json = assert_json_status(&base_patch, "ok");
    let base_patch_id = base_patch_json["patch_id"]
        .as_str()
        .expect("base patch_id should be string")
        .to_string();

    let base_revision = run_mycel(&[
        "store",
        "commit-revision",
        &store_root,
        "--doc-id",
        "doc:author-smoke-nested-parent-choice",
        "--parent",
        &genesis_revision_id,
        "--patch",
        &base_patch_id,
        "--signing-key",
        &key_file,
        "--timestamp",
        "32",
        "--json",
    ]);
    assert_success(&base_revision);
    let base_revision_json = assert_json_status(&base_revision, "ok");
    let base_revision_id = base_revision_json["revision_id"]
        .as_str()
        .expect("base revision_id should be string")
        .to_string();

    let wrapper_patch = run_mycel(&[
        "store",
        "create-patch",
        &store_root,
        "--doc-id",
        "doc:author-smoke-nested-parent-choice",
        "--base-revision",
        &base_revision_id,
        "--ops",
        &wrapper_ops_file,
        "--signing-key",
        &key_file,
        "--timestamp",
        "33",
        "--json",
    ]);
    assert_success(&wrapper_patch);
    let wrapper_patch_json = assert_json_status(&wrapper_patch, "ok");
    let wrapper_patch_id = wrapper_patch_json["patch_id"]
        .as_str()
        .expect("wrapper patch_id should be string")
        .to_string();

    let wrapper_revision = run_mycel(&[
        "store",
        "commit-revision",
        &store_root,
        "--doc-id",
        "doc:author-smoke-nested-parent-choice",
        "--parent",
        &base_revision_id,
        "--patch",
        &wrapper_patch_id,
        "--signing-key",
        &key_file,
        "--timestamp",
        "34",
        "--json",
    ]);
    assert_success(&wrapper_revision);
    let wrapper_revision_json = assert_json_status(&wrapper_revision, "ok");
    let wrapper_revision_id = wrapper_revision_json["revision_id"]
        .as_str()
        .expect("wrapper revision_id should be string")
        .to_string();

    let left_patch = run_mycel(&[
        "store",
        "create-patch",
        &store_root,
        "--doc-id",
        "doc:author-smoke-nested-parent-choice",
        "--base-revision",
        &base_revision_id,
        "--ops",
        &left_ops_file,
        "--signing-key",
        &key_file,
        "--timestamp",
        "35",
        "--json",
    ]);
    assert_success(&left_patch);
    let left_patch_json = assert_json_status(&left_patch, "ok");
    let left_patch_id = left_patch_json["patch_id"]
        .as_str()
        .expect("left patch_id should be string")
        .to_string();

    let left_revision = run_mycel(&[
        "store",
        "commit-revision",
        &store_root,
        "--doc-id",
        "doc:author-smoke-nested-parent-choice",
        "--parent",
        &base_revision_id,
        "--patch",
        &left_patch_id,
        "--signing-key",
        &key_file,
        "--timestamp",
        "36",
        "--json",
    ]);
    assert_success(&left_revision);
    let left_revision_json = assert_json_status(&left_revision, "ok");
    let left_revision_id = left_revision_json["revision_id"]
        .as_str()
        .expect("left revision_id should be string")
        .to_string();

    let right_patch = run_mycel(&[
        "store",
        "create-patch",
        &store_root,
        "--doc-id",
        "doc:author-smoke-nested-parent-choice",
        "--base-revision",
        &base_revision_id,
        "--ops",
        &right_ops_file,
        "--signing-key",
        &key_file,
        "--timestamp",
        "37",
        "--json",
    ]);
    assert_success(&right_patch);
    let right_patch_json = assert_json_status(&right_patch, "ok");
    let right_patch_id = right_patch_json["patch_id"]
        .as_str()
        .expect("right patch_id should be string")
        .to_string();

    let right_revision = run_mycel(&[
        "store",
        "commit-revision",
        &store_root,
        "--doc-id",
        "doc:author-smoke-nested-parent-choice",
        "--parent",
        &base_revision_id,
        "--patch",
        &right_patch_id,
        "--signing-key",
        &key_file,
        "--timestamp",
        "38",
        "--json",
    ]);
    assert_success(&right_revision);
    let right_revision_json = assert_json_status(&right_revision, "ok");
    let right_revision_id = right_revision_json["revision_id"]
        .as_str()
        .expect("right revision_id should be string")
        .to_string();

    let merge = run_mycel(&[
        "store",
        "create-merge-revision",
        &store_root,
        "--doc-id",
        "doc:author-smoke-nested-parent-choice",
        "--parent",
        &base_revision_id,
        "--parent",
        &wrapper_revision_id,
        "--parent",
        &left_revision_id,
        "--parent",
        &right_revision_id,
        "--resolved-state",
        &resolved_state_file,
        "--signing-key",
        &key_file,
        "--timestamp",
        "39",
        "--json",
    ]);
    assert_success(&merge);
    let merge_json = assert_json_status(&merge, "ok");
    assert_eq!(merge_json["merge_outcome"], "multi-variant");
    assert!(
        merge_json["merge_reasons"]
            .as_array()
            .is_some_and(|reasons| reasons.iter().any(|reason| {
                reason.as_str().is_some_and(|reason| {
                    reason.contains("selected a non-primary parent placement")
                })
            })),
        "expected nested parent multi-variant reason, got {merge_json}"
    );
    assert!(
        merge_json["merge_reasons"]
            .as_array()
            .is_some_and(|reasons| reasons.iter().any(|reason| {
                reason
                    .as_str()
                    .is_some_and(|reason| reason.contains("multiple competing parent placements"))
            })),
        "expected competing nested parent placement reason, got {merge_json}"
    );
    assert_eq!(merge_json["patch_op_count"], 4);
    assert_eq!(
        merge_json["parent_revision_ids"].as_array().map(Vec::len),
        Some(4)
    );

    let index = run_mycel(&["store", "index", &store_root, "--json"]);
    assert_success(&index);
    let index_json = assert_json_status(&index, "ok");
    assert_eq!(index_json["stored_object_count"], 12);
    assert_eq!(
        index_json["doc_revisions"]["doc:author-smoke-nested-parent-choice"]
            .as_array()
            .map(Vec::len),
        Some(6)
    );
    assert_eq!(
        index_json["object_ids_by_type"]["patch"]
            .as_array()
            .map(Vec::len),
        Some(5)
    );
    assert_eq!(
        index_json["object_ids_by_type"]["revision"]
            .as_array()
            .map(Vec::len),
        Some(6)
    );
}

#[test]
fn store_merge_authoring_flow_reports_anchor_nested_parent_choice_as_multi_variant() {
    let store_dir = create_temp_dir("store-merge-nested-parent-anchor-root");
    let (_key_dir, key_path) = write_signing_key_file("store-merge-nested-parent-anchor-key");
    let (_resolved_dir, resolved_state_path) =
        write_nested_parent_anchor_choice_resolved_state_file(
            "store-merge-nested-parent-anchor-state",
        );
    let store_root = path_arg(&store_dir.path().to_path_buf());
    let key_file = path_arg(&key_path);
    let resolved_state_file = path_arg(&resolved_state_path);

    let init = run_mycel(&["store", "init", &store_root, "--json"]);
    assert_success(&init);

    let document = run_mycel(&[
        "store",
        "create-document",
        &store_root,
        "--doc-id",
        "doc:author-smoke-nested-parent-anchor-choice",
        "--title",
        "Author Smoke Nested Parent Anchor Choice",
        "--language",
        "en",
        "--signing-key",
        &key_file,
        "--timestamp",
        "30",
        "--json",
    ]);
    assert_success(&document);
    let document_json = assert_json_status(&document, "ok");
    let genesis_revision_id = document_json["genesis_revision_id"]
        .as_str()
        .expect("genesis revision should be string")
        .to_string();

    let base_ops_dir = create_temp_dir("store-merge-nested-parent-anchor-base-ops");
    let base_ops_path = base_ops_dir.path().join("ops.json");
    fs::write(
        &base_ops_path,
        serde_json::to_string_pretty(&json!([
            {
                "op": "insert_block",
                "new_block": {
                    "block_id": "blk:nested-left",
                    "block_type": "paragraph",
                    "content": "Left",
                    "attrs": {},
                    "children": []
                }
            },
            {
                "op": "insert_block",
                "new_block": {
                    "block_id": "blk:nested-right",
                    "block_type": "paragraph",
                    "content": "Right",
                    "attrs": {},
                    "children": []
                }
            },
            {
                "op": "insert_block",
                "new_block": {
                    "block_id": "blk:nested-leaf",
                    "block_type": "paragraph",
                    "content": "Leaf",
                    "attrs": {},
                    "children": []
                }
            }
        ]))
        .expect("nested parent anchor base ops JSON should serialize"),
    )
    .expect("nested parent anchor base ops JSON should write");
    let base_ops_file = path_arg(&base_ops_path);

    let subsection_ops_dir = create_temp_dir("store-merge-nested-parent-anchor-subsection-ops");
    let subsection_ops_path = subsection_ops_dir.path().join("ops.json");
    fs::write(
        &subsection_ops_path,
        serde_json::to_string_pretty(&json!([
            {
                "op": "insert_block",
                "parent_block_id": "blk:nested-left",
                "new_block": {
                    "block_id": "blk:nested-subsection",
                    "block_type": "paragraph",
                    "content": "Subsection",
                    "attrs": {},
                    "children": []
                }
            }
        ]))
        .expect("nested parent anchor subsection ops JSON should serialize"),
    )
    .expect("nested parent anchor subsection ops JSON should write");
    let subsection_ops_file = path_arg(&subsection_ops_path);

    let left_ops_dir = create_temp_dir("store-merge-nested-parent-anchor-left-ops");
    let left_ops_path = left_ops_dir.path().join("ops.json");
    fs::write(
        &left_ops_path,
        serde_json::to_string_pretty(&json!([
            {
                "op": "move_block",
                "block_id": "blk:nested-leaf",
                "parent_block_id": "blk:nested-left"
            }
        ]))
        .expect("nested parent anchor left ops JSON should serialize"),
    )
    .expect("nested parent anchor left ops JSON should write");
    let left_ops_file = path_arg(&left_ops_path);

    let right_ops_dir = create_temp_dir("store-merge-nested-parent-anchor-right-ops");
    let right_ops_path = right_ops_dir.path().join("ops.json");
    fs::write(
        &right_ops_path,
        serde_json::to_string_pretty(&json!([
            {
                "op": "move_block",
                "block_id": "blk:nested-leaf",
                "parent_block_id": "blk:nested-right"
            }
        ]))
        .expect("nested parent anchor right ops JSON should serialize"),
    )
    .expect("nested parent anchor right ops JSON should write");
    let right_ops_file = path_arg(&right_ops_path);

    let base_patch = run_mycel(&[
        "store",
        "create-patch",
        &store_root,
        "--doc-id",
        "doc:author-smoke-nested-parent-anchor-choice",
        "--base-revision",
        &genesis_revision_id,
        "--ops",
        &base_ops_file,
        "--signing-key",
        &key_file,
        "--timestamp",
        "31",
        "--json",
    ]);
    assert_success(&base_patch);
    let base_patch_json = assert_json_status(&base_patch, "ok");
    let base_patch_id = base_patch_json["patch_id"]
        .as_str()
        .expect("base patch_id should be string")
        .to_string();

    let base_revision = run_mycel(&[
        "store",
        "commit-revision",
        &store_root,
        "--doc-id",
        "doc:author-smoke-nested-parent-anchor-choice",
        "--parent",
        &genesis_revision_id,
        "--patch",
        &base_patch_id,
        "--signing-key",
        &key_file,
        "--timestamp",
        "32",
        "--json",
    ]);
    assert_success(&base_revision);
    let base_revision_json = assert_json_status(&base_revision, "ok");
    let base_revision_id = base_revision_json["revision_id"]
        .as_str()
        .expect("base revision_id should be string")
        .to_string();

    let subsection_patch = run_mycel(&[
        "store",
        "create-patch",
        &store_root,
        "--doc-id",
        "doc:author-smoke-nested-parent-anchor-choice",
        "--base-revision",
        &base_revision_id,
        "--ops",
        &subsection_ops_file,
        "--signing-key",
        &key_file,
        "--timestamp",
        "33",
        "--json",
    ]);
    assert_success(&subsection_patch);
    let subsection_patch_json = assert_json_status(&subsection_patch, "ok");
    let subsection_patch_id = subsection_patch_json["patch_id"]
        .as_str()
        .expect("subsection patch_id should be string")
        .to_string();

    let subsection_revision = run_mycel(&[
        "store",
        "commit-revision",
        &store_root,
        "--doc-id",
        "doc:author-smoke-nested-parent-anchor-choice",
        "--parent",
        &base_revision_id,
        "--patch",
        &subsection_patch_id,
        "--signing-key",
        &key_file,
        "--timestamp",
        "34",
        "--json",
    ]);
    assert_success(&subsection_revision);
    let subsection_revision_json = assert_json_status(&subsection_revision, "ok");
    let subsection_revision_id = subsection_revision_json["revision_id"]
        .as_str()
        .expect("subsection revision_id should be string")
        .to_string();

    let left_patch = run_mycel(&[
        "store",
        "create-patch",
        &store_root,
        "--doc-id",
        "doc:author-smoke-nested-parent-anchor-choice",
        "--base-revision",
        &base_revision_id,
        "--ops",
        &left_ops_file,
        "--signing-key",
        &key_file,
        "--timestamp",
        "35",
        "--json",
    ]);
    assert_success(&left_patch);
    let left_patch_json = assert_json_status(&left_patch, "ok");
    let left_patch_id = left_patch_json["patch_id"]
        .as_str()
        .expect("left patch_id should be string")
        .to_string();

    let left_revision = run_mycel(&[
        "store",
        "commit-revision",
        &store_root,
        "--doc-id",
        "doc:author-smoke-nested-parent-anchor-choice",
        "--parent",
        &base_revision_id,
        "--patch",
        &left_patch_id,
        "--signing-key",
        &key_file,
        "--timestamp",
        "36",
        "--json",
    ]);
    assert_success(&left_revision);
    let left_revision_json = assert_json_status(&left_revision, "ok");
    let left_revision_id = left_revision_json["revision_id"]
        .as_str()
        .expect("left revision_id should be string")
        .to_string();

    let right_patch = run_mycel(&[
        "store",
        "create-patch",
        &store_root,
        "--doc-id",
        "doc:author-smoke-nested-parent-anchor-choice",
        "--base-revision",
        &base_revision_id,
        "--ops",
        &right_ops_file,
        "--signing-key",
        &key_file,
        "--timestamp",
        "37",
        "--json",
    ]);
    assert_success(&right_patch);
    let right_patch_json = assert_json_status(&right_patch, "ok");
    let right_patch_id = right_patch_json["patch_id"]
        .as_str()
        .expect("right patch_id should be string")
        .to_string();

    let right_revision = run_mycel(&[
        "store",
        "commit-revision",
        &store_root,
        "--doc-id",
        "doc:author-smoke-nested-parent-anchor-choice",
        "--parent",
        &base_revision_id,
        "--patch",
        &right_patch_id,
        "--signing-key",
        &key_file,
        "--timestamp",
        "38",
        "--json",
    ]);
    assert_success(&right_revision);
    let right_revision_json = assert_json_status(&right_revision, "ok");
    let right_revision_id = right_revision_json["revision_id"]
        .as_str()
        .expect("right revision_id should be string")
        .to_string();

    let merge = run_mycel(&[
        "store",
        "create-merge-revision",
        &store_root,
        "--doc-id",
        "doc:author-smoke-nested-parent-anchor-choice",
        "--parent",
        &base_revision_id,
        "--parent",
        &subsection_revision_id,
        "--parent",
        &left_revision_id,
        "--parent",
        &right_revision_id,
        "--resolved-state",
        &resolved_state_file,
        "--signing-key",
        &key_file,
        "--timestamp",
        "39",
        "--json",
    ]);
    assert_success(&merge);
    let merge_json = assert_json_status(&merge, "ok");
    assert_eq!(merge_json["merge_outcome"], "multi-variant");
    assert!(
        merge_json["merge_reasons"]
            .as_array()
            .is_some_and(|reasons| reasons.iter().any(|reason| {
                reason.as_str().is_some_and(|reason| {
                    reason.contains("selected a non-primary parent placement")
                })
            })),
        "expected anchor nested parent multi-variant reason, got {merge_json}"
    );
    assert!(
        merge_json["merge_reasons"]
            .as_array()
            .is_some_and(|reasons| reasons.iter().any(|reason| {
                reason
                    .as_str()
                    .is_some_and(|reason| reason.contains("multiple competing parent placements"))
            })),
        "expected competing anchor nested parent placement reason, got {merge_json}"
    );
}

#[test]
fn store_merge_authoring_flow_rejects_novel_nested_parent_choice_as_manual_curation_required() {
    let store_dir = create_temp_dir("store-merge-nested-parent-manual-root");
    let (_key_dir, key_path) = write_signing_key_file("store-merge-nested-parent-manual-key");
    let (_resolved_dir, resolved_state_path) =
        write_nested_parent_manual_resolved_state_file("store-merge-nested-parent-manual-state");
    let store_root = path_arg(&store_dir.path().to_path_buf());
    let key_file = path_arg(&key_path);
    let resolved_state_file = path_arg(&resolved_state_path);

    let init = run_mycel(&["store", "init", &store_root, "--json"]);
    assert_success(&init);

    let document = run_mycel(&[
        "store",
        "create-document",
        &store_root,
        "--doc-id",
        "doc:author-smoke-nested-parent-manual",
        "--title",
        "Author Smoke Nested Parent Manual",
        "--language",
        "en",
        "--signing-key",
        &key_file,
        "--timestamp",
        "40",
        "--json",
    ]);
    assert_success(&document);
    let document_json = assert_json_status(&document, "ok");
    let genesis_revision_id = document_json["genesis_revision_id"]
        .as_str()
        .expect("genesis revision should be string")
        .to_string();

    let base_ops_dir = create_temp_dir("store-merge-nested-parent-manual-base-ops");
    let base_ops_path = base_ops_dir.path().join("ops.json");
    fs::write(
        &base_ops_path,
        serde_json::to_string_pretty(&json!([
            {
                "op": "insert_block",
                "new_block": {
                    "block_id": "blk:manual-left",
                    "block_type": "paragraph",
                    "content": "Left",
                    "attrs": {},
                    "children": []
                }
            },
            {
                "op": "insert_block",
                "new_block": {
                    "block_id": "blk:manual-right",
                    "block_type": "paragraph",
                    "content": "Right",
                    "attrs": {},
                    "children": []
                }
            },
            {
                "op": "insert_block",
                "new_block": {
                    "block_id": "blk:manual-leaf",
                    "block_type": "paragraph",
                    "content": "Leaf",
                    "attrs": {},
                    "children": []
                }
            }
        ]))
        .expect("nested parent manual base ops JSON should serialize"),
    )
    .expect("nested parent manual base ops JSON should write");
    let base_ops_file = path_arg(&base_ops_path);

    let wrapper_ops_dir = create_temp_dir("store-merge-nested-parent-manual-wrapper-ops");
    let wrapper_ops_path = wrapper_ops_dir.path().join("ops.json");
    fs::write(
        &wrapper_ops_path,
        serde_json::to_string_pretty(&json!([
            {
                "op": "insert_block",
                "parent_block_id": "blk:manual-right",
                "new_block": {
                    "block_id": "blk:manual-wrapper",
                    "block_type": "paragraph",
                    "content": "Wrapper",
                    "attrs": {},
                    "children": []
                }
            }
        ]))
        .expect("nested parent manual wrapper ops JSON should serialize"),
    )
    .expect("nested parent manual wrapper ops JSON should write");
    let wrapper_ops_file = path_arg(&wrapper_ops_path);

    let move_ops_dir = create_temp_dir("store-merge-nested-parent-manual-move-ops");
    let move_ops_path = move_ops_dir.path().join("ops.json");
    fs::write(
        &move_ops_path,
        serde_json::to_string_pretty(&json!([
            {
                "op": "move_block",
                "block_id": "blk:manual-leaf",
                "parent_block_id": "blk:manual-left"
            }
        ]))
        .expect("nested parent manual move ops JSON should serialize"),
    )
    .expect("nested parent manual move ops JSON should write");
    let move_ops_file = path_arg(&move_ops_path);

    let base_patch = run_mycel(&[
        "store",
        "create-patch",
        &store_root,
        "--doc-id",
        "doc:author-smoke-nested-parent-manual",
        "--base-revision",
        &genesis_revision_id,
        "--ops",
        &base_ops_file,
        "--signing-key",
        &key_file,
        "--timestamp",
        "41",
        "--json",
    ]);
    assert_success(&base_patch);
    let base_patch_json = assert_json_status(&base_patch, "ok");
    let base_patch_id = base_patch_json["patch_id"]
        .as_str()
        .expect("base patch_id should be string")
        .to_string();

    let base_revision = run_mycel(&[
        "store",
        "commit-revision",
        &store_root,
        "--doc-id",
        "doc:author-smoke-nested-parent-manual",
        "--parent",
        &genesis_revision_id,
        "--patch",
        &base_patch_id,
        "--signing-key",
        &key_file,
        "--timestamp",
        "42",
        "--json",
    ]);
    assert_success(&base_revision);
    let base_revision_json = assert_json_status(&base_revision, "ok");
    let base_revision_id = base_revision_json["revision_id"]
        .as_str()
        .expect("base revision_id should be string")
        .to_string();

    let wrapper_patch = run_mycel(&[
        "store",
        "create-patch",
        &store_root,
        "--doc-id",
        "doc:author-smoke-nested-parent-manual",
        "--base-revision",
        &base_revision_id,
        "--ops",
        &wrapper_ops_file,
        "--signing-key",
        &key_file,
        "--timestamp",
        "43",
        "--json",
    ]);
    assert_success(&wrapper_patch);
    let wrapper_patch_json = assert_json_status(&wrapper_patch, "ok");
    let wrapper_patch_id = wrapper_patch_json["patch_id"]
        .as_str()
        .expect("wrapper patch_id should be string")
        .to_string();

    let wrapper_revision = run_mycel(&[
        "store",
        "commit-revision",
        &store_root,
        "--doc-id",
        "doc:author-smoke-nested-parent-manual",
        "--parent",
        &base_revision_id,
        "--patch",
        &wrapper_patch_id,
        "--signing-key",
        &key_file,
        "--timestamp",
        "44",
        "--json",
    ]);
    assert_success(&wrapper_revision);
    let wrapper_revision_json = assert_json_status(&wrapper_revision, "ok");
    let wrapper_revision_id = wrapper_revision_json["revision_id"]
        .as_str()
        .expect("wrapper revision_id should be string")
        .to_string();

    let move_patch = run_mycel(&[
        "store",
        "create-patch",
        &store_root,
        "--doc-id",
        "doc:author-smoke-nested-parent-manual",
        "--base-revision",
        &base_revision_id,
        "--ops",
        &move_ops_file,
        "--signing-key",
        &key_file,
        "--timestamp",
        "45",
        "--json",
    ]);
    assert_success(&move_patch);
    let move_patch_json = assert_json_status(&move_patch, "ok");
    let move_patch_id = move_patch_json["patch_id"]
        .as_str()
        .expect("move patch_id should be string")
        .to_string();

    let move_revision = run_mycel(&[
        "store",
        "commit-revision",
        &store_root,
        "--doc-id",
        "doc:author-smoke-nested-parent-manual",
        "--parent",
        &base_revision_id,
        "--patch",
        &move_patch_id,
        "--signing-key",
        &key_file,
        "--timestamp",
        "46",
        "--json",
    ]);
    assert_success(&move_revision);
    let move_revision_json = assert_json_status(&move_revision, "ok");
    let move_revision_id = move_revision_json["revision_id"]
        .as_str()
        .expect("move revision_id should be string")
        .to_string();

    let merge = run_mycel(&[
        "store",
        "create-merge-revision",
        &store_root,
        "--doc-id",
        "doc:author-smoke-nested-parent-manual",
        "--parent",
        &base_revision_id,
        "--parent",
        &wrapper_revision_id,
        "--parent",
        &move_revision_id,
        "--resolved-state",
        &resolved_state_file,
        "--signing-key",
        &key_file,
        "--timestamp",
        "47",
    ]);

    assert!(
        !merge.status.success(),
        "expected manual-curation failure, stdout: {}, stderr: {}",
        String::from_utf8_lossy(&merge.stdout),
        String::from_utf8_lossy(&merge.stderr)
    );
    assert_stderr_contains(
        &merge,
        "merge resolution is manual-curation-required: resolved block 'blk:manual-leaf' does not match any parent placement",
    );
}

#[test]
fn store_merge_authoring_flow_marks_nested_sibling_choice_through_inserted_sibling_as_multi_variant(
) {
    let store_dir = create_temp_dir("store-merge-nested-sibling-manual-root");
    let (_key_dir, key_path) = write_signing_key_file("store-merge-nested-sibling-manual-key");
    let (_resolved_dir, resolved_state_path) =
        write_nested_sibling_manual_resolved_state_file("store-merge-nested-sibling-manual-state");
    let store_root = path_arg(&store_dir.path().to_path_buf());
    let key_file = path_arg(&key_path);
    let resolved_state_file = path_arg(&resolved_state_path);

    let init = run_mycel(&["store", "init", &store_root, "--json"]);
    assert_success(&init);

    let document = run_mycel(&[
        "store",
        "create-document",
        &store_root,
        "--doc-id",
        "doc:author-smoke-nested-sibling-manual",
        "--title",
        "Author Smoke Nested Sibling Manual",
        "--language",
        "en",
        "--signing-key",
        &key_file,
        "--timestamp",
        "50",
        "--json",
    ]);
    assert_success(&document);
    let document_json = assert_json_status(&document, "ok");
    let genesis_revision_id = document_json["genesis_revision_id"]
        .as_str()
        .expect("genesis revision should be string")
        .to_string();

    let base_ops_dir = create_temp_dir("store-merge-nested-sibling-manual-base-ops");
    let base_ops_path = base_ops_dir.path().join("ops.json");
    fs::write(
        &base_ops_path,
        serde_json::to_string_pretty(&json!([
            {
                "op": "insert_block",
                "new_block": {
                    "block_id": "blk:nested-parent",
                    "block_type": "paragraph",
                    "content": "Parent",
                    "attrs": {},
                    "children": [
                        {
                            "block_id": "blk:nested-child-a",
                            "block_type": "paragraph",
                            "content": "Child A",
                            "attrs": {},
                            "children": []
                        },
                        {
                            "block_id": "blk:nested-child-b",
                            "block_type": "paragraph",
                            "content": "Child B",
                            "attrs": {},
                            "children": []
                        },
                        {
                            "block_id": "blk:nested-child-c",
                            "block_type": "paragraph",
                            "content": "Child C",
                            "attrs": {},
                            "children": []
                        }
                    ]
                }
            }
        ]))
        .expect("nested sibling manual base ops JSON should serialize"),
    )
    .expect("nested sibling manual base ops JSON should write");
    let base_ops_file = path_arg(&base_ops_path);

    let insert_ops_dir = create_temp_dir("store-merge-nested-sibling-manual-insert-ops");
    let insert_ops_path = insert_ops_dir.path().join("ops.json");
    fs::write(
        &insert_ops_path,
        serde_json::to_string_pretty(&json!([
            {
                "op": "insert_block_after",
                "after_block_id": "blk:nested-child-b",
                "new_block": {
                    "block_id": "blk:nested-child-d",
                    "block_type": "paragraph",
                    "content": "Child D",
                    "attrs": {},
                    "children": []
                }
            }
        ]))
        .expect("nested sibling manual insert ops JSON should serialize"),
    )
    .expect("nested sibling manual insert ops JSON should write");
    let insert_ops_file = path_arg(&insert_ops_path);

    let move_ops_dir = create_temp_dir("store-merge-nested-sibling-manual-move-ops");
    let move_ops_path = move_ops_dir.path().join("ops.json");
    fs::write(
        &move_ops_path,
        serde_json::to_string_pretty(&json!([
            {
                "op": "move_block",
                "block_id": "blk:nested-child-a",
                "parent_block_id": "blk:nested-parent",
                "after_block_id": "blk:nested-child-b"
            }
        ]))
        .expect("nested sibling manual move ops JSON should serialize"),
    )
    .expect("nested sibling manual move ops JSON should write");
    let move_ops_file = path_arg(&move_ops_path);

    let base_patch = run_mycel(&[
        "store",
        "create-patch",
        &store_root,
        "--doc-id",
        "doc:author-smoke-nested-sibling-manual",
        "--base-revision",
        &genesis_revision_id,
        "--ops",
        &base_ops_file,
        "--signing-key",
        &key_file,
        "--timestamp",
        "51",
        "--json",
    ]);
    assert_success(&base_patch);
    let base_patch_json = assert_json_status(&base_patch, "ok");
    let base_patch_id = base_patch_json["patch_id"]
        .as_str()
        .expect("base patch_id should be string")
        .to_string();

    let base_revision = run_mycel(&[
        "store",
        "commit-revision",
        &store_root,
        "--doc-id",
        "doc:author-smoke-nested-sibling-manual",
        "--parent",
        &genesis_revision_id,
        "--patch",
        &base_patch_id,
        "--signing-key",
        &key_file,
        "--timestamp",
        "52",
        "--json",
    ]);
    assert_success(&base_revision);
    let base_revision_json = assert_json_status(&base_revision, "ok");
    let base_revision_id = base_revision_json["revision_id"]
        .as_str()
        .expect("base revision_id should be string")
        .to_string();

    let insert_patch = run_mycel(&[
        "store",
        "create-patch",
        &store_root,
        "--doc-id",
        "doc:author-smoke-nested-sibling-manual",
        "--base-revision",
        &base_revision_id,
        "--ops",
        &insert_ops_file,
        "--signing-key",
        &key_file,
        "--timestamp",
        "53",
        "--json",
    ]);
    assert_success(&insert_patch);
    let insert_patch_json = assert_json_status(&insert_patch, "ok");
    let insert_patch_id = insert_patch_json["patch_id"]
        .as_str()
        .expect("insert patch_id should be string")
        .to_string();

    let insert_revision = run_mycel(&[
        "store",
        "commit-revision",
        &store_root,
        "--doc-id",
        "doc:author-smoke-nested-sibling-manual",
        "--parent",
        &base_revision_id,
        "--patch",
        &insert_patch_id,
        "--signing-key",
        &key_file,
        "--timestamp",
        "54",
        "--json",
    ]);
    assert_success(&insert_revision);
    let insert_revision_json = assert_json_status(&insert_revision, "ok");
    let insert_revision_id = insert_revision_json["revision_id"]
        .as_str()
        .expect("insert revision_id should be string")
        .to_string();

    let move_patch = run_mycel(&[
        "store",
        "create-patch",
        &store_root,
        "--doc-id",
        "doc:author-smoke-nested-sibling-manual",
        "--base-revision",
        &base_revision_id,
        "--ops",
        &move_ops_file,
        "--signing-key",
        &key_file,
        "--timestamp",
        "55",
        "--json",
    ]);
    assert_success(&move_patch);
    let move_patch_json = assert_json_status(&move_patch, "ok");
    let move_patch_id = move_patch_json["patch_id"]
        .as_str()
        .expect("move patch_id should be string")
        .to_string();

    let move_revision = run_mycel(&[
        "store",
        "commit-revision",
        &store_root,
        "--doc-id",
        "doc:author-smoke-nested-sibling-manual",
        "--parent",
        &base_revision_id,
        "--patch",
        &move_patch_id,
        "--signing-key",
        &key_file,
        "--timestamp",
        "56",
        "--json",
    ]);
    assert_success(&move_revision);
    let move_revision_json = assert_json_status(&move_revision, "ok");
    let move_revision_id = move_revision_json["revision_id"]
        .as_str()
        .expect("move revision_id should be string")
        .to_string();

    let merge = run_mycel(&[
        "store",
        "create-merge-revision",
        &store_root,
        "--doc-id",
        "doc:author-smoke-nested-sibling-manual",
        "--parent",
        &base_revision_id,
        "--parent",
        &insert_revision_id,
        "--parent",
        &move_revision_id,
        "--resolved-state",
        &resolved_state_file,
        "--signing-key",
        &key_file,
        "--timestamp",
        "57",
        "--json",
    ]);

    assert_success(&merge);
    let merge_json = assert_json_status(&merge, "ok");
    assert_eq!(merge_json["merge_outcome"], "multi-variant");
    assert!(
        merge_json["merge_reasons"]
            .as_array()
            .is_some_and(|reasons| reasons.iter().any(|reason| {
                reason.as_str().is_some_and(|reason| {
                    reason.contains("selected a non-primary sibling placement")
                })
            })),
        "expected nested sibling multi-variant reason, got {merge_json}"
    );
    assert_eq!(merge_json["patch_op_count"], 2);
}

#[test]
fn store_merge_authoring_flow_marks_deep_composed_branch_reuse_as_multi_variant() {
    let store_dir = create_temp_dir("store-merge-composed-manual-root");
    let (_key_dir, key_path) = write_signing_key_file("store-merge-composed-manual-key");
    let (_resolved_dir, resolved_state_path) =
        write_composed_branch_manual_resolved_state_file("store-merge-composed-manual-state");
    let store_root = path_arg(&store_dir.path().to_path_buf());
    let key_file = path_arg(&key_path);
    let resolved_state_file = path_arg(&resolved_state_path);

    let init = run_mycel(&["store", "init", &store_root, "--json"]);
    assert_success(&init);

    let document = run_mycel(&[
        "store",
        "create-document",
        &store_root,
        "--doc-id",
        "doc:author-smoke-composed-manual",
        "--title",
        "Author Smoke Composed Manual",
        "--language",
        "en",
        "--signing-key",
        &key_file,
        "--timestamp",
        "60",
        "--json",
    ]);
    assert_success(&document);
    let document_json = assert_json_status(&document, "ok");
    let genesis_revision_id = document_json["genesis_revision_id"]
        .as_str()
        .expect("genesis revision should be string")
        .to_string();

    let base_ops_dir = create_temp_dir("store-merge-composed-manual-base-ops");
    let base_ops_path = base_ops_dir.path().join("ops.json");
    fs::write(
        &base_ops_path,
        serde_json::to_string_pretty(&json!([
            {
                "op": "insert_block",
                "new_block": {
                    "block_id": "blk:cmp-anchor",
                    "block_type": "paragraph",
                    "content": "Anchor",
                    "attrs": {},
                    "children": []
                }
            },
            {
                "op": "insert_block",
                "new_block": {
                    "block_id": "blk:cmp-old-parent",
                    "block_type": "paragraph",
                    "content": "Old Parent",
                    "attrs": {},
                    "children": [
                        {
                            "block_id": "blk:cmp-leaf-a",
                            "block_type": "paragraph",
                            "content": "Leaf A",
                            "attrs": {},
                            "children": []
                        },
                        {
                            "block_id": "blk:cmp-leaf-b",
                            "block_type": "paragraph",
                            "content": "Leaf B",
                            "attrs": {},
                            "children": []
                        }
                    ]
                }
            }
        ]))
        .expect("composed manual base ops JSON should serialize"),
    )
    .expect("composed manual base ops JSON should write");
    let base_ops_file = path_arg(&base_ops_path);

    let delete_ops_dir = create_temp_dir("store-merge-composed-manual-delete-ops");
    let delete_ops_path = delete_ops_dir.path().join("ops.json");
    fs::write(
        &delete_ops_path,
        serde_json::to_string_pretty(&json!([
            {
                "op": "delete_block",
                "block_id": "blk:cmp-old-parent"
            }
        ]))
        .expect("composed manual delete ops JSON should serialize"),
    )
    .expect("composed manual delete ops JSON should write");
    let delete_ops_file = path_arg(&delete_ops_path);

    let insert_ops_dir = create_temp_dir("store-merge-composed-manual-insert-ops");
    let insert_ops_path = insert_ops_dir.path().join("ops.json");
    fs::write(
        &insert_ops_path,
        serde_json::to_string_pretty(&json!([
            {
                "op": "insert_block_after",
                "after_block_id": "blk:cmp-anchor",
                "new_block": {
                    "block_id": "blk:cmp-wrapper",
                    "block_type": "paragraph",
                    "content": "Wrapper",
                    "attrs": {},
                    "children": [
                        {
                            "block_id": "blk:cmp-section",
                            "block_type": "paragraph",
                            "content": "Section",
                            "attrs": {},
                            "children": [
                                {
                                    "block_id": "blk:cmp-subsection",
                                    "block_type": "paragraph",
                                    "content": "Subsection",
                                    "attrs": {},
                                    "children": []
                                }
                            ]
                        }
                    ]
                }
            }
        ]))
        .expect("composed manual insert ops JSON should serialize"),
    )
    .expect("composed manual insert ops JSON should write");
    let insert_ops_file = path_arg(&insert_ops_path);

    let base_patch = run_mycel(&[
        "store",
        "create-patch",
        &store_root,
        "--doc-id",
        "doc:author-smoke-composed-manual",
        "--base-revision",
        &genesis_revision_id,
        "--ops",
        &base_ops_file,
        "--signing-key",
        &key_file,
        "--timestamp",
        "61",
        "--json",
    ]);
    assert_success(&base_patch);
    let base_patch_json = assert_json_status(&base_patch, "ok");
    let base_patch_id = base_patch_json["patch_id"]
        .as_str()
        .expect("base patch_id should be string")
        .to_string();

    let base_revision = run_mycel(&[
        "store",
        "commit-revision",
        &store_root,
        "--doc-id",
        "doc:author-smoke-composed-manual",
        "--parent",
        &genesis_revision_id,
        "--patch",
        &base_patch_id,
        "--signing-key",
        &key_file,
        "--timestamp",
        "62",
        "--json",
    ]);
    assert_success(&base_revision);
    let base_revision_json = assert_json_status(&base_revision, "ok");
    let base_revision_id = base_revision_json["revision_id"]
        .as_str()
        .expect("base revision_id should be string")
        .to_string();

    let delete_patch = run_mycel(&[
        "store",
        "create-patch",
        &store_root,
        "--doc-id",
        "doc:author-smoke-composed-manual",
        "--base-revision",
        &base_revision_id,
        "--ops",
        &delete_ops_file,
        "--signing-key",
        &key_file,
        "--timestamp",
        "63",
        "--json",
    ]);
    assert_success(&delete_patch);
    let delete_patch_json = assert_json_status(&delete_patch, "ok");
    let delete_patch_id = delete_patch_json["patch_id"]
        .as_str()
        .expect("delete patch_id should be string")
        .to_string();

    let delete_revision = run_mycel(&[
        "store",
        "commit-revision",
        &store_root,
        "--doc-id",
        "doc:author-smoke-composed-manual",
        "--parent",
        &base_revision_id,
        "--patch",
        &delete_patch_id,
        "--signing-key",
        &key_file,
        "--timestamp",
        "64",
        "--json",
    ]);
    assert_success(&delete_revision);
    let delete_revision_json = assert_json_status(&delete_revision, "ok");
    let delete_revision_id = delete_revision_json["revision_id"]
        .as_str()
        .expect("delete revision_id should be string")
        .to_string();

    let insert_patch = run_mycel(&[
        "store",
        "create-patch",
        &store_root,
        "--doc-id",
        "doc:author-smoke-composed-manual",
        "--base-revision",
        &base_revision_id,
        "--ops",
        &insert_ops_file,
        "--signing-key",
        &key_file,
        "--timestamp",
        "65",
        "--json",
    ]);
    assert_success(&insert_patch);
    let insert_patch_json = assert_json_status(&insert_patch, "ok");
    let insert_patch_id = insert_patch_json["patch_id"]
        .as_str()
        .expect("insert patch_id should be string")
        .to_string();

    let insert_revision = run_mycel(&[
        "store",
        "commit-revision",
        &store_root,
        "--doc-id",
        "doc:author-smoke-composed-manual",
        "--parent",
        &base_revision_id,
        "--patch",
        &insert_patch_id,
        "--signing-key",
        &key_file,
        "--timestamp",
        "66",
        "--json",
    ]);
    assert_success(&insert_revision);
    let insert_revision_json = assert_json_status(&insert_revision, "ok");
    let insert_revision_id = insert_revision_json["revision_id"]
        .as_str()
        .expect("insert revision_id should be string")
        .to_string();

    let merge = run_mycel(&[
        "store",
        "create-merge-revision",
        &store_root,
        "--doc-id",
        "doc:author-smoke-composed-manual",
        "--parent",
        &base_revision_id,
        "--parent",
        &delete_revision_id,
        "--parent",
        &insert_revision_id,
        "--resolved-state",
        &resolved_state_file,
        "--signing-key",
        &key_file,
        "--timestamp",
        "67",
        "--json",
    ]);

    assert_success(&merge);
    let merge_json = assert_json_status(&merge, "ok");
    assert_eq!(merge_json["merge_outcome"], "multi-variant");
    assert!(
        merge_json["merge_reasons"]
            .as_array()
            .is_some_and(|reasons| reasons.iter().any(|reason| {
                reason.as_str().is_some_and(|reason| {
                    reason.contains("selected a non-primary parent placement")
                })
            })),
        "expected composed branch multi-variant reason, got {merge_json}"
    );
    assert_eq!(merge_json["patch_op_count"], 4);
}

#[test]
fn store_merge_authoring_flow_rejects_attr_variant_as_manual_curation_required() {
    let store_dir = create_temp_dir("store-merge-attrs-manual-root");
    let (_key_dir, key_path) = write_signing_key_file("store-merge-attrs-manual-key");
    let (_resolved_dir, resolved_state_path) =
        write_attrs_manual_resolved_state_file("store-merge-attrs-manual-state");
    let store_root = path_arg(&store_dir.path().to_path_buf());
    let key_file = path_arg(&key_path);
    let resolved_state_file = path_arg(&resolved_state_path);

    let init = run_mycel(&["store", "init", &store_root, "--json"]);
    assert_success(&init);

    let document = run_mycel(&[
        "store",
        "create-document",
        &store_root,
        "--doc-id",
        "doc:author-smoke-attrs-manual",
        "--title",
        "Author Smoke Attrs Manual",
        "--language",
        "en",
        "--signing-key",
        &key_file,
        "--timestamp",
        "70",
        "--json",
    ]);
    assert_success(&document);
    let document_json = assert_json_status(&document, "ok");
    let genesis_revision_id = document_json["genesis_revision_id"]
        .as_str()
        .expect("genesis revision should be string")
        .to_string();

    let base_ops_dir = create_temp_dir("store-merge-attrs-manual-base-ops");
    let base_ops_path = base_ops_dir.path().join("ops.json");
    fs::write(
        &base_ops_path,
        serde_json::to_string_pretty(&json!([
            {
                "op": "insert_block",
                "new_block": {
                    "block_id": "blk:merge-attrs",
                    "block_type": "paragraph",
                    "content": "Attrs",
                    "attrs": {},
                    "children": []
                }
            }
        ]))
        .expect("attrs manual base ops JSON should serialize"),
    )
    .expect("attrs manual base ops JSON should write");
    let base_ops_file = path_arg(&base_ops_path);

    let attrs_ops_dir = create_temp_dir("store-merge-attrs-manual-attrs-ops");
    let attrs_ops_path = attrs_ops_dir.path().join("ops.json");
    fs::write(
        &attrs_ops_path,
        serde_json::to_string_pretty(&json!([
            {
                "op": "delete_block",
                "block_id": "blk:merge-attrs"
            },
            {
                "op": "insert_block",
                "new_block": {
                    "block_id": "blk:merge-attrs",
                    "block_type": "paragraph",
                    "content": "Attrs",
                    "attrs": {
                        "style": "note"
                    },
                    "children": []
                }
            }
        ]))
        .expect("attrs manual variant ops JSON should serialize"),
    )
    .expect("attrs manual variant ops JSON should write");
    let attrs_ops_file = path_arg(&attrs_ops_path);

    let base_patch = run_mycel(&[
        "store",
        "create-patch",
        &store_root,
        "--doc-id",
        "doc:author-smoke-attrs-manual",
        "--base-revision",
        &genesis_revision_id,
        "--ops",
        &base_ops_file,
        "--signing-key",
        &key_file,
        "--timestamp",
        "71",
        "--json",
    ]);
    assert_success(&base_patch);
    let base_patch_json = assert_json_status(&base_patch, "ok");
    let base_patch_id = base_patch_json["patch_id"]
        .as_str()
        .expect("base patch_id should be string")
        .to_string();

    let base_revision = run_mycel(&[
        "store",
        "commit-revision",
        &store_root,
        "--doc-id",
        "doc:author-smoke-attrs-manual",
        "--parent",
        &genesis_revision_id,
        "--patch",
        &base_patch_id,
        "--signing-key",
        &key_file,
        "--timestamp",
        "72",
        "--json",
    ]);
    assert_success(&base_revision);
    let base_revision_json = assert_json_status(&base_revision, "ok");
    let base_revision_id = base_revision_json["revision_id"]
        .as_str()
        .expect("base revision_id should be string")
        .to_string();

    let attrs_patch = run_mycel(&[
        "store",
        "create-patch",
        &store_root,
        "--doc-id",
        "doc:author-smoke-attrs-manual",
        "--base-revision",
        &base_revision_id,
        "--ops",
        &attrs_ops_file,
        "--signing-key",
        &key_file,
        "--timestamp",
        "73",
        "--json",
    ]);
    assert_success(&attrs_patch);
    let attrs_patch_json = assert_json_status(&attrs_patch, "ok");
    let attrs_patch_id = attrs_patch_json["patch_id"]
        .as_str()
        .expect("attrs patch_id should be string")
        .to_string();

    let attrs_revision = run_mycel(&[
        "store",
        "commit-revision",
        &store_root,
        "--doc-id",
        "doc:author-smoke-attrs-manual",
        "--parent",
        &base_revision_id,
        "--patch",
        &attrs_patch_id,
        "--signing-key",
        &key_file,
        "--timestamp",
        "74",
        "--json",
    ]);
    assert_success(&attrs_revision);
    let attrs_revision_json = assert_json_status(&attrs_revision, "ok");
    let attrs_revision_id = attrs_revision_json["revision_id"]
        .as_str()
        .expect("attrs revision_id should be string")
        .to_string();

    let merge = run_mycel(&[
        "store",
        "create-merge-revision",
        &store_root,
        "--doc-id",
        "doc:author-smoke-attrs-manual",
        "--parent",
        &base_revision_id,
        "--parent",
        &attrs_revision_id,
        "--resolved-state",
        &resolved_state_file,
        "--signing-key",
        &key_file,
        "--timestamp",
        "75",
    ]);

    assert!(
        !merge.status.success(),
        "expected manual-curation failure, stdout: {}, stderr: {}",
        String::from_utf8_lossy(&merge.stdout),
        String::from_utf8_lossy(&merge.stderr)
    );
    assert_stderr_contains(
        &merge,
        "manual-curation-required: block 'blk:merge-attrs' changes attrs in an unsupported way",
    );
}
