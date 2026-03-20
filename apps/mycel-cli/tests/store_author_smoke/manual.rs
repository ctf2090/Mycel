use super::*;

#[test]
fn store_merge_authoring_flow_requires_manual_curation_for_metadata_removal() {
    let store_dir = create_temp_dir("store-merge-metadata-removal-root");
    let (_key_dir, key_path) = write_signing_key_file("store-merge-metadata-removal-key");
    let store_root = path_arg(&store_dir.path().to_path_buf());
    let key_file = path_arg(&key_path);
    let (_base_ops_dir, base_ops_path) =
        write_metadata_variant_ops_file("store-merge-metadata-removal-base-ops", "base");
    let (_right_ops_dir, right_ops_path) =
        write_metadata_variant_ops_file("store-merge-metadata-removal-right-ops", "right");
    let base_ops_file = path_arg(&base_ops_path);
    let right_ops_file = path_arg(&right_ops_path);

    let init = run_mycel(&["store", "init", &store_root, "--json"]);
    assert_success(&init);

    let document = run_mycel(&[
        "store",
        "create-document",
        &store_root,
        "--doc-id",
        "doc:author-smoke-metadata-removal",
        "--title",
        "Author Smoke Metadata Removal",
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

    let base_patch = run_mycel(&[
        "store",
        "create-patch",
        &store_root,
        "--doc-id",
        "doc:author-smoke-metadata-removal",
        "--base-revision",
        &genesis_revision_id,
        "--ops",
        &base_ops_file,
        "--signing-key",
        &key_file,
        "--timestamp",
        "41",
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
        "doc:author-smoke-metadata-removal",
        "--parent",
        &genesis_revision_id,
        "--patch",
        &base_patch_id,
        "--signing-key",
        &key_file,
        "--timestamp",
        "42",
        "--json",
    ]);
    assert_success(&base_revision);
    let base_revision_json = assert_json_status(&base_revision, "ok");
    let base_revision_id = base_revision_json["revision_id"]
        .as_str()
        .expect("base revision_id should be string")
        .to_string();

    let right_patch = run_mycel(&[
        "store",
        "create-patch",
        &store_root,
        "--doc-id",
        "doc:author-smoke-metadata-removal",
        "--base-revision",
        &base_revision_id,
        "--ops",
        &right_ops_file,
        "--signing-key",
        &key_file,
        "--timestamp",
        "43",
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
        "doc:author-smoke-metadata-removal",
        "--parent",
        &base_revision_id,
        "--patch",
        &right_patch_id,
        "--signing-key",
        &key_file,
        "--timestamp",
        "44",
        "--json",
    ]);
    assert_success(&right_revision);
    let right_revision_json = assert_json_status(&right_revision, "ok");
    let right_revision_id = right_revision_json["revision_id"]
        .as_str()
        .expect("right revision_id should be string")
        .to_string();

    let empty_resolved_dir = create_temp_dir("store-merge-metadata-removal-empty-state");
    let empty_resolved_path = empty_resolved_dir.path().join("resolved-state.json");
    fs::write(
        &empty_resolved_path,
        serde_json::to_string_pretty(&json!({
            "doc_id": "doc:author-smoke-metadata-removal",
            "blocks": [],
            "metadata": {}
        }))
        .expect("metadata removal resolved state JSON should serialize"),
    )
    .expect("metadata removal resolved state JSON should write");
    let resolved_state_file = path_arg(&empty_resolved_path);

    let merge = run_mycel(&[
        "store",
        "create-merge-revision",
        &store_root,
        "--doc-id",
        "doc:author-smoke-metadata-removal",
        "--parent",
        &base_revision_id,
        "--parent",
        &right_revision_id,
        "--resolved-state",
        &resolved_state_file,
        "--signing-key",
        &key_file,
        "--timestamp",
        "45",
    ]);

    assert!(
        !merge.status.success(),
        "expected manual-curation failure, stdout: {}, stderr: {}",
        String::from_utf8_lossy(&merge.stdout),
        String::from_utf8_lossy(&merge.stderr)
    );
    assert_stderr_contains(
        &merge,
        "manual-curation-required: resolved metadata key 'topic' removes primary metadata but v0.1 patch ops cannot express metadata deletion",
    );
}

#[test]
fn store_merge_authoring_flow_rejects_novel_nested_parent_choice_as_manual_curation_required() {
    let store_dir = create_temp_dir("store-merge-nested-parent-manual-root");
    let (_key_dir, key_path) = write_signing_key_file("store-merge-nested-parent-manual-key");
    let (_resolved_dir, resolved_state_path) =
        write_nested_parent_manual_resolved_state_file("store-merge-nested-parent-manual-state");
    let store_root = path_arg(&store_dir.path().to_path_buf());
    let key_file = path_arg(&key_path);
    let resolved_state_file = path_arg(&resolved_state_path);

    let init = run_mycel(&["store", "init", &store_root, "--json"]);
    assert_success(&init);

    let document = run_mycel(&[
        "store",
        "create-document",
        &store_root,
        "--doc-id",
        "doc:author-smoke-nested-parent-manual",
        "--title",
        "Author Smoke Nested Parent Manual",
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

    let base_ops_dir = create_temp_dir("store-merge-nested-parent-manual-base-ops");
    let base_ops_path = base_ops_dir.path().join("ops.json");
    fs::write(
        &base_ops_path,
        serde_json::to_string_pretty(&json!([
            {
                "op": "insert_block",
                "new_block": {
                    "block_id": "blk:manual-left",
                    "block_type": "paragraph",
                    "content": "Left",
                    "attrs": {},
                    "children": []
                }
            },
            {
                "op": "insert_block",
                "new_block": {
                    "block_id": "blk:manual-right",
                    "block_type": "paragraph",
                    "content": "Right",
                    "attrs": {},
                    "children": []
                }
            },
            {
                "op": "insert_block",
                "new_block": {
                    "block_id": "blk:manual-leaf",
                    "block_type": "paragraph",
                    "content": "Leaf",
                    "attrs": {},
                    "children": []
                }
            }
        ]))
        .expect("nested parent manual base ops JSON should serialize"),
    )
    .expect("nested parent manual base ops JSON should write");
    let base_ops_file = path_arg(&base_ops_path);

    let wrapper_ops_dir = create_temp_dir("store-merge-nested-parent-manual-wrapper-ops");
    let wrapper_ops_path = wrapper_ops_dir.path().join("ops.json");
    fs::write(
        &wrapper_ops_path,
        serde_json::to_string_pretty(&json!([
            {
                "op": "insert_block",
                "parent_block_id": "blk:manual-right",
                "new_block": {
                    "block_id": "blk:manual-wrapper",
                    "block_type": "paragraph",
                    "content": "Wrapper",
                    "attrs": {},
                    "children": []
                }
            }
        ]))
        .expect("nested parent manual wrapper ops JSON should serialize"),
    )
    .expect("nested parent manual wrapper ops JSON should write");
    let wrapper_ops_file = path_arg(&wrapper_ops_path);

    let move_ops_dir = create_temp_dir("store-merge-nested-parent-manual-move-ops");
    let move_ops_path = move_ops_dir.path().join("ops.json");
    fs::write(
        &move_ops_path,
        serde_json::to_string_pretty(&json!([
            {
                "op": "move_block",
                "block_id": "blk:manual-leaf",
                "parent_block_id": "blk:manual-left"
            }
        ]))
        .expect("nested parent manual move ops JSON should serialize"),
    )
    .expect("nested parent manual move ops JSON should write");
    let move_ops_file = path_arg(&move_ops_path);

    let base_patch = run_mycel(&[
        "store",
        "create-patch",
        &store_root,
        "--doc-id",
        "doc:author-smoke-nested-parent-manual",
        "--base-revision",
        &genesis_revision_id,
        "--ops",
        &base_ops_file,
        "--signing-key",
        &key_file,
        "--timestamp",
        "41",
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
        "doc:author-smoke-nested-parent-manual",
        "--parent",
        &genesis_revision_id,
        "--patch",
        &base_patch_id,
        "--signing-key",
        &key_file,
        "--timestamp",
        "42",
        "--json",
    ]);
    assert_success(&base_revision);
    let base_revision_json = assert_json_status(&base_revision, "ok");
    let base_revision_id = base_revision_json["revision_id"]
        .as_str()
        .expect("base revision_id should be string")
        .to_string();

    let wrapper_patch = run_mycel(&[
        "store",
        "create-patch",
        &store_root,
        "--doc-id",
        "doc:author-smoke-nested-parent-manual",
        "--base-revision",
        &base_revision_id,
        "--ops",
        &wrapper_ops_file,
        "--signing-key",
        &key_file,
        "--timestamp",
        "43",
        "--json",
    ]);
    assert_success(&wrapper_patch);
    let wrapper_patch_json = assert_json_status(&wrapper_patch, "ok");
    let wrapper_patch_id = wrapper_patch_json["patch_id"]
        .as_str()
        .expect("wrapper patch_id should be string")
        .to_string();

    let wrapper_revision = run_mycel(&[
        "store",
        "commit-revision",
        &store_root,
        "--doc-id",
        "doc:author-smoke-nested-parent-manual",
        "--parent",
        &base_revision_id,
        "--patch",
        &wrapper_patch_id,
        "--signing-key",
        &key_file,
        "--timestamp",
        "44",
        "--json",
    ]);
    assert_success(&wrapper_revision);
    let wrapper_revision_json = assert_json_status(&wrapper_revision, "ok");
    let wrapper_revision_id = wrapper_revision_json["revision_id"]
        .as_str()
        .expect("wrapper revision_id should be string")
        .to_string();

    let move_patch = run_mycel(&[
        "store",
        "create-patch",
        &store_root,
        "--doc-id",
        "doc:author-smoke-nested-parent-manual",
        "--base-revision",
        &base_revision_id,
        "--ops",
        &move_ops_file,
        "--signing-key",
        &key_file,
        "--timestamp",
        "45",
        "--json",
    ]);
    assert_success(&move_patch);
    let move_patch_json = assert_json_status(&move_patch, "ok");
    let move_patch_id = move_patch_json["patch_id"]
        .as_str()
        .expect("move patch_id should be string")
        .to_string();

    let move_revision = run_mycel(&[
        "store",
        "commit-revision",
        &store_root,
        "--doc-id",
        "doc:author-smoke-nested-parent-manual",
        "--parent",
        &base_revision_id,
        "--patch",
        &move_patch_id,
        "--signing-key",
        &key_file,
        "--timestamp",
        "46",
        "--json",
    ]);
    assert_success(&move_revision);
    let move_revision_json = assert_json_status(&move_revision, "ok");
    let move_revision_id = move_revision_json["revision_id"]
        .as_str()
        .expect("move revision_id should be string")
        .to_string();

    let merge = run_mycel(&[
        "store",
        "create-merge-revision",
        &store_root,
        "--doc-id",
        "doc:author-smoke-nested-parent-manual",
        "--parent",
        &base_revision_id,
        "--parent",
        &wrapper_revision_id,
        "--parent",
        &move_revision_id,
        "--resolved-state",
        &resolved_state_file,
        "--signing-key",
        &key_file,
        "--timestamp",
        "47",
    ]);

    assert!(
        !merge.status.success(),
        "expected manual-curation failure, stdout: {}, stderr: {}",
        String::from_utf8_lossy(&merge.stdout),
        String::from_utf8_lossy(&merge.stderr)
    );
    assert_stderr_contains(
        &merge,
        "merge resolution is manual-curation-required: resolved block 'blk:manual-leaf' does not match any parent placement",
    );
}

#[test]
fn store_merge_authoring_flow_rejects_attr_variant_as_manual_curation_required() {
    let store_dir = create_temp_dir("store-merge-attrs-manual-root");
    let (_key_dir, key_path) = write_signing_key_file("store-merge-attrs-manual-key");
    let (_resolved_dir, resolved_state_path) =
        write_attrs_manual_resolved_state_file("store-merge-attrs-manual-state");
    let store_root = path_arg(&store_dir.path().to_path_buf());
    let key_file = path_arg(&key_path);
    let resolved_state_file = path_arg(&resolved_state_path);

    let init = run_mycel(&["store", "init", &store_root, "--json"]);
    assert_success(&init);

    let document = run_mycel(&[
        "store",
        "create-document",
        &store_root,
        "--doc-id",
        "doc:author-smoke-attrs-manual",
        "--title",
        "Author Smoke Attrs Manual",
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

    let base_ops_dir = create_temp_dir("store-merge-attrs-manual-base-ops");
    let base_ops_path = base_ops_dir.path().join("ops.json");
    fs::write(
        &base_ops_path,
        serde_json::to_string_pretty(&json!([
            {
                "op": "insert_block",
                "new_block": {
                    "block_id": "blk:merge-attrs",
                    "block_type": "paragraph",
                    "content": "Attrs",
                    "attrs": {},
                    "children": []
                }
            }
        ]))
        .expect("attrs manual base ops JSON should serialize"),
    )
    .expect("attrs manual base ops JSON should write");
    let base_ops_file = path_arg(&base_ops_path);

    let attrs_ops_dir = create_temp_dir("store-merge-attrs-manual-attrs-ops");
    let attrs_ops_path = attrs_ops_dir.path().join("ops.json");
    fs::write(
        &attrs_ops_path,
        serde_json::to_string_pretty(&json!([
            {
                "op": "delete_block",
                "block_id": "blk:merge-attrs"
            },
            {
                "op": "insert_block",
                "new_block": {
                    "block_id": "blk:merge-attrs",
                    "block_type": "paragraph",
                    "content": "Attrs",
                    "attrs": {
                        "style": "note"
                    },
                    "children": []
                }
            }
        ]))
        .expect("attrs manual variant ops JSON should serialize"),
    )
    .expect("attrs manual variant ops JSON should write");
    let attrs_ops_file = path_arg(&attrs_ops_path);

    let base_patch = run_mycel(&[
        "store",
        "create-patch",
        &store_root,
        "--doc-id",
        "doc:author-smoke-attrs-manual",
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
        "doc:author-smoke-attrs-manual",
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

    let attrs_patch = run_mycel(&[
        "store",
        "create-patch",
        &store_root,
        "--doc-id",
        "doc:author-smoke-attrs-manual",
        "--base-revision",
        &base_revision_id,
        "--ops",
        &attrs_ops_file,
        "--signing-key",
        &key_file,
        "--timestamp",
        "73",
        "--json",
    ]);
    assert_success(&attrs_patch);
    let attrs_patch_json = assert_json_status(&attrs_patch, "ok");
    let attrs_patch_id = attrs_patch_json["patch_id"]
        .as_str()
        .expect("attrs patch_id should be string")
        .to_string();

    let attrs_revision = run_mycel(&[
        "store",
        "commit-revision",
        &store_root,
        "--doc-id",
        "doc:author-smoke-attrs-manual",
        "--parent",
        &base_revision_id,
        "--patch",
        &attrs_patch_id,
        "--signing-key",
        &key_file,
        "--timestamp",
        "74",
        "--json",
    ]);
    assert_success(&attrs_revision);
    let attrs_revision_json = assert_json_status(&attrs_revision, "ok");
    let attrs_revision_id = attrs_revision_json["revision_id"]
        .as_str()
        .expect("attrs revision_id should be string")
        .to_string();

    let merge = run_mycel(&[
        "store",
        "create-merge-revision",
        &store_root,
        "--doc-id",
        "doc:author-smoke-attrs-manual",
        "--parent",
        &base_revision_id,
        "--parent",
        &attrs_revision_id,
        "--resolved-state",
        &resolved_state_file,
        "--signing-key",
        &key_file,
        "--timestamp",
        "75",
    ]);

    assert!(
        !merge.status.success(),
        "expected manual-curation failure, stdout: {}, stderr: {}",
        String::from_utf8_lossy(&merge.stdout),
        String::from_utf8_lossy(&merge.stderr)
    );
    assert_stderr_contains(
        &merge,
        "manual-curation-required: block 'blk:merge-attrs' changes attrs in an unsupported way",
    );
}
