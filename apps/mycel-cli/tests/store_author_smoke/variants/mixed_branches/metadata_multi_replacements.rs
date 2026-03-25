use super::*;

#[test]
fn store_merge_authoring_flow_reports_selected_metadata_replacement_with_multiple_replacements_and_removal(
) {
    let doc_id = "doc:author-smoke-metadata-select-many";
    let flow = VariantScenarioFlow::new(
        "store-merge-metadata-select-many-root",
        "store-merge-metadata-select-many-key",
        doc_id,
        "Author Smoke Metadata Select Many",
        "126",
    );
    let (_resolved_dir, resolved_state_path) = write_metadata_variant_resolved_state_for_doc_file(
        "store-merge-metadata-select-many-state",
        doc_id,
        "right-a",
    );
    let (_base_ops_dir, base_ops_path) = write_metadata_entries_ops_file(
        "store-merge-metadata-select-many-base-ops",
        &[("topic", "base")],
    );
    let (_replace_a_ops_dir, replace_a_ops_path) = write_metadata_entries_ops_file(
        "store-merge-metadata-select-many-replace-a-ops",
        &[("topic", "right-a")],
    );
    let (_replace_b_ops_dir, replace_b_ops_path) = write_metadata_entries_ops_file(
        "store-merge-metadata-select-many-replace-b-ops",
        &[("topic", "right-b")],
    );
    let resolved_state_file = path_arg(&resolved_state_path);
    let base_ops_file = path_arg(&base_ops_path);
    let replace_a_ops_file = path_arg(&replace_a_ops_path);
    let replace_b_ops_file = path_arg(&replace_b_ops_path);

    let base_revision_id =
        flow.commit_ops_revision(flow.genesis_revision_id(), &base_ops_file, "127", "128");
    let replace_a_revision_id =
        flow.commit_ops_revision(&base_revision_id, &replace_a_ops_file, "129", "130");
    let replace_b_revision_id =
        flow.commit_ops_revision(&base_revision_id, &replace_b_ops_file, "131", "132");
    let merge_json = flow.create_merge_revision(
        &[
            &base_revision_id,
            &replace_a_revision_id,
            &replace_b_revision_id,
            flow.genesis_revision_id(),
        ],
        &resolved_state_file,
        "133",
    );

    assert_eq!(merge_json["merge_outcome"], "multi-variant");
    assert!(
        merge_json["merge_reasons"].as_array().is_some_and(|reasons| reasons
            .iter()
            .any(|reason| reason.as_str().is_some_and(|reason| reason.contains(
                "metadata key 'topic' adopted a non-primary parent replacement while competing non-primary replacements and a removal remained"
            )))),
        "expected richer selected metadata reason, got {merge_json}"
    );
    assert!(
        merge_json["merge_reason_details"]
            .as_array()
            .is_some_and(|details| details.iter().any(|detail| {
                detail["subject_id"] == "topic"
                    && detail["variant_kind"] == "metadata"
                    && detail["reason_kind"] == "selected-non-primary-parent-variant"
                    && detail["branch_kind"]
                        == "adopted-non-primary-replacement-while-competing-replacements-and-removal-remain"
            })),
        "expected richer selected metadata branch detail, got {merge_json}"
    );
    assert!(
        merge_json["merge_reason_details"]
            .as_array()
            .is_some_and(|details| details.iter().any(|detail| {
                detail["subject_id"] == "topic"
                    && detail["variant_kind"] == "metadata"
                    && detail["reason_kind"]
                        == "multiple-competing-alternatives-remain-after-selected-variant"
                    && detail["branch_kind"]
                        == "multiple-competing-non-primary-replacements-and-removals"
            })),
        "expected richer competing metadata branch detail, got {merge_json}"
    );
}

#[test]
fn store_merge_authoring_flow_reports_kept_primary_metadata_over_multiple_replacements_and_removals(
) {
    let doc_id = "doc:author-smoke-metadata-keep-many";
    let flow = VariantScenarioFlow::new(
        "store-merge-metadata-keep-many-root",
        "store-merge-metadata-keep-many-key",
        doc_id,
        "Author Smoke Metadata Keep Many",
        "134",
    );
    let (_resolved_dir, resolved_state_path) = write_metadata_variant_resolved_state_for_doc_file(
        "store-merge-metadata-keep-many-state",
        doc_id,
        "base",
    );
    let (_base_ops_dir, base_ops_path) = write_metadata_entries_ops_file(
        "store-merge-metadata-keep-many-base-ops",
        &[("topic", "base")],
    );
    let (_replace_a_ops_dir, replace_a_ops_path) = write_metadata_entries_ops_file(
        "store-merge-metadata-keep-many-replace-a-ops",
        &[("topic", "right-a")],
    );
    let (_replace_b_ops_dir, replace_b_ops_path) = write_metadata_entries_ops_file(
        "store-merge-metadata-keep-many-replace-b-ops",
        &[("topic", "right-b")],
    );
    let resolved_state_file = path_arg(&resolved_state_path);
    let base_ops_file = path_arg(&base_ops_path);
    let replace_a_ops_file = path_arg(&replace_a_ops_path);
    let replace_b_ops_file = path_arg(&replace_b_ops_path);

    let base_revision_id =
        flow.commit_ops_revision(flow.genesis_revision_id(), &base_ops_file, "135", "136");
    let replace_a_revision_id =
        flow.commit_ops_revision(&base_revision_id, &replace_a_ops_file, "137", "138");
    let replace_b_revision_id =
        flow.commit_ops_revision(&base_revision_id, &replace_b_ops_file, "139", "140");
    let merge_json = flow.create_merge_revision(
        &[
            &base_revision_id,
            &replace_a_revision_id,
            &replace_b_revision_id,
            flow.genesis_revision_id(),
        ],
        &resolved_state_file,
        "141",
    );

    assert_eq!(merge_json["merge_outcome"], "multi-variant");
    assert!(
        merge_json["merge_reasons"].as_array().is_some_and(|reasons| reasons
            .iter()
            .any(|reason| reason.as_str().is_some_and(|reason| reason.contains(
                "metadata key 'topic' kept the primary parent variant over multiple competing non-primary replacements and removals"
            )))),
        "expected richer kept-primary metadata reason, got {merge_json}"
    );
    assert!(
        merge_json["merge_reason_details"]
            .as_array()
            .is_some_and(|details| details.iter().any(|detail| {
                detail["subject_id"] == "topic"
                    && detail["variant_kind"] == "metadata"
                    && detail["reason_kind"]
                        == "kept-primary-parent-variant-over-competing-non-primary-alternative"
                    && detail["branch_kind"]
                        == "kept-primary-variant-over-multiple-competing-non-primary-replacements-and-removals"
            })),
        "expected richer kept-primary metadata branch detail, got {merge_json}"
    );
    assert!(
        merge_json["merge_reason_details"]
            .as_array()
            .is_some_and(|details| details.iter().any(|detail| {
                detail["subject_id"] == "topic"
                    && detail["variant_kind"] == "metadata"
                    && detail["reason_kind"]
                        == "multiple-competing-alternatives-remain-after-keeping-primary-variant"
                    && detail["branch_kind"]
                        == "multiple-competing-non-primary-replacements-and-removals"
            })),
        "expected richer multiple competing kept-primary metadata branch detail, got {merge_json}"
    );
}
