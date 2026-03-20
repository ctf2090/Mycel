use super::*;

#[test]
fn store_authoring_flow_creates_document_patch_and_revision() {
    let store_dir = create_temp_dir("store-author-root");
    let (_key_dir, key_path) = write_signing_key_file("store-author-key");
    let (_ops_dir, ops_path) = write_ops_file("store-author-ops");
    let store_root = path_arg(&store_dir.path().to_path_buf());
    let key_file = path_arg(&key_path);
    let ops_file = path_arg(&ops_path);

    let init = run_mycel(&["store", "init", &store_root, "--json"]);
    assert_success(&init);
    let init_json = assert_json_status(&init, "ok");
    assert_eq!(init_json["store_root"], store_root);

    let document = run_mycel(&[
        "store",
        "create-document",
        &store_root,
        "--doc-id",
        "doc:author-smoke",
        "--title",
        "Author Smoke",
        "--language",
        "en",
        "--signing-key",
        &key_file,
        "--timestamp",
        "10",
        "--json",
    ]);
    assert_success(&document);
    let document_json = assert_json_status(&document, "ok");
    let genesis_revision_id = document_json["genesis_revision_id"]
        .as_str()
        .expect("genesis revision should be string")
        .to_string();
    assert_eq!(document_json["written_object_count"], 2);

    let patch = run_mycel(&[
        "store",
        "create-patch",
        &store_root,
        "--doc-id",
        "doc:author-smoke",
        "--base-revision",
        &genesis_revision_id,
        "--ops",
        &ops_file,
        "--signing-key",
        &key_file,
        "--timestamp",
        "11",
        "--json",
    ]);
    assert_success(&patch);
    let patch_json = assert_json_status(&patch, "ok");
    let patch_id = patch_json["patch_id"]
        .as_str()
        .expect("patch_id should be string")
        .to_string();
    assert_eq!(patch_json["written_object_count"], 1);

    let revision = run_mycel(&[
        "store",
        "commit-revision",
        &store_root,
        "--doc-id",
        "doc:author-smoke",
        "--parent",
        &genesis_revision_id,
        "--patch",
        &patch_id,
        "--signing-key",
        &key_file,
        "--timestamp",
        "12",
        "--json",
    ]);
    assert_success(&revision);
    let revision_json = assert_json_status(&revision, "ok");
    assert_eq!(revision_json["written_object_count"], 1);
    assert!(revision_json["recomputed_state_hash"]
        .as_str()
        .is_some_and(|value| value.starts_with("hash:")));

    let index = run_mycel(&["store", "index", &store_root, "--json"]);
    assert_success(&index);
    let index_json = assert_json_status(&index, "ok");
    assert_eq!(index_json["stored_object_count"], 4);
    assert_eq!(
        index_json["doc_revisions"]["doc:author-smoke"]
            .as_array()
            .map(Vec::len),
        Some(2)
    );
    assert_eq!(
        index_json["object_ids_by_type"]["document"]
            .as_array()
            .map(Vec::len),
        Some(1)
    );
    assert_eq!(
        index_json["object_ids_by_type"]["patch"]
            .as_array()
            .map(Vec::len),
        Some(1)
    );
    assert_eq!(
        index_json["object_ids_by_type"]["revision"]
            .as_array()
            .map(Vec::len),
        Some(2)
    );

    let rebuild = run_mycel(&["store", "rebuild", &store_root, "--json"]);
    assert_success(&rebuild);
    let rebuild_json = assert_json_status(&rebuild, "ok");
    assert_eq!(rebuild_json["stored_object_count"], 4);
    assert_eq!(rebuild_json["verified_object_count"], 4);
}

