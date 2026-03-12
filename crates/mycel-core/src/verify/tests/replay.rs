use super::fixtures::*;
use super::*;

#[test]
pub(super) fn revision_replay_verifies_state_hash_from_neighbor_patch() {
    let (signing_key, public_key) = signer_material();
    let dir = write_test_dir("revision-replay-valid");
    let patch_path = dir.join("patch.json");
    let revision_path = dir.join("revision.json");

    let mut patch = json!({
        "type": "patch",
        "version": "mycel/0.1",
        "doc_id": "doc:test",
        "base_revision": "rev:genesis-null",
        "author": public_key,
        "timestamp": 10u64,
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
    });
    let patch_id =
        recompute_object_id(&patch, "patch_id", "patch").expect("patch ID should recompute");
    patch["patch_id"] = Value::String(patch_id.clone());
    patch["signature"] = Value::String(sign_value(&signing_key, &patch));
    std::fs::write(
        &patch_path,
        serde_json::to_string_pretty(&patch).expect("patch should serialize"),
    )
    .expect("patch should write");

    let mut revision = json!({
        "type": "revision",
        "version": "mycel/0.1",
        "doc_id": "doc:test",
        "parents": [],
        "patches": [patch_id],
        "state_hash": state_hash_for_blocks(
            "doc:test",
            vec![BlockObject {
                block_id: "blk:001".to_string(),
                block_type: "paragraph".to_string(),
                content: "Hello".to_string(),
                attrs: Map::new(),
                children: Vec::new()
            }]
        ),
        "author": public_key,
        "timestamp": 11u64
    });
    let revision_id =
        recompute_object_id(&revision, "revision_id", "rev").expect("revision ID should recompute");
    revision["revision_id"] = Value::String(revision_id);
    revision["signature"] = Value::String(sign_value(&signing_key, &revision));
    std::fs::write(
        &revision_path,
        serde_json::to_string_pretty(&revision).expect("revision should serialize"),
    )
    .expect("revision should write");

    let summary = verify_object_path(&revision_path);

    assert!(summary.is_ok(), "expected success, got {summary:?}");
    assert_eq!(summary.state_hash_verification.as_deref(), Some("verified"));
    assert_eq!(
        summary.declared_state_hash.as_deref(),
        summary.recomputed_state_hash.as_deref()
    );

    let _ = std::fs::remove_file(patch_path);
    let _ = std::fs::remove_file(revision_path);
    let _ = std::fs::remove_dir(dir);
}

