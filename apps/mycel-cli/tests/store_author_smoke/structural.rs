use super::*;

#[test]
fn store_merge_authoring_flow_reports_nested_parent_choice_as_multi_variant() {
    let store_dir = create_temp_dir("store-merge-nested-parent-root");
    let (_key_dir, key_path) = write_signing_key_file("store-merge-nested-parent-key");
    let (_resolved_dir, resolved_state_path) =
        write_nested_parent_choice_resolved_state_file("store-merge-nested-parent-state");
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
        "doc:author-smoke-nested-parent-choice",
        "--title",
        "Author Smoke Nested Parent Choice",
        "--language",
        "en",
        "--signing-key",
        &key_file,
        "--timestamp",
        "30",
        "--json",
    ]);
    assert_success(&document);
    let document_json = assert_json_status(&document, "ok");
    let genesis_revision_id = document_json["genesis_revision_id"]
        .as_str()
        .expect("genesis revision should be string")
        .to_string();

    let base_ops_dir = create_temp_dir("store-merge-nested-parent-base-ops");
    let base_ops_path = base_ops_dir.path().join("ops.json");
    fs::write(
        &base_ops_path,
        serde_json::to_string_pretty(&json!([
            {
                "op": "insert_block",
                "new_block": {
                    "block_id": "blk:nested-left",
                    "block_type": "paragraph",
                    "content": "Left",
                    "attrs": {},
                    "children": []
                }
            },
            {
                "op": "insert_block",
                "new_block": {
                    "block_id": "blk:nested-right",
                    "block_type": "paragraph",
                    "content": "Right",
                    "attrs": {},
                    "children": []
                }
            },
            {
                "op": "insert_block",
                "new_block": {
                    "block_id": "blk:nested-leaf",
                    "block_type": "paragraph",
                    "content": "Leaf",
                    "attrs": {},
                    "children": []
                }
            }
        ]))
        .expect("nested parent base ops JSON should serialize"),
    )
    .expect("nested parent base ops JSON should write");
    let base_ops_file = path_arg(&base_ops_path);

    let wrapper_ops_dir = create_temp_dir("store-merge-nested-parent-wrapper-ops");
    let wrapper_ops_path = wrapper_ops_dir.path().join("ops.json");
    fs::write(
        &wrapper_ops_path,
        serde_json::to_string_pretty(&json!([
            {
                "op": "insert_block",
                "new_block": {
                    "block_id": "blk:nested-wrapper",
                    "block_type": "paragraph",
                    "content": "Wrapper",
                    "attrs": {},
                    "children": []
                }
            }
        ]))
        .expect("nested parent wrapper ops JSON should serialize"),
    )
    .expect("nested parent wrapper ops JSON should write");
    let wrapper_ops_file = path_arg(&wrapper_ops_path);

    let left_ops_dir = create_temp_dir("store-merge-nested-parent-left-ops");
    let left_ops_path = left_ops_dir.path().join("ops.json");
    fs::write(
        &left_ops_path,
        serde_json::to_string_pretty(&json!([
            {
                "op": "move_block",
                "block_id": "blk:nested-leaf",
                "parent_block_id": "blk:nested-left"
            }
        ]))
        .expect("nested parent left ops JSON should serialize"),
    )
    .expect("nested parent left ops JSON should write");
    let left_ops_file = path_arg(&left_ops_path);

    let right_ops_dir = create_temp_dir("store-merge-nested-parent-right-ops");
    let right_ops_path = right_ops_dir.path().join("ops.json");
    fs::write(
        &right_ops_path,
        serde_json::to_string_pretty(&json!([
            {
                "op": "move_block",
                "block_id": "blk:nested-leaf",
                "parent_block_id": "blk:nested-right"
            }
        ]))
        .expect("nested parent right ops JSON should serialize"),
    )
    .expect("nested parent right ops JSON should write");
    let right_ops_file = path_arg(&right_ops_path);

    let base_patch = run_mycel(&[
        "store",
        "create-patch",
        &store_root,
        "--doc-id",
        "doc:author-smoke-nested-parent-choice",
        "--base-revision",
        &genesis_revision_id,
        "--ops",
        &base_ops_file,
        "--signing-key",
        &key_file,
        "--timestamp",
        "31",
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
        "doc:author-smoke-nested-parent-choice",
        "--parent",
        &genesis_revision_id,
        "--patch",
        &base_patch_id,
        "--signing-key",
        &key_file,
        "--timestamp",
        "32",
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
        "doc:author-smoke-nested-parent-choice",
        "--base-revision",
        &base_revision_id,
        "--ops",
        &wrapper_ops_file,
        "--signing-key",
        &key_file,
        "--timestamp",
        "33",
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
        "doc:author-smoke-nested-parent-choice",
        "--parent",
        &base_revision_id,
        "--patch",
        &wrapper_patch_id,
        "--signing-key",
        &key_file,
        "--timestamp",
        "34",
        "--json",
    ]);
    assert_success(&wrapper_revision);
    let wrapper_revision_json = assert_json_status(&wrapper_revision, "ok");
    let wrapper_revision_id = wrapper_revision_json["revision_id"]
        .as_str()
        .expect("wrapper revision_id should be string")
        .to_string();

    let left_patch = run_mycel(&[
        "store",
        "create-patch",
        &store_root,
        "--doc-id",
        "doc:author-smoke-nested-parent-choice",
        "--base-revision",
        &base_revision_id,
        "--ops",
        &left_ops_file,
        "--signing-key",
        &key_file,
        "--timestamp",
        "35",
        "--json",
    ]);
    assert_success(&left_patch);
    let left_patch_json = assert_json_status(&left_patch, "ok");
    let left_patch_id = left_patch_json["patch_id"]
        .as_str()
        .expect("left patch_id should be string")
        .to_string();

    let left_revision = run_mycel(&[
        "store",
        "commit-revision",
        &store_root,
        "--doc-id",
        "doc:author-smoke-nested-parent-choice",
        "--parent",
        &base_revision_id,
        "--patch",
        &left_patch_id,
        "--signing-key",
        &key_file,
        "--timestamp",
        "36",
        "--json",
    ]);
    assert_success(&left_revision);
    let left_revision_json = assert_json_status(&left_revision, "ok");
    let left_revision_id = left_revision_json["revision_id"]
        .as_str()
        .expect("left revision_id should be string")
        .to_string();

    let right_patch = run_mycel(&[
        "store",
        "create-patch",
        &store_root,
        "--doc-id",
        "doc:author-smoke-nested-parent-choice",
        "--base-revision",
        &base_revision_id,
        "--ops",
        &right_ops_file,
        "--signing-key",
        &key_file,
        "--timestamp",
        "37",
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
        "doc:author-smoke-nested-parent-choice",
        "--parent",
        &base_revision_id,
        "--patch",
        &right_patch_id,
        "--signing-key",
        &key_file,
        "--timestamp",
        "38",
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
        "doc:author-smoke-nested-parent-choice",
        "--parent",
        &base_revision_id,
        "--parent",
        &wrapper_revision_id,
        "--parent",
        &left_revision_id,
        "--parent",
        &right_revision_id,
        "--resolved-state",
        &resolved_state_file,
        "--signing-key",
        &key_file,
        "--timestamp",
        "39",
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
                    reason.contains("selected a non-primary parent placement")
                })
            })),
        "expected nested parent multi-variant reason, got {merge_json}"
    );
    assert!(
        merge_json["merge_reasons"]
            .as_array()
            .is_some_and(|reasons| reasons.iter().any(|reason| {
                reason
                    .as_str()
                    .is_some_and(|reason| reason.contains("multiple competing parent placements"))
            })),
        "expected competing nested parent placement reason, got {merge_json}"
    );
    assert_eq!(merge_json["patch_op_count"], 4);
    assert_eq!(
        merge_json["parent_revision_ids"].as_array().map(Vec::len),
        Some(4)
    );

    let index = run_mycel(&["store", "index", &store_root, "--json"]);
    assert_success(&index);
    let index_json = assert_json_status(&index, "ok");
    assert_eq!(index_json["stored_object_count"], 12);
    assert_eq!(
        index_json["doc_revisions"]["doc:author-smoke-nested-parent-choice"]
            .as_array()
            .map(Vec::len),
        Some(6)
    );
    assert_eq!(
        index_json["object_ids_by_type"]["patch"]
            .as_array()
            .map(Vec::len),
        Some(5)
    );
    assert_eq!(
        index_json["object_ids_by_type"]["revision"]
            .as_array()
            .map(Vec::len),
        Some(6)
    );
}

