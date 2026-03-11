use std::fs;
use std::path::PathBuf;

use base64::Engine;
use ed25519_dalek::SigningKey;
use serde_json::{json, Value};

use super::{
    commit_revision_to_store, create_document_in_store, create_merge_revision_in_store,
    create_patch_in_store, parse_signing_key_seed, signer_id, DocumentCreateParams, MergeOutcome,
    MergeRevisionCreateParams, PatchCreateParams, RevisionCommitParams,
};
use crate::protocol::{parse_patch_object, BlockObject, PatchOperation};
use crate::replay::replay_revision_from_index;
use crate::store::{load_store_index_manifest, load_stored_object_value};

fn temp_dir(prefix: &str) -> PathBuf {
    let unique = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .expect("system time should be after unix epoch")
        .as_nanos();
    let path = std::env::temp_dir().join(format!("mycel-author-{prefix}-{unique}"));
    fs::create_dir_all(&path).expect("temp dir should be created");
    path
}

fn signing_key() -> SigningKey {
    parse_signing_key_seed(&base64::engine::general_purpose::STANDARD.encode([7u8; 32]))
        .expect("signing key seed should parse")
}

fn paragraph_block(block_id: &str, content: &str) -> BlockObject {
    BlockObject {
        block_id: block_id.to_string(),
        block_type: "paragraph".to_string(),
        content: content.to_string(),
        attrs: serde_json::Map::new(),
        children: Vec::new(),
    }
}

fn paragraph_block_with_children(
    block_id: &str,
    content: &str,
    children: Vec<BlockObject>,
) -> BlockObject {
    BlockObject {
        children,
        ..paragraph_block(block_id, content)
    }
}

fn paragraph_block_with_attrs(
    block_id: &str,
    content: &str,
    attrs: serde_json::Map<String, Value>,
) -> BlockObject {
    BlockObject {
        attrs,
        ..paragraph_block(block_id, content)
    }
}

fn commit_ops_revision(
    store_root: &std::path::Path,
    signing_key: &SigningKey,
    doc_id: &str,
    base_revision: &str,
    patch_timestamp: u64,
    revision_timestamp: u64,
    ops: Value,
) -> String {
    let patch = create_patch_in_store(
        store_root,
        signing_key,
        &PatchCreateParams {
            doc_id: doc_id.to_string(),
            base_revision: base_revision.to_string(),
            timestamp: patch_timestamp,
            ops,
        },
    )
    .expect("patch should be created");
    commit_revision_to_store(
        store_root,
        signing_key,
        &RevisionCommitParams {
            doc_id: doc_id.to_string(),
            parents: vec![base_revision.to_string()],
            patches: vec![patch.patch_id],
            merge_strategy: None,
            timestamp: revision_timestamp,
        },
    )
    .expect("revision should be committed")
    .revision_id
}

#[test]
fn authoring_flow_creates_document_patch_and_revision_in_store() {
    let store_root = temp_dir("flow");
    let signing_key = signing_key();
    let document = create_document_in_store(
        &store_root,
        &signing_key,
        &DocumentCreateParams {
            doc_id: "doc:author-flow".to_string(),
            title: "Author Flow".to_string(),
            language: "en".to_string(),
            timestamp: 10,
        },
    )
    .expect("document should be created");
    assert_eq!(document.written_object_count, 2);

    let patch = create_patch_in_store(
        &store_root,
        &signing_key,
        &PatchCreateParams {
            doc_id: "doc:author-flow".to_string(),
            base_revision: document.genesis_revision_id.clone(),
            timestamp: 11,
            ops: json!([
                {
                    "op": "insert_block",
                    "new_block": {
                        "block_id": "blk:001",
                        "block_type": "paragraph",
                        "content": "Hello authoring",
                        "attrs": {},
                        "children": []
                    }
                }
            ]),
        },
    )
    .expect("patch should be created");
    assert_eq!(patch.written_object_count, 1);

    let revision = commit_revision_to_store(
        &store_root,
        &signing_key,
        &RevisionCommitParams {
            doc_id: "doc:author-flow".to_string(),
            parents: vec![document.genesis_revision_id.clone()],
            patches: vec![patch.patch_id.clone()],
            merge_strategy: None,
            timestamp: 12,
        },
    )
    .expect("revision should be committed");
    assert_eq!(revision.written_object_count, 1);

    let manifest = load_store_index_manifest(&store_root).expect("manifest should load");
    assert_eq!(
        manifest.doc_revisions.get("doc:author-flow").map(Vec::len),
        Some(2)
    );
    assert_eq!(
        manifest
            .author_patches
            .get(&signer_id(&signing_key))
            .map(Vec::len),
        Some(1)
    );

    let mut object_index =
        crate::store::load_store_object_index(&store_root).expect("object index should load");
    object_index.insert(
        "doc:author-flow".to_string(),
        load_stored_object_value(&store_root, "doc:author-flow").expect("document should load"),
    );
    let replay = replay_revision_from_index(
        &load_stored_object_value(&store_root, &revision.revision_id)
            .expect("revision should load"),
        &object_index,
    )
    .expect("revision replay should succeed");
    assert_eq!(replay.revision_id, revision.revision_id);
    assert_eq!(replay.state.doc_id, "doc:author-flow");
    assert_eq!(replay.state.blocks.len(), 1);

    let _ = fs::remove_dir_all(store_root);
}

