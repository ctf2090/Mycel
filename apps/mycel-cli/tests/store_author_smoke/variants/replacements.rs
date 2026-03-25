use super::*;

#[test]
fn store_merge_authoring_flow_reports_content_variant_choice_as_multi_variant() {
    let flow = StoreAuthoringFlow::new(
        "store-merge-content-variant-root",
        "store-merge-content-variant-key",
    );
    let doc_id = "doc:author-smoke-content-variant";
    let (_resolved_dir, resolved_state_path) = write_content_variant_resolved_state_file(
        "store-merge-content-variant-state",
        "Right variant",
    );
    let (_left_ops_dir, left_ops_path) =
        write_content_variant_ops_file("store-merge-content-variant-left-ops", "Left variant");
    let (_right_ops_dir, right_ops_path) =
        write_content_variant_ops_file("store-merge-content-variant-right-ops", "Right variant");
    let (_center_ops_dir, center_ops_path) =
        write_content_variant_ops_file("store-merge-content-variant-center-ops", "Center variant");
    let resolved_state_file = path_arg(&resolved_state_path);
    let left_ops_file = path_arg(&left_ops_path);
    let right_ops_file = path_arg(&right_ops_path);
    let center_ops_file = path_arg(&center_ops_path);
    let genesis_revision_id =
        flow.create_document(doc_id, "Author Smoke Content Variant", "en", "30");

    let base_ops_dir = create_temp_dir("store-merge-content-variant-base-ops");
    let base_ops_path = base_ops_dir.path().join("ops.json");
    fs::write(
        &base_ops_path,
        serde_json::to_string_pretty(&json!([
            {
                "op": "insert_block",
                "new_block": {
                    "block_id": "blk:author-smoke-variant-001",
                    "block_type": "paragraph",
                    "content": "Base",
                    "attrs": {},
                    "children": []
                }
            }
        ]))
        .expect("content variant base ops JSON should serialize"),
    )
    .expect("content variant base ops JSON should write");
    let base_ops_file = path_arg(&base_ops_path);

    let base_patch_id = flow.create_patch(doc_id, &genesis_revision_id, &base_ops_file, "31");
    let base_revision_id = flow.commit_revision(doc_id, &genesis_revision_id, &base_patch_id, "32");
    let left_patch_id = flow.create_patch(doc_id, &base_revision_id, &left_ops_file, "33");
    let left_revision_id = flow.commit_revision(doc_id, &base_revision_id, &left_patch_id, "34");
    let right_patch_id = flow.create_patch(doc_id, &base_revision_id, &right_ops_file, "35");
    let right_revision_id = flow.commit_revision(doc_id, &base_revision_id, &right_patch_id, "36");
    let center_patch_id = flow.create_patch(doc_id, &base_revision_id, &center_ops_file, "37");
    let center_revision_id =
        flow.commit_revision(doc_id, &base_revision_id, &center_patch_id, "38");
    let merge_json = flow.create_merge_revision(
        doc_id,
        &[&left_revision_id, &right_revision_id, &center_revision_id],
        &resolved_state_file,
        "39",
    );
    assert_content_variant_merge_reasons(&merge_json);
}

