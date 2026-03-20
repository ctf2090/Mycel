use super::*;

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