#[test]
fn store_merge_authoring_flow_creates_merge_patch_and_revision() {
    let store_dir = create_temp_dir("store-merge-root");
    let (_key_dir, key_path) = write_signing_key_file("store-merge-key");
    let (_ops_dir, ops_path) = write_ops_file("store-merge-ops");
    let (_resolved_dir, resolved_state_path) = write_resolved_state_file("store-merge-state");
    let store_root = path_arg(&store_dir.path().to_path_buf());
    let key_file = path_arg(&key_path);
    let ops_file = path_arg(&ops_path);
    let resolved_state_file = path_arg(&resolved_state_path);

    let init = run_mycel(&["store", "init", &store_root, "--json"]);
    assert_success(&init);

    let document = run_mycel(&[
        "store",
        "create-document",
        &store_root,
        "--doc-id",
        "doc:author-smoke",
        "--title",
        "Author Smoke Merge",
        "--language",
        "en",
        "--signing-key",
        &key_file,
        "--timestamp",
        "10",
        "--json",
    ]);
    assert_success(&document);
    let document_json = assert_json_status(&document, "ok");
    let genesis_revision_id = document_json["genesis_revision_id"]
        .as_str()
        .expect("genesis revision should be string")
        .to_string();

    let primary_patch = run_mycel(&[
        "store",
        "create-patch",
        &store_root,
        "--doc-id",
        "doc:author-smoke",
        "--base-revision",
        &genesis_revision_id,
        "--ops",
        &ops_file,
        "--signing-key",
        &key_file,
        "--timestamp",
        "11",
        "--json",
    ]);
    assert_success(&primary_patch);
    let primary_patch_json = assert_json_status(&primary_patch, "ok");
    let primary_patch_id = primary_patch_json["patch_id"]
        .as_str()
        .expect("patch_id should be string")
        .to_string();

    let primary_revision = run_mycel(&[
        "store",
        "commit-revision",
        &store_root,
        "--doc-id",
        "doc:author-smoke",
        "--parent",
        &genesis_revision_id,
        "--patch",
        &primary_patch_id,
        "--signing-key",
        &key_file,
        "--timestamp",
        "12",
        "--json",
    ]);
    assert_success(&primary_revision);
    let primary_revision_json = assert_json_status(&primary_revision, "ok");
    let primary_revision_id = primary_revision_json["revision_id"]
        .as_str()
        .expect("revision_id should be string")
        .to_string();

    let side_ops_dir = create_temp_dir("store-merge-side-ops");
    let side_ops_path = side_ops_dir.path().join("ops.json");
    fs::write(
        &side_ops_path,
        serde_json::to_string_pretty(&json!([
            {
                "op": "insert_block",
                "new_block": {
                    "block_id": "blk:author-smoke-merge-002",
                    "block_type": "paragraph",
                    "content": "Merged side branch",
                    "attrs": {},
                    "children": []
                }
            }
        ]))
        .expect("side ops JSON should serialize"),
    )
    .expect("side ops JSON should write");
    let side_ops_file = path_arg(&side_ops_path);

    let side_patch = run_mycel(&[
        "store",
        "create-patch",
        &store_root,
        "--doc-id",
        "doc:author-smoke",
        "--base-revision",
        &genesis_revision_id,
        "--ops",
        &side_ops_file,
        "--signing-key",
        &key_file,
        "--timestamp",
        "13",
        "--json",
    ]);
    assert_success(&side_patch);
    let side_patch_json = assert_json_status(&side_patch, "ok");
    let side_patch_id = side_patch_json["patch_id"]
        .as_str()
        .expect("side patch_id should be string")
        .to_string();

    let side_revision = run_mycel(&[
        "store",
        "commit-revision",
        &store_root,
        "--doc-id",
        "doc:author-smoke",
        "--parent",
        &genesis_revision_id,
        "--patch",
        &side_patch_id,
        "--signing-key",
        &key_file,
        "--timestamp",
        "14",
        "--json",
    ]);
    assert_success(&side_revision);
    let side_revision_json = assert_json_status(&side_revision, "ok");
    let side_revision_id = side_revision_json["revision_id"]
        .as_str()
        .expect("side revision_id should be string")
        .to_string();

    let merge = run_mycel(&[
        "store",
        "create-merge-revision",
        &store_root,
        "--doc-id",
        "doc:author-smoke",
        "--parent",
        &primary_revision_id,
        "--parent",
        &side_revision_id,
        "--resolved-state",
        &resolved_state_file,
        "--signing-key",
        &key_file,
        "--timestamp",
        "15",
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
                        "block 'blk:author-smoke-merge-002' selected a non-primary parent variant",
                    )
                })
            })),
        "expected merge-reason classification for side-branch addition, got {merge_json}"
    );
    assert_eq!(merge_json["patch_op_count"], 1);
    assert_eq!(
        merge_json["parent_revision_ids"].as_array().map(Vec::len),
        Some(2)
    );
    assert!(merge_json["patch_id"]
        .as_str()
        .is_some_and(|value| value.starts_with("patch:")));
    assert!(merge_json["revision_id"]
        .as_str()
        .is_some_and(|value| value.starts_with("rev:")));

    let index = run_mycel(&["store", "index", &store_root, "--json"]);
    assert_success(&index);
    let index_json = assert_json_status(&index, "ok");
    assert_eq!(index_json["stored_object_count"], 8);
    assert_eq!(
        index_json["doc_revisions"]["doc:author-smoke"]
            .as_array()
            .map(Vec::len),
        Some(4)
    );
    assert_eq!(
        index_json["object_ids_by_type"]["patch"]
            .as_array()
            .map(Vec::len),
        Some(3)
    );
    assert_eq!(
        index_json["object_ids_by_type"]["revision"]
            .as_array()
            .map(Vec::len),
        Some(4)
    );
}

