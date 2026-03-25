use super::*;

#[test]
fn store_merge_authoring_flow_reports_block_added_from_non_primary_parent_as_multi_variant() {
    let store_dir = create_temp_dir("store-merge-content-added-root");
    let (_key_dir, key_path) = write_signing_key_file("store-merge-content-added-key");
    let (_resolved_dir, resolved_state_path) =
        write_content_variant_resolved_state_file("store-merge-content-added-state", "right");
    let (_right_ops_dir, right_ops_path) =
        write_content_addition_ops_file("store-merge-content-added-right-ops", "right");
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
        "doc:author-smoke-content-variant",
        "--title",
        "Author Smoke Content Variant",
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
        "doc:author-smoke-content-variant",
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
        "doc:author-smoke-content-variant",
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
        "doc:author-smoke-content-variant",
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
                    reason.contains("block 'blk:author-smoke-variant-001' adopted a non-primary parent addition")
                })
            })),
        "expected added-from-parent content reason, got {merge_json}"
    );
    assert!(
        !merge_json["merge_reasons"]
            .as_array()
            .is_some_and(|reasons| reasons.iter().any(|reason| {
                reason.as_str().is_some_and(|reason| {
                    reason.contains(
                        "block 'blk:author-smoke-variant-001' kept the primary variant while multiple competing non-primary additions remained",
                    )
                })
            })),
        "did not expect competing content reason with only one alternative, got {merge_json}"
    );
    assert!(
        merge_json["merge_reason_details"]
            .as_array()
            .is_some_and(|details| details.iter().any(|detail| {
                detail["subject_id"] == "blk:author-smoke-variant-001"
                    && detail["variant_kind"] == "content"
                    && detail["reason_kind"] == "selected-non-primary-parent-variant"
                    && detail["branch_kind"] == "adopted-non-primary-addition"
            })),
        "expected adopted non-primary content addition detail, got {merge_json}"
    );
    assert_eq!(merge_json["patch_op_count"], 1);
    assert_eq!(
        merge_json["parent_revision_ids"].as_array().map(Vec::len),
        Some(2)
    );
}

#[test]
fn store_merge_authoring_flow_reports_kept_primary_absence_over_non_primary_block_addition() {
    let store_dir = create_temp_dir("store-merge-content-keep-primary-root");
    let (_key_dir, key_path) = write_signing_key_file("store-merge-content-keep-primary-key");
    let (_right_ops_dir, right_ops_path) =
        write_content_addition_ops_file("store-merge-content-keep-primary-right-ops", "right");
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
        "doc:author-smoke-content-variant",
        "--title",
        "Author Smoke Content Variant",
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
        "doc:author-smoke-content-variant",
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
        "doc:author-smoke-content-variant",
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

    let empty_resolved_dir = create_temp_dir("store-merge-content-keep-primary-empty-state");
    let empty_resolved_path = empty_resolved_dir.path().join("resolved-state.json");
    fs::write(
        &empty_resolved_path,
        serde_json::to_string_pretty(&json!({
            "doc_id": "doc:author-smoke-content-variant",
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
        "doc:author-smoke-content-variant",
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
                    reason.contains(
                        "block 'blk:author-smoke-variant-001' kept the primary absence over a competing non-primary addition",
                    )
                })
            })),
        "expected keep-primary content reason, got {merge_json}"
    );
    assert_eq!(merge_json["patch_op_count"], 0);
}