#[test]
fn store_merge_authoring_flow_reports_anchor_nested_parent_choice_as_multi_variant() {
    let store_dir = create_temp_dir("store-merge-nested-parent-anchor-root");
    let (_key_dir, key_path) = write_signing_key_file("store-merge-nested-parent-anchor-key");
    let (_resolved_dir, resolved_state_path) =
        write_nested_parent_anchor_choice_resolved_state_file(
            "store-merge-nested-parent-anchor-state",
        );
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
        "doc:author-smoke-nested-parent-anchor-choice",
        "--title",
        "Author Smoke Nested Parent Anchor Choice",
        "--language",
        "en",
        "--signing-key",
        &key_file,
        "--timestamp",
        "30",
        "--json",
    ]);
    assert_success(&document);
    let document_json = assert_json_status(&document, "ok");
    let genesis_revision_id = document_json["genesis_revision_id"]
        .as_str()
        .expect("genesis revision should be string")
        .to_string();

    let base_ops_dir = create_temp_dir("store-merge-nested-parent-anchor-base-ops");
    let base_ops_path = base_ops_dir.path().join("ops.json");
    fs::write(
        &base_ops_path,
        serde_json::to_string_pretty(&json!([
            {
                "op": "insert_block",
                "new_block": {
                    "block_id": "blk:nested-left",
                    "block_type": "paragraph",
                    "content": "Left",
                    "attrs": {},
                    "children": []
                }
            },
            {
                "op": "insert_block",
                "new_block": {
                    "block_id": "blk:nested-right",
                    "block_type": "paragraph",
                    "content": "Right",
                    "attrs": {},
                    "children": []
                }
            },
            {
                "op": "insert_block",
                "new_block": {
                    "block_id": "blk:nested-leaf",
                    "block_type": "paragraph",
                    "content": "Leaf",
                    "attrs": {},
                    "children": []
                }
            }
        ]))
        .expect("nested parent anchor base ops JSON should serialize"),
    )
    .expect("nested parent anchor base ops JSON should write");
    let base_ops_file = path_arg(&base_ops_path);

    let subsection_ops_dir = create_temp_dir("store-merge-nested-parent-anchor-subsection-ops");
    let subsection_ops_path = subsection_ops_dir.path().join("ops.json");
    fs::write(
        &subsection_ops_path,
        serde_json::to_string_pretty(&json!([
            {
                "op": "insert_block",
                "parent_block_id": "blk:nested-left",
                "new_block": {
                    "block_id": "blk:nested-subsection",
                    "block_type": "paragraph",
                    "content": "Subsection",
                    "attrs": {},
                    "children": []
                }
            }
        ]))
        .expect("nested parent anchor subsection ops JSON should serialize"),
    )
    .expect("nested parent anchor subsection ops JSON should write");
    let subsection_ops_file = path_arg(&subsection_ops_path);

    let left_ops_dir = create_temp_dir("store-merge-nested-parent-anchor-left-ops");
    let left_ops_path = left_ops_dir.path().join("ops.json");
    fs::write(
        &left_ops_path,
        serde_json::to_string_pretty(&json!([
            {
                "op": "move_block",
                "block_id": "blk:nested-leaf",
                "parent_block_id": "blk:nested-left"
            }
        ]))
        .expect("nested parent anchor left ops JSON should serialize"),
    )
    .expect("nested parent anchor left ops JSON should write");
    let left_ops_file = path_arg(&left_ops_path);

    let right_ops_dir = create_temp_dir("store-merge-nested-parent-anchor-right-ops");
    let right_ops_path = right_ops_dir.path().join("ops.json");
    fs::write(
        &right_ops_path,
        serde_json::to_string_pretty(&json!([
            {
                "op": "move_block",
                "block_id": "blk:nested-leaf",
                "parent_block_id": "blk:nested-right"
            }
        ]))
        .expect("nested parent anchor right ops JSON should serialize"),
    )
    .expect("nested parent anchor right ops JSON should write");
    let right_ops_file = path_arg(&right_ops_path);

    let base_patch = run_mycel(&[
        "store",
        "create-patch",
        &store_root,
        "--doc-id",
        "doc:author-smoke-nested-parent-anchor-choice",
        "--base-revision",
        &genesis_revision_id,
        "--ops",
        &base_ops_file,
        "--signing-key",
        &key_file,
        "--timestamp",
        "31",
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
        "doc:author-smoke-nested-parent-anchor-choice",
        "--parent",
        &genesis_revision_id,
        "--patch",
        &base_patch_id,
        "--signing-key",
        &key_file,
        "--timestamp",
        "32",
        "--json",
    ]);
    assert_success(&base_revision);
    let base_revision_json = assert_json_status(&base_revision, "ok");
    let base_revision_id = base_revision_json["revision_id"]
        .as_str()
        .expect("base revision_id should be string")
        .to_string();

    let subsection_patch = run_mycel(&[
        "store",
        "create-patch",
        &store_root,
        "--doc-id",
        "doc:author-smoke-nested-parent-anchor-choice",
        "--base-revision",
        &base_revision_id,
        "--ops",
        &subsection_ops_file,
        "--signing-key",
        &key_file,
        "--timestamp",
        "33",
        "--json",
    ]);
    assert_success(&subsection_patch);
    let subsection_patch_json = assert_json_status(&subsection_patch, "ok");
    let subsection_patch_id = subsection_patch_json["patch_id"]
        .as_str()
        .expect("subsection patch_id should be string")
        .to_string();

    let subsection_revision = run_mycel(&[
        "store",
        "commit-revision",
        &store_root,
        "--doc-id",
        "doc:author-smoke-nested-parent-anchor-choice",
        "--parent",
        &base_revision_id,
        "--patch",
        &subsection_patch_id,
        "--signing-key",
        &key_file,
        "--timestamp",
        "34",
        "--json",
    ]);
    assert_success(&subsection_revision);
    let subsection_revision_json = assert_json_status(&subsection_revision, "ok");
    let subsection_revision_id = subsection_revision_json["revision_id"]
        .as_str()
        .expect("subsection revision_id should be string")
        .to_string();

    let left_patch = run_mycel(&[
        "store",
        "create-patch",
        &store_root,
        "--doc-id",
        "doc:author-smoke-nested-parent-anchor-choice",
        "--base-revision",
        &base_revision_id,
        "--ops",
        &left_ops_file,
        "--signing-key",
        &key_file,
        "--timestamp",
        "35",
        "--json",
    ]);
    assert_success(&left_patch);
    let left_patch_json = assert_json_status(&left_patch, "ok");
    let left_patch_id = left_patch_json["patch_id"]
        .as_str()
        .expect("left patch_id should be string")
        .to_string();

    let left_revision = run_mycel(&[
        "store",
        "commit-revision",
        &store_root,
        "--doc-id",
        "doc:author-smoke-nested-parent-anchor-choice",
        "--parent",
        &base_revision_id,
        "--patch",
        &left_patch_id,
        "--signing-key",
        &key_file,
        "--timestamp",
        "36",
        "--json",
    ]);
    assert_success(&left_revision);
    let left_revision_json = assert_json_status(&left_revision, "ok");
    let left_revision_id = left_revision_json["revision_id"]
        .as_str()
        .expect("left revision_id should be string")
        .to_string();

    let right_patch = run_mycel(&[
        "store",
        "create-patch",
        &store_root,
        "--doc-id",
        "doc:author-smoke-nested-parent-anchor-choice",
        "--base-revision",
        &base_revision_id,
        "--ops",
        &right_ops_file,
        "--signing-key",
        &key_file,
        "--timestamp",
        "37",
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
        "doc:author-smoke-nested-parent-anchor-choice",
        "--parent",
        &base_revision_id,
        "--patch",
        &right_patch_id,
        "--signing-key",
        &key_file,
        "--timestamp",
        "38",
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
        "doc:author-smoke-nested-parent-anchor-choice",
        "--parent",
        &base_revision_id,
        "--parent",
        &subsection_revision_id,
        "--parent",
        &left_revision_id,
        "--parent",
        &right_revision_id,
        "--resolved-state",
        &resolved_state_file,
        "--signing-key",
        &key_file,
        "--timestamp",
        "39",
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
                    reason.contains("selected a non-primary parent placement")
                })
            })),
        "expected anchor nested parent multi-variant reason, got {merge_json}"
    );
    assert!(
        merge_json["merge_reasons"]
            .as_array()
            .is_some_and(|reasons| reasons.iter().any(|reason| {
                reason
                    .as_str()
                    .is_some_and(|reason| reason.contains("multiple competing parent placements"))
            })),
        "expected competing anchor nested parent placement reason, got {merge_json}"
    );
}

