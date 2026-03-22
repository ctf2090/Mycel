use super::*;
use crate::author::types::{MergeReasonKind, MergeReasonSubjectKind, MergeReasonVariantKind};

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
    assert!(
        summary
            .merge_reasons
            .iter()
            .any(|reason| reason.contains("multiple competing parent placements")),
        "expected competing parent placement reason, got {summary:?}"
    );
    let parent_detail = summary
        .merge_reason_details
        .iter()
        .find(|detail| {
            detail.subject_id == "blk:nested-leaf"
                && detail.reason_kind == MergeReasonKind::SelectedNonPrimaryParentVariant
                && detail.variant_kind == MergeReasonVariantKind::ParentPlacement
        })
        .expect("expected structured parent placement detail");
    assert_eq!(parent_detail.subject_kind, MergeReasonSubjectKind::Block);
    assert_eq!(parent_detail.primary_variant, "<root>");
    assert_eq!(parent_detail.resolved_variant, "blk:nested-left");
    assert_eq!(parent_detail.competing_variants, vec!["blk:nested-right".to_string()]);
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
fn merge_authoring_marks_anchor_based_nested_parent_choice_conflicts_as_multi_variant() {
    let store_root = temp_dir("merge-anchor-nested-parent-choice");
    let signing_key = signing_key();
    let document = create_document_in_store(
        &store_root,
        &signing_key,
        &DocumentCreateParams {
            doc_id: "doc:merge-anchor-nested-parent-choice".to_string(),
            title: "Merge Anchor Nested Parent Choice".to_string(),
            language: "en".to_string(),
            timestamp: 58,
        },
    )
    .expect("document should be created");

    let base_revision_id = commit_ops_revision(
        &store_root,
        &signing_key,
        "doc:merge-anchor-nested-parent-choice",
        &document.genesis_revision_id,
        59,
        60,
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

    let subsection_revision_id = commit_ops_revision(
        &store_root,
        &signing_key,
        "doc:merge-anchor-nested-parent-choice",
        &base_revision_id,
        61,
        62,
        json!([
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
        ]),
    );

    let left_revision_id = commit_ops_revision(
        &store_root,
        &signing_key,
        "doc:merge-anchor-nested-parent-choice",
        &base_revision_id,
        63,
        64,
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
        "doc:merge-anchor-nested-parent-choice",
        &base_revision_id,
        65,
        66,
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
            doc_id: "doc:merge-anchor-nested-parent-choice".to_string(),
            parents: vec![
                base_revision_id,
                subsection_revision_id,
                left_revision_id,
                right_revision_id,
            ],
            resolved_state: crate::replay::DocumentState {
                doc_id: "doc:merge-anchor-nested-parent-choice".to_string(),
                blocks: vec![
                    paragraph_block_with_children(
                        "blk:nested-left",
                        "Left",
                        vec![paragraph_block_with_children(
                            "blk:nested-subsection",
                            "Subsection",
                            vec![paragraph_block("blk:nested-leaf", "Leaf")],
                        )],
                    ),
                    paragraph_block("blk:nested-right", "Right"),
                ],
                metadata: serde_json::Map::new(),
            },
            merge_strategy: "semantic-block-merge".to_string(),
            timestamp: 67,
        },
    )
    .expect("merge revision should be created");

    assert_eq!(summary.merge_outcome, MergeOutcome::MultiVariant);
    assert!(
        summary
            .merge_reasons
            .iter()
            .any(|reason| reason.contains("selected a non-primary parent placement")),
        "expected anchor nested parent multi-variant reason, got {summary:?}"
    );
    assert!(
        summary
            .merge_reasons
            .iter()
            .any(|reason| reason.contains("multiple competing parent placements")),
        "expected anchor competing parent placement reason, got {summary:?}"
    );
    let patch_value = load_stored_object_value(&store_root, &summary.patch_id)
        .expect("generated merge patch should be stored");
    let patch = parse_patch_object(&patch_value).expect("generated patch should parse");
    assert!(patch.ops.iter().any(|op| matches!(
        op,
        PatchOperation::InsertBlock { parent_block_id: Some(parent_block_id), new_block, .. }
        if parent_block_id == "blk:nested-left" && new_block.block_id == "blk:nested-subsection"
    )));
    assert!(patch.ops.iter().any(|op| matches!(
        op,
        PatchOperation::MoveBlock { block_id, parent_block_id: Some(parent_block_id), .. }
        if block_id == "blk:nested-leaf" && parent_block_id == "blk:nested-subsection"
    )));

    let _ = fs::remove_dir_all(store_root);
}