#[test]
fn store_merge_authoring_flow_reports_kept_primary_and_multiple_competing_block_additions() {
    let store_dir = create_temp_dir("store-merge-content-keep-primary-multiple-root");
    let (_key_dir, key_path) =
        write_signing_key_file("store-merge-content-keep-primary-multiple-key");
    let (_right_ops_dir, right_ops_path) = write_content_addition_ops_file(
        "store-merge-content-keep-primary-multiple-right-ops",
        "right",
    );
    let (_center_ops_dir, center_ops_path) = write_content_addition_ops_file(
        "store-merge-content-keep-primary-multiple-center-ops",
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
        "doc:author-smoke-content-keep-primary-multiple",
        "--title",
        "Author Smoke Content Keep Primary Multiple",
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
        "doc:author-smoke-content-keep-primary-multiple",
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
        "doc:author-smoke-content-keep-primary-multiple",
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
        "doc:author-smoke-content-keep-primary-multiple",
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
        "doc:author-smoke-content-keep-primary-multiple",
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
        create_temp_dir("store-merge-content-keep-primary-multiple-empty-state");
    let empty_resolved_path = empty_resolved_dir.path().join("resolved-state.json");
    fs::write(
        &empty_resolved_path,
        serde_json::to_string_pretty(&json!({
            "doc_id": "doc:author-smoke-content-keep-primary-multiple",
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
        "doc:author-smoke-content-keep-primary-multiple",
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
                    reason.contains(
                        "block 'blk:author-smoke-variant-001' kept the primary absence over a competing non-primary addition",
                    )
                })
            })),
        "expected keep-primary content reason, got {merge_json}"
    );
    assert!(
        merge_json["merge_reasons"]
            .as_array()
            .is_some_and(|reasons| reasons.iter().any(|reason| {
                reason.as_str().is_some_and(|reason| {
                    reason.contains(
                        "block 'blk:author-smoke-variant-001' kept the primary variant while multiple competing non-primary additions remained",
                    )
                })
            })),
        "expected multiple-competing content reason, got {merge_json}"
    );
    assert!(
        merge_json["merge_reason_details"]
            .as_array()
            .is_some_and(|details| details.iter().any(|detail| {
                detail["subject_id"] == "blk:author-smoke-variant-001"
                    && detail["variant_kind"] == "content"
                    && detail["reason_kind"]
                        == "kept-primary-parent-variant-over-competing-non-primary-alternative"
                    && detail["branch_kind"] == "kept-primary-absence-over-non-primary-addition"
                    && detail["competing_variants"]
                        .as_array()
                        .is_some_and(|variants| variants.len() == 2)
            })),
        "expected keep-primary content detail, got {merge_json}"
    );
    assert!(
        merge_json["merge_reason_details"]
            .as_array()
            .is_some_and(|details| details.iter().any(|detail| {
                detail["subject_id"] == "blk:author-smoke-variant-001"
                    && detail["variant_kind"] == "content"
                    && detail["reason_kind"]
                        == "multiple-competing-alternatives-remain-after-keeping-primary-variant"
                    && detail["branch_kind"] == "multiple-competing-non-primary-additions"
                    && detail["competing_variants"]
                        .as_array()
                        .is_some_and(|variants| variants.len() == 2)
            })),
        "expected multiple-competing content detail, got {merge_json}"
    );
    assert_eq!(merge_json["patch_op_count"], 0);
}

