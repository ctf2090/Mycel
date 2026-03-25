use super::*;

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
