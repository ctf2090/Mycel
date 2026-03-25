use super::*;

#[test]
fn store_merge_authoring_flow_preserves_distinct_reasons_for_mixed_metadata_keys() {
    let store_dir = create_temp_dir("store-merge-metadata-mixed-keys-root");
    let (_key_dir, key_path) = write_signing_key_file("store-merge-metadata-mixed-keys-key");
    let (_resolved_dir, resolved_state_path) = write_metadata_entries_resolved_state_for_doc_file(
        "store-merge-metadata-mixed-keys-state",
        "doc:author-smoke-metadata-mixed-keys",
        &[("topic", "right")],
    );
    let (_topic_ops_dir, topic_ops_path) = write_metadata_entries_ops_file(
        "store-merge-metadata-mixed-keys-topic-ops",
        &[("topic", "right")],
    );
    let (_priority_ops_dir, priority_ops_path) = write_metadata_entries_ops_file(
        "store-merge-metadata-mixed-keys-priority-ops",
        &[("priority", "high")],
    );
    let store_root = path_arg(store_dir.path());
    let key_file = path_arg(&key_path);
    let resolved_state_file = path_arg(&resolved_state_path);
    let topic_ops_file = path_arg(&topic_ops_path);
    let priority_ops_file = path_arg(&priority_ops_path);

    let init = run_mycel(&["store", "init", &store_root, "--json"]);
    assert_success(&init);

    let document = run_mycel(&[
        "store",
        "create-document",
        &store_root,
        "--doc-id",
        "doc:author-smoke-metadata-mixed-keys",
        "--title",
        "Author Smoke Metadata Mixed Keys",
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

    let topic_patch = run_mycel(&[
        "store",
        "create-patch",
        &store_root,
        "--doc-id",
        "doc:author-smoke-metadata-mixed-keys",
        "--base-revision",
        &genesis_revision_id,
        "--ops",
        &topic_ops_file,
        "--signing-key",
        &key_file,
        "--timestamp",
        "41",
        "--json",
    ]);
    assert_success(&topic_patch);
    let topic_patch_json = assert_json_status(&topic_patch, "ok");
    let topic_patch_id = topic_patch_json["patch_id"]
        .as_str()
        .expect("topic patch_id should be string")
        .to_string();

    let topic_revision = run_mycel(&[
        "store",
        "commit-revision",
        &store_root,
        "--doc-id",
        "doc:author-smoke-metadata-mixed-keys",
        "--parent",
        &genesis_revision_id,
        "--patch",
        &topic_patch_id,
        "--signing-key",
        &key_file,
        "--timestamp",
        "42",
        "--json",
    ]);
    assert_success(&topic_revision);
    let topic_revision_json = assert_json_status(&topic_revision, "ok");
    let topic_revision_id = topic_revision_json["revision_id"]
        .as_str()
        .expect("topic revision_id should be string")
        .to_string();

    let priority_patch = run_mycel(&[
        "store",
        "create-patch",
        &store_root,
        "--doc-id",
        "doc:author-smoke-metadata-mixed-keys",
        "--base-revision",
        &genesis_revision_id,
        "--ops",
        &priority_ops_file,
        "--signing-key",
        &key_file,
        "--timestamp",
        "43",
        "--json",
    ]);
    assert_success(&priority_patch);
    let priority_patch_json = assert_json_status(&priority_patch, "ok");
    let priority_patch_id = priority_patch_json["patch_id"]
        .as_str()
        .expect("priority patch_id should be string")
        .to_string();

    let priority_revision = run_mycel(&[
        "store",
        "commit-revision",
        &store_root,
        "--doc-id",
        "doc:author-smoke-metadata-mixed-keys",
        "--parent",
        &genesis_revision_id,
        "--patch",
        &priority_patch_id,
        "--signing-key",
        &key_file,
        "--timestamp",
        "44",
        "--json",
    ]);
    assert_success(&priority_revision);
    let priority_revision_json = assert_json_status(&priority_revision, "ok");
    let priority_revision_id = priority_revision_json["revision_id"]
        .as_str()
        .expect("priority revision_id should be string")
        .to_string();

    let merge = run_mycel(&[
        "store",
        "create-merge-revision",
        &store_root,
        "--doc-id",
        "doc:author-smoke-metadata-mixed-keys",
        "--parent",
        &genesis_revision_id,
        "--parent",
        &topic_revision_id,
        "--parent",
        &priority_revision_id,
        "--resolved-state",
        &resolved_state_file,
        "--signing-key",
        &key_file,
        "--timestamp",
        "45",
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
                    reason.contains("metadata key 'topic' adopted a non-primary parent addition")
                })
            })),
        "expected topic selection reason, got {merge_json}"
    );
    assert!(
        merge_json["merge_reasons"]
            .as_array()
            .is_some_and(|reasons| reasons.iter().any(|reason| {
                reason.as_str().is_some_and(|reason| {
                    reason.contains("metadata key 'priority' kept the primary absence over a competing non-primary addition")
                })
            })),
        "expected priority keep-primary reason, got {merge_json}"
    );
    assert!(
        merge_json["merge_reason_details"]
            .as_array()
            .is_some_and(|details| details.iter().any(|detail| {
                detail["subject_id"] == "topic"
                    && detail["variant_kind"] == "metadata"
                    && detail["reason_kind"] == "selected-non-primary-parent-variant"
                    && detail["branch_kind"] == "adopted-non-primary-addition"
            })),
        "expected topic branch kind detail, got {merge_json}"
    );
    assert!(
        merge_json["merge_reason_details"]
            .as_array()
            .is_some_and(|details| details.iter().any(|detail| {
                detail["subject_id"] == "priority"
                    && detail["variant_kind"] == "metadata"
                    && detail["reason_kind"]
                        == "kept-primary-parent-variant-over-competing-non-primary-alternative"
                    && detail["branch_kind"] == "kept-primary-absence-over-non-primary-addition"
            })),
        "expected priority keep-primary branch kind detail, got {merge_json}"
    );
    assert_eq!(merge_json["patch_op_count"], 1);
}

