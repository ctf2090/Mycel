use super::*;

fn assert_content_variant_merge_reasons(merge_json: &serde_json::Value) {
    assert_eq!(merge_json["merge_outcome"], "multi-variant");
    assert!(
        merge_json["merge_reasons"]
            .as_array()
            .is_some_and(|reasons| reasons.iter().any(|reason| {
                reason.as_str().is_some_and(|reason| {
                    reason.contains("adopted a non-primary parent replacement")
                })
            })),
        "expected content variant multi-variant reason, got {merge_json}"
    );
    assert!(
        merge_json["merge_reasons"]
            .as_array()
            .is_some_and(|reasons| reasons.iter().any(|reason| {
                reason.as_str().is_some_and(|reason| {
                    reason.contains(
                        "selected one non-primary replacement while other competing non-primary replacements remained",
                    )
                })
            })),
        "expected competing content variant reason, got {merge_json}"
    );
    assert!(
        merge_json["merge_reason_details"]
            .as_array()
            .is_some_and(|details| details.iter().any(|detail| {
                detail["subject_kind"] == "block"
                    && detail["subject_id"] == "blk:author-smoke-variant-001"
                    && detail["variant_kind"] == "content"
                    && detail["reason_kind"] == "selected-non-primary-parent-variant"
                    && detail["branch_kind"] == "adopted-non-primary-replacement"
                    && detail["resolved_variant"]
                        .as_str()
                        .is_some_and(|variant| variant.contains("Right variant"))
                    && detail["competing_variants"]
                        .as_array()
                        .is_some_and(|variants| variants.len() == 1)
            })),
        "expected structured content variant detail, got {merge_json}"
    );
    assert!(
        merge_json["merge_reason_details"]
            .as_array()
            .is_some_and(|details| details.iter().any(|detail| {
                detail["subject_kind"] == "block"
                    && detail["subject_id"] == "blk:author-smoke-variant-001"
                    && detail["variant_kind"] == "content"
                    && detail["reason_kind"]
                        == "multiple-competing-alternatives-remain-after-selected-variant"
                    && detail["branch_kind"] == "multiple-competing-non-primary-replacements"
                    && detail["competing_variants"]
                        .as_array()
                        .is_some_and(|variants| variants.len() == 2)
            })),
        "expected competing content branch kind detail, got {merge_json}"
    );
    assert_eq!(merge_json["patch_op_count"], 1);
    assert_eq!(
        merge_json["parent_revision_ids"].as_array().map(Vec::len),
        Some(3)
    );
}

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
    let store_dir = create_temp_dir("store-merge-content-duplicate-non-primary-root");
    let (_key_dir, key_path) =
        write_signing_key_file("store-merge-content-duplicate-non-primary-key");
    let (_resolved_dir, resolved_state_path) = write_content_variant_resolved_state_file(
        "store-merge-content-duplicate-non-primary-state",
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
    let store_root = path_arg(store_dir.path());
    let key_file = path_arg(&key_path);
    let resolved_state_file = path_arg(&resolved_state_path);
    let left_ops_file = path_arg(&left_ops_path);
    let right_ops_file = path_arg(&right_ops_path);
    let center_ops_file = path_arg(&center_ops_path);

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
        "48",
        "--json",
    ]);
    assert_success(&document);
    let genesis_revision_id = assert_json_status(&document, "ok")["genesis_revision_id"]
        .as_str()
        .expect("genesis revision should be string")
        .to_string();

    let base_ops_dir = create_temp_dir("store-merge-content-duplicate-non-primary-base-ops");
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
        .expect("duplicate content base ops JSON should serialize"),
    )
    .expect("duplicate content base ops JSON should write");
    let base_ops_file = path_arg(&base_ops_path);

    let base_patch = run_mycel(&[
        "store",
        "create-patch",
        &store_root,
        "--doc-id",
        "doc:author-smoke-content-variant",
        "--base-revision",
        &genesis_revision_id,
        "--ops",
        &base_ops_file,
        "--signing-key",
        &key_file,
        "--timestamp",
        "49",
        "--json",
    ]);
    assert_success(&base_patch);
    let base_patch_id = assert_json_status(&base_patch, "ok")["patch_id"]
        .as_str()
        .expect("base patch_id should be string")
        .to_string();

    let base_revision = run_mycel(&[
        "store",
        "commit-revision",
        &store_root,
        "--doc-id",
        "doc:author-smoke-content-variant",
        "--parent",
        &genesis_revision_id,
        "--patch",
        &base_patch_id,
        "--signing-key",
        &key_file,
        "--timestamp",
        "50",
        "--json",
    ]);
    assert_success(&base_revision);
    let base_revision_id = assert_json_status(&base_revision, "ok")["revision_id"]
        .as_str()
        .expect("base revision_id should be string")
        .to_string();

    let left_patch = run_mycel(&[
        "store",
        "create-patch",
        &store_root,
        "--doc-id",
        "doc:author-smoke-content-variant",
        "--base-revision",
        &base_revision_id,
        "--ops",
        &left_ops_file,
        "--signing-key",
        &key_file,
        "--timestamp",
        "51",
        "--json",
    ]);
    assert_success(&left_patch);
    let left_patch_id = assert_json_status(&left_patch, "ok")["patch_id"]
        .as_str()
        .expect("left patch_id should be string")
        .to_string();

    let left_revision = run_mycel(&[
        "store",
        "commit-revision",
        &store_root,
        "--doc-id",
        "doc:author-smoke-content-variant",
        "--parent",
        &base_revision_id,
        "--patch",
        &left_patch_id,
        "--signing-key",
        &key_file,
        "--timestamp",
        "52",
        "--json",
    ]);
    assert_success(&left_revision);
    let left_revision_id = assert_json_status(&left_revision, "ok")["revision_id"]
        .as_str()
        .expect("left revision_id should be string")
        .to_string();

    let right_patch = run_mycel(&[
        "store",
        "create-patch",
        &store_root,
        "--doc-id",
        "doc:author-smoke-content-variant",
        "--base-revision",
        &left_revision_id,
        "--ops",
        &right_ops_file,
        "--signing-key",
        &key_file,
        "--timestamp",
        "53",
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
        "doc:author-smoke-content-variant",
        "--parent",
        &left_revision_id,
        "--patch",
        &right_patch_id,
        "--signing-key",
        &key_file,
        "--timestamp",
        "54",
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
        "doc:author-smoke-content-variant",
        "--base-revision",
        &left_revision_id,
        "--ops",
        &center_ops_file,
        "--signing-key",
        &key_file,
        "--timestamp",
        "55",
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
        "doc:author-smoke-content-variant",
        "--parent",
        &left_revision_id,
        "--patch",
        &center_patch_id,
        "--signing-key",
        &key_file,
        "--timestamp",
        "56",
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
        "doc:author-smoke-content-variant",
        "--parent",
        &left_revision_id,
        "--parent",
        &right_revision_id,
        "--parent",
        &center_revision_id,
        "--resolved-state",
        &resolved_state_file,
        "--signing-key",
        &key_file,
        "--timestamp",
        "57",
        "--json",
    ]);
    assert_success(&merge);
    let merge_json = assert_json_status(&merge, "ok");
    assert_eq!(merge_json["merge_outcome"], "multi-variant");
    assert!(
        merge_json["merge_reason_details"]
            .as_array()
            .is_some_and(|details| details.iter().any(|detail| {
                detail["subject_id"] == "blk:author-smoke-variant-001"
                    && detail["variant_kind"] == "content"
                    && detail["reason_kind"] == "selected-non-primary-parent-variant"
                    && detail["competing_variants"]
                        .as_array()
                        .is_some_and(|variants| {
                            variants.len() == 1
                                && variants.iter().all(|variant| {
                                    variant.as_str().is_some_and(|variant| {
                                        variant.contains("\"content\":\"right\"")
                                    })
                                })
                        })
            })),
        "expected selected duplicate content replacement detail, got {merge_json}"
    );
    assert!(
        merge_json["merge_reason_details"]
            .as_array()
            .is_some_and(|details| details.iter().any(|detail| {
                detail["subject_id"] == "blk:author-smoke-variant-001"
                    && detail["variant_kind"] == "content"
                    && detail["reason_kind"]
                        == "multiple-competing-alternatives-remain-after-selected-variant"
                    && detail["branch_kind"] == "multiple-competing-non-primary-replacements"
                    && detail["competing_variants"]
                        .as_array()
                        .is_some_and(|variants| {
                            variants.len() == 2
                                && variants.iter().all(|variant| {
                                    variant.as_str().is_some_and(|variant| {
                                        variant.contains("\"content\":\"right\"")
                                    })
                                })
                        })
            })),
        "expected duplicate competing content replacements detail, got {merge_json}"
    );
}