#[test]
fn merge_authoring_reports_multi_variant_when_parents_disagree() {
    let store_root = temp_dir("merge-multi-variant");
    let signing_key = signing_key();
    let document = create_document_in_store(
        &store_root,
        &signing_key,
        &DocumentCreateParams {
            doc_id: "doc:merge-variant".to_string(),
            title: "Merge Variant".to_string(),
            language: "en".to_string(),
            timestamp: 10,
        },
    )
    .expect("document should be created");

    let base_patch = create_patch_in_store(
        &store_root,
        &signing_key,
        &PatchCreateParams {
            doc_id: "doc:merge-variant".to_string(),
            base_revision: document.genesis_revision_id.clone(),
            timestamp: 11,
            ops: json!([
                {
                    "op": "insert_block",
                    "new_block": {
                        "block_id": "blk:merge-001",
                        "block_type": "paragraph",
                        "content": "Base",
                        "attrs": {},
                        "children": []
                    }
                }
            ]),
        },
    )
    .expect("base patch should be created");
    let base_revision = commit_revision_to_store(
        &store_root,
        &signing_key,
        &RevisionCommitParams {
            doc_id: "doc:merge-variant".to_string(),
            parents: vec![document.genesis_revision_id.clone()],
            patches: vec![base_patch.patch_id],
            merge_strategy: None,
            timestamp: 12,
        },
    )
    .expect("base revision should be committed");

    let left_patch = create_patch_in_store(
        &store_root,
        &signing_key,
        &PatchCreateParams {
            doc_id: "doc:merge-variant".to_string(),
            base_revision: base_revision.revision_id.clone(),
            timestamp: 13,
            ops: json!([
                {
                    "op": "replace_block",
                    "block_id": "blk:merge-001",
                    "new_content": "Left variant"
                }
            ]),
        },
    )
    .expect("left patch should be created");
    let left_revision = commit_revision_to_store(
        &store_root,
        &signing_key,
        &RevisionCommitParams {
            doc_id: "doc:merge-variant".to_string(),
            parents: vec![base_revision.revision_id.clone()],
            patches: vec![left_patch.patch_id],
            merge_strategy: None,
            timestamp: 14,
        },
    )
    .expect("left revision should be committed");

    let right_patch = create_patch_in_store(
        &store_root,
        &signing_key,
        &PatchCreateParams {
            doc_id: "doc:merge-variant".to_string(),
            base_revision: base_revision.revision_id.clone(),
            timestamp: 15,
            ops: json!([
                {
                    "op": "replace_block",
                    "block_id": "blk:merge-001",
                    "new_content": "Right variant"
                }
            ]),
        },
    )
    .expect("right patch should be created");
    let right_revision = commit_revision_to_store(
        &store_root,
        &signing_key,
        &RevisionCommitParams {
            doc_id: "doc:merge-variant".to_string(),
            parents: vec![base_revision.revision_id.clone()],
            patches: vec![right_patch.patch_id],
            merge_strategy: None,
            timestamp: 16,
        },
    )
    .expect("right revision should be committed");

    let summary = create_merge_revision_in_store(
        &store_root,
        &signing_key,
        &MergeRevisionCreateParams {
            doc_id: "doc:merge-variant".to_string(),
            parents: vec![left_revision.revision_id, right_revision.revision_id],
            resolved_state: crate::replay::DocumentState {
                doc_id: "doc:merge-variant".to_string(),
                blocks: vec![paragraph_block("blk:merge-001", "Right variant")],
                metadata: serde_json::Map::new(),
            },
            merge_strategy: "semantic-block-merge".to_string(),
            timestamp: 17,
        },
    )
    .expect("merge revision should be created");

    assert_eq!(summary.merge_outcome, MergeOutcome::MultiVariant);
    assert_eq!(summary.patch_op_count, 1);
    assert!(
        summary
            .merge_reasons
            .iter()
            .any(|reason| reason.contains("selected a non-primary parent variant")),
        "expected multi-variant reason, got {summary:?}"
    );

    let _ = fs::remove_dir_all(store_root);
}