#[test]
fn store_merge_authoring_flow_preserves_duplicate_non_primary_content_additions_when_keeping_primary_absence(
) {
    let store_dir = create_temp_dir("store-merge-content-keep-primary-duplicate-additions-root");
    let (_key_dir, key_path) =
        write_signing_key_file("store-merge-content-keep-primary-duplicate-additions-key");
    let (_right_ops_dir, right_ops_path) = write_content_addition_ops_for_block_file(
        "store-merge-content-keep-primary-duplicate-additions-right-ops",
        "blk:author-smoke-added-keep-duplicate",
        "right",
    );
    let (_center_ops_dir, center_ops_path) = write_content_addition_ops_for_block_file(
        "store-merge-content-keep-primary-duplicate-additions-center-ops",
        "blk:author-smoke-added-keep-duplicate",
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
        "doc:author-smoke-content-keep-primary-duplicate-additions",
        "--title",
        "Author Smoke Content Keep Primary Duplicate Additions",
        "--language",
        "en",
        "--signing-key",
        &key_file,
        "--timestamp",
        "56",
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
        "doc:author-smoke-content-keep-primary-duplicate-additions",
        "--base-revision",
        &genesis_revision_id,
        "--ops",
        &right_ops_file,
        "--signing-key",
        &key_file,
        "--timestamp",
        "57",
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
        "doc:author-smoke-content-keep-primary-duplicate-additions",
        "--parent",
        &genesis_revision_id,
        "--patch",
        &right_patch_id,
        "--signing-key",
        &key_file,
        "--timestamp",
        "58",
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
        "doc:author-smoke-content-keep-primary-duplicate-additions",
        "--base-revision",
        &genesis_revision_id,
        "--ops",
        &center_ops_file,
        "--signing-key",
        &key_file,
        "--timestamp",
        "59",
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
        "doc:author-smoke-content-keep-primary-duplicate-additions",
        "--parent",
        &genesis_revision_id,
        "--patch",
        &center_patch_id,
        "--signing-key",
        &key_file,
        "--timestamp",
        "60",
        "--json",
    ]);
    assert_success(&center_revision);
    let center_revision_id = assert_json_status(&center_revision, "ok")["revision_id"]
        .as_str()
        .expect("center revision_id should be string")
        .to_string();

    let empty_resolved_dir =
        create_temp_dir("store-merge-content-keep-primary-duplicate-additions-empty-state");
    let empty_resolved_path = empty_resolved_dir.path().join("resolved-state.json");
    fs::write(
        &empty_resolved_path,
        serde_json::to_string_pretty(&json!({
            "doc_id": "doc:author-smoke-content-keep-primary-duplicate-additions",
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
        "doc:author-smoke-content-keep-primary-duplicate-additions",
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
        "61",
        "--json",
    ]);
    assert_success(&merge);
    let merge_json = assert_json_status(&merge, "ok");
    assert_eq!(merge_json["merge_outcome"], "multi-variant");
    assert!(
        merge_json["merge_reason_details"]
            .as_array()
            .is_some_and(|details| details.iter().any(|detail| {
                detail["subject_id"] == "blk:author-smoke-added-keep-duplicate"
                    && detail["variant_kind"] == "content"
                    && detail["reason_kind"]
                        == "kept-primary-parent-variant-over-competing-non-primary-alternative"
                    && detail["branch_kind"] == "kept-primary-absence-over-non-primary-addition"
                    && detail["competing_variants"] == json!([
                        "{\"attrs\":{},\"block_id\":\"blk:author-smoke-added-keep-duplicate\",\"block_type\":\"paragraph\",\"content\":\"right\"}",
                        "{\"attrs\":{},\"block_id\":\"blk:author-smoke-added-keep-duplicate\",\"block_type\":\"paragraph\",\"content\":\"right\"}"
                    ])
            })),
        "expected keep-primary duplicate content additions detail, got {merge_json}"
    );
    assert!(
        merge_json["merge_reason_details"]
            .as_array()
            .is_some_and(|details| details.iter().any(|detail| {
                detail["subject_id"] == "blk:author-smoke-added-keep-duplicate"
                    && detail["variant_kind"] == "content"
                    && detail["reason_kind"]
                        == "multiple-competing-alternatives-remain-after-keeping-primary-variant"
                    && detail["branch_kind"] == "multiple-competing-non-primary-additions"
                    && detail["competing_variants"] == json!([
                        "{\"attrs\":{},\"block_id\":\"blk:author-smoke-added-keep-duplicate\",\"block_type\":\"paragraph\",\"content\":\"right\"}",
                        "{\"attrs\":{},\"block_id\":\"blk:author-smoke-added-keep-duplicate\",\"block_type\":\"paragraph\",\"content\":\"right\"}"
                    ])
            })),
        "expected multiple competing duplicate content additions detail, got {merge_json}"
    );
    assert_eq!(merge_json["patch_op_count"], 0);
}

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
fn store_merge_authoring_flow_reports_selected_content_addition_with_competing_additions() {
    let store_dir = create_temp_dir("store-merge-content-selected-addition-competing-root");
    let (_key_dir, key_path) =
        write_signing_key_file("store-merge-content-selected-addition-competing-key");
    let (_resolved_dir, resolved_state_path) = write_content_entries_resolved_state_for_doc_file(
        "store-merge-content-selected-addition-competing-state",
        "doc:author-smoke-content-selected-addition-competing",
        &[("blk:author-smoke-added-choice", "right")],
    );
    let (_right_ops_dir, right_ops_path) = write_content_addition_ops_for_block_file(
        "store-merge-content-selected-addition-competing-right-ops",
        "blk:author-smoke-added-choice",
        "right",
    );
    let (_center_ops_dir, center_ops_path) = write_content_addition_ops_for_block_file(
        "store-merge-content-selected-addition-competing-center-ops",
        "blk:author-smoke-added-choice",
        "center",
    );
    let store_root = path_arg(store_dir.path());
    let key_file = path_arg(&key_path);
    let resolved_state_file = path_arg(&resolved_state_path);
    let right_ops_file = path_arg(&right_ops_path);
    let center_ops_file = path_arg(&center_ops_path);

    let init = run_mycel(&["store", "init", &store_root, "--json"]);
    assert_success(&init);

    let document = run_mycel(&[
        "store",
        "create-document",
        &store_root,
        "--doc-id",
        "doc:author-smoke-content-selected-addition-competing",
        "--title",
        "Author Smoke Content Selected Addition Competing",
        "--language",
        "en",
        "--signing-key",
        &key_file,
        "--timestamp",
        "44",
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
        "doc:author-smoke-content-selected-addition-competing",
        "--base-revision",
        &genesis_revision_id,
        "--ops",
        &right_ops_file,
        "--signing-key",
        &key_file,
        "--timestamp",
        "45",
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
        "doc:author-smoke-content-selected-addition-competing",
        "--parent",
        &genesis_revision_id,
        "--patch",
        &right_patch_id,
        "--signing-key",
        &key_file,
        "--timestamp",
        "46",
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
        "doc:author-smoke-content-selected-addition-competing",
        "--base-revision",
        &genesis_revision_id,
        "--ops",
        &center_ops_file,
        "--signing-key",
        &key_file,
        "--timestamp",
        "47",
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
        "doc:author-smoke-content-selected-addition-competing",
        "--parent",
        &genesis_revision_id,
        "--patch",
        &center_patch_id,
        "--signing-key",
        &key_file,
        "--timestamp",
        "48",
        "--json",
    ]);
    assert_success(&center_revision);
    let center_revision_id = assert_json_status(&center_revision, "ok")["revision_id"]
        .as_str()
        .expect("center revision_id should be string")
        .to_string();

    let merge = run_mycel(&[
        "store",
        "create-merge-revision",
        &store_root,
        "--doc-id",
        "doc:author-smoke-content-selected-addition-competing",
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
        "49",
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
                        "block 'blk:author-smoke-added-choice' adopted a non-primary parent addition",
                    )
                })
            })),
        "expected selected content addition reason, got {merge_json}"
    );
    assert!(
        merge_json["merge_reasons"]
            .as_array()
            .is_some_and(|reasons| reasons.iter().any(|reason| {
                    reason.as_str().is_some_and(|reason| {
                        reason.contains(
                        "block 'blk:author-smoke-added-choice' selected one non-primary addition while other competing non-primary additions remained",
                    )
                })
            })),
        "expected competing content addition reason, got {merge_json}"
    );
    assert!(
        merge_json["merge_reason_details"]
            .as_array()
            .is_some_and(|details| details.iter().any(|detail| {
                detail["subject_id"] == "blk:author-smoke-added-choice"
                    && detail["variant_kind"] == "content"
                    && detail["reason_kind"] == "selected-non-primary-parent-variant"
                    && detail["branch_kind"] == "adopted-non-primary-addition"
            })),
        "expected selected content addition detail, got {merge_json}"
    );
    assert!(
        merge_json["merge_reason_details"]
            .as_array()
            .is_some_and(|details| details.iter().any(|detail| {
                detail["subject_id"] == "blk:author-smoke-added-choice"
                    && detail["variant_kind"] == "content"
                    && detail["reason_kind"]
                        == "multiple-competing-alternatives-remain-after-selected-variant"
                    && detail["branch_kind"] == "multiple-competing-non-primary-additions"
                    && detail["competing_variants"]
                        .as_array()
                        .is_some_and(|variants| variants.len() == 2)
            })),
        "expected competing content addition detail, got {merge_json}"
    );
    assert_eq!(merge_json["patch_op_count"], 1);
}