#[test]
fn store_merge_authoring_flow_preserves_distinct_reasons_for_mixed_content_blocks() {
    let store_dir = create_temp_dir("store-merge-content-mixed-blocks-root");
    let (_key_dir, key_path) = write_signing_key_file("store-merge-content-mixed-blocks-key");
    let (_topic_resolved_dir, resolved_state_path) =
        write_content_entries_resolved_state_for_doc_file(
            "store-merge-content-mixed-blocks-state",
            "doc:author-smoke-content-mixed-blocks",
            &[
                ("blk:author-smoke-topic", "Right"),
                ("blk:author-smoke-priority", "Base"),
            ],
        );
    let (_topic_ops_dir, topic_ops_path) = write_content_addition_ops_for_block_file(
        "store-merge-content-mixed-blocks-topic-ops",
        "blk:author-smoke-topic",
        "Right",
    );
    let (_priority_ops_dir, priority_ops_path) = write_content_variant_ops_for_block_file(
        "store-merge-content-mixed-blocks-priority-ops",
        "blk:author-smoke-priority",
        "High",
    );
    let store_root = path_arg(store_dir.path());
    let key_file = path_arg(&key_path);
    let resolved_state_file = path_arg(&resolved_state_path);
    let topic_ops_file = path_arg(&topic_ops_path);
    let priority_ops_file = path_arg(&priority_ops_path);

    let init = run_mycel(&["store", "init", &store_root, "--json"]);
    assert_success(&init);

    let document = run_mycel(&[
        "store",
        "create-document",
        &store_root,
        "--doc-id",
        "doc:author-smoke-content-mixed-blocks",
        "--title",
        "Author Smoke Content Mixed Blocks",
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

    let base_ops_dir = create_temp_dir("store-merge-content-mixed-blocks-base-ops");
    let base_ops_path = base_ops_dir.path().join("ops.json");
    fs::write(
        &base_ops_path,
        serde_json::to_string_pretty(&json!([
            {
                "op": "insert_block",
                "new_block": {
                    "block_id": "blk:author-smoke-priority",
                    "block_type": "paragraph",
                    "content": "Base",
                    "attrs": {},
                    "children": []
                }
            }
        ]))
        .expect("content mixed base ops JSON should serialize"),
    )
    .expect("content mixed base ops JSON should write");
    let base_ops_file = path_arg(&base_ops_path);

    let base_patch = run_mycel(&[
        "store",
        "create-patch",
        &store_root,
        "--doc-id",
        "doc:author-smoke-content-mixed-blocks",
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
        "doc:author-smoke-content-mixed-blocks",
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

    let topic_patch = run_mycel(&[
        "store",
        "create-patch",
        &store_root,
        "--doc-id",
        "doc:author-smoke-content-mixed-blocks",
        "--base-revision",
        &base_revision_id,
        "--ops",
        &topic_ops_file,
        "--signing-key",
        &key_file,
        "--timestamp",
        "63",
        "--json",
    ]);
    assert_success(&topic_patch);
    let topic_patch_json = assert_json_status(&topic_patch, "ok");
    let topic_patch_id = topic_patch_json["patch_id"]
        .as_str()
        .expect("topic patch_id should be string")
        .to_string();

    let topic_revision = run_mycel(&[
        "store",
        "commit-revision",
        &store_root,
        "--doc-id",
        "doc:author-smoke-content-mixed-blocks",
        "--parent",
        &base_revision_id,
        "--patch",
        &topic_patch_id,
        "--signing-key",
        &key_file,
        "--timestamp",
        "64",
        "--json",
    ]);
    assert_success(&topic_revision);
    let topic_revision_json = assert_json_status(&topic_revision, "ok");
    let topic_revision_id = topic_revision_json["revision_id"]
        .as_str()
        .expect("topic revision_id should be string")
        .to_string();

    let priority_patch = run_mycel(&[
        "store",
        "create-patch",
        &store_root,
        "--doc-id",
        "doc:author-smoke-content-mixed-blocks",
        "--base-revision",
        &base_revision_id,
        "--ops",
        &priority_ops_file,
        "--signing-key",
        &key_file,
        "--timestamp",
        "65",
        "--json",
    ]);
    assert_success(&priority_patch);
    let priority_patch_json = assert_json_status(&priority_patch, "ok");
    let priority_patch_id = priority_patch_json["patch_id"]
        .as_str()
        .expect("priority patch_id should be string")
        .to_string();

    let priority_revision = run_mycel(&[
        "store",
        "commit-revision",
        &store_root,
        "--doc-id",
        "doc:author-smoke-content-mixed-blocks",
        "--parent",
        &base_revision_id,
        "--patch",
        &priority_patch_id,
        "--signing-key",
        &key_file,
        "--timestamp",
        "66",
        "--json",
    ]);
    assert_success(&priority_revision);
    let priority_revision_json = assert_json_status(&priority_revision, "ok");
    let priority_revision_id = priority_revision_json["revision_id"]
        .as_str()
        .expect("priority revision_id should be string")
        .to_string();

    let merge = run_mycel(&[
        "store",
        "create-merge-revision",
        &store_root,
        "--doc-id",
        "doc:author-smoke-content-mixed-blocks",
        "--parent",
        &base_revision_id,
        "--parent",
        &topic_revision_id,
        "--parent",
        &priority_revision_id,
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
                    reason.contains(
                        "block 'blk:author-smoke-topic' adopted a non-primary parent addition",
                    )
                })
            })),
        "expected topic selection reason, got {merge_json}"
    );
    assert!(
        merge_json["merge_reasons"]
            .as_array()
            .is_some_and(|reasons| reasons.iter().any(|reason| {
                reason.as_str().is_some_and(|reason| {
                    reason.contains(
                        "block 'blk:author-smoke-priority' kept the primary parent variant over a competing non-primary replacement",
                    )
                })
            })),
        "expected priority keep-primary reason, got {merge_json}"
    );
    assert!(
        merge_json["merge_reason_details"]
            .as_array()
            .is_some_and(|details| details.iter().any(|detail| {
                detail["subject_id"] == "blk:author-smoke-topic"
                    && detail["variant_kind"] == "content"
                    && detail["reason_kind"] == "selected-non-primary-parent-variant"
                    && detail["branch_kind"] == "adopted-non-primary-addition"
            })),
        "expected topic branch kind detail, got {merge_json}"
    );
    assert!(
        merge_json["merge_reason_details"]
            .as_array()
            .is_some_and(|details| details.iter().any(|detail| {
                detail["subject_id"] == "blk:author-smoke-priority"
                    && detail["variant_kind"] == "content"
                    && detail["reason_kind"]
                        == "kept-primary-parent-variant-over-competing-non-primary-alternative"
                    && detail["branch_kind"] == "kept-primary-variant-over-non-primary-replacement"
            })),
        "expected priority keep-primary branch kind detail, got {merge_json}"
    );
    assert_eq!(merge_json["patch_op_count"], 1);
}
