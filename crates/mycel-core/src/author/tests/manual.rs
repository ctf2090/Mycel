use super::*;

#[test]
fn merge_authoring_requires_manual_curation_for_metadata_removal() {
    let store_root = temp_dir("merge-metadata-removal");
    let signing_key = signing_key();
    let document = create_document_in_store(
        &store_root,
        &signing_key,
        &DocumentCreateParams {
            doc_id: "doc:merge-metadata-removal".to_string(),
            title: "Merge Metadata Removal".to_string(),
            language: "en".to_string(),
            timestamp: 20,
        },
    )
    .expect("document should be created");

    let base_revision_id = commit_ops_revision(
        &store_root,
        &signing_key,
        "doc:merge-metadata-removal",
        &document.genesis_revision_id,
        21,
        22,
        json!([
            {
                "op": "set_metadata",
                "metadata": {
                    "topic": "base"
                }
            }
        ]),
    );

    let right_revision_id = commit_ops_revision(
        &store_root,
        &signing_key,
        "doc:merge-metadata-removal",
        &base_revision_id,
        23,
        24,
        json!([
            {
                "op": "set_metadata",
                "metadata": {
                    "topic": "right"
                }
            }
        ]),
    );

    let error = create_merge_revision_in_store(
        &store_root,
        &signing_key,
        &MergeRevisionCreateParams {
            doc_id: "doc:merge-metadata-removal".to_string(),
            parents: vec![base_revision_id, right_revision_id],
            resolved_state: crate::replay::DocumentState {
                doc_id: "doc:merge-metadata-removal".to_string(),
                blocks: Vec::new(),
                metadata: serde_json::Map::new(),
            },
            merge_strategy: "semantic-block-merge".to_string(),
            timestamp: 25,
        },
    )
    .expect_err("metadata removal should require manual curation");

    assert!(
        error.to_string().contains(
            "manual-curation-required: resolved metadata key 'topic' removes primary metadata but v0.1 patch ops cannot express metadata deletion"
        ),
        "expected metadata removal manual-curation error, got {error}"
    );

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