#[test]
fn merge_authoring_supports_structural_move_and_insert_ops() {
    let store_root = temp_dir("merge-structural");
    let signing_key = signing_key();
    let document = create_document_in_store(
        &store_root,
        &signing_key,
        &DocumentCreateParams {
            doc_id: "doc:merge-structural".to_string(),
            title: "Merge Structural".to_string(),
            language: "en".to_string(),
            timestamp: 20,
        },
    )
    .expect("document should be created");

    let base_patch = create_patch_in_store(
        &store_root,
        &signing_key,
        &PatchCreateParams {
            doc_id: "doc:merge-structural".to_string(),
            base_revision: document.genesis_revision_id.clone(),
            timestamp: 21,
            ops: json!([
                {
                    "op": "insert_block",
                    "new_block": {
                        "block_id": "blk:merge-a",
                        "block_type": "paragraph",
                        "content": "A",
                        "attrs": {},
                        "children": []
                    }
                },
                {
                    "op": "insert_block",
                    "new_block": {
                        "block_id": "blk:merge-b",
                        "block_type": "paragraph",
                        "content": "B",
                        "attrs": {},
                        "children": []
                    }
                }
            ]),
        },
    )
    .expect("base patch should be created");
    let base_revision = commit_revision_to_store(
        &store_root,
        &signing_key,
        &RevisionCommitParams {
            doc_id: "doc:merge-structural".to_string(),
            parents: vec![document.genesis_revision_id.clone()],
            patches: vec![base_patch.patch_id],
            merge_strategy: None,
            timestamp: 22,
        },
    )
    .expect("base revision should be committed");

    let move_patch = create_patch_in_store(
        &store_root,
        &signing_key,
        &PatchCreateParams {
            doc_id: "doc:merge-structural".to_string(),
            base_revision: base_revision.revision_id.clone(),
            timestamp: 23,
            ops: json!([
                {
                    "op": "move_block",
                    "block_id": "blk:merge-a",
                    "after_block_id": "blk:merge-b"
                }
            ]),
        },
    )
    .expect("move patch should be created");
    let move_revision = commit_revision_to_store(
        &store_root,
        &signing_key,
        &RevisionCommitParams {
            doc_id: "doc:merge-structural".to_string(),
            parents: vec![base_revision.revision_id.clone()],
            patches: vec![move_patch.patch_id],
            merge_strategy: None,
            timestamp: 24,
        },
    )
    .expect("move revision should be committed");

    let insert_patch = create_patch_in_store(
        &store_root,
        &signing_key,
        &PatchCreateParams {
            doc_id: "doc:merge-structural".to_string(),
            base_revision: base_revision.revision_id.clone(),
            timestamp: 25,
            ops: json!([
                {
                    "op": "insert_block",
                    "new_block": {
                        "block_id": "blk:merge-c",
                        "block_type": "paragraph",
                        "content": "C",
                        "attrs": {},
                        "children": []
                    }
                }
            ]),
        },
    )
    .expect("insert patch should be created");
    let insert_revision = commit_revision_to_store(
        &store_root,
        &signing_key,
        &RevisionCommitParams {
            doc_id: "doc:merge-structural".to_string(),
            parents: vec![base_revision.revision_id.clone()],
            patches: vec![insert_patch.patch_id],
            merge_strategy: None,
            timestamp: 26,
        },
    )
    .expect("insert revision should be committed");

    let summary = create_merge_revision_in_store(
        &store_root,
        &signing_key,
        &MergeRevisionCreateParams {
            doc_id: "doc:merge-structural".to_string(),
            parents: vec![
                base_revision.revision_id.clone(),
                move_revision.revision_id.clone(),
                insert_revision.revision_id.clone(),
            ],
            resolved_state: crate::replay::DocumentState {
                doc_id: "doc:merge-structural".to_string(),
                blocks: vec![
                    paragraph_block("blk:merge-b", "B"),
                    paragraph_block("blk:merge-a", "A"),
                    paragraph_block("blk:merge-c", "C"),
                ],
                metadata: serde_json::Map::new(),
            },
            merge_strategy: "semantic-block-merge".to_string(),
            timestamp: 27,
        },
    )
    .expect("merge revision should be created");

    assert_eq!(summary.merge_outcome, MergeOutcome::AutoMerged);
    assert_eq!(summary.patch_op_count, 2);
    let patch_value = load_stored_object_value(&store_root, &summary.patch_id)
        .expect("generated merge patch should be stored");
    let patch = parse_patch_object(&patch_value).expect("generated patch should parse");
    assert_eq!(patch.ops.len(), 2);
    assert!(patch.ops.iter().any(|op| matches!(
        op,
        PatchOperation::MoveBlock { block_id, after_block_id: Some(after_block_id), .. }
        if block_id == "blk:merge-a" && after_block_id == "blk:merge-b"
    )));
    assert!(patch.ops.iter().any(|op| matches!(
        op,
        PatchOperation::InsertBlockAfter { after_block_id, new_block }
        if after_block_id == "blk:merge-a" && new_block.block_id == "blk:merge-c"
    )));

    let _ = fs::remove_dir_all(store_root);
}