#[test]
fn store_merge_authoring_flow_preserves_duplicate_non_primary_metadata_replacements() {
    let store_dir = create_temp_dir("store-merge-metadata-duplicate-non-primary-root");
    let (_key_dir, key_path) =
        write_signing_key_file("store-merge-metadata-duplicate-non-primary-key");
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
    let store_root = path_arg(store_dir.path());
    let key_file = path_arg(&key_path);
    let resolved_state_file = path_arg(&resolved_state_path);
    let left_ops_file = path_arg(&left_ops_path);
    let right_ops_file = path_arg(&right_ops_path);
    let center_ops_file = path_arg(&center_ops_path);

    let init = run_mycel(&["store", "init", &store_root, "--json"]);
    assert_success(&init);

    let document = run_mycel(&[
        "store",
        "create-document",
        &store_root,
        "--doc-id",
        "doc:author-smoke-metadata-variant",
        "--title",
        "Author Smoke Metadata Variant",
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

    let left_patch = run_mycel(&[
        "store",
        "create-patch",
        &store_root,
        "--doc-id",
        "doc:author-smoke-metadata-variant",
        "--base-revision",
        &genesis_revision_id,
        "--ops",
        &left_ops_file,
        "--signing-key",
        &key_file,
        "--timestamp",
        "57",
        "--json",
    ]);
    assert_success(&left_patch);
    let left_patch_id = assert_json_status(&left_patch, "ok")["patch_id"]
        .as_str()
        .expect("left patch_id should be string")
        .to_string();

    let left_revision = run_mycel(&[
        "store",
        "commit-revision",
        &store_root,
        "--doc-id",
        "doc:author-smoke-metadata-variant",
        "--parent",
        &genesis_revision_id,
        "--patch",
        &left_patch_id,
        "--signing-key",
        &key_file,
        "--timestamp",
        "58",
        "--json",
    ]);
    assert_success(&left_revision);
    let left_revision_id = assert_json_status(&left_revision, "ok")["revision_id"]
        .as_str()
        .expect("left revision_id should be string")
        .to_string();

    let right_patch = run_mycel(&[
        "store",
        "create-patch",
        &store_root,
        "--doc-id",
        "doc:author-smoke-metadata-variant",
        "--base-revision",
        &genesis_revision_id,
        "--ops",
        &right_ops_file,
        "--signing-key",
        &key_file,
        "--timestamp",
        "59",
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
        "doc:author-smoke-metadata-variant",
        "--parent",
        &genesis_revision_id,
        "--patch",
        &right_patch_id,
        "--signing-key",
        &key_file,
        "--timestamp",
        "60",
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
        "doc:author-smoke-metadata-variant",
        "--base-revision",
        &genesis_revision_id,
        "--ops",
        &center_ops_file,
        "--signing-key",
        &key_file,
        "--timestamp",
        "61",
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
        "doc:author-smoke-metadata-variant",
        "--parent",
        &genesis_revision_id,
        "--patch",
        &center_patch_id,
        "--signing-key",
        &key_file,
        "--timestamp",
        "62",
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
        "doc:author-smoke-metadata-variant",
        "--parent",
        &left_revision_id,
        "--parent",
        &right_revision_id,
        "--parent",
        &center_revision_id,
        "--resolved-state",
        &resolved_state_file,
        "--signing-key",
        &key_file,
        "--timestamp",
        "63",
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
                    && detail["competing_variants"] == json!(["\"right\""])
            })),
        "expected selected duplicate metadata replacement detail, got {merge_json}"
    );
    assert!(
        merge_json["merge_reason_details"]
            .as_array()
            .is_some_and(|details| details.iter().any(|detail| {
                detail["subject_id"] == "topic"
                    && detail["variant_kind"] == "metadata"
                    && detail["reason_kind"]
                        == "multiple-competing-alternatives-remain-after-selected-variant"
                    && detail["branch_kind"] == "multiple-competing-non-primary-replacements"
                    && detail["competing_variants"] == json!(["\"right\"", "\"right\""])
            })),
        "expected duplicate competing metadata replacements detail, got {merge_json}"
    );
}

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

#[test]
fn store_merge_authoring_flow_reports_mixed_content_replacement_and_removal_branches() {
    let store_dir = create_temp_dir("store-merge-content-mixed-replace-remove-root");
    let (_key_dir, key_path) =
        write_signing_key_file("store-merge-content-mixed-replace-remove-key");
    let (_resolved_dir, resolved_state_path) = write_content_entries_resolved_state_for_doc_file(
        "store-merge-content-mixed-replace-remove-state",
        "doc:author-smoke-content-mixed-replace-remove",
        &[("blk:author-smoke-mixed-001", "Base")],
    );
    let (_replace_ops_dir, replace_ops_path) = write_content_variant_ops_for_block_file(
        "store-merge-content-mixed-replace-remove-replace-ops",
        "blk:author-smoke-mixed-001",
        "Right",
    );
    let (_delete_ops_dir, delete_ops_path) = write_content_delete_ops_for_block_file(
        "store-merge-content-mixed-replace-remove-delete-ops",
        "blk:author-smoke-mixed-001",
    );
    let store_root = path_arg(store_dir.path());
    let key_file = path_arg(&key_path);
    let resolved_state_file = path_arg(&resolved_state_path);
    let replace_ops_file = path_arg(&replace_ops_path);
    let delete_ops_file = path_arg(&delete_ops_path);

    let init = run_mycel(&["store", "init", &store_root, "--json"]);
    assert_success(&init);

    let document = run_mycel(&[
        "store",
        "create-document",
        &store_root,
        "--doc-id",
        "doc:author-smoke-content-mixed-replace-remove",
        "--title",
        "Author Smoke Content Mixed Replace Remove",
        "--language",
        "en",
        "--signing-key",
        &key_file,
        "--timestamp",
        "70",
        "--json",
    ]);
    assert_success(&document);
    let document_json = assert_json_status(&document, "ok");
    let genesis_revision_id = document_json["genesis_revision_id"]
        .as_str()
        .expect("genesis revision should be string")
        .to_string();

    let base_ops_dir = create_temp_dir("store-merge-content-mixed-replace-remove-base-ops");
    let base_ops_path = base_ops_dir.path().join("ops.json");
    fs::write(
        &base_ops_path,
        serde_json::to_string_pretty(&json!([
            {
                "op": "insert_block",
                "new_block": {
                    "block_id": "blk:author-smoke-mixed-001",
                    "block_type": "paragraph",
                    "content": "Base",
                    "attrs": {},
                    "children": []
                }
            }
        ]))
        .expect("mixed replace/remove base ops JSON should serialize"),
    )
    .expect("mixed replace/remove base ops JSON should write");
    let base_ops_file = path_arg(&base_ops_path);

    let base_patch = run_mycel(&[
        "store",
        "create-patch",
        &store_root,
        "--doc-id",
        "doc:author-smoke-content-mixed-replace-remove",
        "--base-revision",
        &genesis_revision_id,
        "--ops",
        &base_ops_file,
        "--signing-key",
        &key_file,
        "--timestamp",
        "71",
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
        "doc:author-smoke-content-mixed-replace-remove",
        "--parent",
        &genesis_revision_id,
        "--patch",
        &base_patch_id,
        "--signing-key",
        &key_file,
        "--timestamp",
        "72",
        "--json",
    ]);
    assert_success(&base_revision);
    let base_revision_json = assert_json_status(&base_revision, "ok");
    let base_revision_id = base_revision_json["revision_id"]
        .as_str()
        .expect("base revision_id should be string")
        .to_string();

    let replace_patch = run_mycel(&[
        "store",
        "create-patch",
        &store_root,
        "--doc-id",
        "doc:author-smoke-content-mixed-replace-remove",
        "--base-revision",
        &base_revision_id,
        "--ops",
        &replace_ops_file,
        "--signing-key",
        &key_file,
        "--timestamp",
        "73",
        "--json",
    ]);
    assert_success(&replace_patch);
    let replace_patch_json = assert_json_status(&replace_patch, "ok");
    let replace_patch_id = replace_patch_json["patch_id"]
        .as_str()
        .expect("replace patch_id should be string")
        .to_string();

    let replace_revision = run_mycel(&[
        "store",
        "commit-revision",
        &store_root,
        "--doc-id",
        "doc:author-smoke-content-mixed-replace-remove",
        "--parent",
        &base_revision_id,
        "--patch",
        &replace_patch_id,
        "--signing-key",
        &key_file,
        "--timestamp",
        "74",
        "--json",
    ]);
    assert_success(&replace_revision);
    let replace_revision_json = assert_json_status(&replace_revision, "ok");
    let replace_revision_id = replace_revision_json["revision_id"]
        .as_str()
        .expect("replace revision_id should be string")
        .to_string();

    let delete_patch = run_mycel(&[
        "store",
        "create-patch",
        &store_root,
        "--doc-id",
        "doc:author-smoke-content-mixed-replace-remove",
        "--base-revision",
        &base_revision_id,
        "--ops",
        &delete_ops_file,
        "--signing-key",
        &key_file,
        "--timestamp",
        "75",
        "--json",
    ]);
    assert_success(&delete_patch);
    let delete_patch_json = assert_json_status(&delete_patch, "ok");
    let delete_patch_id = delete_patch_json["patch_id"]
        .as_str()
        .expect("delete patch_id should be string")
        .to_string();

    let delete_revision = run_mycel(&[
        "store",
        "commit-revision",
        &store_root,
        "--doc-id",
        "doc:author-smoke-content-mixed-replace-remove",
        "--parent",
        &base_revision_id,
        "--patch",
        &delete_patch_id,
        "--signing-key",
        &key_file,
        "--timestamp",
        "76",
        "--json",
    ]);
    assert_success(&delete_revision);
    let delete_revision_json = assert_json_status(&delete_revision, "ok");
    let delete_revision_id = delete_revision_json["revision_id"]
        .as_str()
        .expect("delete revision_id should be string")
        .to_string();

    let merge = run_mycel(&[
        "store",
        "create-merge-revision",
        &store_root,
        "--doc-id",
        "doc:author-smoke-content-mixed-replace-remove",
        "--parent",
        &base_revision_id,
        "--parent",
        &replace_revision_id,
        "--parent",
        &delete_revision_id,
        "--resolved-state",
        &resolved_state_file,
        "--signing-key",
        &key_file,
        "--timestamp",
        "77",
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
                        "block 'blk:author-smoke-mixed-001' kept the primary parent variant over mixed competing non-primary alternatives",
                    )
                })
            })),
        "expected mixed keep-primary reason, got {merge_json}"
    );
    assert!(
        merge_json["merge_reasons"]
            .as_array()
            .is_some_and(|reasons| reasons.iter().any(|reason| {
                reason.as_str().is_some_and(|reason| {
                    reason.contains(
                        "block 'blk:author-smoke-mixed-001' kept the primary variant while multiple competing non-primary replacements and removals remained",
                    )
                })
            })),
        "expected mixed multiple-competing reason, got {merge_json}"
    );
    assert!(
        merge_json["merge_reason_details"]
            .as_array()
            .is_some_and(|details| details.iter().any(|detail| {
                detail["subject_id"] == "blk:author-smoke-mixed-001"
                    && detail["variant_kind"] == "content"
                    && detail["reason_kind"]
                        == "kept-primary-parent-variant-over-competing-non-primary-alternative"
                    && detail["branch_kind"]
                        == "kept-primary-variant-over-mixed-non-primary-alternatives"
            })),
        "expected mixed keep-primary branch kind detail, got {merge_json}"
    );
    assert!(
        merge_json["merge_reason_details"]
            .as_array()
            .is_some_and(|details| details.iter().any(|detail| {
                detail["subject_id"] == "blk:author-smoke-mixed-001"
                    && detail["variant_kind"] == "content"
                    && detail["reason_kind"]
                        == "multiple-competing-alternatives-remain-after-keeping-primary-variant"
                    && detail["branch_kind"] == "multiple-competing-mixed-non-primary-alternatives"
            })),
        "expected mixed competing branch kind detail, got {merge_json}"
    );
    assert_eq!(merge_json["patch_op_count"], 0);
}