#[test]
fn store_merge_authoring_flow_supports_structural_move_and_insert() {
    let store_dir = create_temp_dir("store-merge-structural-root");
    let (_key_dir, key_path) = write_signing_key_file("store-merge-structural-key");
    let (_resolved_dir, resolved_state_path) =
        write_structural_resolved_state_file("store-merge-structural-state");
    let (_move_ops_dir, move_ops_path) =
        write_structural_move_ops_file("store-merge-structural-move-ops");
    let (_insert_ops_dir, insert_ops_path) =
        write_structural_insert_ops_file("store-merge-structural-insert-ops");
    let store_root = path_arg(&store_dir.path().to_path_buf());
    let key_file = path_arg(&key_path);
    let resolved_state_file = path_arg(&resolved_state_path);
    let move_ops_file = path_arg(&move_ops_path);
    let insert_ops_file = path_arg(&insert_ops_path);

    let init = run_mycel(&["store", "init", &store_root, "--json"]);
    assert_success(&init);

    let document = run_mycel(&[
        "store",
        "create-document",
        &store_root,
        "--doc-id",
        "doc:author-smoke-structural",
        "--title",
        "Author Smoke Structural Merge",
        "--language",
        "en",
        "--signing-key",
        &key_file,
        "--timestamp",
        "20",
        "--json",
    ]);
    assert_success(&document);
    let document_json = assert_json_status(&document, "ok");
    let genesis_revision_id = document_json["genesis_revision_id"]
        .as_str()
        .expect("genesis revision should be string")
        .to_string();

    let base_ops_dir = create_temp_dir("store-merge-structural-base-ops");
    let base_ops_path = base_ops_dir.path().join("ops.json");
    fs::write(
        &base_ops_path,
        serde_json::to_string_pretty(&json!([
            {
                "op": "insert_block",
                "new_block": {
                    "block_id": "blk:author-smoke-001",
                    "block_type": "paragraph",
                    "content": "Hello author smoke",
                    "attrs": {},
                    "children": []
                }
            },
            {
                "op": "insert_block",
                "new_block": {
                    "block_id": "blk:author-smoke-002",
                    "block_type": "paragraph",
                    "content": "Second structural block",
                    "attrs": {},
                    "children": []
                }
            }
        ]))
        .expect("base structural ops JSON should serialize"),
    )
    .expect("base structural ops JSON should write");
    let base_ops_file = path_arg(&base_ops_path);

    let base_patch = run_mycel(&[
        "store",
        "create-patch",
        &store_root,
        "--doc-id",
        "doc:author-smoke-structural",
        "--base-revision",
        &genesis_revision_id,
        "--ops",
        &base_ops_file,
        "--signing-key",
        &key_file,
        "--timestamp",
        "21",
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
        "doc:author-smoke-structural",
        "--parent",
        &genesis_revision_id,
        "--patch",
        &base_patch_id,
        "--signing-key",
        &key_file,
        "--timestamp",
        "22",
        "--json",
    ]);
    assert_success(&base_revision);
    let base_revision_json = assert_json_status(&base_revision, "ok");
    let base_revision_id = base_revision_json["revision_id"]
        .as_str()
        .expect("base revision_id should be string")
        .to_string();

    let move_patch = run_mycel(&[
        "store",
        "create-patch",
        &store_root,
        "--doc-id",
        "doc:author-smoke-structural",
        "--base-revision",
        &base_revision_id,
        "--ops",
        &move_ops_file,
        "--signing-key",
        &key_file,
        "--timestamp",
        "23",
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
        "doc:author-smoke-structural",
        "--parent",
        &base_revision_id,
        "--patch",
        &move_patch_id,
        "--signing-key",
        &key_file,
        "--timestamp",
        "24",
        "--json",
    ]);
    assert_success(&move_revision);
    let move_revision_json = assert_json_status(&move_revision, "ok");
    let move_revision_id = move_revision_json["revision_id"]
        .as_str()
        .expect("move revision_id should be string")
        .to_string();

    let insert_patch = run_mycel(&[
        "store",
        "create-patch",
        &store_root,
        "--doc-id",
        "doc:author-smoke-structural",
        "--base-revision",
        &base_revision_id,
        "--ops",
        &insert_ops_file,
        "--signing-key",
        &key_file,
        "--timestamp",
        "25",
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
        "doc:author-smoke-structural",
        "--parent",
        &base_revision_id,
        "--patch",
        &insert_patch_id,
        "--signing-key",
        &key_file,
        "--timestamp",
        "26",
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
        "doc:author-smoke-structural",
        "--parent",
        &base_revision_id,
        "--parent",
        &move_revision_id,
        "--parent",
        &insert_revision_id,
        "--resolved-state",
        &resolved_state_file,
        "--signing-key",
        &key_file,
        "--timestamp",
        "27",
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
        "expected structural sibling multi-variant reason, got {merge_json}"
    );
    assert_eq!(merge_json["patch_op_count"], 2);
    assert_eq!(
        merge_json["parent_revision_ids"].as_array().map(Vec::len),
        Some(3)
    );

    let index = run_mycel(&["store", "index", &store_root, "--json"]);
    assert_success(&index);
    let index_json = assert_json_status(&index, "ok");
    assert_eq!(index_json["stored_object_count"], 10);
    assert_eq!(
        index_json["doc_revisions"]["doc:author-smoke-structural"]
            .as_array()
            .map(Vec::len),
        Some(5)
    );
    assert_eq!(
        index_json["object_ids_by_type"]["patch"]
            .as_array()
            .map(Vec::len),
        Some(4)
    );
    assert_eq!(
        index_json["object_ids_by_type"]["revision"]
            .as_array()
            .map(Vec::len),
        Some(5)
    );
}