#[test]
fn merge_authoring_supports_reparenting_into_a_newly_inserted_parent() {
    let store_root = temp_dir("merge-reparent-new-parent");
    let signing_key = signing_key();
    let document = create_document_in_store(
        &store_root,
        &signing_key,
        &DocumentCreateParams {
            doc_id: "doc:merge-reparent".to_string(),
            title: "Merge Reparent".to_string(),
            language: "en".to_string(),
            timestamp: 28,
        },
    )
    .expect("document should be created");

    let base_revision_id = commit_ops_revision(
        &store_root,
        &signing_key,
        "doc:merge-reparent",
        &document.genesis_revision_id,
        29,
        30,
        json!([
            {
                "op": "insert_block",
                "new_block": {
                    "block_id": "blk:reparent-leaf",
                    "block_type": "paragraph",
                    "content": "Leaf",
                    "attrs": {},
                    "children": []
                }
            }
        ]),
    );

    let parent_revision_id = commit_ops_revision(
        &store_root,
        &signing_key,
        "doc:merge-reparent",
        &base_revision_id,
        31,
        32,
        json!([
            {
                "op": "insert_block",
                "new_block": {
                    "block_id": "blk:reparent-parent",
                    "block_type": "paragraph",
                    "content": "Parent",
                    "attrs": {},
                    "children": []
                }
            }
        ]),
    );

    let summary = create_merge_revision_in_store(
        &store_root,
        &signing_key,
        &MergeRevisionCreateParams {
            doc_id: "doc:merge-reparent".to_string(),
            parents: vec![base_revision_id, parent_revision_id],
            resolved_state: crate::replay::DocumentState {
                doc_id: "doc:merge-reparent".to_string(),
                blocks: vec![paragraph_block_with_children(
                    "blk:reparent-parent",
                    "Parent",
                    vec![paragraph_block("blk:reparent-leaf", "Leaf")],
                )],
                metadata: serde_json::Map::new(),
            },
            merge_strategy: "semantic-block-merge".to_string(),
            timestamp: 33,
        },
    )
    .expect("merge revision should be created");

    assert_eq!(summary.merge_outcome, MergeOutcome::AutoMerged);
    assert_eq!(summary.patch_op_count, 2);
    let patch_value = load_stored_object_value(&store_root, &summary.patch_id)
        .expect("generated merge patch should be stored");
    let patch = parse_patch_object(&patch_value).expect("generated patch should parse");
    assert_eq!(patch.ops.len(), 2);
    assert!(patch.ops.iter().any(|op| matches!(
        op,
        PatchOperation::InsertBlock { new_block, .. }
        if new_block.block_id == "blk:reparent-parent" && new_block.children.is_empty()
    )));
    assert!(patch.ops.iter().any(|op| matches!(
        op,
        PatchOperation::MoveBlock { block_id, parent_block_id: Some(parent_block_id), after_block_id: None }
        if block_id == "blk:reparent-leaf" && parent_block_id == "blk:reparent-parent"
    )));

    let _ = fs::remove_dir_all(store_root);
}