#[test]
fn merge_authoring_marks_multiple_competing_nested_sibling_placements_as_multi_variant() {
    let store_root = temp_dir("merge-nested-sibling-competing");
    let signing_key = signing_key();
    let document = create_document_in_store(
        &store_root,
        &signing_key,
        &DocumentCreateParams {
            doc_id: "doc:merge-nested-sibling-competing".to_string(),
            title: "Merge Nested Sibling Competing".to_string(),
            language: "en".to_string(),
            timestamp: 58,
        },
    )
    .expect("document should be created");

    let base_revision_id = commit_ops_revision(
        &store_root,
        &signing_key,
        "doc:merge-nested-sibling-competing",
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
        ]),
    );

    let moved_after_b_revision_id = commit_ops_revision(
        &store_root,
        &signing_key,
        "doc:merge-nested-sibling-competing",
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

    let moved_after_c_revision_id = commit_ops_revision(
        &store_root,
        &signing_key,
        "doc:merge-nested-sibling-competing",
        &base_revision_id,
        63,
        64,
        json!([
            {
                "op": "move_block",
                "block_id": "blk:nested-child-a",
                "parent_block_id": "blk:nested-parent",
                "after_block_id": "blk:nested-child-c"
            }
        ]),
    );

    let summary = create_merge_revision_in_store(
        &store_root,
        &signing_key,
        &MergeRevisionCreateParams {
            doc_id: "doc:merge-nested-sibling-competing".to_string(),
            parents: vec![
                base_revision_id,
                moved_after_b_revision_id,
                moved_after_c_revision_id,
            ],
            resolved_state: crate::replay::DocumentState {
                doc_id: "doc:merge-nested-sibling-competing".to_string(),
                blocks: vec![paragraph_block_with_children(
                    "blk:nested-parent",
                    "Parent",
                    vec![
                        paragraph_block("blk:nested-child-b", "Child B"),
                        paragraph_block("blk:nested-child-c", "Child C"),
                        paragraph_block("blk:nested-child-a", "Child A"),
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
            .any(|reason| reason.contains("selected a non-primary sibling placement")),
        "expected nested sibling multi-variant reason, got {summary:?}"
    );
    assert!(
        summary
            .merge_reasons
            .iter()
            .any(|reason| reason.contains("multiple competing sibling placements")),
        "expected competing sibling placement reason, got {summary:?}"
    );
    let sibling_detail = summary
        .merge_reason_details
        .iter()
        .find(|detail| {
            detail.subject_id == "blk:nested-child-a"
                && detail.reason_kind == MergeReasonKind::SelectedNonPrimaryParentVariant
                && detail.variant_kind == MergeReasonVariantKind::SiblingPlacement
        })
        .expect("expected structured sibling placement detail");
    assert_eq!(sibling_detail.subject_kind, MergeReasonSubjectKind::Block);
    assert_eq!(sibling_detail.primary_variant, "<start>");
    assert_eq!(sibling_detail.resolved_variant, "blk:nested-child-c");
    assert_eq!(
        sibling_detail.competing_variants,
        vec!["blk:nested-child-b".to_string()]
    );
    let patch_value = load_stored_object_value(&store_root, &summary.patch_id)
        .expect("generated merge patch should be stored");
    let patch = parse_patch_object(&patch_value).expect("generated patch should parse");
    assert!(patch.ops.iter().any(|op| matches!(
        op,
        PatchOperation::MoveBlock { block_id, parent_block_id: Some(parent_block_id), after_block_id: Some(after_block_id) }
        if block_id == "blk:nested-child-a"
            && parent_block_id == "blk:nested-parent"
            && after_block_id == "blk:nested-child-c"
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
fn merge_authoring_marks_nested_sibling_choice_through_inserted_sibling_as_multi_variant() {
    let store_root = temp_dir("merge-nested-sibling-manual");
    let signing_key = signing_key();
    let document = create_document_in_store(
        &store_root,
        &signing_key,
        &DocumentCreateParams {
            doc_id: "doc:merge-nested-sibling-manual".to_string(),
            title: "Merge Nested Sibling Manual".to_string(),
            language: "en".to_string(),
            timestamp: 74,
        },
    )
    .expect("document should be created");

    let base_revision_id = commit_ops_revision(
        &store_root,
        &signing_key,
        "doc:merge-nested-sibling-manual",
        &document.genesis_revision_id,
        75,
        76,
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
        ]),
    );

    let insert_revision_id = commit_ops_revision(
        &store_root,
        &signing_key,
        "doc:merge-nested-sibling-manual",
        &base_revision_id,
        77,
        78,
        json!([
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
        ]),
    );

    let moved_revision_id = commit_ops_revision(
        &store_root,
        &signing_key,
        "doc:merge-nested-sibling-manual",
        &base_revision_id,
        79,
        80,
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
            doc_id: "doc:merge-nested-sibling-manual".to_string(),
            parents: vec![base_revision_id, insert_revision_id, moved_revision_id],
            resolved_state: crate::replay::DocumentState {
                doc_id: "doc:merge-nested-sibling-manual".to_string(),
                blocks: vec![paragraph_block_with_children(
                    "blk:nested-parent",
                    "Parent",
                    vec![
                        paragraph_block("blk:nested-child-b", "Child B"),
                        paragraph_block("blk:nested-child-d", "Child D"),
                        paragraph_block("blk:nested-child-a", "Child A"),
                        paragraph_block("blk:nested-child-c", "Child C"),
                    ],
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
            .any(|reason| reason.contains("selected a non-primary sibling placement")),
        "expected nested sibling multi-variant reason, got {summary:?}"
    );
    let patch_value = load_stored_object_value(&store_root, &summary.patch_id)
        .expect("generated merge patch should be stored");
    let patch = parse_patch_object(&patch_value).expect("generated patch should parse");
    assert!(patch.ops.iter().any(|op| matches!(
        op,
        PatchOperation::InsertBlockAfter { after_block_id, new_block }
        if after_block_id == "blk:nested-child-b" && new_block.block_id == "blk:nested-child-d"
    )));
    assert!(patch.ops.iter().any(|op| matches!(
        op,
        PatchOperation::MoveBlock { block_id, parent_block_id: Some(parent_block_id), after_block_id: Some(after_block_id) }
        if block_id == "blk:nested-child-a"
            && parent_block_id == "blk:nested-parent"
            && after_block_id == "blk:nested-child-d"
    )));

    let _ = fs::remove_dir_all(store_root);
}

#[test]
fn merge_authoring_marks_nested_sibling_choice_through_inserted_sibling_chain_as_multi_variant() {
    let store_root = temp_dir("merge-nested-sibling-chain");
    let signing_key = signing_key();
    let document = create_document_in_store(
        &store_root,
        &signing_key,
        &DocumentCreateParams {
            doc_id: "doc:merge-nested-sibling-chain".to_string(),
            title: "Merge Nested Sibling Chain".to_string(),
            language: "en".to_string(),
            timestamp: 82,
        },
    )
    .expect("document should be created");

    let base_revision_id = commit_ops_revision(
        &store_root,
        &signing_key,
        "doc:merge-nested-sibling-chain",
        &document.genesis_revision_id,
        83,
        84,
        json!([
            {
                "op": "insert_block",
                "new_block": {
                    "block_id": "blk:chain-parent",
                    "block_type": "paragraph",
                    "content": "Parent",
                    "attrs": {},
                    "children": [
                        {
                            "block_id": "blk:chain-child-a",
                            "block_type": "paragraph",
                            "content": "Child A",
                            "attrs": {},
                            "children": []
                        },
                        {
                            "block_id": "blk:chain-child-b",
                            "block_type": "paragraph",
                            "content": "Child B",
                            "attrs": {},
                            "children": []
                        },
                        {
                            "block_id": "blk:chain-child-c",
                            "block_type": "paragraph",
                            "content": "Child C",
                            "attrs": {},
                            "children": []
                        }
                    ]
                }
            }
        ]),
    );

    let insert_d_revision_id = commit_ops_revision(
        &store_root,
        &signing_key,
        "doc:merge-nested-sibling-chain",
        &base_revision_id,
        85,
        86,
        json!([
            {
                "op": "insert_block_after",
                "after_block_id": "blk:chain-child-b",
                "new_block": {
                    "block_id": "blk:chain-child-d",
                    "block_type": "paragraph",
                    "content": "Child D",
                    "attrs": {},
                    "children": []
                }
            }
        ]),
    );

    let insert_e_revision_id = commit_ops_revision(
        &store_root,
        &signing_key,
        "doc:merge-nested-sibling-chain",
        &insert_d_revision_id,
        87,
        88,
        json!([
            {
                "op": "insert_block_after",
                "after_block_id": "blk:chain-child-d",
                "new_block": {
                    "block_id": "blk:chain-child-e",
                    "block_type": "paragraph",
                    "content": "Child E",
                    "attrs": {},
                    "children": []
                }
            }
        ]),
    );

    let moved_revision_id = commit_ops_revision(
        &store_root,
        &signing_key,
        "doc:merge-nested-sibling-chain",
        &base_revision_id,
        89,
        90,
        json!([
            {
                "op": "move_block",
                "block_id": "blk:chain-child-a",
                "parent_block_id": "blk:chain-parent",
                "after_block_id": "blk:chain-child-b"
            }
        ]),
    );

    let moved_after_c_revision_id = commit_ops_revision(
        &store_root,
        &signing_key,
        "doc:merge-nested-sibling-chain",
        &base_revision_id,
        91,
        92,
        json!([
            {
                "op": "move_block",
                "block_id": "blk:chain-child-a",
                "parent_block_id": "blk:chain-parent",
                "after_block_id": "blk:chain-child-c"
            }
        ]),
    );

    let summary = create_merge_revision_in_store(
        &store_root,
        &signing_key,
        &MergeRevisionCreateParams {
            doc_id: "doc:merge-nested-sibling-chain".to_string(),
            parents: vec![
                base_revision_id,
                insert_d_revision_id,
                insert_e_revision_id,
                moved_revision_id,
                moved_after_c_revision_id,
            ],
            resolved_state: crate::replay::DocumentState {
                doc_id: "doc:merge-nested-sibling-chain".to_string(),
                blocks: vec![paragraph_block_with_children(
                    "blk:chain-parent",
                    "Parent",
                    vec![
                        paragraph_block("blk:chain-child-b", "Child B"),
                        paragraph_block("blk:chain-child-d", "Child D"),
                        paragraph_block("blk:chain-child-e", "Child E"),
                        paragraph_block("blk:chain-child-a", "Child A"),
                        paragraph_block("blk:chain-child-c", "Child C"),
                    ],
                )],
                metadata: serde_json::Map::new(),
            },
            merge_strategy: "semantic-block-merge".to_string(),
            timestamp: 91,
        },
    )
    .expect("merge revision should be created");

    assert_eq!(summary.merge_outcome, MergeOutcome::MultiVariant);
    assert!(
        summary
            .merge_reasons
            .iter()
            .any(|reason| reason.contains("selected a non-primary sibling placement")),
        "expected nested sibling chain multi-variant reason, got {summary:?}"
    );
    assert!(
        summary
            .merge_reasons
            .iter()
            .any(|reason| reason.contains("multiple competing sibling placements")),
        "expected competing nested sibling chain reason, got {summary:?}"
    );
    let patch_value = load_stored_object_value(&store_root, &summary.patch_id)
        .expect("generated merge patch should be stored");
    let patch = parse_patch_object(&patch_value).expect("generated patch should parse");
    assert!(patch.ops.iter().any(|op| matches!(
        op,
        PatchOperation::InsertBlockAfter { after_block_id, new_block }
        if after_block_id == "blk:chain-child-b" && new_block.block_id == "blk:chain-child-d"
    )));
    assert!(patch.ops.iter().any(|op| matches!(
        op,
        PatchOperation::InsertBlockAfter { after_block_id, new_block }
        if after_block_id == "blk:chain-child-d" && new_block.block_id == "blk:chain-child-e"
    )));
    assert!(patch.ops.iter().any(|op| matches!(
        op,
        PatchOperation::MoveBlock { block_id, parent_block_id: Some(parent_block_id), after_block_id: Some(after_block_id) }
        if block_id == "blk:chain-child-a"
            && parent_block_id == "blk:chain-parent"
            && after_block_id == "blk:chain-child-e"
    )));

    let _ = fs::remove_dir_all(store_root);
}

#[test]
fn merge_authoring_supports_nested_leading_insert_without_manual_curation() {
    let store_root = temp_dir("merge-nested-leading-insert");
    let signing_key = signing_key();
    let document = create_document_in_store(
        &store_root,
        &signing_key,
        &DocumentCreateParams {
            doc_id: "doc:merge-nested-leading-insert".to_string(),
            title: "Merge Nested Leading Insert".to_string(),
            language: "en".to_string(),
            timestamp: 82,
        },
    )
    .expect("document should be created");

    let base_revision_id = commit_ops_revision(
        &store_root,
        &signing_key,
        "doc:merge-nested-leading-insert",
        &document.genesis_revision_id,
        83,
        84,
        json!([
            {
                "op": "insert_block",
                "new_block": {
                    "block_id": "blk:leading-parent",
                    "block_type": "paragraph",
                    "content": "Parent",
                    "attrs": {},
                    "children": [
                        {
                            "block_id": "blk:leading-child-a",
                            "block_type": "paragraph",
                            "content": "Child A",
                            "attrs": {},
                            "children": []
                        },
                        {
                            "block_id": "blk:leading-child-b",
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

    let insert_revision_id = commit_ops_revision(
        &store_root,
        &signing_key,
        "doc:merge-nested-leading-insert",
        &base_revision_id,
        85,
        86,
        json!([
            {
                "op": "insert_block",
                "parent_block_id": "blk:leading-parent",
                "index": 0,
                "new_block": {
                    "block_id": "blk:leading-child-new",
                    "block_type": "paragraph",
                    "content": "Child New",
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
            doc_id: "doc:merge-nested-leading-insert".to_string(),
            parents: vec![base_revision_id, insert_revision_id],
            resolved_state: crate::replay::DocumentState {
                doc_id: "doc:merge-nested-leading-insert".to_string(),
                blocks: vec![paragraph_block_with_children(
                    "blk:leading-parent",
                    "Parent",
                    vec![
                        paragraph_block("blk:leading-child-new", "Child New"),
                        paragraph_block("blk:leading-child-a", "Child A"),
                        paragraph_block("blk:leading-child-b", "Child B"),
                    ],
                )],
                metadata: serde_json::Map::new(),
            },
            merge_strategy: "semantic-block-merge".to_string(),
            timestamp: 87,
        },
    )
    .expect("merge revision should be created");

    assert_eq!(summary.merge_outcome, MergeOutcome::MultiVariant);
    assert!(
        summary
            .merge_reasons
            .iter()
            .any(|reason| reason.contains("selected a non-primary sibling placement")),
        "expected leading-insert sibling multi-variant reason, got {summary:?}"
    );
    let patch_value = load_stored_object_value(&store_root, &summary.patch_id)
        .expect("generated merge patch should be stored");
    let patch = parse_patch_object(&patch_value).expect("generated patch should parse");
    assert_eq!(patch.ops.len(), 1);
    assert!(matches!(
        &patch.ops[0],
        PatchOperation::InsertBlock { parent_block_id: Some(parent_block_id), index: Some(0), new_block }
        if parent_block_id == "blk:leading-parent" && new_block.block_id == "blk:leading-child-new"
    ));

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
fn merge_authoring_marks_deep_composed_branch_reuse_as_multi_variant() {
    let store_root = temp_dir("merge-composed-manual");
    let signing_key = signing_key();
    let document = create_document_in_store(
        &store_root,
        &signing_key,
        &DocumentCreateParams {
            doc_id: "doc:merge-composed-manual".to_string(),
            title: "Merge Composed Manual".to_string(),
            language: "en".to_string(),
            timestamp: 82,
        },
    )
    .expect("document should be created");

    let base_revision_id = commit_ops_revision(
        &store_root,
        &signing_key,
        "doc:merge-composed-manual",
        &document.genesis_revision_id,
        83,
        84,
        json!([
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
        ]),
    );

    let deleted_revision_id = commit_ops_revision(
        &store_root,
        &signing_key,
        "doc:merge-composed-manual",
        &base_revision_id,
        85,
        86,
        json!([
            {
                "op": "delete_block",
                "block_id": "blk:cmp-old-parent"
            }
        ]),
    );

    let inserted_revision_id = commit_ops_revision(
        &store_root,
        &signing_key,
        "doc:merge-composed-manual",
        &base_revision_id,
        87,
        88,
        json!([
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
        ]),
    );

    let summary = create_merge_revision_in_store(
        &store_root,
        &signing_key,
        &MergeRevisionCreateParams {
            doc_id: "doc:merge-composed-manual".to_string(),
            parents: vec![base_revision_id, deleted_revision_id, inserted_revision_id],
            resolved_state: crate::replay::DocumentState {
                doc_id: "doc:merge-composed-manual".to_string(),
                blocks: vec![
                    paragraph_block("blk:cmp-anchor", "Anchor"),
                    paragraph_block_with_children(
                        "blk:cmp-wrapper",
                        "Wrapper",
                        vec![paragraph_block_with_children(
                            "blk:cmp-section",
                            "Section",
                            vec![paragraph_block_with_children(
                                "blk:cmp-subsection",
                                "Subsection",
                                vec![
                                    paragraph_block("blk:cmp-leaf-a", "Leaf A"),
                                    paragraph_block("blk:cmp-leaf-b", "Leaf B"),
                                ],
                            )],
                        )],
                    ),
                ],
                metadata: serde_json::Map::new(),
            },
            merge_strategy: "semantic-block-merge".to_string(),
            timestamp: 89,
        },
    )
    .expect("merge revision should be created");

    assert_eq!(summary.merge_outcome, MergeOutcome::MultiVariant);
    assert!(
        summary
            .merge_reasons
            .iter()
            .any(|reason| reason.contains("selected a non-primary parent placement")),
        "expected composed branch multi-variant reason, got {summary:?}"
    );
    let patch_value = load_stored_object_value(&store_root, &summary.patch_id)
        .expect("generated merge patch should be stored");
    let patch = parse_patch_object(&patch_value).expect("generated patch should parse");
    assert_eq!(patch.ops.len(), 4);
    assert!(patch.ops.iter().any(|op| matches!(
        op,
        PatchOperation::InsertBlockAfter { after_block_id, new_block }
        if after_block_id == "blk:cmp-anchor" && new_block.block_id == "blk:cmp-wrapper"
    )));
    assert!(patch.ops.iter().any(|op| matches!(
        op,
        PatchOperation::MoveBlock { block_id, parent_block_id: Some(parent_block_id), after_block_id: None }
        if block_id == "blk:cmp-leaf-a"
            && parent_block_id == "blk:cmp-subsection"
    )));
    assert!(patch.ops.iter().any(|op| matches!(
        op,
        PatchOperation::MoveBlock { block_id, parent_block_id: Some(parent_block_id), after_block_id: Some(after_block_id) }
        if block_id == "blk:cmp-leaf-b"
            && parent_block_id == "blk:cmp-subsection"
            && after_block_id == "blk:cmp-leaf-a"
    )));
    assert!(patch.ops.iter().any(|op| matches!(
        op,
        PatchOperation::DeleteBlock { block_id }
        if block_id == "blk:cmp-old-parent"
    )));

    let _ = fs::remove_dir_all(store_root);
}

#[test]
fn merge_authoring_supports_multiple_existing_blocks_in_anchored_composed_branch() {
    let store_root = temp_dir("merge-anchored-composed-branch-multi");
    let signing_key = signing_key();
    let document = create_document_in_store(
        &store_root,
        &signing_key,
        &DocumentCreateParams {
            doc_id: "doc:merge-anchored-composed-branch-multi".to_string(),
            title: "Merge Anchored Composed Branch Multi".to_string(),
            language: "en".to_string(),
            timestamp: 82,
        },
    )
    .expect("document should be created");

    let base_revision_id = commit_ops_revision(
        &store_root,
        &signing_key,
        "doc:merge-anchored-composed-branch-multi",
        &document.genesis_revision_id,
        83,
        84,
        json!([
            {
                "op": "insert_block",
                "new_block": {
                    "block_id": "blk:anchored-left",
                    "block_type": "paragraph",
                    "content": "Left",
                    "attrs": {},
                    "children": []
                }
            },
            {
                "op": "insert_block",
                "new_block": {
                    "block_id": "blk:anchored-leaf-a",
                    "block_type": "paragraph",
                    "content": "Leaf A",
                    "attrs": {},
                    "children": []
                }
            },
            {
                "op": "insert_block",
                "new_block": {
                    "block_id": "blk:anchored-leaf-b",
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
        "doc:merge-anchored-composed-branch-multi",
        &base_revision_id,
        85,
        86,
        json!([
            {
                "op": "insert_block",
                "new_block": {
                    "block_id": "blk:anchored-wrapper",
                    "block_type": "paragraph",
                    "content": "Wrapper",
                    "attrs": {},
                    "children": [
                        {
                            "block_id": "blk:anchored-section",
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

    let moved_a_revision_id = commit_ops_revision(
        &store_root,
        &signing_key,
        "doc:merge-anchored-composed-branch-multi",
        &base_revision_id,
        87,
        88,
        json!([
            {
                "op": "move_block",
                "block_id": "blk:anchored-leaf-a",
                "parent_block_id": "blk:anchored-left"
            }
        ]),
    );

    let moved_b_revision_id = commit_ops_revision(
        &store_root,
        &signing_key,
        "doc:merge-anchored-composed-branch-multi",
        &moved_a_revision_id,
        89,
        90,
        json!([
            {
                "op": "move_block",
                "block_id": "blk:anchored-leaf-b",
                "parent_block_id": "blk:anchored-left",
                "after_block_id": "blk:anchored-leaf-a"
            }
        ]),
    );

    let summary = create_merge_revision_in_store(
        &store_root,
        &signing_key,
        &MergeRevisionCreateParams {
            doc_id: "doc:merge-anchored-composed-branch-multi".to_string(),
            parents: vec![base_revision_id, wrapper_revision_id, moved_b_revision_id],
            resolved_state: crate::replay::DocumentState {
                doc_id: "doc:merge-anchored-composed-branch-multi".to_string(),
                blocks: vec![paragraph_block_with_children(
                    "blk:anchored-wrapper",
                    "Wrapper",
                    vec![paragraph_block_with_children(
                        "blk:anchored-section",
                        "Section",
                        vec![
                            paragraph_block("blk:anchored-left", "Left"),
                            paragraph_block("blk:anchored-leaf-a", "Leaf A"),
                            paragraph_block("blk:anchored-leaf-b", "Leaf B"),
                        ],
                    )],
                )],
                metadata: serde_json::Map::new(),
            },
            merge_strategy: "semantic-block-merge".to_string(),
            timestamp: 91,
        },
    )
    .expect("merge revision should be created");

    assert_eq!(summary.merge_outcome, MergeOutcome::MultiVariant);
    assert!(
        summary
            .merge_reasons
            .iter()
            .any(|reason| reason.contains("selected a non-primary parent placement")),
        "expected anchored composed branch multi-variant reason, got {summary:?}"
    );
    assert_eq!(summary.patch_op_count, 4);
    let patch_value = load_stored_object_value(&store_root, &summary.patch_id)
        .expect("generated merge patch should be stored");
    let patch = parse_patch_object(&patch_value).expect("generated patch should parse");
    assert!(patch.ops.iter().any(|op| matches!(
        op,
        PatchOperation::InsertBlock { new_block, .. }
        if new_block.block_id == "blk:anchored-wrapper"
            && new_block.children.len() == 1
            && new_block.children[0].block_id == "blk:anchored-section"
    )));
    assert!(patch.ops.iter().any(|op| matches!(
        op,
        PatchOperation::MoveBlock { block_id, parent_block_id: Some(parent_block_id), after_block_id: None }
        if block_id == "blk:anchored-left" && parent_block_id == "blk:anchored-section"
    )));
    assert!(patch.ops.iter().any(|op| matches!(
        op,
        PatchOperation::MoveBlock { block_id, parent_block_id: Some(parent_block_id), after_block_id: Some(after_block_id) }
        if block_id == "blk:anchored-leaf-a"
            && parent_block_id == "blk:anchored-section"
            && after_block_id == "blk:anchored-left"
    )));
    assert!(patch.ops.iter().any(|op| matches!(
        op,
        PatchOperation::MoveBlock { block_id, parent_block_id: Some(parent_block_id), after_block_id: Some(after_block_id) }
        if block_id == "blk:anchored-leaf-b"
            && parent_block_id == "blk:anchored-section"
            && after_block_id == "blk:anchored-leaf-a"
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
