use super::*;

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