#[test]
fn store_merge_authoring_flow_reports_selected_content_removal_with_competing_removals() {
    let store_dir = create_temp_dir("store-merge-content-select-removal-competing-root");
    let (_key_dir, key_path) =
        write_signing_key_file("store-merge-content-select-removal-competing-key");
    let (_resolved_dir, resolved_state_path) = write_content_entries_resolved_state_for_doc_file(
        "store-merge-content-select-removal-competing-state",
        "doc:author-smoke-content-select-removal-competing",
        &[("blk:author-smoke-unrelated", "Unrelated")],
    );
    let (_unrelated_ops_dir, unrelated_ops_path) = write_content_addition_ops_for_block_file(
        "store-merge-content-select-removal-competing-unrelated-ops",
        "blk:author-smoke-unrelated",
        "Unrelated",
    );
    let store_root = path_arg(store_dir.path());
    let key_file = path_arg(&key_path);
    let resolved_state_file = path_arg(&resolved_state_path);
    let unrelated_ops_file = path_arg(&unrelated_ops_path);

    let init = run_mycel(&["store", "init", &store_root, "--json"]);
    assert_success(&init);

    let document = run_mycel(&[
        "store",
        "create-document",
        &store_root,
        "--doc-id",
        "doc:author-smoke-content-select-removal-competing",
        "--title",
        "Author Smoke Content Select Removal Competing",
        "--language",
        "en",
        "--signing-key",
        &key_file,
        "--timestamp",
        "92",
        "--json",
    ]);
    assert_success(&document);
    let genesis_revision_id = assert_json_status(&document, "ok")["genesis_revision_id"]
        .as_str()
        .expect("genesis revision should be string")
        .to_string();

    let base_ops_dir = create_temp_dir("store-merge-content-select-removal-competing-base-ops");
    let base_ops_path = base_ops_dir.path().join("ops.json");
    fs::write(
        &base_ops_path,
        serde_json::to_string_pretty(&json!([
            {
                "op": "insert_block",
                "new_block": {
                    "block_id": "blk:author-smoke-remove-choice",
                    "block_type": "paragraph",
                    "content": "Base",
                    "attrs": {},
                    "children": []
                }
            }
        ]))
        .expect("selected removal base ops JSON should serialize"),
    )
    .expect("selected removal base ops JSON should write");
    let base_ops_file = path_arg(&base_ops_path);

    let base_patch = run_mycel(&[
        "store",
        "create-patch",
        &store_root,
        "--doc-id",
        "doc:author-smoke-content-select-removal-competing",
        "--base-revision",
        &genesis_revision_id,
        "--ops",
        &base_ops_file,
        "--signing-key",
        &key_file,
        "--timestamp",
        "93",
        "--json",
    ]);
    assert_success(&base_patch);
    let base_patch_id = assert_json_status(&base_patch, "ok")["patch_id"]
        .as_str()
        .expect("base patch_id should be string")
        .to_string();

    let base_revision = run_mycel(&[
        "store",
        "commit-revision",
        &store_root,
        "--doc-id",
        "doc:author-smoke-content-select-removal-competing",
        "--parent",
        &genesis_revision_id,
        "--patch",
        &base_patch_id,
        "--signing-key",
        &key_file,
        "--timestamp",
        "94",
        "--json",
    ]);
    assert_success(&base_revision);
    let base_revision_id = assert_json_status(&base_revision, "ok")["revision_id"]
        .as_str()
        .expect("base revision_id should be string")
        .to_string();

    let unrelated_patch = run_mycel(&[
        "store",
        "create-patch",
        &store_root,
        "--doc-id",
        "doc:author-smoke-content-select-removal-competing",
        "--base-revision",
        &genesis_revision_id,
        "--ops",
        &unrelated_ops_file,
        "--signing-key",
        &key_file,
        "--timestamp",
        "95",
        "--json",
    ]);
    assert_success(&unrelated_patch);
    let unrelated_patch_id = assert_json_status(&unrelated_patch, "ok")["patch_id"]
        .as_str()
        .expect("unrelated patch_id should be string")
        .to_string();

    let unrelated_revision = run_mycel(&[
        "store",
        "commit-revision",
        &store_root,
        "--doc-id",
        "doc:author-smoke-content-select-removal-competing",
        "--parent",
        &genesis_revision_id,
        "--patch",
        &unrelated_patch_id,
        "--signing-key",
        &key_file,
        "--timestamp",
        "96",
        "--json",
    ]);
    assert_success(&unrelated_revision);
    let unrelated_revision_id = assert_json_status(&unrelated_revision, "ok")["revision_id"]
        .as_str()
        .expect("unrelated revision_id should be string")
        .to_string();

    let merge = run_mycel(&[
        "store",
        "create-merge-revision",
        &store_root,
        "--doc-id",
        "doc:author-smoke-content-select-removal-competing",
        "--parent",
        &base_revision_id,
        "--parent",
        &genesis_revision_id,
        "--parent",
        &unrelated_revision_id,
        "--resolved-state",
        &resolved_state_file,
        "--signing-key",
        &key_file,
        "--timestamp",
        "97",
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
                        "block 'blk:author-smoke-remove-choice' adopted a non-primary parent removal",
                    )
                })
            })),
        "expected selected content removal reason, got {merge_json}"
    );
    assert!(
        merge_json["merge_reasons"]
            .as_array()
            .is_some_and(|reasons| reasons.iter().any(|reason| {
                reason.as_str().is_some_and(|reason| {
                    reason.contains(
                        "block 'blk:author-smoke-remove-choice' selected one non-primary removal while other competing non-primary removals remained",
                    )
                })
            })),
        "expected competing content removal reason, got {merge_json}"
    );
    assert!(
        merge_json["merge_reason_details"]
            .as_array()
            .is_some_and(|details| details.iter().any(|detail| {
                detail["subject_id"] == "blk:author-smoke-remove-choice"
                    && detail["variant_kind"] == "content"
                    && detail["reason_kind"] == "selected-non-primary-parent-variant"
                    && detail["branch_kind"] == "adopted-non-primary-removal"
                    && detail["competing_variants"] == json!(["<absent>"])
            })),
        "expected selected content removal detail, got {merge_json}"
    );
    assert!(
        merge_json["merge_reason_details"]
            .as_array()
            .is_some_and(|details| details.iter().any(|detail| {
                detail["subject_id"] == "blk:author-smoke-remove-choice"
                    && detail["variant_kind"] == "content"
                    && detail["reason_kind"]
                        == "multiple-competing-alternatives-remain-after-selected-variant"
                    && detail["branch_kind"] == "multiple-competing-non-primary-removals"
                    && detail["competing_variants"] == json!(["<absent>", "<absent>"])
            })),
        "expected multiple competing content removals detail, got {merge_json}"
    );
    assert_eq!(merge_json["patch_op_count"], 2);
}

