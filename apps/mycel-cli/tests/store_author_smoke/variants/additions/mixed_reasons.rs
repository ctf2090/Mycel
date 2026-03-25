use super::*;

#[test]
fn store_merge_authoring_flow_preserves_distinct_reasons_for_mixed_metadata_keys() {
    let doc_id = "doc:author-smoke-metadata-mixed-keys";
    let flow = VariantScenarioFlow::new(
        "store-merge-metadata-mixed-keys-root",
        "store-merge-metadata-mixed-keys-key",
        doc_id,
        "Author Smoke Metadata Mixed Keys",
        "40",
    );
    let (_resolved_dir, resolved_state_path) = write_metadata_entries_resolved_state_for_doc_file(
        "store-merge-metadata-mixed-keys-state",
        doc_id,
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
    let resolved_state_file = path_arg(&resolved_state_path);
    let topic_ops_file = path_arg(&topic_ops_path);
    let priority_ops_file = path_arg(&priority_ops_path);

    let topic_revision_id =
        flow.commit_ops_revision(flow.genesis_revision_id(), &topic_ops_file, "41", "42");
    let priority_revision_id =
        flow.commit_ops_revision(flow.genesis_revision_id(), &priority_ops_file, "43", "44");
    let merge_json = flow.create_merge_revision(
        &[
            flow.genesis_revision_id(),
            &topic_revision_id,
            &priority_revision_id,
        ],
        &resolved_state_file,
        "45",
    );

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
    let doc_id = "doc:author-smoke-content-mixed-blocks";
    let flow = VariantScenarioFlow::new(
        "store-merge-content-mixed-blocks-root",
        "store-merge-content-mixed-blocks-key",
        doc_id,
        "Author Smoke Content Mixed Blocks",
        "60",
    );
    let (_resolved_dir, resolved_state_path) = write_content_entries_resolved_state_for_doc_file(
        "store-merge-content-mixed-blocks-state",
        doc_id,
        &[
            ("blk:author-smoke-topic", "Right"),
            ("blk:author-smoke-priority", "Base"),
        ],
    );
    let (_base_ops_dir, base_ops_path) = write_insert_block_ops_file(
        "store-merge-content-mixed-blocks-base-ops",
        "blk:author-smoke-priority",
        "Base",
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
    let resolved_state_file = path_arg(&resolved_state_path);
    let base_ops_file = path_arg(&base_ops_path);
    let topic_ops_file = path_arg(&topic_ops_path);
    let priority_ops_file = path_arg(&priority_ops_path);

    let base_revision_id =
        flow.commit_ops_revision(flow.genesis_revision_id(), &base_ops_file, "61", "62");
    let topic_revision_id =
        flow.commit_ops_revision(&base_revision_id, &topic_ops_file, "63", "64");
    let priority_revision_id =
        flow.commit_ops_revision(&base_revision_id, &priority_ops_file, "65", "66");
    let merge_json = flow.create_merge_revision(
        &[&base_revision_id, &topic_revision_id, &priority_revision_id],
        &resolved_state_file,
        "67",
    );

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