#[test]
pub(super) fn merge_revision_replay_verifies_state_hash_from_primary_parent_and_patch() {
    let (signing_key, public_key) = signer_material();
    let dir = write_test_dir("revision-replay-merge-valid");
    let base_patch_path = dir.join("patch-base.json");
    let base_revision_path = dir.join("revision-base.json");
    let side_patch_path = dir.join("patch-side.json");
    let side_revision_path = dir.join("revision-side.json");
    let merge_patch_path = dir.join("patch-merge.json");
    let merge_revision_path = dir.join("revision-merge.json");

    let mut base_patch = json!({
        "type": "patch",
        "version": "mycel/0.1",
        "doc_id": "doc:test",
        "base_revision": "rev:genesis-null",
        "author": public_key,
        "timestamp": 10u64,
        "ops": [
            {
                "op": "insert_block",
                "new_block": {
                    "block_id": "blk:001",
                    "block_type": "paragraph",
                    "content": "Base",
                    "attrs": {},
                    "children": []
                }
            }
        ]
    });
    let base_patch_id = recompute_object_id(&base_patch, "patch_id", "patch")
        .expect("base patch ID should recompute");
    base_patch["patch_id"] = Value::String(base_patch_id.clone());
    base_patch["signature"] = Value::String(sign_value(&signing_key, &base_patch));
    std::fs::write(
        &base_patch_path,
        serde_json::to_string_pretty(&base_patch).expect("base patch should serialize"),
    )
    .expect("base patch should write");

    let mut base_revision = json!({
        "type": "revision",
        "version": "mycel/0.1",
        "doc_id": "doc:test",
        "parents": [],
        "patches": [base_patch_id.clone()],
        "state_hash": state_hash_for_blocks(
            "doc:test",
            vec![BlockObject {
                block_id: "blk:001".to_string(),
                block_type: "paragraph".to_string(),
                content: "Base".to_string(),
                attrs: Map::new(),
                children: Vec::new()
            }]
        ),
        "author": public_key,
        "timestamp": 11u64
    });
    let base_revision_id = recompute_object_id(&base_revision, "revision_id", "rev")
        .expect("base revision ID should recompute");
    base_revision["revision_id"] = Value::String(base_revision_id.clone());
    base_revision["signature"] = Value::String(sign_value(&signing_key, &base_revision));
    std::fs::write(
        &base_revision_path,
        serde_json::to_string_pretty(&base_revision).expect("base revision should serialize"),
    )
    .expect("base revision should write");

    let mut side_patch = json!({
        "type": "patch",
        "version": "mycel/0.1",
        "doc_id": "doc:test",
        "base_revision": "rev:genesis-null",
        "author": public_key,
        "timestamp": 12u64,
        "ops": [
            {
                "op": "insert_block",
                "new_block": {
                    "block_id": "blk:002",
                    "block_type": "paragraph",
                    "content": "Side",
                    "attrs": {},
                    "children": []
                }
            }
        ]
    });
    let side_patch_id = recompute_object_id(&side_patch, "patch_id", "patch")
        .expect("side patch ID should recompute");
    side_patch["patch_id"] = Value::String(side_patch_id.clone());
    side_patch["signature"] = Value::String(sign_value(&signing_key, &side_patch));
    std::fs::write(
        &side_patch_path,
        serde_json::to_string_pretty(&side_patch).expect("side patch should serialize"),
    )
    .expect("side patch should write");

    let mut side_revision = json!({
        "type": "revision",
        "version": "mycel/0.1",
        "doc_id": "doc:test",
        "parents": [],
        "patches": [side_patch_id.clone()],
        "state_hash": state_hash_for_blocks(
            "doc:test",
            vec![BlockObject {
                block_id: "blk:002".to_string(),
                block_type: "paragraph".to_string(),
                content: "Side".to_string(),
                attrs: Map::new(),
                children: Vec::new()
            }]
        ),
        "author": public_key,
        "timestamp": 13u64
    });
    let side_revision_id = recompute_object_id(&side_revision, "revision_id", "rev")
        .expect("side revision ID should recompute");
    side_revision["revision_id"] = Value::String(side_revision_id.clone());
    side_revision["signature"] = Value::String(sign_value(&signing_key, &side_revision));
    std::fs::write(
        &side_revision_path,
        serde_json::to_string_pretty(&side_revision).expect("side revision should serialize"),
    )
    .expect("side revision should write");

    let mut merge_patch = json!({
        "type": "patch",
        "version": "mycel/0.1",
        "doc_id": "doc:test",
        "base_revision": base_revision_id.clone(),
        "author": public_key,
        "timestamp": 14u64,
        "ops": [
            {
                "op": "replace_block",
                "block_id": "blk:001",
                "new_content": "Merged"
            }
        ]
    });
    let merge_patch_id = recompute_object_id(&merge_patch, "patch_id", "patch")
        .expect("merge patch ID should recompute");
    merge_patch["patch_id"] = Value::String(merge_patch_id.clone());
    merge_patch["signature"] = Value::String(sign_value(&signing_key, &merge_patch));
    std::fs::write(
        &merge_patch_path,
        serde_json::to_string_pretty(&merge_patch).expect("merge patch should serialize"),
    )
    .expect("merge patch should write");

    let mut merge_revision = json!({
        "type": "revision",
        "version": "mycel/0.1",
        "doc_id": "doc:test",
        "parents": [base_revision_id.clone(), side_revision_id.clone()],
        "patches": [merge_patch_id],
        "merge_strategy": "semantic-block-merge",
        "state_hash": state_hash_for_blocks(
            "doc:test",
            vec![BlockObject {
                block_id: "blk:001".to_string(),
                block_type: "paragraph".to_string(),
                content: "Merged".to_string(),
                attrs: Map::new(),
                children: Vec::new()
            }]
        ),
        "author": public_key,
        "timestamp": 15u64
    });
    let merge_revision_id = recompute_object_id(&merge_revision, "revision_id", "rev")
        .expect("merge revision ID should recompute");
    merge_revision["revision_id"] = Value::String(merge_revision_id);
    merge_revision["signature"] = Value::String(sign_value(&signing_key, &merge_revision));
    std::fs::write(
        &merge_revision_path,
        serde_json::to_string_pretty(&merge_revision).expect("merge revision should serialize"),
    )
    .expect("merge revision should write");

    let summary = verify_object_path(&merge_revision_path);

    assert!(summary.is_ok(), "expected success, got {summary:?}");
    assert_eq!(summary.state_hash_verification.as_deref(), Some("verified"));
    assert_eq!(
        summary.declared_state_hash.as_deref(),
        summary.recomputed_state_hash.as_deref()
    );

    let _ = std::fs::remove_file(base_patch_path);
    let _ = std::fs::remove_file(base_revision_path);
    let _ = std::fs::remove_file(side_patch_path);
    let _ = std::fs::remove_file(side_revision_path);
    let _ = std::fs::remove_file(merge_patch_path);
    let _ = std::fs::remove_file(merge_revision_path);
    let _ = std::fs::remove_dir(dir);
}