#[test]
fn merge_authoring_supports_reparenting_through_a_composed_parent_chain() {
    let store_root = temp_dir("merge-reparent-parent-chain");
    let signing_key = signing_key();
    let document = create_document_in_store(
        &store_root,
        &signing_key,
        &DocumentCreateParams {
            doc_id: "doc:merge-parent-chain".to_string(),
            title: "Merge Parent Chain".to_string(),
            language: "en".to_string(),
            timestamp: 34,
        },
    )
    .expect("document should be created");

    let base_revision_id = commit_ops_revision(
        &store_root,
        &signing_key,
        "doc:merge-parent-chain",
        &document.genesis_revision_id,
        35,
        36,
        json!([
            {
                "op": "insert_block",
                "new_block": {
                    "block_id": "blk:chain-parent",
                    "block_type": "paragraph",
                    "content": "Chain Parent",
                    "attrs": {},
                    "children": []
                }
            },
            {
                "op": "insert_block",
                "new_block": {
                    "block_id": "blk:chain-leaf",
                    "block_type": "paragraph",
                    "content": "Chain Leaf",
                    "attrs": {},
                    "children": []
                }
            }
        ]),
    );

    let wrapper_revision_id = commit_ops_revision(
        &store_root,
        &signing_key,
        "doc:merge-parent-chain",
        &base_revision_id,
        37,
        38,
        json!([
            {
                "op": "insert_block",
                "new_block": {
                    "block_id": "blk:chain-wrapper",
                    "block_type": "paragraph",
                    "content": "Chain Wrapper",
                    "attrs": {},
                    "children": []
                }
            }
        ]),
    );

    let summary = create_merge_revision_in_store(
        &store_root,
        &signing_key,
        &MergeRevisionCreateParams {
            doc_id: "doc:merge-parent-chain".to_string(),
            parents: vec![base_revision_id, wrapper_revision_id],
            resolved_state: crate::replay::DocumentState {
                doc_id: "doc:merge-parent-chain".to_string(),
                blocks: vec![paragraph_block_with_children(
                    "blk:chain-wrapper",
                    "Chain Wrapper",
                    vec![paragraph_block_with_children(
                        "blk:chain-parent",
                        "Chain Parent",
                        vec![paragraph_block("blk:chain-leaf", "Chain Leaf")],
                    )],
                )],
                metadata: serde_json::Map::new(),
            },
            merge_strategy: "semantic-block-merge".to_string(),
            timestamp: 39,
        },
    )
    .expect("merge revision should be created");

    assert_eq!(summary.merge_outcome, MergeOutcome::AutoMerged);
    assert_eq!(summary.patch_op_count, 3);
    let patch_value = load_stored_object_value(&store_root, &summary.patch_id)
        .expect("generated merge patch should be stored");
    let patch = parse_patch_object(&patch_value).expect("generated patch should parse");
    assert_eq!(patch.ops.len(), 3);
    assert!(patch.ops.iter().any(|op| matches!(
        op,
        PatchOperation::InsertBlock { new_block, .. }
        if new_block.block_id == "blk:chain-wrapper" && new_block.children.is_empty()
    )));
    assert!(patch.ops.iter().any(|op| matches!(
        op,
        PatchOperation::MoveBlock { block_id, parent_block_id: Some(parent_block_id), after_block_id: None }
        if block_id == "blk:chain-parent" && parent_block_id == "blk:chain-wrapper"
    )));
    assert!(patch.ops.iter().any(|op| matches!(
        op,
        PatchOperation::MoveBlock { block_id, parent_block_id: Some(parent_block_id), after_block_id: None }
        if block_id == "blk:chain-leaf" && parent_block_id == "blk:chain-parent"
    )));

    let _ = fs::remove_dir_all(store_root);
}