#[test]
fn store_merge_authoring_flow_reports_kept_primary_content_over_multiple_removals() {
    let store_dir = create_temp_dir("store-merge-content-keep-primary-removals-root");
    let (_key_dir, key_path) =
        write_signing_key_file("store-merge-content-keep-primary-removals-key");
    let (_resolved_dir, resolved_state_path) = write_content_entries_resolved_state_for_doc_file(
        "store-merge-content-keep-primary-removals-state",
        "doc:author-smoke-content-keep-primary-removals",
        &[
            ("blk:author-smoke-remove-keep", "Base"),
            ("blk:author-smoke-other", "Other"),
        ],
    );
    let (_other_ops_dir, other_ops_path) = write_content_addition_ops_for_block_file(
        "store-merge-content-keep-primary-removals-other-ops",
        "blk:author-smoke-other",
        "Other",
    );
    let store_root = path_arg(store_dir.path());
    let key_file = path_arg(&key_path);
    let resolved_state_file = path_arg(&resolved_state_path);
    let other_ops_file = path_arg(&other_ops_path);

    let init = run_mycel(&["store", "init", &store_root, "--json"]);
    assert_success(&init);

    let document = run_mycel(&[
        "store",
        "create-document",
        &store_root,
        "--doc-id",
        "doc:author-smoke-content-keep-primary-removals",
        "--title",
        "Author Smoke Content Keep Primary Removals",
        "--language",
        "en",
        "--signing-key",
        &key_file,
        "--timestamp",
        "98",
        "--json",
    ]);
    assert_success(&document);
    let genesis_revision_id = assert_json_status(&document, "ok")["genesis_revision_id"]
        .as_str()
        .expect("genesis revision should be string")
        .to_string();

    let base_ops_dir = create_temp_dir("store-merge-content-keep-primary-removals-base-ops");
    let base_ops_path = base_ops_dir.path().join("ops.json");
    fs::write(
        &base_ops_path,
        serde_json::to_string_pretty(&json!([
            {
                "op": "insert_block",
                "new_block": {
                    "block_id": "blk:author-smoke-remove-keep",
                    "block_type": "paragraph",
                    "content": "Base",
                    "attrs": {},
                    "children": []
                }
            }
        ]))
        .expect("keep-primary removals base ops JSON should serialize"),
    )
    .expect("keep-primary removals base ops JSON should write");
    let base_ops_file = path_arg(&base_ops_path);

    let base_patch = run_mycel(&[
        "store",
        "create-patch",
        &store_root,
        "--doc-id",
        "doc:author-smoke-content-keep-primary-removals",
        "--base-revision",
        &genesis_revision_id,
        "--ops",
        &base_ops_file,
        "--signing-key",
        &key_file,
        "--timestamp",
        "99",
        "--json",
    ]);
    assert_success(&base_patch);
    let base_patch_id = assert_json_status(&base_patch, "ok")["patch_id"]
        .as_str()
        .expect("base patch_id should be string")
        .to_string();

    let base_revision = run_mycel(&[
        "store",
        "commit-revision",
        &store_root,
        "--doc-id",
        "doc:author-smoke-content-keep-primary-removals",
        "--parent",
        &genesis_revision_id,
        "--patch",
        &base_patch_id,
        "--signing-key",
        &key_file,
        "--timestamp",
        "100",
        "--json",
    ]);
    assert_success(&base_revision);
    let base_revision_id = assert_json_status(&base_revision, "ok")["revision_id"]
        .as_str()
        .expect("base revision_id should be string")
        .to_string();

    let other_patch = run_mycel(&[
        "store",
        "create-patch",
        &store_root,
        "--doc-id",
        "doc:author-smoke-content-keep-primary-removals",
        "--base-revision",
        &genesis_revision_id,
        "--ops",
        &other_ops_file,
        "--signing-key",
        &key_file,
        "--timestamp",
        "101",
        "--json",
    ]);
    assert_success(&other_patch);
    let other_patch_id = assert_json_status(&other_patch, "ok")["patch_id"]
        .as_str()
        .expect("other patch_id should be string")
        .to_string();

    let other_revision = run_mycel(&[
        "store",
        "commit-revision",
        &store_root,
        "--doc-id",
        "doc:author-smoke-content-keep-primary-removals",
        "--parent",
        &genesis_revision_id,
        "--patch",
        &other_patch_id,
        "--signing-key",
        &key_file,
        "--timestamp",
        "102",
        "--json",
    ]);
    assert_success(&other_revision);
    let other_revision_id = assert_json_status(&other_revision, "ok")["revision_id"]
        .as_str()
        .expect("other revision_id should be string")
        .to_string();

    let merge = run_mycel(&[
        "store",
        "create-merge-revision",
        &store_root,
        "--doc-id",
        "doc:author-smoke-content-keep-primary-removals",
        "--parent",
        &base_revision_id,
        "--parent",
        &genesis_revision_id,
        "--parent",
        &other_revision_id,
        "--resolved-state",
        &resolved_state_file,
        "--signing-key",
        &key_file,
        "--timestamp",
        "103",
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
                        "block 'blk:author-smoke-remove-keep' kept the primary parent variant over a competing non-primary removal",
                    )
                })
            })),
        "expected keep-primary content removal reason, got {merge_json}"
    );
    assert!(
        merge_json["merge_reasons"]
            .as_array()
            .is_some_and(|reasons| reasons.iter().any(|reason| {
                reason.as_str().is_some_and(|reason| {
                    reason.contains(
                        "block 'blk:author-smoke-remove-keep' kept the primary variant while multiple competing non-primary removals remained",
                    )
                })
            })),
        "expected multiple competing content removals reason, got {merge_json}"
    );
    assert!(
        merge_json["merge_reason_details"]
            .as_array()
            .is_some_and(|details| details.iter().any(|detail| {
                detail["subject_id"] == "blk:author-smoke-remove-keep"
                    && detail["variant_kind"] == "content"
                    && detail["reason_kind"]
                        == "kept-primary-parent-variant-over-competing-non-primary-alternative"
                    && detail["branch_kind"] == "kept-primary-variant-over-non-primary-removal"
                    && detail["competing_variants"] == json!(["<absent>", "<absent>"])
            })),
        "expected keep-primary content removal detail, got {merge_json}"
    );
    assert!(
        merge_json["merge_reason_details"]
            .as_array()
            .is_some_and(|details| details.iter().any(|detail| {
                detail["subject_id"] == "blk:author-smoke-remove-keep"
                    && detail["variant_kind"] == "content"
                    && detail["reason_kind"]
                        == "multiple-competing-alternatives-remain-after-keeping-primary-variant"
                    && detail["branch_kind"] == "multiple-competing-non-primary-removals"
                    && detail["competing_variants"] == json!(["<absent>", "<absent>"])
            })),
        "expected multiple competing content removals detail, got {merge_json}"
    );
    assert_eq!(merge_json["patch_op_count"], 1);
}

#[test]
fn store_merge_authoring_flow_reports_selected_replacement_with_competing_removal_branch() {
    let store_dir = create_temp_dir("store-merge-content-select-replace-with-removal-root");
    let (_key_dir, key_path) =
        write_signing_key_file("store-merge-content-select-replace-with-removal-key");
    let (_resolved_dir, resolved_state_path) = write_content_entries_resolved_state_for_doc_file(
        "store-merge-content-select-replace-with-removal-state",
        "doc:author-smoke-content-select-replace-with-removal",
        &[("blk:author-smoke-select-001", "Right")],
    );
    let (_replace_ops_dir, replace_ops_path) = write_content_variant_ops_for_block_file(
        "store-merge-content-select-replace-with-removal-replace-ops",
        "blk:author-smoke-select-001",
        "Right",
    );
    let (_delete_ops_dir, delete_ops_path) = write_content_delete_ops_for_block_file(
        "store-merge-content-select-replace-with-removal-delete-ops",
        "blk:author-smoke-select-001",
    );
    let store_root = path_arg(store_dir.path());
    let key_file = path_arg(&key_path);
    let resolved_state_file = path_arg(&resolved_state_path);
    let replace_ops_file = path_arg(&replace_ops_path);
    let delete_ops_file = path_arg(&delete_ops_path);

    let init = run_mycel(&["store", "init", &store_root, "--json"]);
    assert_success(&init);

    let document = run_mycel(&[
        "store",
        "create-document",
        &store_root,
        "--doc-id",
        "doc:author-smoke-content-select-replace-with-removal",
        "--title",
        "Author Smoke Content Select Replace With Removal",
        "--language",
        "en",
        "--signing-key",
        &key_file,
        "--timestamp",
        "78",
        "--json",
    ]);
    assert_success(&document);
    let document_json = assert_json_status(&document, "ok");
    let genesis_revision_id = document_json["genesis_revision_id"]
        .as_str()
        .expect("genesis revision should be string")
        .to_string();

    let base_ops_dir = create_temp_dir("store-merge-content-select-replace-with-removal-base-ops");
    let base_ops_path = base_ops_dir.path().join("ops.json");
    fs::write(
        &base_ops_path,
        serde_json::to_string_pretty(&json!([
            {
                "op": "insert_block",
                "new_block": {
                    "block_id": "blk:author-smoke-select-001",
                    "block_type": "paragraph",
                    "content": "Base",
                    "attrs": {},
                    "children": []
                }
            }
        ]))
        .expect("selected replace/remove base ops JSON should serialize"),
    )
    .expect("selected replace/remove base ops JSON should write");
    let base_ops_file = path_arg(&base_ops_path);

    let base_patch = run_mycel(&[
        "store",
        "create-patch",
        &store_root,
        "--doc-id",
        "doc:author-smoke-content-select-replace-with-removal",
        "--base-revision",
        &genesis_revision_id,
        "--ops",
        &base_ops_file,
        "--signing-key",
        &key_file,
        "--timestamp",
        "79",
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
        "doc:author-smoke-content-select-replace-with-removal",
        "--parent",
        &genesis_revision_id,
        "--patch",
        &base_patch_id,
        "--signing-key",
        &key_file,
        "--timestamp",
        "80",
        "--json",
    ]);
    assert_success(&base_revision);
    let base_revision_json = assert_json_status(&base_revision, "ok");
    let base_revision_id = base_revision_json["revision_id"]
        .as_str()
        .expect("base revision_id should be string")
        .to_string();

    let replace_patch = run_mycel(&[
        "store",
        "create-patch",
        &store_root,
        "--doc-id",
        "doc:author-smoke-content-select-replace-with-removal",
        "--base-revision",
        &base_revision_id,
        "--ops",
        &replace_ops_file,
        "--signing-key",
        &key_file,
        "--timestamp",
        "81",
        "--json",
    ]);
    assert_success(&replace_patch);
    let replace_patch_json = assert_json_status(&replace_patch, "ok");
    let replace_patch_id = replace_patch_json["patch_id"]
        .as_str()
        .expect("replace patch_id should be string")
        .to_string();

    let replace_revision = run_mycel(&[
        "store",
        "commit-revision",
        &store_root,
        "--doc-id",
        "doc:author-smoke-content-select-replace-with-removal",
        "--parent",
        &base_revision_id,
        "--patch",
        &replace_patch_id,
        "--signing-key",
        &key_file,
        "--timestamp",
        "82",
        "--json",
    ]);
    assert_success(&replace_revision);
    let replace_revision_json = assert_json_status(&replace_revision, "ok");
    let replace_revision_id = replace_revision_json["revision_id"]
        .as_str()
        .expect("replace revision_id should be string")
        .to_string();

    let delete_patch = run_mycel(&[
        "store",
        "create-patch",
        &store_root,
        "--doc-id",
        "doc:author-smoke-content-select-replace-with-removal",
        "--base-revision",
        &base_revision_id,
        "--ops",
        &delete_ops_file,
        "--signing-key",
        &key_file,
        "--timestamp",
        "83",
        "--json",
    ]);
    assert_success(&delete_patch);
    let delete_patch_json = assert_json_status(&delete_patch, "ok");
    let delete_patch_id = delete_patch_json["patch_id"]
        .as_str()
        .expect("delete patch_id should be string")
        .to_string();

    let delete_revision = run_mycel(&[
        "store",
        "commit-revision",
        &store_root,
        "--doc-id",
        "doc:author-smoke-content-select-replace-with-removal",
        "--parent",
        &base_revision_id,
        "--patch",
        &delete_patch_id,
        "--signing-key",
        &key_file,
        "--timestamp",
        "84",
        "--json",
    ]);
    assert_success(&delete_revision);
    let delete_revision_json = assert_json_status(&delete_revision, "ok");
    let delete_revision_id = delete_revision_json["revision_id"]
        .as_str()
        .expect("delete revision_id should be string")
        .to_string();

    let merge = run_mycel(&[
        "store",
        "create-merge-revision",
        &store_root,
        "--doc-id",
        "doc:author-smoke-content-select-replace-with-removal",
        "--parent",
        &base_revision_id,
        "--parent",
        &replace_revision_id,
        "--parent",
        &delete_revision_id,
        "--resolved-state",
        &resolved_state_file,
        "--signing-key",
        &key_file,
        "--timestamp",
        "85",
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
                        "block 'blk:author-smoke-select-001' adopted a non-primary parent replacement while a competing non-primary removal remained",
                    )
                })
            })),
        "expected mixed selected replacement reason, got {merge_json}"
    );
    assert!(
        merge_json["merge_reason_details"]
            .as_array()
            .is_some_and(|details| details.iter().any(|detail| {
                detail["subject_id"] == "blk:author-smoke-select-001"
                    && detail["variant_kind"] == "content"
                    && detail["reason_kind"] == "selected-non-primary-parent-variant"
                    && detail["branch_kind"]
                        == "adopted-non-primary-replacement-while-competing-removal-remains"
            })),
        "expected mixed selected replacement branch kind detail, got {merge_json}"
    );
    assert!(
        merge_json["merge_reason_details"]
            .as_array()
            .is_some_and(|details| details.iter().any(|detail| {
                detail["subject_id"] == "blk:author-smoke-select-001"
                    && detail["variant_kind"] == "content"
                    && detail["reason_kind"]
                        == "multiple-competing-alternatives-remain-after-selected-variant"
                    && detail["branch_kind"] == "multiple-competing-mixed-non-primary-alternatives"
            })),
        "expected mixed competing content branch kind detail, got {merge_json}"
    );
    assert_eq!(merge_json["patch_op_count"], 1);
}

