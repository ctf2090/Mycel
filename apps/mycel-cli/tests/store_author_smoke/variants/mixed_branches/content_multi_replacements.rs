use super::*;

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
