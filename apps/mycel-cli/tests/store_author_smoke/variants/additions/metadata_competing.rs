use super::*;

#[test]
fn store_merge_authoring_flow_reports_selected_metadata_addition_with_competing_additions() {
    let doc_id = "doc:author-smoke-metadata-selected-addition-competing";
    let flow = VariantScenarioFlow::new(
        "store-merge-metadata-selected-addition-competing-root",
        "store-merge-metadata-selected-addition-competing-key",
        doc_id,
        "Author Smoke Metadata Selected Addition Competing",
        "56",
    );
    let (_resolved_dir, resolved_state_path) = write_metadata_variant_resolved_state_for_doc_file(
        "store-merge-metadata-selected-addition-competing-state",
        doc_id,
        "right",
    );
    let (_right_ops_dir, right_ops_path) = write_metadata_variant_ops_file(
        "store-merge-metadata-selected-addition-competing-right-ops",
        "right",
    );
    let (_center_ops_dir, center_ops_path) = write_metadata_variant_ops_file(
        "store-merge-metadata-selected-addition-competing-center-ops",
        "center",
    );
    let resolved_state_file = path_arg(&resolved_state_path);
    let right_ops_file = path_arg(&right_ops_path);
    let center_ops_file = path_arg(&center_ops_path);

    let right_revision_id =
        flow.commit_ops_revision(flow.genesis_revision_id(), &right_ops_file, "57", "58");
    let center_revision_id =
        flow.commit_ops_revision(flow.genesis_revision_id(), &center_ops_file, "59", "60");
    let merge_json = flow.create_merge_revision(
        &[
            flow.genesis_revision_id(),
            &right_revision_id,
            &center_revision_id,
        ],
        &resolved_state_file,
        "61",
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
        "expected selected metadata addition reason, got {merge_json}"
    );
    assert!(
        merge_json["merge_reasons"]
            .as_array()
            .is_some_and(|reasons| reasons.iter().any(|reason| {
                reason.as_str().is_some_and(|reason| {
                    reason.contains(
                        "metadata key 'topic' selected one non-primary addition while other competing non-primary additions remained",
                    )
                })
            })),
        "expected competing metadata addition reason, got {merge_json}"
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
        "expected selected metadata addition detail, got {merge_json}"
    );
    assert!(
        merge_json["merge_reason_details"]
            .as_array()
            .is_some_and(|details| details.iter().any(|detail| {
                detail["subject_id"] == "topic"
                    && detail["variant_kind"] == "metadata"
                    && detail["reason_kind"]
                        == "multiple-competing-alternatives-remain-after-selected-variant"
                    && detail["branch_kind"] == "multiple-competing-non-primary-additions"
                    && detail["competing_variants"]
                        .as_array()
                        .is_some_and(|variants| variants.len() == 2)
            })),
        "expected competing metadata addition detail, got {merge_json}"
    );
    assert_eq!(merge_json["patch_op_count"], 1);
}

#[test]
fn store_merge_authoring_flow_preserves_duplicate_non_primary_metadata_additions() {
    let doc_id = "doc:author-smoke-metadata-duplicate-additions";
    let flow = VariantScenarioFlow::new(
        "store-merge-metadata-duplicate-additions-root",
        "store-merge-metadata-duplicate-additions-key",
        doc_id,
        "Author Smoke Metadata Duplicate Additions",
        "62",
    );
    let (_resolved_dir, resolved_state_path) = write_metadata_entries_resolved_state_for_doc_file(
        "store-merge-metadata-duplicate-additions-state",
        doc_id,
        &[("topic", "right")],
    );
    let (_right_ops_dir, right_ops_path) = write_metadata_variant_ops_file(
        "store-merge-metadata-duplicate-additions-right-ops",
        "right",
    );
    let (_center_ops_dir, center_ops_path) = write_metadata_variant_ops_file(
        "store-merge-metadata-duplicate-additions-center-ops",
        "right",
    );
    let resolved_state_file = path_arg(&resolved_state_path);
    let right_ops_file = path_arg(&right_ops_path);
    let center_ops_file = path_arg(&center_ops_path);

    let right_revision_id =
        flow.commit_ops_revision(flow.genesis_revision_id(), &right_ops_file, "63", "64");
    let center_revision_id =
        flow.commit_ops_revision(flow.genesis_revision_id(), &center_ops_file, "65", "66");
    let merge_json = flow.create_merge_revision(
        &[
            flow.genesis_revision_id(),
            &right_revision_id,
            &center_revision_id,
        ],
        &resolved_state_file,
        "67",
    );

    assert_eq!(merge_json["merge_outcome"], "multi-variant");
    assert!(
        merge_json["merge_reason_details"]
            .as_array()
            .is_some_and(|details| details.iter().any(|detail| {
                detail["subject_id"] == "topic"
                    && detail["variant_kind"] == "metadata"
                    && detail["reason_kind"] == "selected-non-primary-parent-variant"
                    && detail["branch_kind"] == "adopted-non-primary-addition"
                    && detail["competing_variants"] == json!(["\"right\""])
            })),
        "expected selected duplicate metadata addition detail, got {merge_json}"
    );
    assert!(
        merge_json["merge_reason_details"]
            .as_array()
            .is_some_and(|details| details.iter().any(|detail| {
                detail["subject_id"] == "topic"
                    && detail["variant_kind"] == "metadata"
                    && detail["reason_kind"]
                        == "multiple-competing-alternatives-remain-after-selected-variant"
                    && detail["branch_kind"] == "multiple-competing-non-primary-additions"
                    && detail["competing_variants"] == json!(["\"right\"", "\"right\""])
            })),
        "expected duplicate competing metadata additions detail, got {merge_json}"
    );
    assert_eq!(merge_json["patch_op_count"], 1);
}