#[test]
fn store_merge_authoring_flow_reports_selected_metadata_replacement_with_competing_removal_branch()
{
    let store_dir = create_temp_dir("store-merge-metadata-select-replace-with-removal-root");
    let (_key_dir, key_path) =
        write_signing_key_file("store-merge-metadata-select-replace-with-removal-key");
    let (_resolved_dir, resolved_state_path) = write_metadata_variant_resolved_state_for_doc_file(
        "store-merge-metadata-select-replace-with-removal-state",
        "doc:author-smoke-metadata-select-replace-with-removal",
        "right",
    );
    let (_replace_ops_dir, replace_ops_path) = write_metadata_entries_ops_file(
        "store-merge-metadata-select-replace-with-removal-replace-ops",
        &[("topic", "right")],
    );
    let store_root = path_arg(store_dir.path());
    let key_file = path_arg(&key_path);
    let resolved_state_file = path_arg(&resolved_state_path);
    let replace_ops_file = path_arg(&replace_ops_path);

    let init = run_mycel(&["store", "init", &store_root, "--json"]);
    assert_success(&init);

    let document = run_mycel(&[
        "store",
        "create-document",
        &store_root,
        "--doc-id",
        "doc:author-smoke-metadata-select-replace-with-removal",
        "--title",
        "Author Smoke Metadata Select Replace With Removal",
        "--language",
        "en",
        "--signing-key",
        &key_file,
        "--timestamp",
        "86",
        "--json",
    ]);
    assert_success(&document);
    let document_json = assert_json_status(&document, "ok");
    let genesis_revision_id = document_json["genesis_revision_id"]
        .as_str()
        .expect("genesis revision should be string")
        .to_string();

    let (_base_ops_dir, base_ops_path) = write_metadata_entries_ops_file(
        "store-merge-metadata-select-replace-with-removal-base-ops",
        &[("topic", "base")],
    );
    let base_ops_file = path_arg(&base_ops_path);

    let base_patch = run_mycel(&[
        "store",
        "create-patch",
        &store_root,
        "--doc-id",
        "doc:author-smoke-metadata-select-replace-with-removal",
        "--base-revision",
        &genesis_revision_id,
        "--ops",
        &base_ops_file,
        "--signing-key",
        &key_file,
        "--timestamp",
        "87",
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
        "doc:author-smoke-metadata-select-replace-with-removal",
        "--parent",
        &genesis_revision_id,
        "--patch",
        &base_patch_id,
        "--signing-key",
        &key_file,
        "--timestamp",
        "88",
        "--json",
    ]);
    assert_success(&base_revision);
    let base_revision_json = assert_json_status(&base_revision, "ok");
    let base_revision_id = base_revision_json["revision_id"]
        .as_str()
        .expect("base revision_id should be string")
        .to_string();

    let replace_patch = run_mycel(&[
        "store",
        "create-patch",
        &store_root,
        "--doc-id",
        "doc:author-smoke-metadata-select-replace-with-removal",
        "--base-revision",
        &base_revision_id,
        "--ops",
        &replace_ops_file,
        "--signing-key",
        &key_file,
        "--timestamp",
        "89",
        "--json",
    ]);
    assert_success(&replace_patch);
    let replace_patch_json = assert_json_status(&replace_patch, "ok");
    let replace_patch_id = replace_patch_json["patch_id"]
        .as_str()
        .expect("replace patch_id should be string")
        .to_string();

    let replace_revision = run_mycel(&[
        "store",
        "commit-revision",
        &store_root,
        "--doc-id",
        "doc:author-smoke-metadata-select-replace-with-removal",
        "--parent",
        &base_revision_id,
        "--patch",
        &replace_patch_id,
        "--signing-key",
        &key_file,
        "--timestamp",
        "90",
        "--json",
    ]);
    assert_success(&replace_revision);
    let replace_revision_json = assert_json_status(&replace_revision, "ok");
    let replace_revision_id = replace_revision_json["revision_id"]
        .as_str()
        .expect("replace revision_id should be string")
        .to_string();

    let merge = run_mycel(&[
        "store",
        "create-merge-revision",
        &store_root,
        "--doc-id",
        "doc:author-smoke-metadata-select-replace-with-removal",
        "--parent",
        &base_revision_id,
        "--parent",
        &replace_revision_id,
        "--parent",
        &genesis_revision_id,
        "--resolved-state",
        &resolved_state_file,
        "--signing-key",
        &key_file,
        "--timestamp",
        "91",
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
                        "metadata key 'topic' adopted a non-primary parent replacement while a competing non-primary removal remained",
                    )
                })
            })),
        "expected mixed selected metadata replacement reason, got {merge_json}"
    );
    assert!(
        merge_json["merge_reason_details"]
            .as_array()
            .is_some_and(|details| details.iter().any(|detail| {
                detail["subject_id"] == "topic"
                    && detail["variant_kind"] == "metadata"
                    && detail["reason_kind"] == "selected-non-primary-parent-variant"
                    && detail["branch_kind"]
                        == "adopted-non-primary-replacement-while-competing-removal-remains"
            })),
        "expected mixed selected metadata replacement branch kind detail, got {merge_json}"
    );
    assert!(
        merge_json["merge_reason_details"]
            .as_array()
            .is_some_and(|details| details.iter().any(|detail| {
                detail["subject_id"] == "topic"
                    && detail["variant_kind"] == "metadata"
                    && detail["reason_kind"]
                        == "multiple-competing-alternatives-remain-after-selected-variant"
                    && detail["branch_kind"] == "multiple-competing-mixed-non-primary-alternatives"
            })),
        "expected mixed competing metadata branch kind detail, got {merge_json}"
    );
    assert_eq!(merge_json["patch_op_count"], 1);
}