#[test]
fn store_merge_authoring_flow_preserves_duplicate_non_primary_content_additions() {
    let store_dir = create_temp_dir("store-merge-content-duplicate-additions-root");
    let (_key_dir, key_path) =
        write_signing_key_file("store-merge-content-duplicate-additions-key");
    let (_resolved_dir, resolved_state_path) = write_content_entries_resolved_state_for_doc_file(
        "store-merge-content-duplicate-additions-state",
        "doc:author-smoke-content-duplicate-additions",
        &[("blk:author-smoke-added-duplicate", "right")],
    );
    let (_right_ops_dir, right_ops_path) = write_content_addition_ops_for_block_file(
        "store-merge-content-duplicate-additions-right-ops",
        "blk:author-smoke-added-duplicate",
        "right",
    );
    let (_center_ops_dir, center_ops_path) = write_content_addition_ops_for_block_file(
        "store-merge-content-duplicate-additions-center-ops",
        "blk:author-smoke-added-duplicate",
        "right",
    );
    let store_root = path_arg(store_dir.path());
    let key_file = path_arg(&key_path);
    let resolved_state_file = path_arg(&resolved_state_path);
    let right_ops_file = path_arg(&right_ops_path);
    let center_ops_file = path_arg(&center_ops_path);

    let init = run_mycel(&["store", "init", &store_root, "--json"]);
    assert_success(&init);

    let document = run_mycel(&[
        "store",
        "create-document",
        &store_root,
        "--doc-id",
        "doc:author-smoke-content-duplicate-additions",
        "--title",
        "Author Smoke Content Duplicate Additions",
        "--language",
        "en",
        "--signing-key",
        &key_file,
        "--timestamp",
        "50",
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
        "doc:author-smoke-content-duplicate-additions",
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
    let right_patch_id = assert_json_status(&right_patch, "ok")["patch_id"]
        .as_str()
        .expect("right patch_id should be string")
        .to_string();

    let right_revision = run_mycel(&[
        "store",
        "commit-revision",
        &store_root,
        "--doc-id",
        "doc:author-smoke-content-duplicate-additions",
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
    let right_revision_id = assert_json_status(&right_revision, "ok")["revision_id"]
        .as_str()
        .expect("right revision_id should be string")
        .to_string();

    let center_patch = run_mycel(&[
        "store",
        "create-patch",
        &store_root,
        "--doc-id",
        "doc:author-smoke-content-duplicate-additions",
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
    let center_patch_id = assert_json_status(&center_patch, "ok")["patch_id"]
        .as_str()
        .expect("center patch_id should be string")
        .to_string();

    let center_revision = run_mycel(&[
        "store",
        "commit-revision",
        &store_root,
        "--doc-id",
        "doc:author-smoke-content-duplicate-additions",
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
    let center_revision_id = assert_json_status(&center_revision, "ok")["revision_id"]
        .as_str()
        .expect("center revision_id should be string")
        .to_string();

    let merge = run_mycel(&[
        "store",
        "create-merge-revision",
        &store_root,
        "--doc-id",
        "doc:author-smoke-content-duplicate-additions",
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
        merge_json["merge_reason_details"].as_array().is_some_and(|details| details.iter().any(|detail| {
            detail["subject_id"] == "blk:author-smoke-added-duplicate"
                && detail["variant_kind"] == "content"
                && detail["reason_kind"] == "selected-non-primary-parent-variant"
                && detail["branch_kind"] == "adopted-non-primary-addition"
                && detail["competing_variants"].as_array().is_some_and(|variants| variants == &vec![json!("{\"attrs\":{},\"block_id\":\"blk:author-smoke-added-duplicate\",\"block_type\":\"paragraph\",\"content\":\"right\"}")])
        })),
        "expected selected duplicate content addition detail, got {merge_json}"
    );
    assert!(
        merge_json["merge_reason_details"].as_array().is_some_and(|details| details.iter().any(|detail| {
            detail["subject_id"] == "blk:author-smoke-added-duplicate"
                && detail["variant_kind"] == "content"
                && detail["reason_kind"] == "multiple-competing-alternatives-remain-after-selected-variant"
                && detail["branch_kind"] == "multiple-competing-non-primary-additions"
                && detail["competing_variants"] == json!([
                    "{\"attrs\":{},\"block_id\":\"blk:author-smoke-added-duplicate\",\"block_type\":\"paragraph\",\"content\":\"right\"}",
                    "{\"attrs\":{},\"block_id\":\"blk:author-smoke-added-duplicate\",\"block_type\":\"paragraph\",\"content\":\"right\"}"
                ])
        })),
        "expected duplicate competing content additions detail, got {merge_json}"
    );
    assert_eq!(merge_json["patch_op_count"], 1);
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

#[test]
fn store_merge_authoring_flow_reports_selected_metadata_addition_with_competing_additions() {
    let store_dir = create_temp_dir("store-merge-metadata-selected-addition-competing-root");
    let (_key_dir, key_path) =
        write_signing_key_file("store-merge-metadata-selected-addition-competing-key");
    let (_resolved_dir, resolved_state_path) = write_metadata_variant_resolved_state_for_doc_file(
        "store-merge-metadata-selected-addition-competing-state",
        "doc:author-smoke-metadata-selected-addition-competing",
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
    let store_root = path_arg(store_dir.path());
    let key_file = path_arg(&key_path);
    let resolved_state_file = path_arg(&resolved_state_path);
    let right_ops_file = path_arg(&right_ops_path);
    let center_ops_file = path_arg(&center_ops_path);

    let init = run_mycel(&["store", "init", &store_root, "--json"]);
    assert_success(&init);

    let document = run_mycel(&[
        "store",
        "create-document",
        &store_root,
        "--doc-id",
        "doc:author-smoke-metadata-selected-addition-competing",
        "--title",
        "Author Smoke Metadata Selected Addition Competing",
        "--language",
        "en",
        "--signing-key",
        &key_file,
        "--timestamp",
        "56",
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
        "doc:author-smoke-metadata-selected-addition-competing",
        "--base-revision",
        &genesis_revision_id,
        "--ops",
        &right_ops_file,
        "--signing-key",
        &key_file,
        "--timestamp",
        "57",
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
        "doc:author-smoke-metadata-selected-addition-competing",
        "--parent",
        &genesis_revision_id,
        "--patch",
        &right_patch_id,
        "--signing-key",
        &key_file,
        "--timestamp",
        "58",
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
        "doc:author-smoke-metadata-selected-addition-competing",
        "--base-revision",
        &genesis_revision_id,
        "--ops",
        &center_ops_file,
        "--signing-key",
        &key_file,
        "--timestamp",
        "59",
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
        "doc:author-smoke-metadata-selected-addition-competing",
        "--parent",
        &genesis_revision_id,
        "--patch",
        &center_patch_id,
        "--signing-key",
        &key_file,
        "--timestamp",
        "60",
        "--json",
    ]);
    assert_success(&center_revision);
    let center_revision_id = assert_json_status(&center_revision, "ok")["revision_id"]
        .as_str()
        .expect("center revision_id should be string")
        .to_string();

    let merge = run_mycel(&[
        "store",
        "create-merge-revision",
        &store_root,
        "--doc-id",
        "doc:author-smoke-metadata-selected-addition-competing",
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
        "61",
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
    let store_dir = create_temp_dir("store-merge-metadata-duplicate-additions-root");
    let (_key_dir, key_path) =
        write_signing_key_file("store-merge-metadata-duplicate-additions-key");
    let (_resolved_dir, resolved_state_path) = write_metadata_entries_resolved_state_for_doc_file(
        "store-merge-metadata-duplicate-additions-state",
        "doc:author-smoke-metadata-duplicate-additions",
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
    let store_root = path_arg(store_dir.path());
    let key_file = path_arg(&key_path);
    let resolved_state_file = path_arg(&resolved_state_path);
    let right_ops_file = path_arg(&right_ops_path);
    let center_ops_file = path_arg(&center_ops_path);

    let init = run_mycel(&["store", "init", &store_root, "--json"]);
    assert_success(&init);

    let document = run_mycel(&[
        "store",
        "create-document",
        &store_root,
        "--doc-id",
        "doc:author-smoke-metadata-duplicate-additions",
        "--title",
        "Author Smoke Metadata Duplicate Additions",
        "--language",
        "en",
        "--signing-key",
        &key_file,
        "--timestamp",
        "62",
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
        "doc:author-smoke-metadata-duplicate-additions",
        "--base-revision",
        &genesis_revision_id,
        "--ops",
        &right_ops_file,
        "--signing-key",
        &key_file,
        "--timestamp",
        "63",
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
        "doc:author-smoke-metadata-duplicate-additions",
        "--parent",
        &genesis_revision_id,
        "--patch",
        &right_patch_id,
        "--signing-key",
        &key_file,
        "--timestamp",
        "64",
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
        "doc:author-smoke-metadata-duplicate-additions",
        "--base-revision",
        &genesis_revision_id,
        "--ops",
        &center_ops_file,
        "--signing-key",
        &key_file,
        "--timestamp",
        "65",
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
        "doc:author-smoke-metadata-duplicate-additions",
        "--parent",
        &genesis_revision_id,
        "--patch",
        &center_patch_id,
        "--signing-key",
        &key_file,
        "--timestamp",
        "66",
        "--json",
    ]);
    assert_success(&center_revision);
    let center_revision_id = assert_json_status(&center_revision, "ok")["revision_id"]
        .as_str()
        .expect("center revision_id should be string")
        .to_string();

    let merge = run_mycel(&[
        "store",
        "create-merge-revision",
        &store_root,
        "--doc-id",
        "doc:author-smoke-metadata-duplicate-additions",
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
        "67",
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