#[test]
fn merge_authoring_marks_non_primary_structural_parent_choice_as_multi_variant() {
    let store_root = temp_dir("merge-parent-choice");
    let signing_key = signing_key();
    let document = create_document_in_store(
        &store_root,
        &signing_key,
        &DocumentCreateParams {
            doc_id: "doc:merge-parent-choice".to_string(),
            title: "Merge Parent Choice".to_string(),
            language: "en".to_string(),
            timestamp: 30,
        },
    )
    .expect("document should be created");

    let base_revision_id = commit_ops_revision(
        &store_root,
        &signing_key,
        "doc:merge-parent-choice",
        &document.genesis_revision_id,
        31,
        32,
        json!([
            {
                "op": "insert_block",
                "new_block": {
                    "block_id": "blk:merge-parent",
                    "block_type": "paragraph",
                    "content": "Parent",
                    "attrs": {},
                    "children": []
                }
            },
            {
                "op": "insert_block",
                "new_block": {
                    "block_id": "blk:merge-leaf",
                    "block_type": "paragraph",
                    "content": "Leaf",
                    "attrs": {},
                    "children": []
                }
            }
        ]),
    );

    let moved_revision_id = commit_ops_revision(
        &store_root,
        &signing_key,
        "doc:merge-parent-choice",
        &base_revision_id,
        33,
        34,
        json!([
            {
                "op": "move_block",
                "block_id": "blk:merge-leaf",
                "parent_block_id": "blk:merge-parent"
            }
        ]),
    );

    let summary = create_merge_revision_in_store(
        &store_root,
        &signing_key,
        &MergeRevisionCreateParams {
            doc_id: "doc:merge-parent-choice".to_string(),
            parents: vec![base_revision_id, moved_revision_id],
            resolved_state: crate::replay::DocumentState {
                doc_id: "doc:merge-parent-choice".to_string(),
                blocks: vec![paragraph_block_with_children(
                    "blk:merge-parent",
                    "Parent",
                    vec![paragraph_block("blk:merge-leaf", "Leaf")],
                )],
                metadata: serde_json::Map::new(),
            },
            merge_strategy: "semantic-block-merge".to_string(),
            timestamp: 35,
        },
    )
    .expect("merge revision should be created");

    assert_eq!(summary.merge_outcome, MergeOutcome::MultiVariant);
    assert!(
        summary
            .merge_reasons
            .iter()
            .any(|reason| reason.contains("selected a non-primary parent placement")),
        "expected structural multi-variant reason, got {summary:?}"
    );
    let patch_value = load_stored_object_value(&store_root, &summary.patch_id)
        .expect("generated merge patch should be stored");
    let patch = parse_patch_object(&patch_value).expect("generated patch should parse");
    assert!(patch.ops.iter().any(|op| matches!(
        op,
        PatchOperation::MoveBlock { block_id, parent_block_id: Some(parent_block_id), after_block_id: None }
        if block_id == "blk:merge-leaf" && parent_block_id == "blk:merge-parent"
    )));

    let _ = fs::remove_dir_all(store_root);
}

#[test]
fn merge_authoring_rejects_parent_matched_attr_variant_as_manual_curation_required() {
    let store_root = temp_dir("merge-attrs-manual");
    let signing_key = signing_key();
    let document = create_document_in_store(
        &store_root,
        &signing_key,
        &DocumentCreateParams {
            doc_id: "doc:merge-attrs".to_string(),
            title: "Merge Attrs".to_string(),
            language: "en".to_string(),
            timestamp: 40,
        },
    )
    .expect("document should be created");

    let base_revision_id = commit_ops_revision(
        &store_root,
        &signing_key,
        "doc:merge-attrs",
        &document.genesis_revision_id,
        41,
        42,
        json!([
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
        ]),
    );

    let attrs_revision_id = commit_ops_revision(
        &store_root,
        &signing_key,
        "doc:merge-attrs",
        &base_revision_id,
        43,
        44,
        json!([
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
        ]),
    );

    let mut attrs = serde_json::Map::new();
    attrs.insert("style".to_string(), Value::String("note".to_string()));
    let error = create_merge_revision_in_store(
        &store_root,
        &signing_key,
        &MergeRevisionCreateParams {
            doc_id: "doc:merge-attrs".to_string(),
            parents: vec![base_revision_id, attrs_revision_id],
            resolved_state: crate::replay::DocumentState {
                doc_id: "doc:merge-attrs".to_string(),
                blocks: vec![paragraph_block_with_attrs(
                    "blk:merge-attrs",
                    "Attrs",
                    attrs,
                )],
                metadata: serde_json::Map::new(),
            },
            merge_strategy: "semantic-block-merge".to_string(),
            timestamp: 45,
        },
    )
    .expect_err("merge revision should require manual curation");

    assert!(
        error.to_string().contains(
            "manual-curation-required: block 'blk:merge-attrs' changes attrs in an unsupported way"
        ),
        "expected attrs manual-curation error, got {error}"
    );

    let _ = fs::remove_dir_all(store_root);
}