#[test]
pub(super) fn merge_revision_does_not_implicitly_include_secondary_parent_content() {
    let (signing_key, public_key) = signer_material();
    let mut base_revision = json!({
        "type": "revision",
        "version": "mycel/0.1",
        "doc_id": "doc:test",
        "parents": [],
        "patches": [],
        "state_hash": state_hash_for_blocks(
            "doc:test",
            vec![BlockObject {
                block_id: "blk:001".to_string(),
                block_type: "paragraph".to_string(),
                content: "Base".to_string(),
                attrs: Map::new(),
                children: Vec::new()
            }]
        ),
        "author": public_key,
        "timestamp": 9u64
    });
    let base_revision_id = recompute_object_id(&base_revision, "revision_id", "rev")
        .expect("base revision ID should recompute");
    base_revision["revision_id"] = Value::String(base_revision_id.clone());
    base_revision["signature"] = Value::String(sign_value(&signing_key, &base_revision));

    let mut side_revision = json!({
        "type": "revision",
        "version": "mycel/0.1",
        "doc_id": "doc:test",
        "parents": [],
        "patches": [],
        "state_hash": state_hash_for_blocks(
            "doc:test",
            vec![BlockObject {
                block_id: "blk:002".to_string(),
                block_type: "paragraph".to_string(),
                content: "Side".to_string(),
                attrs: Map::new(),
                children: Vec::new()
            }]
        ),
        "author": public_key,
        "timestamp": 10u64
    });
    let side_revision_id = recompute_object_id(&side_revision, "revision_id", "rev")
        .expect("side revision ID should recompute");
    side_revision["revision_id"] = Value::String(side_revision_id.clone());
    side_revision["signature"] = Value::String(sign_value(&signing_key, &side_revision));

    let mut merge_revision = json!({
        "type": "revision",
        "version": "mycel/0.1",
        "doc_id": "doc:test",
        "parents": [base_revision_id.clone(), side_revision_id.clone()],
        "patches": [],
        "merge_strategy": "semantic-block-merge",
        "state_hash": state_hash_for_blocks(
            "doc:test",
            vec![
                BlockObject {
                    block_id: "blk:001".to_string(),
                    block_type: "paragraph".to_string(),
                    content: "Base".to_string(),
                    attrs: Map::new(),
                    children: Vec::new()
                },
                BlockObject {
                    block_id: "blk:002".to_string(),
                    block_type: "paragraph".to_string(),
                    content: "Side".to_string(),
                    attrs: Map::new(),
                    children: Vec::new()
                }
            ]
        ),
        "author": public_key,
        "timestamp": 11u64
    });
    let merge_revision_id = recompute_object_id(&merge_revision, "revision_id", "rev")
        .expect("merge revision ID should recompute");
    merge_revision["revision_id"] = Value::String(merge_revision_id);
    merge_revision["signature"] = Value::String(sign_value(&signing_key, &merge_revision));

    let object_index = HashMap::from([
        (base_revision_id, base_revision),
        (side_revision_id, side_revision),
    ]);
    let summary = verify_object_value_with_object_index(&merge_revision, Some(&object_index));

    assert!(!summary.is_ok(), "expected failure, got {summary:?}");
    assert_eq!(summary.state_hash_verification.as_deref(), Some("failed"));
    assert!(
        summary
            .errors
            .iter()
            .any(|message| message
                .contains("declared state_hash does not match replayed state hash")),
        "expected ancestry-only state mismatch error, got {summary:?}"
    );
}

