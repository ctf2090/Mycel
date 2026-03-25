use super::*;

#[test]
fn store_merge_authoring_flow_reports_added_metadata_from_non_primary_parent_as_multi_variant() {
    let store_dir = create_temp_dir("store-merge-metadata-added-root");
    let (_key_dir, key_path) = write_signing_key_file("store-merge-metadata-added-key");
    let (_resolved_dir, resolved_state_path) = write_metadata_variant_resolved_state_for_doc_file(
        "store-merge-metadata-added-state",
        "doc:author-smoke-metadata-added",
        "right",
    );
    let (_right_ops_dir, right_ops_path) =
        write_metadata_variant_ops_file("store-merge-metadata-added-right-ops", "right");
    let store_root = path_arg(store_dir.path());
    let key_file = path_arg(&key_path);
    let resolved_state_file = path_arg(&resolved_state_path);
    let right_ops_file = path_arg(&right_ops_path);

    let init = run_mycel(&["store", "init", &store_root, "--json"]);
    assert_success(&init);

    let document = run_mycel(&[
        "store",
        "create-document",
        &store_root,
        "--doc-id",
        "doc:author-smoke-metadata-added",
        "--title",
        "Author Smoke Metadata Added",
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

    let right_patch = run_mycel(&[
        "store",
        "create-patch",
        &store_root,
        "--doc-id",
        "doc:author-smoke-metadata-added",
        "--base-revision",
        &genesis_revision_id,
        "--ops",
        &right_ops_file,
        "--signing-key",
        &key_file,
        "--timestamp",
        "41",
        "--json",
    ]);
    assert_success(&right_patch);
    let right_patch_json = assert_json_status(&right_patch, "ok");
    let right_patch_id = right_patch_json["patch_id"]
        .as_str()
        .expect("right patch_id should be string")
        .to_string();

    let right_revision = run_mycel(&[
        "store",
        "commit-revision",
        &store_root,
        "--doc-id",
        "doc:author-smoke-metadata-added",
        "--parent",
        &genesis_revision_id,
        "--patch",
        &right_patch_id,
        "--signing-key",
        &key_file,
        "--timestamp",
        "42",
        "--json",
    ]);
    assert_success(&right_revision);
    let right_revision_json = assert_json_status(&right_revision, "ok");
    let right_revision_id = right_revision_json["revision_id"]
        .as_str()
        .expect("right revision_id should be string")
        .to_string();

    let merge = run_mycel(&[
        "store",
        "create-merge-revision",
        &store_root,
        "--doc-id",
        "doc:author-smoke-metadata-added",
        "--parent",
        &genesis_revision_id,
        "--parent",
        &right_revision_id,
        "--resolved-state",
        &resolved_state_file,
        "--signing-key",
        &key_file,
        "--timestamp",
        "43",
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
        "expected metadata added-from-parent multi-variant reason, got {merge_json}"
    );
    assert!(
        !merge_json["merge_reasons"]
            .as_array()
            .is_some_and(|reasons| reasons.iter().any(|reason| {
                reason.as_str().is_some_and(|reason| {
                    reason.contains(
                        "metadata key 'topic' kept the primary variant while multiple competing non-primary additions remained",
                    )
                })
            })),
        "did not expect competing metadata reason with only one alternative, got {merge_json}"
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
        "expected adopted non-primary metadata addition detail, got {merge_json}"
    );
    assert_eq!(merge_json["patch_op_count"], 1);
    assert_eq!(
        merge_json["parent_revision_ids"].as_array().map(Vec::len),
        Some(2)
    );
}

#[test]
fn store_merge_authoring_flow_reports_kept_primary_metadata_over_non_primary_addition() {
    let store_dir = create_temp_dir("store-merge-metadata-keep-primary-root");
    let (_key_dir, key_path) = write_signing_key_file("store-merge-metadata-keep-primary-key");
    let (_right_ops_dir, right_ops_path) =
        write_metadata_variant_ops_file("store-merge-metadata-keep-primary-right-ops", "right");
    let store_root = path_arg(store_dir.path());
    let key_file = path_arg(&key_path);
    let right_ops_file = path_arg(&right_ops_path);

    let init = run_mycel(&["store", "init", &store_root, "--json"]);
    assert_success(&init);

    let document = run_mycel(&[
        "store",
        "create-document",
        &store_root,
        "--doc-id",
        "doc:author-smoke-metadata-keep-primary",
        "--title",
        "Author Smoke Metadata Keep Primary",
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

    let right_patch = run_mycel(&[
        "store",
        "create-patch",
        &store_root,
        "--doc-id",
        "doc:author-smoke-metadata-keep-primary",
        "--base-revision",
        &genesis_revision_id,
        "--ops",
        &right_ops_file,
        "--signing-key",
        &key_file,
        "--timestamp",
        "41",
        "--json",
    ]);
    assert_success(&right_patch);
    let right_patch_json = assert_json_status(&right_patch, "ok");
    let right_patch_id = right_patch_json["patch_id"]
        .as_str()
        .expect("right patch_id should be string")
        .to_string();

    let right_revision = run_mycel(&[
        "store",
        "commit-revision",
        &store_root,
        "--doc-id",
        "doc:author-smoke-metadata-keep-primary",
        "--parent",
        &genesis_revision_id,
        "--patch",
        &right_patch_id,
        "--signing-key",
        &key_file,
        "--timestamp",
        "42",
        "--json",
    ]);
    assert_success(&right_revision);
    let right_revision_json = assert_json_status(&right_revision, "ok");
    let right_revision_id = right_revision_json["revision_id"]
        .as_str()
        .expect("right revision_id should be string")
        .to_string();

    let empty_resolved_dir = create_temp_dir("store-merge-metadata-keep-primary-empty-state");
    let empty_resolved_path = empty_resolved_dir.path().join("resolved-state.json");
    fs::write(
        &empty_resolved_path,
        serde_json::to_string_pretty(&json!({
            "doc_id": "doc:author-smoke-metadata-keep-primary",
            "blocks": [],
            "metadata": {}
        }))
        .expect("empty resolved state JSON should serialize"),
    )
    .expect("empty resolved state JSON should write");
    let resolved_state_file = path_arg(&empty_resolved_path);

    let merge = run_mycel(&[
        "store",
        "create-merge-revision",
        &store_root,
        "--doc-id",
        "doc:author-smoke-metadata-keep-primary",
        "--parent",
        &genesis_revision_id,
        "--parent",
        &right_revision_id,
        "--resolved-state",
        &resolved_state_file,
        "--signing-key",
        &key_file,
        "--timestamp",
        "43",
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
                    reason.contains("metadata key 'topic' kept the primary absence over a competing non-primary addition")
                })
            })),
        "expected metadata keep-primary multi-variant reason, got {merge_json}"
    );
    assert_eq!(merge_json["patch_op_count"], 0);
}

#[test]
fn store_merge_authoring_flow_reports_kept_primary_and_multiple_competing_metadata_additions() {
    let store_dir = create_temp_dir("store-merge-metadata-keep-primary-multiple-root");
    let (_key_dir, key_path) =
        write_signing_key_file("store-merge-metadata-keep-primary-multiple-key");
    let (_right_ops_dir, right_ops_path) = write_metadata_variant_ops_file(
        "store-merge-metadata-keep-primary-multiple-right-ops",
        "right",
    );
    let (_center_ops_dir, center_ops_path) = write_metadata_variant_ops_file(
        "store-merge-metadata-keep-primary-multiple-center-ops",
        "center",
    );
    let store_root = path_arg(store_dir.path());
    let key_file = path_arg(&key_path);
    let right_ops_file = path_arg(&right_ops_path);
    let center_ops_file = path_arg(&center_ops_path);

    let init = run_mycel(&["store", "init", &store_root, "--json"]);
    assert_success(&init);

    let document = run_mycel(&[
        "store",
        "create-document",
        &store_root,
        "--doc-id",
        "doc:author-smoke-metadata-keep-primary-multiple",
        "--title",
        "Author Smoke Metadata Keep Primary Multiple",
        "--language",
        "en",
        "--signing-key",
        &key_file,
        "--timestamp",
        "50",
        "--json",
    ]);
    assert_success(&document);
    let document_json = assert_json_status(&document, "ok");
    let genesis_revision_id = document_json["genesis_revision_id"]
        .as_str()
        .expect("genesis revision should be string")
        .to_string();

    let right_patch = run_mycel(&[
        "store",
        "create-patch",
        &store_root,
        "--doc-id",
        "doc:author-smoke-metadata-keep-primary-multiple",
        "--base-revision",
        &genesis_revision_id,
        "--ops",
        &right_ops_file,
        "--signing-key",
        &key_file,
        "--timestamp",
        "51",
        "--json",
    ]);
    assert_success(&right_patch);
    let right_patch_json = assert_json_status(&right_patch, "ok");
    let right_patch_id = right_patch_json["patch_id"]
        .as_str()
        .expect("right patch_id should be string")
        .to_string();

    let right_revision = run_mycel(&[
        "store",
        "commit-revision",
        &store_root,
        "--doc-id",
        "doc:author-smoke-metadata-keep-primary-multiple",
        "--parent",
        &genesis_revision_id,
        "--patch",
        &right_patch_id,
        "--signing-key",
        &key_file,
        "--timestamp",
        "52",
        "--json",
    ]);
    assert_success(&right_revision);
    let right_revision_json = assert_json_status(&right_revision, "ok");
    let right_revision_id = right_revision_json["revision_id"]
        .as_str()
        .expect("right revision_id should be string")
        .to_string();

    let center_patch = run_mycel(&[
        "store",
        "create-patch",
        &store_root,
        "--doc-id",
        "doc:author-smoke-metadata-keep-primary-multiple",
        "--base-revision",
        &genesis_revision_id,
        "--ops",
        &center_ops_file,
        "--signing-key",
        &key_file,
        "--timestamp",
        "53",
        "--json",
    ]);
    assert_success(&center_patch);
    let center_patch_json = assert_json_status(&center_patch, "ok");
    let center_patch_id = center_patch_json["patch_id"]
        .as_str()
        .expect("center patch_id should be string")
        .to_string();

    let center_revision = run_mycel(&[
        "store",
        "commit-revision",
        &store_root,
        "--doc-id",
        "doc:author-smoke-metadata-keep-primary-multiple",
        "--parent",
        &genesis_revision_id,
        "--patch",
        &center_patch_id,
        "--signing-key",
        &key_file,
        "--timestamp",
        "54",
        "--json",
    ]);
    assert_success(&center_revision);
    let center_revision_json = assert_json_status(&center_revision, "ok");
    let center_revision_id = center_revision_json["revision_id"]
        .as_str()
        .expect("center revision_id should be string")
        .to_string();

    let empty_resolved_dir =
        create_temp_dir("store-merge-metadata-keep-primary-multiple-empty-state");
    let empty_resolved_path = empty_resolved_dir.path().join("resolved-state.json");
    fs::write(
        &empty_resolved_path,
        serde_json::to_string_pretty(&json!({
            "doc_id": "doc:author-smoke-metadata-keep-primary-multiple",
            "blocks": [],
            "metadata": {}
        }))
        .expect("empty resolved state JSON should serialize"),
    )
    .expect("empty resolved state JSON should write");
    let resolved_state_file = path_arg(&empty_resolved_path);

    let merge = run_mycel(&[
        "store",
        "create-merge-revision",
        &store_root,
        "--doc-id",
        "doc:author-smoke-metadata-keep-primary-multiple",
        "--parent",
        &genesis_revision_id,
        "--parent",
        &right_revision_id,
        "--parent",
        &center_revision_id,
        "--resolved-state",
        &resolved_state_file,
        "--signing-key",
        &key_file,
        "--timestamp",
        "55",
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
                    reason.contains("metadata key 'topic' kept the primary absence over a competing non-primary addition")
                })
            })),
        "expected keep-primary metadata reason, got {merge_json}"
    );
    assert!(
        merge_json["merge_reasons"]
            .as_array()
            .is_some_and(|reasons| reasons.iter().any(|reason| {
                reason.as_str().is_some_and(|reason| {
                    reason.contains(
                        "metadata key 'topic' kept the primary variant while multiple competing non-primary additions remained",
                    )
                })
            })),
        "expected multiple-competing metadata reason, got {merge_json}"
    );
    assert!(
        merge_json["merge_reason_details"]
            .as_array()
            .is_some_and(|details| details.iter().any(|detail| {
                detail["subject_id"] == "topic"
                    && detail["variant_kind"] == "metadata"
                    && detail["reason_kind"]
                        == "kept-primary-parent-variant-over-competing-non-primary-alternative"
                    && detail["branch_kind"] == "kept-primary-absence-over-non-primary-addition"
                    && detail["competing_variants"]
                        .as_array()
                        .is_some_and(|variants| variants.len() == 2)
            })),
        "expected keep-primary metadata detail, got {merge_json}"
    );
    assert!(
        merge_json["merge_reason_details"]
            .as_array()
            .is_some_and(|details| details.iter().any(|detail| {
                detail["subject_id"] == "topic"
                    && detail["variant_kind"] == "metadata"
                    && detail["reason_kind"]
                        == "multiple-competing-alternatives-remain-after-keeping-primary-variant"
                    && detail["branch_kind"] == "multiple-competing-non-primary-additions"
                    && detail["competing_variants"]
                        .as_array()
                        .is_some_and(|variants| variants.len() == 2)
            })),
        "expected multiple-competing metadata detail, got {merge_json}"
    );
    assert_eq!(merge_json["patch_op_count"], 0);
}