#[test]
fn store_merge_authoring_flow_reports_kept_primary_metadata_over_multiple_removals() {
    let store_dir = create_temp_dir("store-merge-metadata-keep-primary-removals-root");
    let (_key_dir, key_path) =
        write_signing_key_file("store-merge-metadata-keep-primary-removals-key");
    let (_resolved_dir, resolved_state_path) = write_metadata_entries_resolved_state_for_doc_file(
        "store-merge-metadata-keep-primary-removals-state",
        "doc:author-smoke-metadata-keep-primary-removals",
        &[("topic", "base"), ("priority", "high")],
    );
    let (_priority_ops_dir, priority_ops_path) = write_metadata_entries_ops_file(
        "store-merge-metadata-keep-primary-removals-priority-ops",
        &[("priority", "high")],
    );
    let store_root = path_arg(store_dir.path());
    let key_file = path_arg(&key_path);
    let resolved_state_file = path_arg(&resolved_state_path);
    let priority_ops_file = path_arg(&priority_ops_path);

    let init = run_mycel(&["store", "init", &store_root, "--json"]);
    assert_success(&init);

    let document = run_mycel(&[
        "store",
        "create-document",
        &store_root,
        "--doc-id",
        "doc:author-smoke-metadata-keep-primary-removals",
        "--title",
        "Author Smoke Metadata Keep Primary Removals",
        "--language",
        "en",
        "--signing-key",
        &key_file,
        "--timestamp",
        "104",
        "--json",
    ]);
    assert_success(&document);
    let genesis_revision_id = assert_json_status(&document, "ok")["genesis_revision_id"]
        .as_str()
        .expect("genesis revision should be string")
        .to_string();

    let (_base_ops_dir, base_ops_path) = write_metadata_entries_ops_file(
        "store-merge-metadata-keep-primary-removals-base-ops",
        &[("topic", "base")],
    );
    let base_ops_file = path_arg(&base_ops_path);

    let base_patch = run_mycel(&[
        "store",
        "create-patch",
        &store_root,
        "--doc-id",
        "doc:author-smoke-metadata-keep-primary-removals",
        "--base-revision",
        &genesis_revision_id,
        "--ops",
        &base_ops_file,
        "--signing-key",
        &key_file,
        "--timestamp",
        "105",
        "--json",
    ]);
    assert_success(&base_patch);
    let base_patch_id = assert_json_status(&base_patch, "ok")["patch_id"]
        .as_str()
        .expect("base patch_id should be string")
        .to_string();

    let base_revision = run_mycel(&[
        "store",
        "commit-revision",
        &store_root,
        "--doc-id",
        "doc:author-smoke-metadata-keep-primary-removals",
        "--parent",
        &genesis_revision_id,
        "--patch",
        &base_patch_id,
        "--signing-key",
        &key_file,
        "--timestamp",
        "106",
        "--json",
    ]);
    assert_success(&base_revision);
    let base_revision_id = assert_json_status(&base_revision, "ok")["revision_id"]
        .as_str()
        .expect("base revision_id should be string")
        .to_string();

    let priority_patch = run_mycel(&[
        "store",
        "create-patch",
        &store_root,
        "--doc-id",
        "doc:author-smoke-metadata-keep-primary-removals",
        "--base-revision",
        &genesis_revision_id,
        "--ops",
        &priority_ops_file,
        "--signing-key",
        &key_file,
        "--timestamp",
        "107",
        "--json",
    ]);
    assert_success(&priority_patch);
    let priority_patch_id = assert_json_status(&priority_patch, "ok")["patch_id"]
        .as_str()
        .expect("priority patch_id should be string")
        .to_string();

    let priority_revision = run_mycel(&[
        "store",
        "commit-revision",
        &store_root,
        "--doc-id",
        "doc:author-smoke-metadata-keep-primary-removals",
        "--parent",
        &genesis_revision_id,
        "--patch",
        &priority_patch_id,
        "--signing-key",
        &key_file,
        "--timestamp",
        "108",
        "--json",
    ]);
    assert_success(&priority_revision);
    let priority_revision_id = assert_json_status(&priority_revision, "ok")["revision_id"]
        .as_str()
        .expect("priority revision_id should be string")
        .to_string();

    let merge = run_mycel(&[
        "store",
        "create-merge-revision",
        &store_root,
        "--doc-id",
        "doc:author-smoke-metadata-keep-primary-removals",
        "--parent",
        &base_revision_id,
        "--parent",
        &genesis_revision_id,
        "--parent",
        &priority_revision_id,
        "--resolved-state",
        &resolved_state_file,
        "--signing-key",
        &key_file,
        "--timestamp",
        "109",
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
                        "metadata key 'topic' kept the primary parent variant over a competing non-primary removal",
                    )
                })
            })),
        "expected keep-primary metadata removal reason, got {merge_json}"
    );
    assert!(
        merge_json["merge_reasons"]
            .as_array()
            .is_some_and(|reasons| reasons.iter().any(|reason| {
                reason.as_str().is_some_and(|reason| {
                    reason.contains(
                        "metadata key 'topic' kept the primary variant while multiple competing non-primary removals remained",
                    )
                })
            })),
        "expected multiple competing metadata removals reason, got {merge_json}"
    );
    assert!(
        merge_json["merge_reason_details"]
            .as_array()
            .is_some_and(|details| details.iter().any(|detail| {
                detail["subject_id"] == "topic"
                    && detail["variant_kind"] == "metadata"
                    && detail["reason_kind"]
                        == "kept-primary-parent-variant-over-competing-non-primary-alternative"
                    && detail["branch_kind"] == "kept-primary-variant-over-non-primary-removal"
                    && detail["competing_variants"] == json!(["<absent>", "<absent>"])
            })),
        "expected keep-primary metadata removal detail, got {merge_json}"
    );
    assert!(
        merge_json["merge_reason_details"]
            .as_array()
            .is_some_and(|details| details.iter().any(|detail| {
                detail["subject_id"] == "topic"
                    && detail["variant_kind"] == "metadata"
                    && detail["reason_kind"]
                        == "multiple-competing-alternatives-remain-after-keeping-primary-variant"
                    && detail["branch_kind"] == "multiple-competing-non-primary-removals"
                    && detail["competing_variants"] == json!(["<absent>", "<absent>"])
            })),
        "expected multiple competing metadata removals detail, got {merge_json}"
    );
    assert_eq!(merge_json["patch_op_count"], 1);
}

#[test]
fn store_merge_authoring_flow_reports_selected_content_replacement_with_multiple_replacements_and_removal(
) {
    let store_dir = create_temp_dir("store-merge-content-select-many-root");
    let (_key_dir, key_path) = write_signing_key_file("store-merge-content-select-many-key");
    let (_resolved_dir, resolved_state_path) = write_content_entries_resolved_state_for_doc_file(
        "store-merge-content-select-many-state",
        "doc:author-smoke-content-select-many",
        &[("blk:author-smoke-select-many-001", "Right A")],
    );
    let (_base_ops_dir, base_ops_path) = write_content_addition_ops_for_block_file(
        "store-merge-content-select-many-base-ops",
        "blk:author-smoke-select-many-001",
        "Base",
    );
    let (_replace_a_ops_dir, replace_a_ops_path) = write_content_variant_ops_for_block_file(
        "store-merge-content-select-many-replace-a-ops",
        "blk:author-smoke-select-many-001",
        "Right A",
    );
    let (_replace_b_ops_dir, replace_b_ops_path) = write_content_variant_ops_for_block_file(
        "store-merge-content-select-many-replace-b-ops",
        "blk:author-smoke-select-many-001",
        "Right B",
    );
    let store_root = path_arg(store_dir.path());
    let key_file = path_arg(&key_path);
    let resolved_state_file = path_arg(&resolved_state_path);
    let base_ops_file = path_arg(&base_ops_path);
    let replace_a_ops_file = path_arg(&replace_a_ops_path);
    let replace_b_ops_file = path_arg(&replace_b_ops_path);

    let init = run_mycel(&["store", "init", &store_root, "--json"]);
    assert_success(&init);

    let document = run_mycel(&[
        "store",
        "create-document",
        &store_root,
        "--doc-id",
        "doc:author-smoke-content-select-many",
        "--title",
        "Author Smoke Content Select Many",
        "--language",
        "en",
        "--signing-key",
        &key_file,
        "--timestamp",
        "110",
        "--json",
    ]);
    assert_success(&document);
    let genesis_revision_id = assert_json_status(&document, "ok")["genesis_revision_id"]
        .as_str()
        .expect("genesis revision should be string")
        .to_string();

    let base_patch = run_mycel(&[
        "store",
        "create-patch",
        &store_root,
        "--doc-id",
        "doc:author-smoke-content-select-many",
        "--base-revision",
        &genesis_revision_id,
        "--ops",
        &base_ops_file,
        "--signing-key",
        &key_file,
        "--timestamp",
        "111",
        "--json",
    ]);
    assert_success(&base_patch);
    let base_patch_id = assert_json_status(&base_patch, "ok")["patch_id"]
        .as_str()
        .expect("base patch_id should be string")
        .to_string();

    let base_revision = run_mycel(&[
        "store",
        "commit-revision",
        &store_root,
        "--doc-id",
        "doc:author-smoke-content-select-many",
        "--parent",
        &genesis_revision_id,
        "--patch",
        &base_patch_id,
        "--signing-key",
        &key_file,
        "--timestamp",
        "112",
        "--json",
    ]);
    assert_success(&base_revision);
    let base_revision_id = assert_json_status(&base_revision, "ok")["revision_id"]
        .as_str()
        .expect("base revision_id should be string")
        .to_string();

    let replace_a_patch = run_mycel(&[
        "store",
        "create-patch",
        &store_root,
        "--doc-id",
        "doc:author-smoke-content-select-many",
        "--base-revision",
        &base_revision_id,
        "--ops",
        &replace_a_ops_file,
        "--signing-key",
        &key_file,
        "--timestamp",
        "113",
        "--json",
    ]);
    assert_success(&replace_a_patch);
    let replace_a_patch_id = assert_json_status(&replace_a_patch, "ok")["patch_id"]
        .as_str()
        .expect("replace_a patch_id should be string")
        .to_string();

    let replace_a_revision = run_mycel(&[
        "store",
        "commit-revision",
        &store_root,
        "--doc-id",
        "doc:author-smoke-content-select-many",
        "--parent",
        &base_revision_id,
        "--patch",
        &replace_a_patch_id,
        "--signing-key",
        &key_file,
        "--timestamp",
        "114",
        "--json",
    ]);
    assert_success(&replace_a_revision);
    let replace_a_revision_id = assert_json_status(&replace_a_revision, "ok")["revision_id"]
        .as_str()
        .expect("replace_a revision_id should be string")
        .to_string();

    let replace_b_patch = run_mycel(&[
        "store",
        "create-patch",
        &store_root,
        "--doc-id",
        "doc:author-smoke-content-select-many",
        "--base-revision",
        &base_revision_id,
        "--ops",
        &replace_b_ops_file,
        "--signing-key",
        &key_file,
        "--timestamp",
        "115",
        "--json",
    ]);
    assert_success(&replace_b_patch);
    let replace_b_patch_id = assert_json_status(&replace_b_patch, "ok")["patch_id"]
        .as_str()
        .expect("replace_b patch_id should be string")
        .to_string();

    let replace_b_revision = run_mycel(&[
        "store",
        "commit-revision",
        &store_root,
        "--doc-id",
        "doc:author-smoke-content-select-many",
        "--parent",
        &base_revision_id,
        "--patch",
        &replace_b_patch_id,
        "--signing-key",
        &key_file,
        "--timestamp",
        "116",
        "--json",
    ]);
    assert_success(&replace_b_revision);
    let replace_b_revision_id = assert_json_status(&replace_b_revision, "ok")["revision_id"]
        .as_str()
        .expect("replace_b revision_id should be string")
        .to_string();

    let merge = run_mycel(&[
        "store",
        "create-merge-revision",
        &store_root,
        "--doc-id",
        "doc:author-smoke-content-select-many",
        "--parent",
        &base_revision_id,
        "--parent",
        &replace_a_revision_id,
        "--parent",
        &replace_b_revision_id,
        "--parent",
        &genesis_revision_id,
        "--resolved-state",
        &resolved_state_file,
        "--signing-key",
        &key_file,
        "--timestamp",
        "117",
        "--json",
    ]);
    assert_success(&merge);
    let merge_json = assert_json_status(&merge, "ok");
    assert_eq!(merge_json["merge_outcome"], "multi-variant");
    assert!(
        merge_json["merge_reasons"].as_array().is_some_and(|reasons| reasons
            .iter()
            .any(|reason| reason.as_str().is_some_and(|reason| reason.contains(
                "block 'blk:author-smoke-select-many-001' adopted a non-primary parent replacement while competing non-primary replacements and a removal remained"
            )))),
        "expected richer selected content replacement reason, got {merge_json}"
    );
    assert!(
        merge_json["merge_reason_details"]
            .as_array()
            .is_some_and(|details| details.iter().any(|detail| {
                detail["subject_id"] == "blk:author-smoke-select-many-001"
                    && detail["variant_kind"] == "content"
                    && detail["reason_kind"] == "selected-non-primary-parent-variant"
                    && detail["branch_kind"]
                        == "adopted-non-primary-replacement-while-competing-replacements-and-removal-remain"
            })),
        "expected richer selected content branch detail, got {merge_json}"
    );
    assert!(
        merge_json["merge_reason_details"]
            .as_array()
            .is_some_and(|details| details.iter().any(|detail| {
                detail["subject_id"] == "blk:author-smoke-select-many-001"
                    && detail["variant_kind"] == "content"
                    && detail["reason_kind"]
                        == "multiple-competing-alternatives-remain-after-selected-variant"
                    && detail["branch_kind"]
                        == "multiple-competing-non-primary-replacements-and-removals"
            })),
        "expected richer competing content branch detail, got {merge_json}"
    );
}