#[test]
fn store_merge_authoring_flow_reports_metadata_variant_choice_as_multi_variant() {
    let flow = StoreAuthoringFlow::new(
        "store-merge-metadata-variant-root",
        "store-merge-metadata-variant-key",
    );
    let doc_id = "doc:author-smoke-metadata-variant";
    let (_resolved_dir, resolved_state_path) =
        write_metadata_variant_resolved_state_file("store-merge-metadata-variant-state", "right");
    let (_left_ops_dir, left_ops_path) =
        write_metadata_variant_ops_file("store-merge-metadata-variant-left-ops", "left");
    let (_right_ops_dir, right_ops_path) =
        write_metadata_variant_ops_file("store-merge-metadata-variant-right-ops", "right");
    let (_center_ops_dir, center_ops_path) =
        write_metadata_variant_ops_file("store-merge-metadata-variant-center-ops", "center");
    let resolved_state_file = path_arg(&resolved_state_path);
    let left_ops_file = path_arg(&left_ops_path);
    let right_ops_file = path_arg(&right_ops_path);
    let center_ops_file = path_arg(&center_ops_path);
    let genesis_revision_id =
        flow.create_document(doc_id, "Author Smoke Metadata Variant", "en", "40");
    let left_patch_id = flow.create_patch(doc_id, &genesis_revision_id, &left_ops_file, "41");
    let left_revision_id = flow.commit_revision(doc_id, &genesis_revision_id, &left_patch_id, "42");
    let right_patch_id = flow.create_patch(doc_id, &genesis_revision_id, &right_ops_file, "43");
    let right_revision_id =
        flow.commit_revision(doc_id, &genesis_revision_id, &right_patch_id, "44");
    let center_patch_id = flow.create_patch(doc_id, &genesis_revision_id, &center_ops_file, "45");
    let center_revision_id =
        flow.commit_revision(doc_id, &genesis_revision_id, &center_patch_id, "46");
    let merge_json = flow.create_merge_revision(
        doc_id,
        &[&left_revision_id, &right_revision_id, &center_revision_id],
        &resolved_state_file,
        "47",
    );
    assert_eq!(merge_json["merge_outcome"], "multi-variant");
    assert!(
        merge_json["merge_reasons"]
            .as_array()
            .is_some_and(|reasons| reasons.iter().any(|reason| {
                reason.as_str().is_some_and(|reason| {
                    reason.contains("metadata key 'topic' adopted a non-primary parent replacement")
                })
            })),
        "expected metadata variant multi-variant reason, got {merge_json}"
    );
    assert!(
        merge_json["merge_reasons"]
            .as_array()
            .is_some_and(|reasons| reasons.iter().any(|reason| {
                reason.as_str().is_some_and(|reason| {
                    reason.contains(
                        "metadata key 'topic' selected one non-primary replacement while other competing non-primary replacements remained",
                    )
                })
            })),
        "expected competing metadata variant reason, got {merge_json}"
    );
    assert!(
        merge_json["merge_reason_details"]
            .as_array()
            .is_some_and(|details| details.iter().any(|detail| {
                detail["subject_kind"] == "metadata-key"
                    && detail["subject_id"] == "topic"
                    && detail["variant_kind"] == "metadata"
                    && detail["reason_kind"] == "selected-non-primary-parent-variant"
                    && detail["branch_kind"] == "adopted-non-primary-replacement"
                    && detail["primary_variant"] == "\"left\""
                    && detail["resolved_variant"] == "\"right\""
                    && detail["competing_variants"]
                        .as_array()
                        .is_some_and(|variants| variants.len() == 1)
            })),
        "expected structured metadata variant detail, got {merge_json}"
    );
    assert!(
        merge_json["merge_reason_details"]
            .as_array()
            .is_some_and(|details| details.iter().any(|detail| {
                detail["subject_kind"] == "metadata-key"
                    && detail["subject_id"] == "topic"
                    && detail["variant_kind"] == "metadata"
                    && detail["reason_kind"]
                        == "multiple-competing-alternatives-remain-after-selected-variant"
                    && detail["branch_kind"] == "multiple-competing-non-primary-replacements"
                    && detail["competing_variants"]
                        .as_array()
                        .is_some_and(|variants| variants.len() == 2)
            })),
        "expected competing metadata branch kind detail, got {merge_json}"
    );
    assert_eq!(merge_json["patch_op_count"], 1);
    assert_eq!(
        merge_json["parent_revision_ids"].as_array().map(Vec::len),
        Some(3)
    );
}

