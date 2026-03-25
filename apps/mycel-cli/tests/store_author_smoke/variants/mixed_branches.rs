use super::*;

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