#[test]
pub(super) fn revision_replay_rejects_state_hash_mismatch() {
    let (signing_key, public_key) = signer_material();
    let dir = write_test_dir("revision-replay-mismatch");
    let patch_path = dir.join("patch.json");
    let revision_path = dir.join("revision.json");

    let mut patch = json!({
        "type": "patch",
        "version": "mycel/0.1",
        "doc_id": "doc:test",
        "base_revision": "rev:genesis-null",
        "author": public_key,
        "timestamp": 10u64,
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
    });
    let patch_id =
        recompute_object_id(&patch, "patch_id", "patch").expect("patch ID should recompute");
    patch["patch_id"] = Value::String(patch_id.clone());
    patch["signature"] = Value::String(sign_value(&signing_key, &patch));
    std::fs::write(
        &patch_path,
        serde_json::to_string_pretty(&patch).expect("patch should serialize"),
    )
    .expect("patch should write");

    let mut revision = json!({
        "type": "revision",
        "version": "mycel/0.1",
        "doc_id": "doc:test",
        "parents": [],
        "patches": [patch_id],
        "state_hash": "hash:wrong",
        "author": public_key,
        "timestamp": 11u64
    });
    let revision_id =
        recompute_object_id(&revision, "revision_id", "rev").expect("revision ID should recompute");
    revision["revision_id"] = Value::String(revision_id);
    revision["signature"] = Value::String(sign_value(&signing_key, &revision));
    std::fs::write(
        &revision_path,
        serde_json::to_string_pretty(&revision).expect("revision should serialize"),
    )
    .expect("revision should write");

    let summary = verify_object_path(&revision_path);

    assert!(!summary.is_ok(), "expected failure, got {summary:?}");
    assert_eq!(summary.state_hash_verification.as_deref(), Some("failed"));
    assert!(
        summary
            .errors
            .iter()
            .any(|message| message
                .contains("declared state_hash does not match replayed state hash")),
        "expected state-hash mismatch error, got {summary:?}"
    );

    let _ = std::fs::remove_file(patch_path);
    let _ = std::fs::remove_file(revision_path);
    let _ = std::fs::remove_dir(dir);
}