#[test]
fn store_merge_authoring_flow_preserves_duplicate_non_primary_content_replacements() {
    let flow = StoreAuthoringFlow::new(
        "store-merge-content-duplicate-non-primary-root",
        "store-merge-content-duplicate-non-primary-key",
    );
    let doc_id = "doc:author-smoke-content-variant";
    let (_resolved_dir, resolved_state_path) = write_single_block_resolved_state_file(
        "store-merge-content-duplicate-non-primary-state",
        doc_id,
        "blk:author-smoke-variant-001",
        "right",
    );
    let (_left_ops_dir, left_ops_path) = write_content_variant_ops_file(
        "store-merge-content-duplicate-non-primary-left-ops",
        "left",
    );
    let (_right_ops_dir, right_ops_path) = write_content_variant_ops_file(
        "store-merge-content-duplicate-non-primary-right-ops",
        "right",
    );
    let (_center_ops_dir, center_ops_path) = write_content_variant_ops_file(
        "store-merge-content-duplicate-non-primary-center-ops",
        "right",
    );
    let resolved_state_file = path_arg(&resolved_state_path);
    let left_ops_file = path_arg(&left_ops_path);
    let right_ops_file = path_arg(&right_ops_path);
    let center_ops_file = path_arg(&center_ops_path);

    let genesis_revision_id =
        flow.create_document(doc_id, "Author Smoke Content Variant", "en", "48");
    let (_base_ops_dir, base_ops_path) = write_insert_block_ops_file(
        "store-merge-content-duplicate-non-primary-base-ops",
        "blk:author-smoke-variant-001",
        "Base",
    );
    let base_ops_file = path_arg(&base_ops_path);

    let base_patch_id = flow.create_patch(doc_id, &genesis_revision_id, &base_ops_file, "49");
    let base_revision_id = flow.commit_revision(doc_id, &genesis_revision_id, &base_patch_id, "50");
    let left_patch_id = flow.create_patch(doc_id, &base_revision_id, &left_ops_file, "51");
    let left_revision_id = flow.commit_revision(doc_id, &base_revision_id, &left_patch_id, "52");
    let right_patch_id = flow.create_patch(doc_id, &left_revision_id, &right_ops_file, "53");
    let right_revision_id = flow.commit_revision(doc_id, &left_revision_id, &right_patch_id, "54");
    let center_patch_id = flow.create_patch(doc_id, &left_revision_id, &center_ops_file, "55");
    let center_revision_id =
        flow.commit_revision(doc_id, &left_revision_id, &center_patch_id, "56");
    let merge_json = flow.create_merge_revision(
        doc_id,
        &[&left_revision_id, &right_revision_id, &center_revision_id],
        &resolved_state_file,
        "57",
    );
    assert_duplicate_non_primary_content_replacement_reasons(&merge_json);
}

#[test]
fn store_merge_authoring_flow_preserves_duplicate_non_primary_metadata_replacements() {
    let flow = StoreAuthoringFlow::new(
        "store-merge-metadata-duplicate-non-primary-root",
        "store-merge-metadata-duplicate-non-primary-key",
    );
    let doc_id = "doc:author-smoke-metadata-variant";
    let (_resolved_dir, resolved_state_path) = write_metadata_variant_resolved_state_file(
        "store-merge-metadata-duplicate-non-primary-state",
        "right",
    );
    let (_left_ops_dir, left_ops_path) = write_metadata_variant_ops_file(
        "store-merge-metadata-duplicate-non-primary-left-ops",
        "left",
    );
    let (_right_ops_dir, right_ops_path) = write_metadata_variant_ops_file(
        "store-merge-metadata-duplicate-non-primary-right-ops",
        "right",
    );
    let (_center_ops_dir, center_ops_path) = write_metadata_variant_ops_file(
        "store-merge-metadata-duplicate-non-primary-center-ops",
        "right",
    );
    let resolved_state_file = path_arg(&resolved_state_path);
    let left_ops_file = path_arg(&left_ops_path);
    let right_ops_file = path_arg(&right_ops_path);
    let center_ops_file = path_arg(&center_ops_path);

    let genesis_revision_id =
        flow.create_document(doc_id, "Author Smoke Metadata Variant", "en", "56");
    let left_patch_id = flow.create_patch(doc_id, &genesis_revision_id, &left_ops_file, "57");
    let left_revision_id = flow.commit_revision(doc_id, &genesis_revision_id, &left_patch_id, "58");
    let right_patch_id = flow.create_patch(doc_id, &genesis_revision_id, &right_ops_file, "59");
    let right_revision_id =
        flow.commit_revision(doc_id, &genesis_revision_id, &right_patch_id, "60");
    let center_patch_id = flow.create_patch(doc_id, &genesis_revision_id, &center_ops_file, "61");
    let center_revision_id =
        flow.commit_revision(doc_id, &genesis_revision_id, &center_patch_id, "62");
    let merge_json = flow.create_merge_revision(
        doc_id,
        &[&left_revision_id, &right_revision_id, &center_revision_id],
        &resolved_state_file,
        "63",
    );
    assert_duplicate_non_primary_metadata_replacement_reasons(&merge_json);
}