#[test]
fn store_merge_authoring_flow_preserves_duplicate_non_primary_metadata_additions_when_keeping_primary_absence(
) {
    let store_dir = create_temp_dir("store-merge-metadata-keep-primary-duplicate-additions-root");
    let (_key_dir, key_path) =
        write_signing_key_file("store-merge-metadata-keep-primary-duplicate-additions-key");
    let (_right_ops_dir, right_ops_path) = write_metadata_variant_ops_file(
        "store-merge-metadata-keep-primary-duplicate-additions-right-ops",
        "right",
    );
    let (_center_ops_dir, center_ops_path) = write_metadata_variant_ops_file(
        "store-merge-metadata-keep-primary-duplicate-additions-center-ops",
        "right",
    );
    let store_root = path_arg(store_dir.path());
    let key_file = path_arg(&key_path);
    let right_ops_file = path_arg(&right_ops_path);
    let center_ops_file = path_arg(&center_ops_path);

    let init = run_mycel(&["store", "init", &store_root, "--json"]);
    assert_success(&init);

    let document = run_mycel(&[
        "store",
        "create-document",
        &store_root,
        "--doc-id",
        "doc:author-smoke-metadata-keep-primary-duplicate-additions",
        "--title",
        "Author Smoke Metadata Keep Primary Duplicate Additions",
        "--language",
        "en",
        "--signing-key",
        &key_file,
        "--timestamp",
        "68",
        "--json",
    ]);
    assert_success(&document);
    let genesis_revision_id = assert_json_status(&document, "ok")["genesis_revision_id"]
        .as_str()
        .expect("genesis revision should be string")
        .to_string();

    let right_patch = run_mycel(&[
        "store",
        "create-patch",
        &store_root,
        "--doc-id",
        "doc:author-smoke-metadata-keep-primary-duplicate-additions",
        "--base-revision",
        &genesis_revision_id,
        "--ops",
        &right_ops_file,
        "--signing-key",
        &key_file,
        "--timestamp",
        "69",
        "--json",
    ]);
    assert_success(&right_patch);
    let right_patch_id = assert_json_status(&right_patch, "ok")["patch_id"]
        .as_str()
        .expect("right patch_id should be string")
        .to_string();

    let right_revision = run_mycel(&[
        "store",
        "commit-revision",
        &store_root,
        "--doc-id",
        "doc:author-smoke-metadata-keep-primary-duplicate-additions",
        "--parent",
        &genesis_revision_id,
        "--patch",
        &right_patch_id,
        "--signing-key",
        &key_file,
        "--timestamp",
        "70",
        "--json",
    ]);
    assert_success(&right_revision);
    let right_revision_id = assert_json_status(&right_revision, "ok")["revision_id"]
        .as_str()
        .expect("right revision_id should be string")
        .to_string();

    let center_patch = run_mycel(&[
        "store",
        "create-patch",
        &store_root,
        "--doc-id",
        "doc:author-smoke-metadata-keep-primary-duplicate-additions",
        "--base-revision",
        &genesis_revision_id,
        "--ops",
        &center_ops_file,
        "--signing-key",
        &key_file,
        "--timestamp",
        "71",
        "--json",
    ]);
    assert_success(&center_patch);
    let center_patch_id = assert_json_status(&center_patch, "ok")["patch_id"]
        .as_str()
        .expect("center patch_id should be string")
        .to_string();

    let center_revision = run_mycel(&[
        "store",
        "commit-revision",
        &store_root,
        "--doc-id",
        "doc:author-smoke-metadata-keep-primary-duplicate-additions",
        "--parent",
        &genesis_revision_id,
        "--patch",
        &center_patch_id,
        "--signing-key",
        &key_file,
        "--timestamp",
        "72",
        "--json",
    ]);
    assert_success(&center_revision);
    let center_revision_id = assert_json_status(&center_revision, "ok")["revision_id"]
        .as_str()
        .expect("center revision_id should be string")
        .to_string();

    let empty_resolved_dir =
        create_temp_dir("store-merge-metadata-keep-primary-duplicate-additions-empty-state");
    let empty_resolved_path = empty_resolved_dir.path().join("resolved-state.json");
    fs::write(
        &empty_resolved_path,
        serde_json::to_string_pretty(&json!({
            "doc_id": "doc:author-smoke-metadata-keep-primary-duplicate-additions",
            "blocks": [],
            "metadata": {}
        }))
        .expect("empty resolved state JSON should serialize"),
    )
    .expect("empty resolved state JSON should write");
    let resolved_state_file = path_arg(&empty_resolved_path);

    let merge = run_mycel(&[
        "store",
        "create-merge-revision",
        &store_root,
        "--doc-id",
        "doc:author-smoke-metadata-keep-primary-duplicate-additions",
        "--parent",
        &genesis_revision_id,
        "--parent",
        &right_revision_id,
        "--parent",
        &center_revision_id,
        "--resolved-state",
        &resolved_state_file,
        "--signing-key",
        &key_file,
        "--timestamp",
        "73",
        "--json",
    ]);
    assert_success(&merge);
    let merge_json = assert_json_status(&merge, "ok");
    assert_eq!(merge_json["merge_outcome"], "multi-variant");
    assert!(
        merge_json["merge_reason_details"]
            .as_array()
            .is_some_and(|details| details.iter().any(|detail| {
                detail["subject_id"] == "topic"
                    && detail["variant_kind"] == "metadata"
                    && detail["reason_kind"]
                        == "kept-primary-parent-variant-over-competing-non-primary-alternative"
                    && detail["branch_kind"] == "kept-primary-absence-over-non-primary-addition"
                    && detail["competing_variants"] == json!(["\"right\"", "\"right\""])
            })),
        "expected keep-primary duplicate metadata additions detail, got {merge_json}"
    );
    assert!(
        merge_json["merge_reason_details"]
            .as_array()
            .is_some_and(|details| details.iter().any(|detail| {
                detail["subject_id"] == "topic"
                    && detail["variant_kind"] == "metadata"
                    && detail["reason_kind"]
                        == "multiple-competing-alternatives-remain-after-keeping-primary-variant"
                    && detail["branch_kind"] == "multiple-competing-non-primary-additions"
                    && detail["competing_variants"] == json!(["\"right\"", "\"right\""])
            })),
        "expected multiple competing duplicate metadata additions detail, got {merge_json}"
    );
    assert_eq!(merge_json["patch_op_count"], 0);
}
