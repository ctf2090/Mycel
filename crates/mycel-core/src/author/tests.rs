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
fn commit_revision_to_store_updates_only_target_document_in_multi_doc_store() {
    let store_root = temp_dir("author-multi-doc");
    let signing_key = signing_key();

    let doc_a = create_document_in_store(
        &store_root,
        &signing_key,
        &DocumentCreateParams {
            doc_id: "doc:author-multi-a".to_string(),
            title: "Author Multi A".to_string(),
            language: "en".to_string(),
            timestamp: 10,
        },
    )
    .expect("doc A should be created");
    let doc_b = create_document_in_store(
        &store_root,
        &signing_key,
        &DocumentCreateParams {
            doc_id: "doc:author-multi-b".to_string(),
            title: "Author Multi B".to_string(),
            language: "en".to_string(),
            timestamp: 11,
        },
    )
    .expect("doc B should be created");

    let patch_b = create_patch_in_store(
        &store_root,
        &signing_key,
        &PatchCreateParams {
            doc_id: "doc:author-multi-b".to_string(),
            base_revision: doc_b.genesis_revision_id.clone(),
            timestamp: 12,
            ops: json!([
                {
                    "op": "insert_block",
                    "new_block": {
                        "block_id": "blk:multi-b-001",
                        "block_type": "paragraph",
                        "content": "Doc B line",
                        "attrs": {},
                        "children": []
                    }
                }
            ]),
        },
    )
    .expect("doc B patch should be created");
    let revision_b = commit_revision_to_store(
        &store_root,
        &signing_key,
        &RevisionCommitParams {
            doc_id: "doc:author-multi-b".to_string(),
            parents: vec![doc_b.genesis_revision_id.clone()],
            patches: vec![patch_b.patch_id],
            merge_strategy: None,
            timestamp: 13,
        },
    )
    .expect("doc B revision should be committed");

    let patch_a = create_patch_in_store(
        &store_root,
        &signing_key,
        &PatchCreateParams {
            doc_id: "doc:author-multi-a".to_string(),
            base_revision: doc_a.genesis_revision_id.clone(),
            timestamp: 14,
            ops: json!([
                {
                    "op": "insert_block",
                    "new_block": {
                        "block_id": "blk:multi-a-001",
                        "block_type": "paragraph",
                        "content": "Doc A line",
                        "attrs": {},
                        "children": []
                    }
                }
            ]),
        },
    )
    .expect("doc A patch should be created");
    let revision_a = commit_revision_to_store(
        &store_root,
        &signing_key,
        &RevisionCommitParams {
            doc_id: "doc:author-multi-a".to_string(),
            parents: vec![doc_a.genesis_revision_id.clone()],
            patches: vec![patch_a.patch_id],
            merge_strategy: None,
            timestamp: 15,
        },
    )
    .expect("doc A revision should be committed");

    let manifest = load_store_index_manifest(&store_root).expect("manifest should load");
    let mut doc_a_revisions = manifest
        .doc_revisions
        .get("doc:author-multi-a")
        .cloned()
        .expect("doc A revisions should be present");
    doc_a_revisions.sort();
    let mut expected_doc_a_revisions = vec![
        doc_a.genesis_revision_id.clone(),
        revision_a.revision_id.clone(),
    ];
    expected_doc_a_revisions.sort();
    assert_eq!(doc_a_revisions, expected_doc_a_revisions);

    let mut doc_b_revisions = manifest
        .doc_revisions
        .get("doc:author-multi-b")
        .cloned()
        .expect("doc B revisions should be present");
    doc_b_revisions.sort();
    let mut expected_doc_b_revisions = vec![
        doc_b.genesis_revision_id.clone(),
        revision_b.revision_id.clone(),
    ];
    expected_doc_b_revisions.sort();
    assert_eq!(doc_b_revisions, expected_doc_b_revisions);
    assert_eq!(
        manifest.doc_heads.get("doc:author-multi-a"),
        Some(&vec![revision_a.revision_id.clone()])
    );
    assert_eq!(
        manifest.doc_heads.get("doc:author-multi-b"),
        Some(&vec![revision_b.revision_id.clone()])
    );

    let mut object_index =
        crate::store::load_store_object_index(&store_root).expect("object index should load");
    object_index.insert(
        "doc:author-multi-a".to_string(),
        load_stored_object_value(&store_root, "doc:author-multi-a").expect("doc A should load"),
    );
    let replay = replay_revision_from_index(
        &load_stored_object_value(&store_root, &revision_a.revision_id)
            .expect("doc A revision should load"),
        &object_index,
    )
    .expect("doc A replay should succeed");
    assert_eq!(replay.state.doc_id, "doc:author-multi-a");
    assert_eq!(replay.state.blocks.len(), 1);
    assert_eq!(replay.state.blocks[0].content, "Doc A line");

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
fn merge_authoring_updates_only_target_document_in_multi_doc_store() {
    let store_root = temp_dir("merge-multi-doc");
    let signing_key = signing_key();

    let doc_a = create_document_in_store(
        &store_root,
        &signing_key,
        &DocumentCreateParams {
            doc_id: "doc:merge-multi-a".to_string(),
            title: "Merge Multi A".to_string(),
            language: "en".to_string(),
            timestamp: 20,
        },
    )
    .expect("doc A should be created");
    let doc_b = create_document_in_store(
        &store_root,
        &signing_key,
        &DocumentCreateParams {
            doc_id: "doc:merge-multi-b".to_string(),
            title: "Merge Multi B".to_string(),
            language: "en".to_string(),
            timestamp: 21,
        },
    )
    .expect("doc B should be created");

    let base_patch_a = create_patch_in_store(
        &store_root,
        &signing_key,
        &PatchCreateParams {
            doc_id: "doc:merge-multi-a".to_string(),
            base_revision: doc_a.genesis_revision_id.clone(),
            timestamp: 22,
            ops: json!([
                {
                    "op": "insert_block",
                    "new_block": {
                        "block_id": "blk:merge-multi-a",
                        "block_type": "paragraph",
                        "content": "Base A",
                        "attrs": {},
                        "children": []
                    }
                }
            ]),
        },
    )
    .expect("doc A base patch should be created");
    let base_revision_a = commit_revision_to_store(
        &store_root,
        &signing_key,
        &RevisionCommitParams {
            doc_id: "doc:merge-multi-a".to_string(),
            parents: vec![doc_a.genesis_revision_id.clone()],
            patches: vec![base_patch_a.patch_id],
            merge_strategy: None,
            timestamp: 23,
        },
    )
    .expect("doc A base revision should be committed");

    let left_patch_a = create_patch_in_store(
        &store_root,
        &signing_key,
        &PatchCreateParams {
            doc_id: "doc:merge-multi-a".to_string(),
            base_revision: base_revision_a.revision_id.clone(),
            timestamp: 24,
            ops: json!([
                {
                    "op": "replace_block",
                    "block_id": "blk:merge-multi-a",
                    "new_content": "Left A"
                }
            ]),
        },
    )
    .expect("doc A left patch should be created");
    let left_revision_a = commit_revision_to_store(
        &store_root,
        &signing_key,
        &RevisionCommitParams {
            doc_id: "doc:merge-multi-a".to_string(),
            parents: vec![base_revision_a.revision_id.clone()],
            patches: vec![left_patch_a.patch_id],
            merge_strategy: None,
            timestamp: 25,
        },
    )
    .expect("doc A left revision should be committed");

    let right_patch_a = create_patch_in_store(
        &store_root,
        &signing_key,
        &PatchCreateParams {
            doc_id: "doc:merge-multi-a".to_string(),
            base_revision: base_revision_a.revision_id.clone(),
            timestamp: 26,
            ops: json!([
                {
                    "op": "replace_block",
                    "block_id": "blk:merge-multi-a",
                    "new_content": "Right A"
                }
            ]),
        },
    )
    .expect("doc A right patch should be created");
    let right_revision_a = commit_revision_to_store(
        &store_root,
        &signing_key,
        &RevisionCommitParams {
            doc_id: "doc:merge-multi-a".to_string(),
            parents: vec![base_revision_a.revision_id.clone()],
            patches: vec![right_patch_a.patch_id],
            merge_strategy: None,
            timestamp: 27,
        },
    )
    .expect("doc A right revision should be committed");

    let patch_b = create_patch_in_store(
        &store_root,
        &signing_key,
        &PatchCreateParams {
            doc_id: "doc:merge-multi-b".to_string(),
            base_revision: doc_b.genesis_revision_id.clone(),
            timestamp: 28,
            ops: json!([
                {
                    "op": "insert_block",
                    "new_block": {
                        "block_id": "blk:merge-multi-b",
                        "block_type": "paragraph",
                        "content": "Doc B line",
                        "attrs": {},
                        "children": []
                    }
                }
            ]),
        },
    )
    .expect("doc B patch should be created");
    let revision_b = commit_revision_to_store(
        &store_root,
        &signing_key,
        &RevisionCommitParams {
            doc_id: "doc:merge-multi-b".to_string(),
            parents: vec![doc_b.genesis_revision_id.clone()],
            patches: vec![patch_b.patch_id],
            merge_strategy: None,
            timestamp: 29,
        },
    )
    .expect("doc B revision should be committed");

    let summary = create_merge_revision_in_store(
        &store_root,
        &signing_key,
        &MergeRevisionCreateParams {
            doc_id: "doc:merge-multi-a".to_string(),
            parents: vec![
                left_revision_a.revision_id.clone(),
                right_revision_a.revision_id.clone(),
            ],
            resolved_state: crate::replay::DocumentState {
                doc_id: "doc:merge-multi-a".to_string(),
                blocks: vec![paragraph_block("blk:merge-multi-a", "Right A")],
                metadata: serde_json::Map::new(),
            },
            merge_strategy: "semantic-block-merge".to_string(),
            timestamp: 30,
        },
    )
    .expect("merge revision should be created");

    let manifest = load_store_index_manifest(&store_root).expect("manifest should load");
    assert_eq!(
        manifest.doc_heads.get("doc:merge-multi-a"),
        Some(&vec![summary.revision_id.clone()])
    );
    assert_eq!(
        manifest.doc_heads.get("doc:merge-multi-b"),
        Some(&vec![revision_b.revision_id.clone()])
    );
    assert!(
        manifest
            .doc_revisions
            .get("doc:merge-multi-b")
            .is_some_and(|revisions| revisions
                == &vec![
                    doc_b.genesis_revision_id.clone(),
                    revision_b.revision_id.clone()
                ]),
        "doc B revisions should remain unchanged"
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

    assert_eq!(summary.merge_outcome, MergeOutcome::MultiVariant);
    assert!(
        summary
            .merge_reasons
            .iter()
            .any(|reason| reason.contains("selected a non-primary sibling placement")),
        "expected structural sibling multi-variant reason, got {summary:?}"
    );
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
fn merge_authoring_supports_nested_reparenting_of_multiple_existing_blocks() {
    let store_root = temp_dir("merge-nested-reparent-multi");
    let signing_key = signing_key();
    let document = create_document_in_store(
        &store_root,
        &signing_key,
        &DocumentCreateParams {
            doc_id: "doc:merge-nested-reparent-multi".to_string(),
            title: "Merge Nested Reparent Multi".to_string(),
            language: "en".to_string(),
            timestamp: 40,
        },
    )
    .expect("document should be created");

    let base_revision_id = commit_ops_revision(
        &store_root,
        &signing_key,
        "doc:merge-nested-reparent-multi",
        &document.genesis_revision_id,
        41,
        42,
        json!([
            {
                "op": "insert_block",
                "new_block": {
                    "block_id": "blk:nested-leaf-a",
                    "block_type": "paragraph",
                    "content": "Leaf A",
                    "attrs": {},
                    "children": []
                }
            },
            {
                "op": "insert_block",
                "new_block": {
                    "block_id": "blk:nested-leaf-b",
                    "block_type": "paragraph",
                    "content": "Leaf B",
                    "attrs": {},
                    "children": []
                }
            }
        ]),
    );

    let wrapper_revision_id = commit_ops_revision(
        &store_root,
        &signing_key,
        "doc:merge-nested-reparent-multi",
        &base_revision_id,
        43,
        44,
        json!([
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
        ]),
    );

    let section_revision_id = commit_ops_revision(
        &store_root,
        &signing_key,
        "doc:merge-nested-reparent-multi",
        &base_revision_id,
        45,
        46,
        json!([
            {
                "op": "insert_block",
                "new_block": {
                    "block_id": "blk:nested-section",
                    "block_type": "paragraph",
                    "content": "Section",
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
            doc_id: "doc:merge-nested-reparent-multi".to_string(),
            parents: vec![base_revision_id, wrapper_revision_id, section_revision_id],
            resolved_state: crate::replay::DocumentState {
                doc_id: "doc:merge-nested-reparent-multi".to_string(),
                blocks: vec![paragraph_block_with_children(
                    "blk:nested-wrapper",
                    "Wrapper",
                    vec![paragraph_block_with_children(
                        "blk:nested-section",
                        "Section",
                        vec![
                            paragraph_block("blk:nested-leaf-a", "Leaf A"),
                            paragraph_block("blk:nested-leaf-b", "Leaf B"),
                        ],
                    )],
                )],
                metadata: serde_json::Map::new(),
            },
            merge_strategy: "semantic-block-merge".to_string(),
            timestamp: 47,
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
        if new_block.block_id == "blk:nested-wrapper"
            && new_block.children.len() == 1
            && new_block.children[0].block_id == "blk:nested-section"
            && new_block.children[0].children.is_empty()
    )));
    assert!(patch.ops.iter().any(|op| matches!(
        op,
        PatchOperation::MoveBlock { block_id, parent_block_id: Some(parent_block_id), after_block_id: None }
        if block_id == "blk:nested-leaf-a" && parent_block_id == "blk:nested-section"
    )));
    assert!(patch.ops.iter().any(|op| matches!(
        op,
        PatchOperation::MoveBlock { block_id, parent_block_id: Some(parent_block_id), after_block_id: Some(after_block_id) }
        if block_id == "blk:nested-leaf-b"
            && parent_block_id == "blk:nested-section"
            && after_block_id == "blk:nested-leaf-a"
    )));

    let _ = fs::remove_dir_all(store_root);
}

#[test]
fn merge_authoring_supports_reparenting_into_a_later_sibling_parent() {
    let store_root = temp_dir("merge-reparent-later-sibling");
    let signing_key = signing_key();
    let document = create_document_in_store(
        &store_root,
        &signing_key,
        &DocumentCreateParams {
            doc_id: "doc:merge-reparent-later-sibling".to_string(),
            title: "Merge Reparent Later Sibling".to_string(),
            language: "en".to_string(),
            timestamp: 48,
        },
    )
    .expect("document should be created");

    let base_revision_id = commit_ops_revision(
        &store_root,
        &signing_key,
        "doc:merge-reparent-later-sibling",
        &document.genesis_revision_id,
        49,
        50,
        json!([
            {
                "op": "insert_block",
                "new_block": {
                    "block_id": "blk:sibling-parent-a",
                    "block_type": "paragraph",
                    "content": "Parent A",
                    "attrs": {},
                    "children": []
                }
            },
            {
                "op": "insert_block",
                "new_block": {
                    "block_id": "blk:sibling-parent-b",
                    "block_type": "paragraph",
                    "content": "Parent B",
                    "attrs": {},
                    "children": []
                }
            },
            {
                "op": "insert_block",
                "parent_block_id": "blk:sibling-parent-a",
                "new_block": {
                    "block_id": "blk:sibling-leaf",
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
        "doc:merge-reparent-later-sibling",
        &base_revision_id,
        51,
        52,
        json!([
            {
                "op": "move_block",
                "block_id": "blk:sibling-leaf",
                "parent_block_id": "blk:sibling-parent-b"
            }
        ]),
    );

    let summary = create_merge_revision_in_store(
        &store_root,
        &signing_key,
        &MergeRevisionCreateParams {
            doc_id: "doc:merge-reparent-later-sibling".to_string(),
            parents: vec![base_revision_id, moved_revision_id],
            resolved_state: crate::replay::DocumentState {
                doc_id: "doc:merge-reparent-later-sibling".to_string(),
                blocks: vec![
                    paragraph_block_with_children("blk:sibling-parent-a", "Parent A", vec![]),
                    paragraph_block_with_children(
                        "blk:sibling-parent-b",
                        "Parent B",
                        vec![paragraph_block("blk:sibling-leaf", "Leaf")],
                    ),
                ],
                metadata: serde_json::Map::new(),
            },
            merge_strategy: "semantic-block-merge".to_string(),
            timestamp: 53,
        },
    )
    .expect("merge revision should be created");

    assert_eq!(summary.merge_outcome, MergeOutcome::MultiVariant);
    let patch_value = load_stored_object_value(&store_root, &summary.patch_id)
        .expect("generated merge patch should be stored");
    let patch = parse_patch_object(&patch_value).expect("generated patch should parse");
    assert!(patch.ops.iter().any(|op| matches!(
        op,
        PatchOperation::MoveBlock { block_id, parent_block_id: Some(parent_block_id), after_block_id: None }
        if block_id == "blk:sibling-leaf" && parent_block_id == "blk:sibling-parent-b"
    )));

    let _ = fs::remove_dir_all(store_root);
}

#[test]
fn merge_authoring_supports_nested_reparenting_into_a_later_sibling_branch() {
    let store_root = temp_dir("merge-nested-later-sibling-branch");
    let signing_key = signing_key();
    let document = create_document_in_store(
        &store_root,
        &signing_key,
        &DocumentCreateParams {
            doc_id: "doc:merge-nested-later-sibling-branch".to_string(),
            title: "Merge Nested Later Sibling Branch".to_string(),
            language: "en".to_string(),
            timestamp: 54,
        },
    )
    .expect("document should be created");

    let base_revision_id = commit_ops_revision(
        &store_root,
        &signing_key,
        "doc:merge-nested-later-sibling-branch",
        &document.genesis_revision_id,
        55,
        56,
        json!([
            {
                "op": "insert_block",
                "new_block": {
                    "block_id": "blk:branch-root",
                    "block_type": "paragraph",
                    "content": "Root",
                    "attrs": {},
                    "children": [
                        {
                            "block_id": "blk:branch-a",
                            "block_type": "paragraph",
                            "content": "Branch A",
                            "attrs": {},
                            "children": [
                                {
                                    "block_id": "blk:branch-leaf",
                                    "block_type": "paragraph",
                                    "content": "Leaf",
                                    "attrs": {},
                                    "children": []
                                }
                            ]
                        },
                        {
                            "block_id": "blk:branch-b",
                            "block_type": "paragraph",
                            "content": "Branch B",
                            "attrs": {},
                            "children": []
                        }
                    ]
                }
            }
        ]),
    );

    let moved_revision_id = commit_ops_revision(
        &store_root,
        &signing_key,
        "doc:merge-nested-later-sibling-branch",
        &base_revision_id,
        57,
        58,
        json!([
            {
                "op": "move_block",
                "block_id": "blk:branch-leaf",
                "parent_block_id": "blk:branch-b"
            }
        ]),
    );

    let summary = create_merge_revision_in_store(
        &store_root,
        &signing_key,
        &MergeRevisionCreateParams {
            doc_id: "doc:merge-nested-later-sibling-branch".to_string(),
            parents: vec![base_revision_id, moved_revision_id],
            resolved_state: crate::replay::DocumentState {
                doc_id: "doc:merge-nested-later-sibling-branch".to_string(),
                blocks: vec![paragraph_block_with_children(
                    "blk:branch-root",
                    "Root",
                    vec![
                        paragraph_block_with_children("blk:branch-a", "Branch A", vec![]),
                        paragraph_block_with_children(
                            "blk:branch-b",
                            "Branch B",
                            vec![paragraph_block("blk:branch-leaf", "Leaf")],
                        ),
                    ],
                )],
                metadata: serde_json::Map::new(),
            },
            merge_strategy: "semantic-block-merge".to_string(),
            timestamp: 59,
        },
    )
    .expect("merge revision should be created");

    assert_eq!(summary.merge_outcome, MergeOutcome::MultiVariant);
    let patch_value = load_stored_object_value(&store_root, &summary.patch_id)
        .expect("generated merge patch should be stored");
    let patch = parse_patch_object(&patch_value).expect("generated patch should parse");
    assert!(patch.ops.iter().any(|op| matches!(
        op,
        PatchOperation::MoveBlock { block_id, parent_block_id: Some(parent_block_id), after_block_id: None }
        if block_id == "blk:branch-leaf" && parent_block_id == "blk:branch-b"
    )));

    let _ = fs::remove_dir_all(store_root);
}

#[test]
fn merge_authoring_supports_composed_parent_inserted_at_a_later_sibling_position() {
    let store_root = temp_dir("merge-composed-later-sibling-position");
    let signing_key = signing_key();
    let document = create_document_in_store(
        &store_root,
        &signing_key,
        &DocumentCreateParams {
            doc_id: "doc:merge-composed-later-sibling-position".to_string(),
            title: "Merge Composed Later Sibling Position".to_string(),
            language: "en".to_string(),
            timestamp: 60,
        },
    )
    .expect("document should be created");

    let base_revision_id = commit_ops_revision(
        &store_root,
        &signing_key,
        "doc:merge-composed-later-sibling-position",
        &document.genesis_revision_id,
        61,
        62,
        json!([
            {
                "op": "insert_block",
                "new_block": {
                    "block_id": "blk:cmp-a",
                    "block_type": "paragraph",
                    "content": "A",
                    "attrs": {},
                    "children": [
                        {
                            "block_id": "blk:cmp-leaf",
                            "block_type": "paragraph",
                            "content": "Leaf",
                            "attrs": {},
                            "children": []
                        }
                    ]
                }
            },
            {
                "op": "insert_block",
                "new_block": {
                    "block_id": "blk:cmp-b",
                    "block_type": "paragraph",
                    "content": "B",
                    "attrs": {},
                    "children": []
                }
            }
        ]),
    );

    let inserted_revision_id = commit_ops_revision(
        &store_root,
        &signing_key,
        "doc:merge-composed-later-sibling-position",
        &base_revision_id,
        63,
        64,
        json!([
            {
                "op": "insert_block_after",
                "after_block_id": "blk:cmp-b",
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
                            "children": []
                        }
                    ]
                }
            }
        ]),
    );

    let moved_revision_id = commit_ops_revision(
        &store_root,
        &signing_key,
        "doc:merge-composed-later-sibling-position",
        &base_revision_id,
        65,
        66,
        json!([
            {
                "op": "move_block",
                "block_id": "blk:cmp-leaf",
                "after_block_id": "blk:cmp-b"
            }
        ]),
    );

    let summary = create_merge_revision_in_store(
        &store_root,
        &signing_key,
        &MergeRevisionCreateParams {
            doc_id: "doc:merge-composed-later-sibling-position".to_string(),
            parents: vec![base_revision_id, inserted_revision_id, moved_revision_id],
            resolved_state: crate::replay::DocumentState {
                doc_id: "doc:merge-composed-later-sibling-position".to_string(),
                blocks: vec![
                    paragraph_block_with_children("blk:cmp-a", "A", vec![]),
                    paragraph_block("blk:cmp-b", "B"),
                    paragraph_block_with_children(
                        "blk:cmp-wrapper",
                        "Wrapper",
                        vec![paragraph_block_with_children(
                            "blk:cmp-section",
                            "Section",
                            vec![paragraph_block("blk:cmp-leaf", "Leaf")],
                        )],
                    ),
                ],
                metadata: serde_json::Map::new(),
            },
            merge_strategy: "semantic-block-merge".to_string(),
            timestamp: 67,
        },
    )
    .expect("merge revision should be created");

    assert_eq!(summary.merge_outcome, MergeOutcome::AutoMerged);
    let patch_value = load_stored_object_value(&store_root, &summary.patch_id)
        .expect("generated merge patch should be stored");
    let patch = parse_patch_object(&patch_value).expect("generated patch should parse");
    assert!(patch.ops.iter().any(|op| matches!(
        op,
        PatchOperation::InsertBlockAfter { after_block_id, new_block }
        if after_block_id == "blk:cmp-b"
            && new_block.block_id == "blk:cmp-wrapper"
            && new_block.children.len() == 1
            && new_block.children[0].block_id == "blk:cmp-section"
    )));
    assert!(patch.ops.iter().any(|op| matches!(
        op,
        PatchOperation::MoveBlock { block_id, parent_block_id: Some(parent_block_id), after_block_id: None }
        if block_id == "blk:cmp-leaf" && parent_block_id == "blk:cmp-section"
    )));

    let _ = fs::remove_dir_all(store_root);
}

#[test]
fn merge_authoring_supports_multiple_existing_blocks_reparented_into_a_deep_composed_branch() {
    let store_root = temp_dir("merge-deep-composed-branch-multi");
    let signing_key = signing_key();
    let document = create_document_in_store(
        &store_root,
        &signing_key,
        &DocumentCreateParams {
            doc_id: "doc:merge-deep-composed-branch-multi".to_string(),
            title: "Merge Deep Composed Branch Multi".to_string(),
            language: "en".to_string(),
            timestamp: 68,
        },
    )
    .expect("document should be created");

    let base_revision_id = commit_ops_revision(
        &store_root,
        &signing_key,
        "doc:merge-deep-composed-branch-multi",
        &document.genesis_revision_id,
        69,
        70,
        json!([
            {
                "op": "insert_block",
                "new_block": {
                    "block_id": "blk:deep-anchor",
                    "block_type": "paragraph",
                    "content": "Anchor",
                    "attrs": {},
                    "children": []
                }
            },
            {
                "op": "insert_block",
                "new_block": {
                    "block_id": "blk:deep-leaf-a",
                    "block_type": "paragraph",
                    "content": "Leaf A",
                    "attrs": {},
                    "children": []
                }
            },
            {
                "op": "insert_block",
                "new_block": {
                    "block_id": "blk:deep-leaf-b",
                    "block_type": "paragraph",
                    "content": "Leaf B",
                    "attrs": {},
                    "children": []
                }
            }
        ]),
    );

    let inserted_revision_id = commit_ops_revision(
        &store_root,
        &signing_key,
        "doc:merge-deep-composed-branch-multi",
        &base_revision_id,
        71,
        72,
        json!([
            {
                "op": "insert_block_after",
                "after_block_id": "blk:deep-anchor",
                "new_block": {
                    "block_id": "blk:deep-wrapper",
                    "block_type": "paragraph",
                    "content": "Wrapper",
                    "attrs": {},
                    "children": [
                        {
                            "block_id": "blk:deep-section",
                            "block_type": "paragraph",
                            "content": "Section",
                            "attrs": {},
                            "children": [
                                {
                                    "block_id": "blk:deep-subsection",
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
        ]),
    );

    let reordered_a_revision_id = commit_ops_revision(
        &store_root,
        &signing_key,
        "doc:merge-deep-composed-branch-multi",
        &base_revision_id,
        73,
        74,
        json!([
            {
                "op": "move_block",
                "block_id": "blk:deep-leaf-a",
                "after_block_id": "blk:deep-anchor"
            }
        ]),
    );

    let reordered_b_revision_id = commit_ops_revision(
        &store_root,
        &signing_key,
        "doc:merge-deep-composed-branch-multi",
        &base_revision_id,
        75,
        76,
        json!([
            {
                "op": "move_block",
                "block_id": "blk:deep-leaf-b",
                "after_block_id": "blk:deep-leaf-a"
            }
        ]),
    );

    let summary = create_merge_revision_in_store(
        &store_root,
        &signing_key,
        &MergeRevisionCreateParams {
            doc_id: "doc:merge-deep-composed-branch-multi".to_string(),
            parents: vec![
                base_revision_id,
                inserted_revision_id,
                reordered_a_revision_id,
                reordered_b_revision_id,
            ],
            resolved_state: crate::replay::DocumentState {
                doc_id: "doc:merge-deep-composed-branch-multi".to_string(),
                blocks: vec![
                    paragraph_block("blk:deep-anchor", "Anchor"),
                    paragraph_block_with_children(
                        "blk:deep-wrapper",
                        "Wrapper",
                        vec![paragraph_block_with_children(
                            "blk:deep-section",
                            "Section",
                            vec![paragraph_block_with_children(
                                "blk:deep-subsection",
                                "Subsection",
                                vec![
                                    paragraph_block("blk:deep-leaf-a", "Leaf A"),
                                    paragraph_block("blk:deep-leaf-b", "Leaf B"),
                                ],
                            )],
                        )],
                    ),
                ],
                metadata: serde_json::Map::new(),
            },
            merge_strategy: "semantic-block-merge".to_string(),
            timestamp: 77,
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
        PatchOperation::InsertBlockAfter { after_block_id, new_block }
        if after_block_id == "blk:deep-anchor"
            && new_block.block_id == "blk:deep-wrapper"
            && new_block.children.len() == 1
            && new_block.children[0].block_id == "blk:deep-section"
            && new_block.children[0].children.len() == 1
            && new_block.children[0].children[0].block_id == "blk:deep-subsection"
    )));
    assert!(patch.ops.iter().any(|op| matches!(
        op,
        PatchOperation::MoveBlock { block_id, parent_block_id: Some(parent_block_id), after_block_id: None }
        if block_id == "blk:deep-leaf-a" && parent_block_id == "blk:deep-subsection"
    )));
    assert!(patch.ops.iter().any(|op| matches!(
        op,
        PatchOperation::MoveBlock { block_id, parent_block_id: Some(parent_block_id), after_block_id: Some(after_block_id) }
        if block_id == "blk:deep-leaf-b"
            && parent_block_id == "blk:deep-subsection"
            && after_block_id == "blk:deep-leaf-a"
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
fn merge_authoring_marks_nested_parent_choice_conflicts_as_multi_variant() {
    let store_root = temp_dir("merge-nested-parent-choice");
    let signing_key = signing_key();
    let document = create_document_in_store(
        &store_root,
        &signing_key,
        &DocumentCreateParams {
            doc_id: "doc:merge-nested-parent-choice".to_string(),
            title: "Merge Nested Parent Choice".to_string(),
            language: "en".to_string(),
            timestamp: 48,
        },
    )
    .expect("document should be created");

    let base_revision_id = commit_ops_revision(
        &store_root,
        &signing_key,
        "doc:merge-nested-parent-choice",
        &document.genesis_revision_id,
        49,
        50,
        json!([
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
        ]),
    );

    let wrapper_revision_id = commit_ops_revision(
        &store_root,
        &signing_key,
        "doc:merge-nested-parent-choice",
        &base_revision_id,
        51,
        52,
        json!([
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
        ]),
    );

    let left_revision_id = commit_ops_revision(
        &store_root,
        &signing_key,
        "doc:merge-nested-parent-choice",
        &base_revision_id,
        53,
        54,
        json!([
            {
                "op": "move_block",
                "block_id": "blk:nested-leaf",
                "parent_block_id": "blk:nested-left"
            }
        ]),
    );

    let right_revision_id = commit_ops_revision(
        &store_root,
        &signing_key,
        "doc:merge-nested-parent-choice",
        &base_revision_id,
        55,
        56,
        json!([
            {
                "op": "move_block",
                "block_id": "blk:nested-leaf",
                "parent_block_id": "blk:nested-right"
            }
        ]),
    );

    let summary = create_merge_revision_in_store(
        &store_root,
        &signing_key,
        &MergeRevisionCreateParams {
            doc_id: "doc:merge-nested-parent-choice".to_string(),
            parents: vec![
                base_revision_id,
                wrapper_revision_id,
                left_revision_id,
                right_revision_id,
            ],
            resolved_state: crate::replay::DocumentState {
                doc_id: "doc:merge-nested-parent-choice".to_string(),
                blocks: vec![paragraph_block_with_children(
                    "blk:nested-wrapper",
                    "Wrapper",
                    vec![
                        paragraph_block_with_children(
                            "blk:nested-left",
                            "Left",
                            vec![paragraph_block("blk:nested-leaf", "Leaf")],
                        ),
                        paragraph_block("blk:nested-right", "Right"),
                    ],
                )],
                metadata: serde_json::Map::new(),
            },
            merge_strategy: "semantic-block-merge".to_string(),
            timestamp: 57,
        },
    )
    .expect("merge revision should be created");

    assert_eq!(summary.merge_outcome, MergeOutcome::MultiVariant);
    assert!(
        summary
            .merge_reasons
            .iter()
            .any(|reason| reason.contains("selected a non-primary parent placement")),
        "expected nested structural multi-variant reason, got {summary:?}"
    );
    let patch_value = load_stored_object_value(&store_root, &summary.patch_id)
        .expect("generated merge patch should be stored");
    let patch = parse_patch_object(&patch_value).expect("generated patch should parse");
    assert!(patch.ops.iter().any(|op| matches!(
        op,
        PatchOperation::InsertBlock { new_block, .. }
        if new_block.block_id == "blk:nested-wrapper"
    )));
    assert!(patch.ops.iter().any(|op| matches!(
        op,
        PatchOperation::MoveBlock { block_id, parent_block_id: Some(parent_block_id), .. }
        if block_id == "blk:nested-left" && parent_block_id == "blk:nested-wrapper"
    )));
    assert!(patch.ops.iter().any(|op| matches!(
        op,
        PatchOperation::MoveBlock { block_id, parent_block_id: Some(parent_block_id), .. }
        if block_id == "blk:nested-leaf" && parent_block_id == "blk:nested-left"
    )));

    let _ = fs::remove_dir_all(store_root);
}

#[test]
fn merge_authoring_marks_nested_non_primary_sibling_choice_as_multi_variant() {
    let store_root = temp_dir("merge-nested-sibling-choice");
    let signing_key = signing_key();
    let document = create_document_in_store(
        &store_root,
        &signing_key,
        &DocumentCreateParams {
            doc_id: "doc:merge-nested-sibling-choice".to_string(),
            title: "Merge Nested Sibling Choice".to_string(),
            language: "en".to_string(),
            timestamp: 58,
        },
    )
    .expect("document should be created");

    let base_revision_id = commit_ops_revision(
        &store_root,
        &signing_key,
        "doc:merge-nested-sibling-choice",
        &document.genesis_revision_id,
        59,
        60,
        json!([
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
                        }
                    ]
                }
            }
        ]),
    );

    let reordered_revision_id = commit_ops_revision(
        &store_root,
        &signing_key,
        "doc:merge-nested-sibling-choice",
        &base_revision_id,
        61,
        62,
        json!([
            {
                "op": "move_block",
                "block_id": "blk:nested-child-a",
                "parent_block_id": "blk:nested-parent",
                "after_block_id": "blk:nested-child-b"
            }
        ]),
    );

    let summary = create_merge_revision_in_store(
        &store_root,
        &signing_key,
        &MergeRevisionCreateParams {
            doc_id: "doc:merge-nested-sibling-choice".to_string(),
            parents: vec![base_revision_id, reordered_revision_id],
            resolved_state: crate::replay::DocumentState {
                doc_id: "doc:merge-nested-sibling-choice".to_string(),
                blocks: vec![paragraph_block_with_children(
                    "blk:nested-parent",
                    "Parent",
                    vec![
                        paragraph_block("blk:nested-child-b", "Child B"),
                        paragraph_block("blk:nested-child-a", "Child A"),
                    ],
                )],
                metadata: serde_json::Map::new(),
            },
            merge_strategy: "semantic-block-merge".to_string(),
            timestamp: 63,
        },
    )
    .expect("merge revision should be created");

    assert_eq!(summary.merge_outcome, MergeOutcome::MultiVariant);
    assert!(
        summary
            .merge_reasons
            .iter()
            .any(|reason| reason.contains("selected a non-primary sibling placement")),
        "expected nested sibling multi-variant reason, got {summary:?}"
    );
    let patch_value = load_stored_object_value(&store_root, &summary.patch_id)
        .expect("generated merge patch should be stored");
    let patch = parse_patch_object(&patch_value).expect("generated patch should parse");
    assert!(patch.ops.iter().any(|op| matches!(
        op,
        PatchOperation::MoveBlock { block_id, parent_block_id: Some(parent_block_id), after_block_id: Some(after_block_id) }
        if block_id == "blk:nested-child-a"
            && parent_block_id == "blk:nested-parent"
            && after_block_id == "blk:nested-child-b"
    )));

    let _ = fs::remove_dir_all(store_root);
}

#[test]
fn merge_authoring_rejects_novel_nested_parent_choice_when_other_parent_moves_block() {
    let store_root = temp_dir("merge-novel-nested-parent-manual");
    let signing_key = signing_key();
    let document = create_document_in_store(
        &store_root,
        &signing_key,
        &DocumentCreateParams {
            doc_id: "doc:merge-novel-nested-parent-manual".to_string(),
            title: "Merge Novel Nested Parent Manual".to_string(),
            language: "en".to_string(),
            timestamp: 58,
        },
    )
    .expect("document should be created");

    let base_revision_id = commit_ops_revision(
        &store_root,
        &signing_key,
        "doc:merge-novel-nested-parent-manual",
        &document.genesis_revision_id,
        59,
        60,
        json!([
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
                    "block_id": "blk:manual-leaf",
                    "block_type": "paragraph",
                    "content": "Leaf",
                    "attrs": {},
                    "children": []
                }
            }
        ]),
    );

    let wrapper_revision_id = commit_ops_revision(
        &store_root,
        &signing_key,
        "doc:merge-novel-nested-parent-manual",
        &base_revision_id,
        61,
        62,
        json!([
            {
                "op": "insert_block",
                "new_block": {
                    "block_id": "blk:manual-wrapper",
                    "block_type": "paragraph",
                    "content": "Wrapper",
                    "attrs": {},
                    "children": []
                }
            }
        ]),
    );

    let moved_revision_id = commit_ops_revision(
        &store_root,
        &signing_key,
        "doc:merge-novel-nested-parent-manual",
        &base_revision_id,
        63,
        64,
        json!([
            {
                "op": "move_block",
                "block_id": "blk:manual-leaf",
                "parent_block_id": "blk:manual-left"
            }
        ]),
    );

    let summary = create_merge_revision_in_store(
        &store_root,
        &signing_key,
        &MergeRevisionCreateParams {
            doc_id: "doc:merge-novel-nested-parent-manual".to_string(),
            parents: vec![base_revision_id, wrapper_revision_id, moved_revision_id],
            resolved_state: crate::replay::DocumentState {
                doc_id: "doc:merge-novel-nested-parent-manual".to_string(),
                blocks: vec![paragraph_block_with_children(
                    "blk:manual-wrapper",
                    "Wrapper",
                    vec![
                        paragraph_block("blk:manual-left", "Left"),
                        paragraph_block("blk:manual-leaf", "Leaf"),
                    ],
                )],
                metadata: serde_json::Map::new(),
            },
            merge_strategy: "semantic-block-merge".to_string(),
            timestamp: 65,
        },
    )
    .expect("merge revision should be created");

    assert_eq!(summary.merge_outcome, MergeOutcome::MultiVariant);
    assert!(
        summary
            .merge_reasons
            .iter()
            .any(|reason| reason.contains("selected a non-primary parent placement")),
        "expected nested placement multi-variant reason, got {summary:?}"
    );
    let patch_value = load_stored_object_value(&store_root, &summary.patch_id)
        .expect("generated merge patch should be stored");
    let patch = parse_patch_object(&patch_value).expect("generated patch should parse");
    assert!(patch.ops.iter().any(|op| matches!(
        op,
        PatchOperation::InsertBlock { new_block, .. }
        if new_block.block_id == "blk:manual-wrapper"
    )));
    assert!(patch.ops.iter().any(|op| matches!(
        op,
        PatchOperation::MoveBlock { block_id, parent_block_id: Some(parent_block_id), after_block_id: None }
        if block_id == "blk:manual-left" && parent_block_id == "blk:manual-wrapper"
    )));
    assert!(patch.ops.iter().any(|op| matches!(
        op,
        PatchOperation::MoveBlock { block_id, parent_block_id: Some(parent_block_id), after_block_id: Some(after_block_id) }
        if block_id == "blk:manual-leaf"
            && parent_block_id == "blk:manual-wrapper"
            && after_block_id == "blk:manual-left"
    )));

    let _ = fs::remove_dir_all(store_root);
}

#[test]
fn merge_authoring_marks_composed_descendant_of_non_primary_parent_as_multi_variant() {
    let store_root = temp_dir("merge-composed-non-primary-parent-choice");
    let signing_key = signing_key();
    let document = create_document_in_store(
        &store_root,
        &signing_key,
        &DocumentCreateParams {
            doc_id: "doc:merge-composed-non-primary-parent-choice".to_string(),
            title: "Merge Composed Non Primary Parent Choice".to_string(),
            language: "en".to_string(),
            timestamp: 66,
        },
    )
    .expect("document should be created");

    let base_revision_id = commit_ops_revision(
        &store_root,
        &signing_key,
        "doc:merge-composed-non-primary-parent-choice",
        &document.genesis_revision_id,
        67,
        68,
        json!([
            {
                "op": "insert_block",
                "new_block": {
                    "block_id": "blk:composed-parent",
                    "block_type": "paragraph",
                    "content": "Parent",
                    "attrs": {},
                    "children": []
                }
            },
            {
                "op": "insert_block",
                "new_block": {
                    "block_id": "blk:composed-leaf",
                    "block_type": "paragraph",
                    "content": "Leaf",
                    "attrs": {},
                    "children": []
                }
            }
        ]),
    );

    let subsection_revision_id = commit_ops_revision(
        &store_root,
        &signing_key,
        "doc:merge-composed-non-primary-parent-choice",
        &base_revision_id,
        69,
        70,
        json!([
            {
                "op": "insert_block",
                "parent_block_id": "blk:composed-parent",
                "new_block": {
                    "block_id": "blk:composed-subsection",
                    "block_type": "paragraph",
                    "content": "Subsection",
                    "attrs": {},
                    "children": []
                }
            }
        ]),
    );

    let moved_revision_id = commit_ops_revision(
        &store_root,
        &signing_key,
        "doc:merge-composed-non-primary-parent-choice",
        &base_revision_id,
        71,
        72,
        json!([
            {
                "op": "move_block",
                "block_id": "blk:composed-leaf",
                "parent_block_id": "blk:composed-parent"
            }
        ]),
    );

    let summary = create_merge_revision_in_store(
        &store_root,
        &signing_key,
        &MergeRevisionCreateParams {
            doc_id: "doc:merge-composed-non-primary-parent-choice".to_string(),
            parents: vec![base_revision_id, subsection_revision_id, moved_revision_id],
            resolved_state: crate::replay::DocumentState {
                doc_id: "doc:merge-composed-non-primary-parent-choice".to_string(),
                blocks: vec![paragraph_block_with_children(
                    "blk:composed-parent",
                    "Parent",
                    vec![paragraph_block_with_children(
                        "blk:composed-subsection",
                        "Subsection",
                        vec![paragraph_block("blk:composed-leaf", "Leaf")],
                    )],
                )],
                metadata: serde_json::Map::new(),
            },
            merge_strategy: "semantic-block-merge".to_string(),
            timestamp: 73,
        },
    )
    .expect("merge revision should be created");

    assert_eq!(summary.merge_outcome, MergeOutcome::MultiVariant);
    assert!(
        summary
            .merge_reasons
            .iter()
            .any(|reason| reason.contains("selected a non-primary parent placement")),
        "expected composed non-primary parent multi-variant reason, got {summary:?}"
    );
    let patch_value = load_stored_object_value(&store_root, &summary.patch_id)
        .expect("generated merge patch should be stored");
    let patch = parse_patch_object(&patch_value).expect("generated patch should parse");
    assert!(patch.ops.iter().any(|op| matches!(
        op,
        PatchOperation::InsertBlock { parent_block_id: Some(parent_block_id), new_block, .. }
        if parent_block_id == "blk:composed-parent" && new_block.block_id == "blk:composed-subsection"
    )));
    assert!(patch.ops.iter().any(|op| matches!(
        op,
        PatchOperation::MoveBlock { block_id, parent_block_id: Some(parent_block_id), after_block_id: None }
        if block_id == "blk:composed-leaf" && parent_block_id == "blk:composed-subsection"
    )));

    let _ = fs::remove_dir_all(store_root);
}

#[test]
fn merge_authoring_marks_multiple_siblings_under_composed_non_primary_parent_as_multi_variant() {
    let store_root = temp_dir("merge-composed-non-primary-parent-siblings");
    let signing_key = signing_key();
    let document = create_document_in_store(
        &store_root,
        &signing_key,
        &DocumentCreateParams {
            doc_id: "doc:merge-composed-non-primary-parent-siblings".to_string(),
            title: "Merge Composed Non Primary Parent Siblings".to_string(),
            language: "en".to_string(),
            timestamp: 74,
        },
    )
    .expect("document should be created");

    let base_revision_id = commit_ops_revision(
        &store_root,
        &signing_key,
        "doc:merge-composed-non-primary-parent-siblings",
        &document.genesis_revision_id,
        75,
        76,
        json!([
            {
                "op": "insert_block",
                "new_block": {
                    "block_id": "blk:composed-parent-siblings",
                    "block_type": "paragraph",
                    "content": "Parent",
                    "attrs": {},
                    "children": []
                }
            },
            {
                "op": "insert_block",
                "new_block": {
                    "block_id": "blk:composed-leaf-a",
                    "block_type": "paragraph",
                    "content": "Leaf A",
                    "attrs": {},
                    "children": []
                }
            },
            {
                "op": "insert_block",
                "new_block": {
                    "block_id": "blk:composed-leaf-b",
                    "block_type": "paragraph",
                    "content": "Leaf B",
                    "attrs": {},
                    "children": []
                }
            }
        ]),
    );

    let subsection_revision_id = commit_ops_revision(
        &store_root,
        &signing_key,
        "doc:merge-composed-non-primary-parent-siblings",
        &base_revision_id,
        77,
        78,
        json!([
            {
                "op": "insert_block",
                "parent_block_id": "blk:composed-parent-siblings",
                "new_block": {
                    "block_id": "blk:composed-subsection-siblings",
                    "block_type": "paragraph",
                    "content": "Subsection",
                    "attrs": {},
                    "children": []
                }
            }
        ]),
    );

    let moved_revision_id = commit_ops_revision(
        &store_root,
        &signing_key,
        "doc:merge-composed-non-primary-parent-siblings",
        &base_revision_id,
        79,
        80,
        json!([
            {
                "op": "move_block",
                "block_id": "blk:composed-leaf-a",
                "parent_block_id": "blk:composed-parent-siblings"
            },
            {
                "op": "move_block",
                "block_id": "blk:composed-leaf-b",
                "parent_block_id": "blk:composed-parent-siblings",
                "after_block_id": "blk:composed-leaf-a"
            }
        ]),
    );

    let summary = create_merge_revision_in_store(
        &store_root,
        &signing_key,
        &MergeRevisionCreateParams {
            doc_id: "doc:merge-composed-non-primary-parent-siblings".to_string(),
            parents: vec![base_revision_id, subsection_revision_id, moved_revision_id],
            resolved_state: crate::replay::DocumentState {
                doc_id: "doc:merge-composed-non-primary-parent-siblings".to_string(),
                blocks: vec![paragraph_block_with_children(
                    "blk:composed-parent-siblings",
                    "Parent",
                    vec![paragraph_block_with_children(
                        "blk:composed-subsection-siblings",
                        "Subsection",
                        vec![
                            paragraph_block("blk:composed-leaf-a", "Leaf A"),
                            paragraph_block("blk:composed-leaf-b", "Leaf B"),
                        ],
                    )],
                )],
                metadata: serde_json::Map::new(),
            },
            merge_strategy: "semantic-block-merge".to_string(),
            timestamp: 81,
        },
    )
    .expect("merge revision should be created");

    assert_eq!(summary.merge_outcome, MergeOutcome::MultiVariant);
    assert!(
        summary
            .merge_reasons
            .iter()
            .any(|reason| reason.contains("selected a non-primary parent placement")),
        "expected composed sibling multi-variant reason, got {summary:?}"
    );
    let patch_value = load_stored_object_value(&store_root, &summary.patch_id)
        .expect("generated merge patch should be stored");
    let patch = parse_patch_object(&patch_value).expect("generated patch should parse");
    assert!(patch.ops.iter().any(|op| matches!(
        op,
        PatchOperation::InsertBlock { parent_block_id: Some(parent_block_id), new_block, .. }
        if parent_block_id == "blk:composed-parent-siblings"
            && new_block.block_id == "blk:composed-subsection-siblings"
    )));
    assert!(patch.ops.iter().any(|op| matches!(
        op,
        PatchOperation::MoveBlock { block_id, parent_block_id: Some(parent_block_id), after_block_id: None }
        if block_id == "blk:composed-leaf-a" && parent_block_id == "blk:composed-subsection-siblings"
    )));
    assert!(patch.ops.iter().any(|op| matches!(
        op,
        PatchOperation::MoveBlock { block_id, parent_block_id: Some(parent_block_id), after_block_id: Some(after_block_id) }
        if block_id == "blk:composed-leaf-b"
            && parent_block_id == "blk:composed-subsection-siblings"
            && after_block_id == "blk:composed-leaf-a"
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