#[test]
pub(super) fn revision_replay_rejects_genesis_patch_with_non_genesis_base_revision() {
    let (signing_key, public_key) = signer_material();
    let mut patch = json!({
        "type": "patch",
        "version": "mycel/0.1",
        "doc_id": "doc:test",
        "base_revision": "rev:wrong-base",
        "author": public_key,
        "timestamp": 10u64,
        "ops": []
    });
    let patch_id =
        recompute_object_id(&patch, "patch_id", "patch").expect("patch ID should recompute");
    patch["patch_id"] = Value::String(patch_id.clone());
    patch["signature"] = Value::String(sign_value(&signing_key, &patch));

    let mut revision = json!({
        "type": "revision",
        "version": "mycel/0.1",
        "doc_id": "doc:test",
        "parents": [],
        "patches": [patch_id.clone()],
        "state_hash": "hash:test",
        "author": public_key,
        "timestamp": 11u64
    });
    let revision_id =
        recompute_object_id(&revision, "revision_id", "rev").expect("revision ID should recompute");
    revision["revision_id"] = Value::String(revision_id);
    revision["signature"] = Value::String(sign_value(&signing_key, &revision));

    let object_index = HashMap::from([(patch_id, patch)]);
    let summary = verify_object_value_with_object_index(&revision, Some(&object_index));

    assert!(!summary.is_ok(), "expected failure, got {summary:?}");
    assert_eq!(summary.state_hash_verification.as_deref(), Some("failed"));
    assert!(
        summary.errors.iter().any(|message| {
            message.contains("base_revision 'rev:wrong-base'")
                && message.contains("expected 'rev:genesis-null'")
        }),
        "expected genesis base_revision mismatch error, got {summary:?}"
    );
}

#[test]
pub(super) fn revision_replay_rejects_non_genesis_patch_with_wrong_parent_base_revision() {
    let (signing_key, public_key) = signer_material();
    let base_hash = compute_state_hash(&DocumentState {
        doc_id: "doc:test".to_string(),
        blocks: Vec::new(),
        metadata: Map::new(),
    })
    .expect("empty state hash should compute");
    let mut base_revision = json!({
        "type": "revision",
        "version": "mycel/0.1",
        "doc_id": "doc:test",
        "parents": [],
        "patches": [],
        "state_hash": base_hash,
        "author": public_key,
        "timestamp": 9u64
    });
    let base_revision_id = recompute_object_id(&base_revision, "revision_id", "rev")
        .expect("base revision ID should recompute");
    base_revision["revision_id"] = Value::String(base_revision_id.clone());
    base_revision["signature"] = Value::String(sign_value(&signing_key, &base_revision));

    let mut patch = json!({
        "type": "patch",
        "version": "mycel/0.1",
        "doc_id": "doc:test",
        "base_revision": "rev:wrong-base",
        "author": public_key,
        "timestamp": 10u64,
        "ops": []
    });
    let patch_id =
        recompute_object_id(&patch, "patch_id", "patch").expect("patch ID should recompute");
    patch["patch_id"] = Value::String(patch_id.clone());
    patch["signature"] = Value::String(sign_value(&signing_key, &patch));

    let mut revision = json!({
        "type": "revision",
        "version": "mycel/0.1",
        "doc_id": "doc:test",
        "parents": [base_revision_id.clone()],
        "patches": [patch_id.clone()],
        "state_hash": compute_state_hash(&DocumentState {
            doc_id: "doc:test".to_string(),
            blocks: Vec::new(),
            metadata: Map::new(),
        })
        .expect("empty state hash should compute"),
        "author": public_key,
        "timestamp": 11u64
    });
    let revision_id =
        recompute_object_id(&revision, "revision_id", "rev").expect("revision ID should recompute");
    revision["revision_id"] = Value::String(revision_id);
    revision["signature"] = Value::String(sign_value(&signing_key, &revision));

    let object_index = HashMap::from([(base_revision_id, base_revision), (patch_id, patch)]);
    let summary = verify_object_value_with_object_index(&revision, Some(&object_index));

    assert!(!summary.is_ok(), "expected failure, got {summary:?}");
    assert_eq!(summary.state_hash_verification.as_deref(), Some("failed"));
    assert!(
        summary.errors.iter().any(|message| {
            message.contains("base_revision 'rev:wrong-base'")
                && message.contains("does not match expected 'rev:")
        }),
        "expected non-genesis base_revision mismatch error, got {summary:?}"
    );
}

