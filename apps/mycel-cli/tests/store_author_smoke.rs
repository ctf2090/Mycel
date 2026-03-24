use std::fs;
use std::path::{Path, PathBuf};

use base64::Engine;
use serde_json::{json, Value};

mod common;

use common::{
    assert_exit_code, assert_json_status, assert_stdout_contains, assert_success, create_temp_dir,
    parse_json_stdout, run_mycel,
};

fn path_arg(path: &Path) -> String {
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

fn write_content_variant_ops_for_block_file(
    prefix: &str,
    block_id: &str,
    content: &str,
) -> (common::TempDir, PathBuf) {
    let dir = create_temp_dir(prefix);
    let path = dir.path().join("ops.json");
    fs::write(
        &path,
        serde_json::to_string_pretty(&json!([
            {
                "op": "replace_block",
                "block_id": block_id,
                "new_content": content
            }
        ]))
        .expect("content variant ops JSON should serialize"),
    )
    .expect("content variant ops JSON should write");
    (dir, path)
}

fn write_content_addition_ops_file(prefix: &str, content: &str) -> (common::TempDir, PathBuf) {
    let dir = create_temp_dir(prefix);
    let path = dir.path().join("ops.json");
    fs::write(
        &path,
        serde_json::to_string_pretty(&json!([
            {
                "op": "insert_block",
                "new_block": {
                    "block_id": "blk:author-smoke-variant-001",
                    "block_type": "paragraph",
                    "content": content,
                    "attrs": {},
                    "children": []
                }
            }
        ]))
        .expect("content addition ops JSON should serialize"),
    )
    .expect("content addition ops JSON should write");
    (dir, path)
}

fn write_content_addition_ops_for_block_file(
    prefix: &str,
    block_id: &str,
    content: &str,
) -> (common::TempDir, PathBuf) {
    let dir = create_temp_dir(prefix);
    let path = dir.path().join("ops.json");
    fs::write(
        &path,
        serde_json::to_string_pretty(&json!([
            {
                "op": "insert_block",
                "new_block": {
                    "block_id": block_id,
                    "block_type": "paragraph",
                    "content": content,
                    "attrs": {},
                    "children": []
                }
            }
        ]))
        .expect("content addition ops JSON should serialize"),
    )
    .expect("content addition ops JSON should write");
    (dir, path)
}

fn write_content_delete_ops_for_block_file(
    prefix: &str,
    block_id: &str,
) -> (common::TempDir, PathBuf) {
    let dir = create_temp_dir(prefix);
    let path = dir.path().join("ops.json");
    fs::write(
        &path,
        serde_json::to_string_pretty(&json!([
            {
                "op": "delete_block",
                "block_id": block_id
            }
        ]))
        .expect("content delete ops JSON should serialize"),
    )
    .expect("content delete ops JSON should write");
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

fn write_content_entries_resolved_state_for_doc_file(
    prefix: &str,
    doc_id: &str,
    entries: &[(&str, &str)],
) -> (common::TempDir, PathBuf) {
    let dir = create_temp_dir(prefix);
    let path = dir.path().join("resolved-state.json");
    let blocks = entries
        .iter()
        .map(|(block_id, content)| {
            json!({
                "block_id": block_id,
                "block_type": "paragraph",
                "content": content,
                "attrs": {},
                "children": []
            })
        })
        .collect::<Vec<_>>();
    fs::write(
        &path,
        serde_json::to_string_pretty(&json!({
            "doc_id": doc_id,
            "blocks": blocks,
            "metadata": {}
        }))
        .expect("content entries resolved state JSON should serialize"),
    )
    .expect("content entries resolved state JSON should write");
    (dir, path)
}

fn write_metadata_variant_ops_file(prefix: &str, topic: &str) -> (common::TempDir, PathBuf) {
    let dir = create_temp_dir(prefix);
    let path = dir.path().join("ops.json");
    fs::write(
        &path,
        serde_json::to_string_pretty(&json!([
            {
                "op": "set_metadata",
                "metadata": {
                    "topic": topic
                }
            }
        ]))
        .expect("metadata variant ops JSON should serialize"),
    )
    .expect("metadata variant ops JSON should write");
    (dir, path)
}

fn write_metadata_entries_ops_file(
    prefix: &str,
    entries: &[(&str, &str)],
) -> (common::TempDir, PathBuf) {
    let dir = create_temp_dir(prefix);
    let path = dir.path().join("ops.json");
    let metadata = serde_json::Map::from_iter(entries.iter().map(|(key, value)| {
        (
            (*key).to_string(),
            serde_json::Value::String((*value).to_string()),
        )
    }));
    fs::write(
        &path,
        serde_json::to_string_pretty(&json!([
            {
                "op": "set_metadata",
                "metadata": metadata
            }
        ]))
        .expect("metadata entries ops JSON should serialize"),
    )
    .expect("metadata entries ops JSON should write");
    (dir, path)
}

fn write_metadata_variant_resolved_state_file(
    prefix: &str,
    topic: &str,
) -> (common::TempDir, PathBuf) {
    let dir = create_temp_dir(prefix);
    let path = dir.path().join("resolved-state.json");
    fs::write(
        &path,
        serde_json::to_string_pretty(&json!({
            "doc_id": "doc:author-smoke-metadata-variant",
            "blocks": [],
            "metadata": {
                "topic": topic
            }
        }))
        .expect("metadata variant resolved state JSON should serialize"),
    )
    .expect("metadata variant resolved state JSON should write");
    (dir, path)
}

fn write_metadata_variant_resolved_state_for_doc_file(
    prefix: &str,
    doc_id: &str,
    topic: &str,
) -> (common::TempDir, PathBuf) {
    let dir = create_temp_dir(prefix);
    let path = dir.path().join("resolved-state.json");
    fs::write(
        &path,
        serde_json::to_string_pretty(&json!({
            "doc_id": doc_id,
            "blocks": [],
            "metadata": {
                "topic": topic
            }
        }))
        .expect("metadata variant resolved state JSON should serialize"),
    )
    .expect("metadata variant resolved state JSON should write");
    (dir, path)
}

fn write_metadata_entries_resolved_state_for_doc_file(
    prefix: &str,
    doc_id: &str,
    entries: &[(&str, &str)],
) -> (common::TempDir, PathBuf) {
    let dir = create_temp_dir(prefix);
    let path = dir.path().join("resolved-state.json");
    let metadata = serde_json::Map::from_iter(entries.iter().map(|(key, value)| {
        (
            (*key).to_string(),
            serde_json::Value::String((*value).to_string()),
        )
    }));
    fs::write(
        &path,
        serde_json::to_string_pretty(&json!({
            "doc_id": doc_id,
            "blocks": [],
            "metadata": metadata
        }))
        .expect("metadata entries resolved state JSON should serialize"),
    )
    .expect("metadata entries resolved state JSON should write");
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

struct StoreAuthoringFlow {
    _store_dir: common::TempDir,
    _key_dir: common::TempDir,
    store_root: String,
    key_file: String,
}

impl StoreAuthoringFlow {
    fn new(store_prefix: &str, key_prefix: &str) -> Self {
        let store_dir = create_temp_dir(store_prefix);
        let (_key_dir, key_path) = write_signing_key_file(key_prefix);
        let store_root = path_arg(store_dir.path());
        let key_file = path_arg(&key_path);

        let init = run_mycel(&["store", "init", &store_root, "--json"]);
        assert_success(&init);

        Self {
            _store_dir: store_dir,
            _key_dir,
            store_root,
            key_file,
        }
    }

    fn create_document(
        &self,
        doc_id: &str,
        title: &str,
        language: &str,
        timestamp: &str,
    ) -> String {
        let document = run_mycel(&[
            "store",
            "create-document",
            &self.store_root,
            "--doc-id",
            doc_id,
            "--title",
            title,
            "--language",
            language,
            "--signing-key",
            &self.key_file,
            "--timestamp",
            timestamp,
            "--json",
        ]);
        assert_success(&document);
        assert_json_status(&document, "ok")["genesis_revision_id"]
            .as_str()
            .expect("genesis revision should be string")
            .to_string()
    }

    fn create_patch(
        &self,
        doc_id: &str,
        base_revision_id: &str,
        ops_file: &str,
        timestamp: &str,
    ) -> String {
        let patch = run_mycel(&[
            "store",
            "create-patch",
            &self.store_root,
            "--doc-id",
            doc_id,
            "--base-revision",
            base_revision_id,
            "--ops",
            ops_file,
            "--signing-key",
            &self.key_file,
            "--timestamp",
            timestamp,
            "--json",
        ]);
        assert_success(&patch);
        assert_json_status(&patch, "ok")["patch_id"]
            .as_str()
            .expect("patch_id should be string")
            .to_string()
    }

    fn commit_revision(
        &self,
        doc_id: &str,
        parent_revision_id: &str,
        patch_id: &str,
        timestamp: &str,
    ) -> String {
        let revision = run_mycel(&[
            "store",
            "commit-revision",
            &self.store_root,
            "--doc-id",
            doc_id,
            "--parent",
            parent_revision_id,
            "--patch",
            patch_id,
            "--signing-key",
            &self.key_file,
            "--timestamp",
            timestamp,
            "--json",
        ]);
        assert_success(&revision);
        assert_json_status(&revision, "ok")["revision_id"]
            .as_str()
            .expect("revision_id should be string")
            .to_string()
    }

    fn create_merge_revision(
        &self,
        doc_id: &str,
        parent_revision_ids: &[&str],
        resolved_state_file: &str,
        timestamp: &str,
    ) -> Value {
        let mut args = vec![
            "store",
            "create-merge-revision",
            &self.store_root,
            "--doc-id",
            doc_id,
        ];
        for parent in parent_revision_ids {
            args.extend(["--parent", parent]);
        }
        args.extend([
            "--resolved-state",
            resolved_state_file,
            "--signing-key",
            &self.key_file,
            "--timestamp",
            timestamp,
            "--json",
        ]);

        let merge = run_mycel(&args);
        assert_success(&merge);
        assert_json_status(&merge, "ok").clone()
    }
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

#[path = "store_author_smoke/authoring.rs"]
mod authoring;
#[path = "store_author_smoke/manual.rs"]
mod manual;
#[path = "store_author_smoke/structural.rs"]
mod structural;
#[path = "store_author_smoke/variants.rs"]
mod variants;