#[test]
fn store_merge_authoring_flow_marks_nested_sibling_choice_through_inserted_sibling_as_multi_variant(
) {
    let store_dir = create_temp_dir("store-merge-nested-sibling-manual-root");
    let (_key_dir, key_path) = write_signing_key_file("store-merge-nested-sibling-manual-key");
    let (_resolved_dir, resolved_state_path) =
        write_nested_sibling_manual_resolved_state_file("store-merge-nested-sibling-manual-state");
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
        "doc:author-smoke-nested-sibling-manual",
        "--title",
        "Author Smoke Nested Sibling Manual",
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

    let base_ops_dir = create_temp_dir("store-merge-nested-sibling-manual-base-ops");
    let base_ops_path = base_ops_dir.path().join("ops.json");
    fs::write(
        &base_ops_path,
        serde_json::to_string_pretty(&json!([
            {
                "op": "insert_block",
                "new_block": {
                    "block_id": "blk:nested-parent",
                    "block_type": "paragraph",
                    "content": "Parent",
                    "attrs": {},
                    "children": [
                        {
                            "block_id": "blk:nested-child-a",
                            "block_type": "paragraph",
                            "content": "Child A",
                            "attrs": {},
                            "children": []
                        },
                        {
                            "block_id": "blk:nested-child-b",
                            "block_type": "paragraph",
                            "content": "Child B",
                            "attrs": {},
                            "children": []
                        },
                        {
                            "block_id": "blk:nested-child-c",
                            "block_type": "paragraph",
                            "content": "Child C",
                            "attrs": {},
                            "children": []
                        }
                    ]
                }
            }
        ]))
        .expect("nested sibling manual base ops JSON should serialize"),
    )
    .expect("nested sibling manual base ops JSON should write");
    let base_ops_file = path_arg(&base_ops_path);

    let insert_ops_dir = create_temp_dir("store-merge-nested-sibling-manual-insert-ops");
    let insert_ops_path = insert_ops_dir.path().join("ops.json");
    fs::write(
        &insert_ops_path,
        serde_json::to_string_pretty(&json!([
            {
                "op": "insert_block_after",
                "after_block_id": "blk:nested-child-b",
                "new_block": {
                    "block_id": "blk:nested-child-d",
                    "block_type": "paragraph",
                    "content": "Child D",
                    "attrs": {},
                    "children": []
                }
            }
        ]))
        .expect("nested sibling manual insert ops JSON should serialize"),
    )
    .expect("nested sibling manual insert ops JSON should write");
    let insert_ops_file = path_arg(&insert_ops_path);

    let move_ops_dir = create_temp_dir("store-merge-nested-sibling-manual-move-ops");
    let move_ops_path = move_ops_dir.path().join("ops.json");
    fs::write(
        &move_ops_path,
        serde_json::to_string_pretty(&json!([
            {
                "op": "move_block",
                "block_id": "blk:nested-child-a",
                "parent_block_id": "blk:nested-parent",
                "after_block_id": "blk:nested-child-b"
            }
        ]))
        .expect("nested sibling manual move ops JSON should serialize"),
    )
    .expect("nested sibling manual move ops JSON should write");
    let move_ops_file = path_arg(&move_ops_path);

    let base_patch = run_mycel(&[
        "store",
        "create-patch",
        &store_root,
        "--doc-id",
        "doc:author-smoke-nested-sibling-manual",
        "--base-revision",
        &genesis_revision_id,
        "--ops",
        &base_ops_file,
        "--signing-key",
        &key_file,
        "--timestamp",
        "51",
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
        "doc:author-smoke-nested-sibling-manual",
        "--parent",
        &genesis_revision_id,
        "--patch",
        &base_patch_id,
        "--signing-key",
        &key_file,
        "--timestamp",
        "52",
        "--json",
    ]);
    assert_success(&base_revision);
    let base_revision_json = assert_json_status(&base_revision, "ok");
    let base_revision_id = base_revision_json["revision_id"]
        .as_str()
        .expect("base revision_id should be string")
        .to_string();

    let insert_patch = run_mycel(&[
        "store",
        "create-patch",
        &store_root,
        "--doc-id",
        "doc:author-smoke-nested-sibling-manual",
        "--base-revision",
        &base_revision_id,
        "--ops",
        &insert_ops_file,
        "--signing-key",
        &key_file,
        "--timestamp",
        "53",
        "--json",
    ]);
    assert_success(&insert_patch);
    let insert_patch_json = assert_json_status(&insert_patch, "ok");
    let insert_patch_id = insert_patch_json["patch_id"]
        .as_str()
        .expect("insert patch_id should be string")
        .to_string();

    let insert_revision = run_mycel(&[
        "store",
        "commit-revision",
        &store_root,
        "--doc-id",
        "doc:author-smoke-nested-sibling-manual",
        "--parent",
        &base_revision_id,
        "--patch",
        &insert_patch_id,
        "--signing-key",
        &key_file,
        "--timestamp",
        "54",
        "--json",
    ]);
    assert_success(&insert_revision);
    let insert_revision_json = assert_json_status(&insert_revision, "ok");
    let insert_revision_id = insert_revision_json["revision_id"]
        .as_str()
        .expect("insert revision_id should be string")
        .to_string();

    let move_patch = run_mycel(&[
        "store",
        "create-patch",
        &store_root,
        "--doc-id",
        "doc:author-smoke-nested-sibling-manual",
        "--base-revision",
        &base_revision_id,
        "--ops",
        &move_ops_file,
        "--signing-key",
        &key_file,
        "--timestamp",
        "55",
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
        "doc:author-smoke-nested-sibling-manual",
        "--parent",
        &base_revision_id,
        "--patch",
        &move_patch_id,
        "--signing-key",
        &key_file,
        "--timestamp",
        "56",
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
        "doc:author-smoke-nested-sibling-manual",
        "--parent",
        &base_revision_id,
        "--parent",
        &insert_revision_id,
        "--parent",
        &move_revision_id,
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
        merge_json["merge_reasons"]
            .as_array()
            .is_some_and(|reasons| reasons.iter().any(|reason| {
                reason.as_str().is_some_and(|reason| {
                    reason.contains("selected a non-primary sibling placement")
                })
            })),
        "expected nested sibling multi-variant reason, got {merge_json}"
    );
    assert_eq!(merge_json["patch_op_count"], 2);
}

#[test]
fn store_merge_authoring_flow_marks_deep_composed_branch_reuse_as_multi_variant() {
    let store_dir = create_temp_dir("store-merge-composed-manual-root");
    let (_key_dir, key_path) = write_signing_key_file("store-merge-composed-manual-key");
    let (_resolved_dir, resolved_state_path) =
        write_composed_branch_manual_resolved_state_file("store-merge-composed-manual-state");
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
        "doc:author-smoke-composed-manual",
        "--title",
        "Author Smoke Composed Manual",
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

    let base_ops_dir = create_temp_dir("store-merge-composed-manual-base-ops");
    let base_ops_path = base_ops_dir.path().join("ops.json");
    fs::write(
        &base_ops_path,
        serde_json::to_string_pretty(&json!([
            {
                "op": "insert_block",
                "new_block": {
                    "block_id": "blk:cmp-anchor",
                    "block_type": "paragraph",
                    "content": "Anchor",
                    "attrs": {},
                    "children": []
                }
            },
            {
                "op": "insert_block",
                "new_block": {
                    "block_id": "blk:cmp-old-parent",
                    "block_type": "paragraph",
                    "content": "Old Parent",
                    "attrs": {},
                    "children": [
                        {
                            "block_id": "blk:cmp-leaf-a",
                            "block_type": "paragraph",
                            "content": "Leaf A",
                            "attrs": {},
                            "children": []
                        },
                        {
                            "block_id": "blk:cmp-leaf-b",
                            "block_type": "paragraph",
                            "content": "Leaf B",
                            "attrs": {},
                            "children": []
                        }
                    ]
                }
            }
        ]))
        .expect("composed manual base ops JSON should serialize"),
    )
    .expect("composed manual base ops JSON should write");
    let base_ops_file = path_arg(&base_ops_path);

    let delete_ops_dir = create_temp_dir("store-merge-composed-manual-delete-ops");
    let delete_ops_path = delete_ops_dir.path().join("ops.json");
    fs::write(
        &delete_ops_path,
        serde_json::to_string_pretty(&json!([
            {
                "op": "delete_block",
                "block_id": "blk:cmp-old-parent"
            }
        ]))
        .expect("composed manual delete ops JSON should serialize"),
    )
    .expect("composed manual delete ops JSON should write");
    let delete_ops_file = path_arg(&delete_ops_path);

    let insert_ops_dir = create_temp_dir("store-merge-composed-manual-insert-ops");
    let insert_ops_path = insert_ops_dir.path().join("ops.json");
    fs::write(
        &insert_ops_path,
        serde_json::to_string_pretty(&json!([
            {
                "op": "insert_block_after",
                "after_block_id": "blk:cmp-anchor",
                "new_block": {
                    "block_id": "blk:cmp-wrapper",
                    "block_type": "paragraph",
                    "content": "Wrapper",
                    "attrs": {},
                    "children": [
                        {
                            "block_id": "blk:cmp-section",
                            "block_type": "paragraph",
                            "content": "Section",
                            "attrs": {},
                            "children": [
                                {
                                    "block_id": "blk:cmp-subsection",
                                    "block_type": "paragraph",
                                    "content": "Subsection",
                                    "attrs": {},
                                    "children": []
                                }
                            ]
                        }
                    ]
                }
            }
        ]))
        .expect("composed manual insert ops JSON should serialize"),
    )
    .expect("composed manual insert ops JSON should write");
    let insert_ops_file = path_arg(&insert_ops_path);

    let base_patch = run_mycel(&[
        "store",
        "create-patch",
        &store_root,
        "--doc-id",
        "doc:author-smoke-composed-manual",
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
        "doc:author-smoke-composed-manual",
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

    let delete_patch = run_mycel(&[
        "store",
        "create-patch",
        &store_root,
        "--doc-id",
        "doc:author-smoke-composed-manual",
        "--base-revision",
        &base_revision_id,
        "--ops",
        &delete_ops_file,
        "--signing-key",
        &key_file,
        "--timestamp",
        "63",
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
        "doc:author-smoke-composed-manual",
        "--parent",
        &base_revision_id,
        "--patch",
        &delete_patch_id,
        "--signing-key",
        &key_file,
        "--timestamp",
        "64",
        "--json",
    ]);
    assert_success(&delete_revision);
    let delete_revision_json = assert_json_status(&delete_revision, "ok");
    let delete_revision_id = delete_revision_json["revision_id"]
        .as_str()
        .expect("delete revision_id should be string")
        .to_string();

    let insert_patch = run_mycel(&[
        "store",
        "create-patch",
        &store_root,
        "--doc-id",
        "doc:author-smoke-composed-manual",
        "--base-revision",
        &base_revision_id,
        "--ops",
        &insert_ops_file,
        "--signing-key",
        &key_file,
        "--timestamp",
        "65",
        "--json",
    ]);
    assert_success(&insert_patch);
    let insert_patch_json = assert_json_status(&insert_patch, "ok");
    let insert_patch_id = insert_patch_json["patch_id"]
        .as_str()
        .expect("insert patch_id should be string")
        .to_string();

    let insert_revision = run_mycel(&[
        "store",
        "commit-revision",
        &store_root,
        "--doc-id",
        "doc:author-smoke-composed-manual",
        "--parent",
        &base_revision_id,
        "--patch",
        &insert_patch_id,
        "--signing-key",
        &key_file,
        "--timestamp",
        "66",
        "--json",
    ]);
    assert_success(&insert_revision);
    let insert_revision_json = assert_json_status(&insert_revision, "ok");
    let insert_revision_id = insert_revision_json["revision_id"]
        .as_str()
        .expect("insert revision_id should be string")
        .to_string();

    let merge = run_mycel(&[
        "store",
        "create-merge-revision",
        &store_root,
        "--doc-id",
        "doc:author-smoke-composed-manual",
        "--parent",
        &base_revision_id,
        "--parent",
        &delete_revision_id,
        "--parent",
        &insert_revision_id,
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
                    reason.contains("selected a non-primary parent placement")
                })
            })),
        "expected composed branch multi-variant reason, got {merge_json}"
    );
    assert_eq!(merge_json["patch_op_count"], 4);
}