#[test]
pub(super) fn merge_revision_replay_rejects_swapped_parent_order() {
    let (signing_key, public_key) = signer_material();
    let mut base_revision = json!({
        "type": "revision",
        "version": "mycel/0.1",
        "doc_id": "doc:test",
        "parents": [],
        "patches": [],
        "state_hash": state_hash_for_blocks(
            "doc:test",
            vec![BlockObject {
                block_id: "blk:001".to_string(),
                block_type: "paragraph".to_string(),
                content: "Base".to_string(),
                attrs: Map::new(),
                children: Vec::new()
            }]
        ),
        "author": public_key,
        "timestamp": 9u64
    });
    let base_revision_id = recompute_object_id(&base_revision, "revision_id", "rev")
        .expect("base revision ID should recompute");
    base_revision["revision_id"] = Value::String(base_revision_id.clone());
    base_revision["signature"] = Value::String(sign_value(&signing_key, &base_revision));

    let mut side_revision = json!({
        "type": "revision",
        "version": "mycel/0.1",
        "doc_id": "doc:test",
        "parents": [],
        "patches": [],
        "state_hash": state_hash_for_blocks(
            "doc:test",
            vec![BlockObject {
                block_id: "blk:002".to_string(),
                block_type: "paragraph".to_string(),
                content: "Side".to_string(),
                attrs: Map::new(),
                children: Vec::new()
            }]
        ),
        "author": public_key,
        "timestamp": 10u64
    });
    let side_revision_id = recompute_object_id(&side_revision, "revision_id", "rev")
        .expect("side revision ID should recompute");
    side_revision["revision_id"] = Value::String(side_revision_id.clone());
    side_revision["signature"] = Value::String(sign_value(&signing_key, &side_revision));

    let mut merge_patch = json!({
        "type": "patch",
        "version": "mycel/0.1",
        "doc_id": "doc:test",
        "base_revision": base_revision_id.clone(),
        "author": public_key,
        "timestamp": 11u64,
        "ops": []
    });
    let merge_patch_id = recompute_object_id(&merge_patch, "patch_id", "patch")
        .expect("merge patch ID should recompute");
    merge_patch["patch_id"] = Value::String(merge_patch_id.clone());
    merge_patch["signature"] = Value::String(sign_value(&signing_key, &merge_patch));

    let mut merge_revision = json!({
        "type": "revision",
        "version": "mycel/0.1",
        "doc_id": "doc:test",
        "parents": [side_revision_id.clone(), base_revision_id.clone()],
        "patches": [merge_patch_id.clone()],
        "merge_strategy": "semantic-block-merge",
        "state_hash": state_hash_for_blocks(
            "doc:test",
            vec![BlockObject {
                block_id: "blk:002".to_string(),
                block_type: "paragraph".to_string(),
                content: "Side".to_string(),
                attrs: Map::new(),
                children: Vec::new()
            }]
        ),
        "author": public_key,
        "timestamp": 12u64
    });
    let merge_revision_id = recompute_object_id(&merge_revision, "revision_id", "rev")
        .expect("merge revision ID should recompute");
    merge_revision["revision_id"] = Value::String(merge_revision_id);
    merge_revision["signature"] = Value::String(sign_value(&signing_key, &merge_revision));

    let object_index = HashMap::from([
        (base_revision_id.clone(), base_revision),
        (side_revision_id.clone(), side_revision),
        (merge_patch_id.clone(), merge_patch),
    ]);
    let summary = verify_object_value_with_object_index(&merge_revision, Some(&object_index));

    assert!(!summary.is_ok(), "expected failure, got {summary:?}");
    assert_eq!(summary.state_hash_verification.as_deref(), Some("failed"));
    assert!(
        summary.errors.iter().any(|message| {
            message.contains(&format!("patch '{merge_patch_id}'"))
                && message.contains(&format!("base_revision '{base_revision_id}'"))
                && message.contains(&format!("expected '{side_revision_id}'"))
        }),
        "expected swapped-parent-order replay error, got {summary:?}"
    );
}