#[test]
fn store_merge_authoring_flow_reports_kept_primary_content_over_multiple_replacements_and_removals()
{
    let store_dir = create_temp_dir("store-merge-content-keep-many-root");
    let (_key_dir, key_path) = write_signing_key_file("store-merge-content-keep-many-key");
    let (_resolved_dir, resolved_state_path) = write_content_entries_resolved_state_for_doc_file(
        "store-merge-content-keep-many-state",
        "doc:author-smoke-content-keep-many",
        &[("blk:author-smoke-keep-many-001", "Base")],
    );
    let (_base_ops_dir, base_ops_path) = write_content_addition_ops_for_block_file(
        "store-merge-content-keep-many-base-ops",
        "blk:author-smoke-keep-many-001",
        "Base",
    );
    let (_replace_a_ops_dir, replace_a_ops_path) = write_content_variant_ops_for_block_file(
        "store-merge-content-keep-many-replace-a-ops",
        "blk:author-smoke-keep-many-001",
        "Right A",
    );
    let (_replace_b_ops_dir, replace_b_ops_path) = write_content_variant_ops_for_block_file(
        "store-merge-content-keep-many-replace-b-ops",
        "blk:author-smoke-keep-many-001",
        "Right B",
    );
    let store_root = path_arg(store_dir.path());
    let key_file = path_arg(&key_path);
    let resolved_state_file = path_arg(&resolved_state_path);
    let base_ops_file = path_arg(&base_ops_path);
    let replace_a_ops_file = path_arg(&replace_a_ops_path);
    let replace_b_ops_file = path_arg(&replace_b_ops_path);

    let init = run_mycel(&["store", "init", &store_root, "--json"]);
    assert_success(&init);

    let document = run_mycel(&[
        "store",
        "create-document",
        &store_root,
        "--doc-id",
        "doc:author-smoke-content-keep-many",
        "--title",
        "Author Smoke Content Keep Many",
        "--language",
        "en",
        "--signing-key",
        &key_file,
        "--timestamp",
        "118",
        "--json",
    ]);
    assert_success(&document);
    let genesis_revision_id = assert_json_status(&document, "ok")["genesis_revision_id"]
        .as_str()
        .expect("genesis revision should be string")
        .to_string();

    let base_patch = run_mycel(&[
        "store",
        "create-patch",
        &store_root,
        "--doc-id",
        "doc:author-smoke-content-keep-many",
        "--base-revision",
        &genesis_revision_id,
        "--ops",
        &base_ops_file,
        "--signing-key",
        &key_file,
        "--timestamp",
        "119",
        "--json",
    ]);
    assert_success(&base_patch);
    let base_patch_id = assert_json_status(&base_patch, "ok")["patch_id"]
        .as_str()
        .expect("base patch_id should be string")
        .to_string();

    let base_revision = run_mycel(&[
        "store",
        "commit-revision",
        &store_root,
        "--doc-id",
        "doc:author-smoke-content-keep-many",
        "--parent",
        &genesis_revision_id,
        "--patch",
        &base_patch_id,
        "--signing-key",
        &key_file,
        "--timestamp",
        "120",
        "--json",
    ]);
    assert_success(&base_revision);
    let base_revision_id = assert_json_status(&base_revision, "ok")["revision_id"]
        .as_str()
        .expect("base revision_id should be string")
        .to_string();

    let replace_a_patch = run_mycel(&[
        "store",
        "create-patch",
        &store_root,
        "--doc-id",
        "doc:author-smoke-content-keep-many",
        "--base-revision",
        &base_revision_id,
        "--ops",
        &replace_a_ops_file,
        "--signing-key",
        &key_file,
        "--timestamp",
        "121",
        "--json",
    ]);
    assert_success(&replace_a_patch);
    let replace_a_patch_id = assert_json_status(&replace_a_patch, "ok")["patch_id"]
        .as_str()
        .expect("replace_a patch_id should be string")
        .to_string();

    let replace_a_revision = run_mycel(&[
        "store",
        "commit-revision",
        &store_root,
        "--doc-id",
        "doc:author-smoke-content-keep-many",
        "--parent",
        &base_revision_id,
        "--patch",
        &replace_a_patch_id,
        "--signing-key",
        &key_file,
        "--timestamp",
        "122",
        "--json",
    ]);
    assert_success(&replace_a_revision);
    let replace_a_revision_id = assert_json_status(&replace_a_revision, "ok")["revision_id"]
        .as_str()
        .expect("replace_a revision_id should be string")
        .to_string();

    let replace_b_patch = run_mycel(&[
        "store",
        "create-patch",
        &store_root,
        "--doc-id",
        "doc:author-smoke-content-keep-many",
        "--base-revision",
        &base_revision_id,
        "--ops",
        &replace_b_ops_file,
        "--signing-key",
        &key_file,
        "--timestamp",
        "123",
        "--json",
    ]);
    assert_success(&replace_b_patch);
    let replace_b_patch_id = assert_json_status(&replace_b_patch, "ok")["patch_id"]
        .as_str()
        .expect("replace_b patch_id should be string")
        .to_string();

    let replace_b_revision = run_mycel(&[
        "store",
        "commit-revision",
        &store_root,
        "--doc-id",
        "doc:author-smoke-content-keep-many",
        "--parent",
        &base_revision_id,
        "--patch",
        &replace_b_patch_id,
        "--signing-key",
        &key_file,
        "--timestamp",
        "124",
        "--json",
    ]);
    assert_success(&replace_b_revision);
    let replace_b_revision_id = assert_json_status(&replace_b_revision, "ok")["revision_id"]
        .as_str()
        .expect("replace_b revision_id should be string")
        .to_string();

    let merge = run_mycel(&[
        "store",
        "create-merge-revision",
        &store_root,
        "--doc-id",
        "doc:author-smoke-content-keep-many",
        "--parent",
        &base_revision_id,
        "--parent",
        &replace_a_revision_id,
        "--parent",
        &replace_b_revision_id,
        "--parent",
        &genesis_revision_id,
        "--resolved-state",
        &resolved_state_file,
        "--signing-key",
        &key_file,
        "--timestamp",
        "125",
        "--json",
    ]);
    assert_success(&merge);
    let merge_json = assert_json_status(&merge, "ok");
    assert_eq!(merge_json["merge_outcome"], "multi-variant");
    assert!(
        merge_json["merge_reasons"].as_array().is_some_and(|reasons| reasons
            .iter()
            .any(|reason| reason.as_str().is_some_and(|reason| reason.contains(
                "block 'blk:author-smoke-keep-many-001' kept the primary parent variant over multiple competing non-primary replacements and removals"
            )))),
        "expected richer kept-primary content reason, got {merge_json}"
    );
    assert!(
        merge_json["merge_reason_details"]
            .as_array()
            .is_some_and(|details| details.iter().any(|detail| {
                detail["subject_id"] == "blk:author-smoke-keep-many-001"
                    && detail["variant_kind"] == "content"
                    && detail["reason_kind"]
                        == "kept-primary-parent-variant-over-competing-non-primary-alternative"
                    && detail["branch_kind"]
                        == "kept-primary-variant-over-multiple-competing-non-primary-replacements-and-removals"
            })),
        "expected richer kept-primary content branch detail, got {merge_json}"
    );
    assert!(
        merge_json["merge_reason_details"]
            .as_array()
            .is_some_and(|details| details.iter().any(|detail| {
                detail["subject_id"] == "blk:author-smoke-keep-many-001"
                    && detail["variant_kind"] == "content"
                    && detail["reason_kind"]
                        == "multiple-competing-alternatives-remain-after-keeping-primary-variant"
                    && detail["branch_kind"]
                        == "multiple-competing-non-primary-replacements-and-removals"
            })),
        "expected richer multiple competing kept-primary content branch detail, got {merge_json}"
    );
}