#[test]
pub(super) fn revision_replay_rejects_patch_from_other_document() {
    let (signing_key, public_key) = signer_material();
    let mut patch = json!({
        "type": "patch",
        "version": "mycel/0.1",
        "doc_id": "doc:other",
        "base_revision": "rev:genesis-null",
        "author": public_key,
        "timestamp": 10u64,
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
    });
    let patch_id =
        recompute_object_id(&patch, "patch_id", "patch").expect("patch ID should recompute");
    patch["patch_id"] = Value::String(patch_id.clone());
    patch["signature"] = Value::String(sign_value(&signing_key, &patch));

    let mut revision = json!({
        "type": "revision",
        "version": "mycel/0.1",
        "doc_id": "doc:test",
        "parents": [],
        "patches": [patch_id.clone()],
        "state_hash": "hash:wrong",
        "author": public_key,
        "timestamp": 11u64
    });
    let revision_id =
        recompute_object_id(&revision, "revision_id", "rev").expect("revision ID should recompute");
    revision["revision_id"] = Value::String(revision_id);
    revision["signature"] = Value::String(sign_value(&signing_key, &revision));

    let object_index = HashMap::from([(patch_id, patch)]);
    let summary = verify_object_value_with_object_index(&revision, Some(&object_index));

    assert!(!summary.is_ok(), "expected failure, got {summary:?}");
    assert_eq!(summary.state_hash_verification.as_deref(), Some("failed"));
    assert!(
        summary.errors.iter().any(|message| {
            message.contains("patch '")
                && message.contains("belongs to 'doc:other' instead of 'doc:test'")
        }),
        "expected cross-document replay error, got {summary:?}"
    );
}

#[test]
pub(super) fn revision_replay_rejects_patch_dependency_that_fails_standalone_verification() {
    let (signing_key, public_key) = signer_material();
    let mut patch = json!({
        "type": "patch",
        "version": "mycel/0.1",
        "doc_id": "doc:test",
        "base_revision": "rev:genesis-null",
        "author": public_key,
        "timestamp": 10u64,
        "ops": []
    });
    let invalid_patch_id = "patch:wrong".to_string();
    patch["patch_id"] = Value::String(invalid_patch_id.clone());
    patch["signature"] = Value::String(sign_value(&signing_key, &patch));

    let mut revision = json!({
        "type": "revision",
        "version": "mycel/0.1",
        "doc_id": "doc:test",
        "parents": [],
        "patches": [invalid_patch_id.clone()],
        "state_hash": state_hash_for_blocks("doc:test", Vec::new()),
        "author": public_key,
        "timestamp": 11u64
    });
    let revision_id =
        recompute_object_id(&revision, "revision_id", "rev").expect("revision ID should recompute");
    revision["revision_id"] = Value::String(revision_id);
    revision["signature"] = Value::String(sign_value(&signing_key, &revision));

    let object_index = HashMap::from([(invalid_patch_id.clone(), patch)]);
    let summary = verify_object_value_with_object_index(&revision, Some(&object_index));

    assert!(!summary.is_ok(), "expected failure, got {summary:?}");
    assert_eq!(summary.state_hash_verification.as_deref(), Some("failed"));
    assert!(
        summary.errors.iter().any(|message| {
            message.contains(&format!("failed to verify patch '{invalid_patch_id}'"))
                && message
                    .contains("declared patch_id does not match recomputed canonical object ID")
        }),
        "expected dependency verification error, got {summary:?}"
    );
}