#[test]
fn store_merge_authoring_flow_reports_selected_metadata_replacement_with_multiple_replacements_and_removal(
) {
    let store_dir = create_temp_dir("store-merge-metadata-select-many-root");
    let (_key_dir, key_path) = write_signing_key_file("store-merge-metadata-select-many-key");
    let (_resolved_dir, resolved_state_path) = write_metadata_variant_resolved_state_for_doc_file(
        "store-merge-metadata-select-many-state",
        "doc:author-smoke-metadata-select-many",
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
    let store_root = path_arg(store_dir.path());
    let key_file = path_arg(&key_path);
    let resolved_state_file = path_arg(&resolved_state_path);
    let base_ops_file = path_arg(&base_ops_path);
    let replace_a_ops_file = path_arg(&replace_a_ops_path);
    let replace_b_ops_file = path_arg(&replace_b_ops_path);

    let init = run_mycel(&["store", "init", &store_root, "--json"]);
    assert_success(&init);

    let document = run_mycel(&[
        "store",
        "create-document",
        &store_root,
        "--doc-id",
        "doc:author-smoke-metadata-select-many",
        "--title",
        "Author Smoke Metadata Select Many",
        "--language",
        "en",
        "--signing-key",
        &key_file,
        "--timestamp",
        "126",
        "--json",
    ]);
    assert_success(&document);
    let genesis_revision_id = assert_json_status(&document, "ok")["genesis_revision_id"]
        .as_str()
        .expect("genesis revision should be string")
        .to_string();

    let base_patch = run_mycel(&[
        "store",
        "create-patch",
        &store_root,
        "--doc-id",
        "doc:author-smoke-metadata-select-many",
        "--base-revision",
        &genesis_revision_id,
        "--ops",
        &base_ops_file,
        "--signing-key",
        &key_file,
        "--timestamp",
        "127",
        "--json",
    ]);
    assert_success(&base_patch);
    let base_patch_id = assert_json_status(&base_patch, "ok")["patch_id"]
        .as_str()
        .expect("base patch_id should be string")
        .to_string();

    let base_revision = run_mycel(&[
        "store",
        "commit-revision",
        &store_root,
        "--doc-id",
        "doc:author-smoke-metadata-select-many",
        "--parent",
        &genesis_revision_id,
        "--patch",
        &base_patch_id,
        "--signing-key",
        &key_file,
        "--timestamp",
        "128",
        "--json",
    ]);
    assert_success(&base_revision);
    let base_revision_id = assert_json_status(&base_revision, "ok")["revision_id"]
        .as_str()
        .expect("base revision_id should be string")
        .to_string();

    let replace_a_patch = run_mycel(&[
        "store",
        "create-patch",
        &store_root,
        "--doc-id",
        "doc:author-smoke-metadata-select-many",
        "--base-revision",
        &base_revision_id,
        "--ops",
        &replace_a_ops_file,
        "--signing-key",
        &key_file,
        "--timestamp",
        "129",
        "--json",
    ]);
    assert_success(&replace_a_patch);
    let replace_a_patch_id = assert_json_status(&replace_a_patch, "ok")["patch_id"]
        .as_str()
        .expect("replace_a patch_id should be string")
        .to_string();

    let replace_a_revision = run_mycel(&[
        "store",
        "commit-revision",
        &store_root,
        "--doc-id",
        "doc:author-smoke-metadata-select-many",
        "--parent",
        &base_revision_id,
        "--patch",
        &replace_a_patch_id,
        "--signing-key",
        &key_file,
        "--timestamp",
        "130",
        "--json",
    ]);
    assert_success(&replace_a_revision);
    let replace_a_revision_id = assert_json_status(&replace_a_revision, "ok")["revision_id"]
        .as_str()
        .expect("replace_a revision_id should be string")
        .to_string();

    let replace_b_patch = run_mycel(&[
        "store",
        "create-patch",
        &store_root,
        "--doc-id",
        "doc:author-smoke-metadata-select-many",
        "--base-revision",
        &base_revision_id,
        "--ops",
        &replace_b_ops_file,
        "--signing-key",
        &key_file,
        "--timestamp",
        "131",
        "--json",
    ]);
    assert_success(&replace_b_patch);
    let replace_b_patch_id = assert_json_status(&replace_b_patch, "ok")["patch_id"]
        .as_str()
        .expect("replace_b patch_id should be string")
        .to_string();

    let replace_b_revision = run_mycel(&[
        "store",
        "commit-revision",
        &store_root,
        "--doc-id",
        "doc:author-smoke-metadata-select-many",
        "--parent",
        &base_revision_id,
        "--patch",
        &replace_b_patch_id,
        "--signing-key",
        &key_file,
        "--timestamp",
        "132",
        "--json",
    ]);
    assert_success(&replace_b_revision);
    let replace_b_revision_id = assert_json_status(&replace_b_revision, "ok")["revision_id"]
        .as_str()
        .expect("replace_b revision_id should be string")
        .to_string();

    let merge = run_mycel(&[
        "store",
        "create-merge-revision",
        &store_root,
        "--doc-id",
        "doc:author-smoke-metadata-select-many",
        "--parent",
        &base_revision_id,
        "--parent",
        &replace_a_revision_id,
        "--parent",
        &replace_b_revision_id,
        "--parent",
        &genesis_revision_id,
        "--resolved-state",
        &resolved_state_file,
        "--signing-key",
        &key_file,
        "--timestamp",
        "133",
        "--json",
    ]);
    assert_success(&merge);
    let merge_json = assert_json_status(&merge, "ok");
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
    let store_dir = create_temp_dir("store-merge-metadata-keep-many-root");
    let (_key_dir, key_path) = write_signing_key_file("store-merge-metadata-keep-many-key");
    let (_resolved_dir, resolved_state_path) = write_metadata_variant_resolved_state_for_doc_file(
        "store-merge-metadata-keep-many-state",
        "doc:author-smoke-metadata-keep-many",
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
    let store_root = path_arg(store_dir.path());
    let key_file = path_arg(&key_path);
    let resolved_state_file = path_arg(&resolved_state_path);
    let base_ops_file = path_arg(&base_ops_path);
    let replace_a_ops_file = path_arg(&replace_a_ops_path);
    let replace_b_ops_file = path_arg(&replace_b_ops_path);

    let init = run_mycel(&["store", "init", &store_root, "--json"]);
    assert_success(&init);

    let document = run_mycel(&[
        "store",
        "create-document",
        &store_root,
        "--doc-id",
        "doc:author-smoke-metadata-keep-many",
        "--title",
        "Author Smoke Metadata Keep Many",
        "--language",
        "en",
        "--signing-key",
        &key_file,
        "--timestamp",
        "134",
        "--json",
    ]);
    assert_success(&document);
    let genesis_revision_id = assert_json_status(&document, "ok")["genesis_revision_id"]
        .as_str()
        .expect("genesis revision should be string")
        .to_string();

    let base_patch = run_mycel(&[
        "store",
        "create-patch",
        &store_root,
        "--doc-id",
        "doc:author-smoke-metadata-keep-many",
        "--base-revision",
        &genesis_revision_id,
        "--ops",
        &base_ops_file,
        "--signing-key",
        &key_file,
        "--timestamp",
        "135",
        "--json",
    ]);
    assert_success(&base_patch);
    let base_patch_id = assert_json_status(&base_patch, "ok")["patch_id"]
        .as_str()
        .expect("base patch_id should be string")
        .to_string();

    let base_revision = run_mycel(&[
        "store",
        "commit-revision",
        &store_root,
        "--doc-id",
        "doc:author-smoke-metadata-keep-many",
        "--parent",
        &genesis_revision_id,
        "--patch",
        &base_patch_id,
        "--signing-key",
        &key_file,
        "--timestamp",
        "136",
        "--json",
    ]);
    assert_success(&base_revision);
    let base_revision_id = assert_json_status(&base_revision, "ok")["revision_id"]
        .as_str()
        .expect("base revision_id should be string")
        .to_string();

    let replace_a_patch = run_mycel(&[
        "store",
        "create-patch",
        &store_root,
        "--doc-id",
        "doc:author-smoke-metadata-keep-many",
        "--base-revision",
        &base_revision_id,
        "--ops",
        &replace_a_ops_file,
        "--signing-key",
        &key_file,
        "--timestamp",
        "137",
        "--json",
    ]);
    assert_success(&replace_a_patch);
    let replace_a_patch_id = assert_json_status(&replace_a_patch, "ok")["patch_id"]
        .as_str()
        .expect("replace_a patch_id should be string")
        .to_string();

    let replace_a_revision = run_mycel(&[
        "store",
        "commit-revision",
        &store_root,
        "--doc-id",
        "doc:author-smoke-metadata-keep-many",
        "--parent",
        &base_revision_id,
        "--patch",
        &replace_a_patch_id,
        "--signing-key",
        &key_file,
        "--timestamp",
        "138",
        "--json",
    ]);
    assert_success(&replace_a_revision);
    let replace_a_revision_id = assert_json_status(&replace_a_revision, "ok")["revision_id"]
        .as_str()
        .expect("replace_a revision_id should be string")
        .to_string();

    let replace_b_patch = run_mycel(&[
        "store",
        "create-patch",
        &store_root,
        "--doc-id",
        "doc:author-smoke-metadata-keep-many",
        "--base-revision",
        &base_revision_id,
        "--ops",
        &replace_b_ops_file,
        "--signing-key",
        &key_file,
        "--timestamp",
        "139",
        "--json",
    ]);
    assert_success(&replace_b_patch);
    let replace_b_patch_id = assert_json_status(&replace_b_patch, "ok")["patch_id"]
        .as_str()
        .expect("replace_b patch_id should be string")
        .to_string();

    let replace_b_revision = run_mycel(&[
        "store",
        "commit-revision",
        &store_root,
        "--doc-id",
        "doc:author-smoke-metadata-keep-many",
        "--parent",
        &base_revision_id,
        "--patch",
        &replace_b_patch_id,
        "--signing-key",
        &key_file,
        "--timestamp",
        "140",
        "--json",
    ]);
    assert_success(&replace_b_revision);
    let replace_b_revision_id = assert_json_status(&replace_b_revision, "ok")["revision_id"]
        .as_str()
        .expect("replace_b revision_id should be string")
        .to_string();

    let merge = run_mycel(&[
        "store",
        "create-merge-revision",
        &store_root,
        "--doc-id",
        "doc:author-smoke-metadata-keep-many",
        "--parent",
        &base_revision_id,
        "--parent",
        &replace_a_revision_id,
        "--parent",
        &replace_b_revision_id,
        "--parent",
        &genesis_revision_id,
        "--resolved-state",
        &resolved_state_file,
        "--signing-key",
        &key_file,
        "--timestamp",
        "141",
        "--json",
    ]);
    assert_success(&merge);
    let merge_json = assert_json_status(&merge, "ok");
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
