use super::*;

#[test]
fn object_verify_json_fails_for_mismatched_revision_id() {
    let mut revision = signed_object(
        json!({
            "type": "revision",
            "version": "mycel/0.1",
            "doc_id": "doc:test",
            "parents": [],
            "patches": [],
            "state_hash": "hash:test-state",
            "timestamp": 1777778890u64
        }),
        "author",
        "revision_id",
        "rev",
    );
    revision["revision_id"] = Value::String("rev:wrong".to_string());
    revision["signature"] = Value::String(sign_value(&signing_key(), &revision));
    let object = write_object_file("object-verify-revision-mismatch", "revision.json", revision);
    let path = path_arg(&object.path);
    let output = run_mycel(&["object", "verify", &path, "--json"]);

    assert_exit_code(&output, 1);
    let json = assert_json_status(&output, "failed");
    assert_eq!(json["object_type"], "revision");
    assert!(
        json["errors"]
            .as_array()
            .is_some_and(|errors| errors.iter().any(|entry| entry
                .as_str()
                .is_some_and(|message| message.contains("declared revision_id does not match")))),
        "expected derived ID mismatch error, stdout: {}",
        stdout_text(&output)
    );
}

#[test]
fn object_verify_json_fails_for_revision_with_wrong_revision_id_prefix() {
    let mut revision = signed_object(
        json!({
            "type": "revision",
            "version": "mycel/0.1",
            "doc_id": "doc:test",
            "parents": [],
            "patches": [],
            "state_hash": "hash:test-state",
            "timestamp": 1777778890u64
        }),
        "author",
        "revision_id",
        "rev",
    );
    revision["revision_id"] = Value::String(
        revision["revision_id"]
            .as_str()
            .expect("revision_id should exist")
            .replacen("rev:", "patch:", 1),
    );
    revision["signature"] = Value::String(sign_value(&signing_key(), &revision));
    let object = write_object_file(
        "object-verify-revision-wrong-derived-id-prefix",
        "revision.json",
        revision,
    );
    let path = path_arg(&object.path);
    let output = run_mycel(&["object", "verify", &path, "--json"]);

    assert_exit_code(&output, 1);
    let json = assert_json_status(&output, "failed");
    assert_eq!(json["object_type"], "revision");
    assert!(
        json["errors"]
            .as_array()
            .is_some_and(|errors| errors.iter().any(|entry| {
                entry.as_str().is_some_and(|message| {
                    message.contains("top-level 'revision_id' must use 'rev:' prefix")
                })
            })),
        "expected revision_id prefix error, stdout: {}",
        stdout_text(&output)
    );
}

#[test]
fn object_verify_json_fails_for_revision_with_non_string_revision_id() {
    let mut revision = signed_object(
        json!({
            "type": "revision",
            "version": "mycel/0.1",
            "doc_id": "doc:test",
            "parents": [],
            "patches": [],
            "state_hash": "hash:test-state",
            "timestamp": 1777778890u64
        }),
        "author",
        "revision_id",
        "rev",
    );
    revision["revision_id"] = json!(7);
    let object = write_object_file(
        "object-verify-revision-non-string-derived-id",
        "revision.json",
        revision,
    );
    let path = path_arg(&object.path);
    let output = run_mycel(&["object", "verify", &path, "--json"]);

    assert_exit_code(&output, 1);
    let json = assert_json_status(&output, "failed");
    assert_eq!(json["object_type"], "revision");
    assert!(
        json["errors"]
            .as_array()
            .is_some_and(|errors| errors.iter().any(|entry| {
                entry.as_str().is_some_and(|message| {
                    message.contains("top-level 'revision_id' must be a string")
                })
            })),
        "expected revision_id type error, stdout: {}",
        stdout_text(&output)
    );
}

#[test]
fn object_verify_json_fails_for_duplicate_revision_parent_ids() {
    let revision = signed_object(
        json!({
            "type": "revision",
            "version": "mycel/0.1",
            "doc_id": "doc:test",
            "parents": ["rev:base", "rev:base"],
            "patches": [],
            "state_hash": "hash:test-state",
            "timestamp": 1777778890u64
        }),
        "author",
        "revision_id",
        "rev",
    );
    let object = write_object_file(
        "object-verify-revision-duplicate-parents",
        "revision.json",
        revision,
    );
    let path = path_arg(&object.path);
    let output = run_mycel(&["object", "verify", &path, "--json"]);

    assert_exit_code(&output, 1);
    let json = assert_json_status(&output, "failed");
    assert_eq!(json["object_type"], "revision");
    assert!(
        json["errors"]
            .as_array()
            .is_some_and(|errors| errors.iter().any(|entry| {
                entry.as_str().is_some_and(|message| {
                    message.contains("top-level 'parents[1]' duplicates 'parents[0]'")
                })
            })),
        "expected duplicate parent error, stdout: {}",
        stdout_text(&output)
    );
}

#[test]
fn object_verify_json_fails_for_duplicate_revision_patch_ids() {
    let revision = signed_object(
        json!({
            "type": "revision",
            "version": "mycel/0.1",
            "doc_id": "doc:test",
            "parents": ["rev:base"],
            "patches": ["patch:test", "patch:test"],
            "state_hash": "hash:test-state",
            "timestamp": 1777778890u64
        }),
        "author",
        "revision_id",
        "rev",
    );
    let object = write_object_file(
        "object-verify-revision-duplicate-patches",
        "revision.json",
        revision,
    );
    let path = path_arg(&object.path);
    let output = run_mycel(&["object", "verify", &path, "--json"]);

    assert_exit_code(&output, 1);
    let json = assert_json_status(&output, "failed");
    assert_eq!(json["object_type"], "revision");
    assert!(
        json["errors"]
            .as_array()
            .is_some_and(|errors| errors.iter().any(|entry| {
                entry.as_str().is_some_and(|message| {
                    message.contains("top-level 'patches[1]' duplicates 'patches[0]'")
                })
            })),
        "expected duplicate patch error, stdout: {}",
        stdout_text(&output)
    );
}

#[test]
fn object_verify_json_fails_for_revision_with_wrong_parent_prefix() {
    let revision = signed_object(
        json!({
            "type": "revision",
            "version": "mycel/0.1",
            "doc_id": "doc:test",
            "parents": ["hash:base"],
            "patches": [],
            "state_hash": "hash:test-state",
            "timestamp": 1777778890u64
        }),
        "author",
        "revision_id",
        "rev",
    );
    let object = write_object_file(
        "object-verify-revision-wrong-parent-prefix",
        "revision.json",
        revision,
    );
    let path = path_arg(&object.path);
    let output = run_mycel(&["object", "verify", &path, "--json"]);

    assert_exit_code(&output, 1);
    let json = assert_json_status(&output, "failed");
    assert_eq!(json["object_type"], "revision");
    assert!(
        json["errors"]
            .as_array()
            .is_some_and(|errors| errors.iter().any(|entry| {
                entry.as_str().is_some_and(|message| {
                    message.contains("top-level 'parents[0]' must use 'rev:' prefix")
                })
            })),
        "expected parent prefix error, stdout: {}",
        stdout_text(&output)
    );
}

#[test]
fn object_verify_json_fails_for_genesis_revision_with_merge_strategy() {
    let revision = signed_object(
        json!({
            "type": "revision",
            "version": "mycel/0.1",
            "doc_id": "doc:test",
            "parents": [],
            "patches": [],
            "merge_strategy": "semantic-block-merge",
            "state_hash": "hash:test-state",
            "timestamp": 1777778890u64
        }),
        "author",
        "revision_id",
        "rev",
    );
    let object = write_object_file(
        "object-verify-revision-genesis-merge-strategy",
        "revision.json",
        revision,
    );
    let path = path_arg(&object.path);
    let output = run_mycel(&["object", "verify", &path, "--json"]);

    assert_exit_code(&output, 1);
    let json = assert_json_status(&output, "failed");
    assert_eq!(json["object_type"], "revision");
    assert!(
        json["errors"]
            .as_array()
            .is_some_and(|errors| errors.iter().any(|entry| {
                entry.as_str().is_some_and(|message| {
                    message.contains(
                        "top-level 'merge_strategy' is not allowed when 'parents' is empty",
                    )
                })
            })),
        "expected genesis merge_strategy error, stdout: {}",
        stdout_text(&output)
    );
}

#[test]
fn object_verify_json_reports_ok_for_revision_with_replayed_state_hash() {
    let dir = create_temp_dir("object-verify-revision-replay");
    let patch_path = dir.path().join("patch.json");
    let revision_path = dir.path().join("revision.json");
    let patch = signed_object(
        json!({
            "type": "patch",
            "version": "mycel/0.1",
            "doc_id": "doc:test",
            "base_revision": "rev:genesis-null",
            "timestamp": 1777778888u64,
            "ops": [
                {
                    "op": "insert_block",
                    "new_block": {
                        "block_id": "blk:001",
                        "block_type": "paragraph",
                        "content": "Hello",
                        "attrs": {},
                        "children": []
                    }
                }
            ]
        }),
        "author",
        "patch_id",
        "patch",
    );
    fs::write(
        &patch_path,
        serde_json::to_string_pretty(&patch).expect("patch JSON should serialize"),
    )
    .expect("patch JSON should be written");
    let expected_state_hash = state_hash(&json!({
        "doc_id": "doc:test",
        "blocks": [
            {
                "block_id": "blk:001",
                "block_type": "paragraph",
                "content": "Hello",
                "attrs": {},
                "children": []
            }
        ]
    }));
    let revision = signed_object(
        json!({
            "type": "revision",
            "version": "mycel/0.1",
            "doc_id": "doc:test",
            "parents": [],
            "patches": [patch["patch_id"].as_str().expect("patch id should exist")],
            "state_hash": expected_state_hash,
            "timestamp": 1777778890u64
        }),
        "author",
        "revision_id",
        "rev",
    );
    fs::write(
        &revision_path,
        serde_json::to_string_pretty(&revision).expect("revision JSON should serialize"),
    )
    .expect("revision JSON should be written");

    let output = run_mycel(&["object", "verify", &path_arg(&revision_path), "--json"]);

    assert_success(&output);
    let json = assert_json_status(&output, "ok");
    assert_eq!(json["object_type"], "revision");
    assert_eq!(json["state_hash_verification"], "verified");
    assert_eq!(json["declared_state_hash"], json["recomputed_state_hash"]);
}

#[test]
fn object_verify_json_reports_ok_for_non_genesis_revision_with_neighbor_patch() {
    let dir = create_temp_dir("object-verify-non-genesis-revision-replay");
    let base_revision_path = dir.path().join("revision-base.json");
    let patch_path = dir.path().join("patch-child.json");
    let revision_path = dir.path().join("revision-child.json");

    let base_revision = signed_object(
        json!({
            "type": "revision",
            "version": "mycel/0.1",
            "doc_id": "doc:test",
            "parents": [],
            "patches": [],
            "state_hash": state_hash(&json!({
                "doc_id": "doc:test",
                "blocks": [],
                "metadata": {}
            })),
            "timestamp": 1777778887u64
        }),
        "author",
        "revision_id",
        "rev",
    );
    fs::write(
        &base_revision_path,
        serde_json::to_string_pretty(&base_revision).expect("base revision JSON should serialize"),
    )
    .expect("base revision JSON should be written");

    let patch = signed_object(
        json!({
            "type": "patch",
            "version": "mycel/0.1",
            "doc_id": "doc:test",
            "base_revision": base_revision["revision_id"].as_str().expect("base revision id should exist"),
            "timestamp": 1777778888u64,
            "ops": [
                {
                    "op": "insert_block",
                    "new_block": {
                        "block_id": "blk:001",
                        "block_type": "paragraph",
                        "content": "Hello",
                        "attrs": {},
                        "children": []
                    }
                }
            ]
        }),
        "author",
        "patch_id",
        "patch",
    );
    fs::write(
        &patch_path,
        serde_json::to_string_pretty(&patch).expect("patch JSON should serialize"),
    )
    .expect("patch JSON should be written");

    let revision = signed_object(
        json!({
            "type": "revision",
            "version": "mycel/0.1",
            "doc_id": "doc:test",
            "parents": [base_revision["revision_id"].as_str().expect("base revision id should exist")],
            "patches": [patch["patch_id"].as_str().expect("patch id should exist")],
            "state_hash": state_hash(&json!({
                "doc_id": "doc:test",
                "blocks": [
                    {
                        "block_id": "blk:001",
                        "block_type": "paragraph",
                        "content": "Hello",
                        "attrs": {},
                        "children": []
                    }
                ]
            })),
            "timestamp": 1777778889u64
        }),
        "author",
        "revision_id",
        "rev",
    );
    fs::write(
        &revision_path,
        serde_json::to_string_pretty(&revision).expect("revision JSON should serialize"),
    )
    .expect("revision JSON should be written");

    let output = run_mycel(&["object", "verify", &path_arg(&revision_path), "--json"]);

    assert_success(&output);
    let json = assert_json_status(&output, "ok");
    assert_eq!(json["object_type"], "revision");
    assert_eq!(json["state_hash_verification"], "verified");
    assert_eq!(json["declared_state_hash"], json["recomputed_state_hash"]);
}

#[test]
fn object_verify_json_fails_for_revision_state_hash_mismatch() {
    let dir = create_temp_dir("object-verify-revision-state-hash-mismatch");
    let patch_path = dir.path().join("patch.json");
    let revision_path = dir.path().join("revision.json");
    let patch = signed_object(
        json!({
            "type": "patch",
            "version": "mycel/0.1",
            "doc_id": "doc:test",
            "base_revision": "rev:genesis-null",
            "timestamp": 1777778888u64,
            "ops": [
                {
                    "op": "insert_block",
                    "new_block": {
                        "block_id": "blk:001",
                        "block_type": "paragraph",
                        "content": "Hello",
                        "attrs": {},
                        "children": []
                    }
                }
            ]
        }),
        "author",
        "patch_id",
        "patch",
    );
    fs::write(
        &patch_path,
        serde_json::to_string_pretty(&patch).expect("patch JSON should serialize"),
    )
    .expect("patch JSON should be written");
    let revision = signed_object(
        json!({
            "type": "revision",
            "version": "mycel/0.1",
            "doc_id": "doc:test",
            "parents": [],
            "patches": [patch["patch_id"].as_str().expect("patch id should exist")],
            "state_hash": "hash:wrong",
            "timestamp": 1777778890u64
        }),
        "author",
        "revision_id",
        "rev",
    );
    fs::write(
        &revision_path,
        serde_json::to_string_pretty(&revision).expect("revision JSON should serialize"),
    )
    .expect("revision JSON should be written");

    let output = run_mycel(&["object", "verify", &path_arg(&revision_path), "--json"]);

    assert_exit_code(&output, 1);
    let json = assert_json_status(&output, "failed");
    assert_eq!(json["object_type"], "revision");
    assert_eq!(json["state_hash_verification"], "failed");
    assert!(
        json["errors"]
            .as_array()
            .is_some_and(
                |errors| errors
                    .iter()
                    .any(|entry| entry.as_str().is_some_and(|message| {
                        message.contains("declared state_hash does not match replayed state hash")
                    }))
            ),
        "expected state-hash mismatch error, stdout: {}",
        stdout_text(&output)
    );
}

#[test]
fn object_verify_json_fails_for_revision_with_wrong_state_hash_prefix() {
    let revision = signed_object(
        json!({
            "type": "revision",
            "version": "mycel/0.1",
            "doc_id": "doc:test",
            "parents": ["rev:base"],
            "patches": [],
            "state_hash": "rev:test",
            "timestamp": 1777778890u64
        }),
        "author",
        "revision_id",
        "rev",
    );
    let object = write_object_file(
        "object-verify-revision-wrong-state-hash-prefix",
        "revision.json",
        revision,
    );
    let path = path_arg(&object.path);
    let output = run_mycel(&["object", "verify", &path, "--json"]);

    assert_exit_code(&output, 1);
    let json = assert_json_status(&output, "failed");
    assert_eq!(json["object_type"], "revision");
    assert!(
        json["errors"]
            .as_array()
            .is_some_and(|errors| errors.iter().any(|entry| {
                entry.as_str().is_some_and(|message| {
                    message.contains("top-level 'state_hash' must use 'hash:' prefix")
                })
            })),
        "expected state_hash prefix error, stdout: {}",
        stdout_text(&output)
    );
}

#[test]
fn object_verify_json_fails_for_revision_missing_state_hash() {
    let revision = signed_object(
        json!({
            "type": "revision",
            "version": "mycel/0.1",
            "doc_id": "doc:test",
            "parents": ["rev:base"],
            "patches": [],
            "timestamp": 1777778890u64
        }),
        "author",
        "revision_id",
        "rev",
    );
    let object = write_object_file(
        "object-verify-revision-missing-state-hash",
        "revision.json",
        revision,
    );
    let path = path_arg(&object.path);
    let output = run_mycel(&["object", "verify", &path, "--json"]);

    assert_exit_code(&output, 1);
    let json = assert_json_status(&output, "failed");
    assert_eq!(json["object_type"], "revision");
    assert!(
        json["errors"]
            .as_array()
            .is_some_and(|errors| errors.iter().any(|entry| {
                entry.as_str().is_some_and(|message| {
                    message.contains("revision object is missing string field 'state_hash'")
                })
            })),
        "expected missing state_hash error, stdout: {}",
        stdout_text(&output)
    );
}

#[test]
fn object_verify_json_fails_for_revision_missing_parents() {
    let revision = signed_object(
        json!({
            "type": "revision",
            "version": "mycel/0.1",
            "doc_id": "doc:test",
            "patches": [],
            "state_hash": "hash:test-state",
            "timestamp": 1777778890u64
        }),
        "author",
        "revision_id",
        "rev",
    );
    let object = write_object_file(
        "object-verify-revision-missing-parents",
        "revision.json",
        revision,
    );
    let path = path_arg(&object.path);
    let output = run_mycel(&["object", "verify", &path, "--json"]);

    assert_exit_code(&output, 1);
    let json = assert_json_status(&output, "failed");
    assert_eq!(json["object_type"], "revision");
    assert!(
        json["errors"]
            .as_array()
            .is_some_and(|errors| errors.iter().any(|entry| {
                entry
                    .as_str()
                    .is_some_and(|message| message.contains("missing array field 'parents'"))
            })),
        "expected missing parents error, stdout: {}",
        stdout_text(&output)
    );
}

#[test]
fn object_verify_json_fails_for_revision_with_non_array_parents() {
    let revision = signed_object(
        json!({
            "type": "revision",
            "version": "mycel/0.1",
            "doc_id": "doc:test",
            "parents": {},
            "patches": [],
            "state_hash": "hash:test-state",
            "timestamp": 1777778890u64
        }),
        "author",
        "revision_id",
        "rev",
    );
    let object = write_object_file(
        "object-verify-revision-non-array-parents",
        "revision.json",
        revision,
    );
    let path = path_arg(&object.path);
    let output = run_mycel(&["object", "verify", &path, "--json"]);

    assert_exit_code(&output, 1);
    let json = assert_json_status(&output, "failed");
    assert_eq!(json["object_type"], "revision");
    assert!(
        json["errors"]
            .as_array()
            .is_some_and(|errors| errors.iter().any(|entry| {
                entry
                    .as_str()
                    .is_some_and(|message| message.contains("top-level 'parents' must be an array"))
            })),
        "expected parents array-shape error, stdout: {}",
        stdout_text(&output)
    );
}

#[test]
fn object_verify_json_fails_for_revision_missing_patches() {
    let revision = signed_object(
        json!({
            "type": "revision",
            "version": "mycel/0.1",
            "doc_id": "doc:test",
            "parents": [],
            "state_hash": "hash:test-state",
            "timestamp": 1777778890u64
        }),
        "author",
        "revision_id",
        "rev",
    );
    let object = write_object_file(
        "object-verify-revision-missing-patches",
        "revision.json",
        revision,
    );
    let path = path_arg(&object.path);
    let output = run_mycel(&["object", "verify", &path, "--json"]);

    assert_exit_code(&output, 1);
    let json = assert_json_status(&output, "failed");
    assert_eq!(json["object_type"], "revision");
    assert!(
        json["errors"]
            .as_array()
            .is_some_and(|errors| errors.iter().any(|entry| {
                entry
                    .as_str()
                    .is_some_and(|message| message.contains("missing array field 'patches'"))
            })),
        "expected missing patches error, stdout: {}",
        stdout_text(&output)
    );
}

#[test]
fn object_verify_json_fails_for_revision_with_non_array_patches() {
    let revision = signed_object(
        json!({
            "type": "revision",
            "version": "mycel/0.1",
            "doc_id": "doc:test",
            "parents": [],
            "patches": {},
            "state_hash": "hash:test-state",
            "timestamp": 1777778890u64
        }),
        "author",
        "revision_id",
        "rev",
    );
    let object = write_object_file(
        "object-verify-revision-non-array-patches",
        "revision.json",
        revision,
    );
    let path = path_arg(&object.path);
    let output = run_mycel(&["object", "verify", &path, "--json"]);

    assert_exit_code(&output, 1);
    let json = assert_json_status(&output, "failed");
    assert_eq!(json["object_type"], "revision");
    assert!(
        json["errors"]
            .as_array()
            .is_some_and(|errors| errors.iter().any(|entry| {
                entry
                    .as_str()
                    .is_some_and(|message| message.contains("top-level 'patches' must be an array"))
            })),
        "expected patches array-shape error, stdout: {}",
        stdout_text(&output)
    );
}

#[test]
fn object_verify_json_fails_for_revision_with_wrong_author_prefix() {
    let mut revision = signed_object(
        json!({
            "type": "revision",
            "version": "mycel/0.1",
            "doc_id": "doc:test",
            "parents": ["rev:base"],
            "patches": [],
            "state_hash": "hash:test-state",
            "timestamp": 1777778890u64
        }),
        "author",
        "revision_id",
        "rev",
    );
    revision["author"] = Value::String("author:test".to_string());
    let object = write_object_file(
        "object-verify-revision-wrong-author-prefix",
        "revision.json",
        revision,
    );
    let path = path_arg(&object.path);
    let output = run_mycel(&["object", "verify", &path, "--json"]);

    assert_exit_code(&output, 1);
    let json = assert_json_status(&output, "failed");
    assert_eq!(json["object_type"], "revision");
    assert!(
        json["errors"]
            .as_array()
            .is_some_and(|errors| errors.iter().any(|entry| {
                entry.as_str().is_some_and(|message| {
                    message.contains("signer field must use format 'pk:ed25519:<base64>'")
                })
            })),
        "expected signer-format error, stdout: {}",
        stdout_text(&output)
    );
}

#[test]
fn object_verify_json_fails_for_revision_with_malformed_author_key() {
    let mut revision = json!({
        "type": "revision",
        "version": "mycel/0.1",
        "revision_id": "rev:placeholder",
        "doc_id": "doc:test",
        "parents": [],
        "patches": [],
        "state_hash": "hash:test-state",
        "author": "pk:ed25519:not-base64",
        "timestamp": 1777778890u64
    });
    revision["signature"] = Value::String(sign_value(&signing_key(), &revision));
    let object = write_object_file(
        "object-verify-revision-malformed-author-key",
        "revision.json",
        revision,
    );
    let path = path_arg(&object.path);
    let output = run_mycel(&["object", "verify", &path, "--json"]);

    assert_exit_code(&output, 1);
    let json = assert_json_status(&output, "failed");
    assert_eq!(json["object_type"], "revision");
    assert!(
        json["errors"]
            .as_array()
            .is_some_and(|errors| errors.iter().any(|entry| {
                entry
                    .as_str()
                    .is_some_and(|message| message.contains("failed to decode Ed25519 public key"))
            })),
        "expected malformed public-key error, stdout: {}",
        stdout_text(&output)
    );
}

#[test]
fn object_verify_json_fails_for_revision_with_invalid_author_key_bytes() {
    let object = write_object_file(
        "object-verify-revision-invalid-author-bytes",
        "revision.json",
        json!({
            "type": "revision",
            "version": "mycel/0.1",
            "revision_id": "rev:placeholder",
            "doc_id": "doc:test",
            "parents": [],
            "patches": [],
            "state_hash": "hash:test-state",
            "author": "pk:ed25519:AA==",
            "timestamp": 1777778890u64,
            "signature": "sig:ed25519:AA=="
        }),
    );
    let path = path_arg(&object.path);
    let output = run_mycel(&["object", "verify", &path, "--json"]);

    assert_exit_code(&output, 1);
    let json = assert_json_status(&output, "failed");
    assert_eq!(json["object_type"], "revision");
    assert!(
        json["errors"]
            .as_array()
            .is_some_and(|errors| errors.iter().any(|entry| {
                entry.as_str().is_some_and(|message| {
                    message.contains("Ed25519 public key must decode to 32 bytes")
                })
            })),
        "expected invalid public-key length error, stdout: {}",
        stdout_text(&output)
    );
}

#[test]
fn object_verify_json_fails_for_revision_missing_author() {
    let mut revision = signed_object(
        json!({
            "type": "revision",
            "version": "mycel/0.1",
            "doc_id": "doc:test",
            "parents": ["rev:base"],
            "patches": [],
            "state_hash": "hash:test-state",
            "timestamp": 1777778890u64
        }),
        "author",
        "revision_id",
        "rev",
    );
    revision
        .as_object_mut()
        .expect("revision should be an object")
        .remove("author");
    let object = write_object_file(
        "object-verify-revision-missing-author",
        "revision.json",
        revision,
    );
    let path = path_arg(&object.path);
    let output = run_mycel(&["object", "verify", &path, "--json"]);

    assert_exit_code(&output, 1);
    let json = assert_json_status(&output, "failed");
    assert_eq!(json["object_type"], "revision");
    assert!(
        json["errors"]
            .as_array()
            .is_some_and(|errors| errors.iter().any(|entry| {
                entry.as_str().is_some_and(|message| {
                    message.contains("revision object is missing string signer field 'author'")
                })
            })),
        "expected missing author signer-field error, stdout: {}",
        stdout_text(&output)
    );
}

#[test]
fn object_verify_json_fails_for_revision_missing_signature() {
    let object = write_object_file(
        "object-verify-revision-missing-signature",
        "revision.json",
        json!({
            "type": "revision",
            "version": "mycel/0.1",
            "revision_id": "rev:placeholder",
            "doc_id": "doc:test",
            "parents": [],
            "patches": [],
            "state_hash": "hash:test-state",
            "author": signer_id(&signing_key()),
            "timestamp": 1777778890u64
        }),
    );
    let path = path_arg(&object.path);
    let output = run_mycel(&["object", "verify", &path, "--json"]);

    assert_exit_code(&output, 1);
    let json = assert_json_status(&output, "failed");
    assert_eq!(json["object_type"], "revision");
    assert!(
        json["errors"]
            .as_array()
            .is_some_and(|errors| errors.iter().any(|entry| {
                entry.as_str().is_some_and(|message| {
                    message.contains("revision object is missing required top-level 'signature'")
                })
            })),
        "expected missing signature error, stdout: {}",
        stdout_text(&output)
    );
}

#[test]
fn object_verify_json_fails_for_revision_missing_timestamp() {
    let revision = signed_object(
        json!({
            "type": "revision",
            "version": "mycel/0.1",
            "doc_id": "doc:test",
            "parents": [],
            "patches": [],
            "state_hash": "hash:test-state"
        }),
        "author",
        "revision_id",
        "rev",
    );
    let object = write_object_file(
        "object-verify-revision-missing-timestamp",
        "revision.json",
        revision,
    );
    let path = path_arg(&object.path);
    let output = run_mycel(&["object", "verify", &path, "--json"]);

    assert_exit_code(&output, 1);
    let json = assert_json_status(&output, "failed");
    assert_eq!(json["object_type"], "revision");
    assert!(
        json["errors"]
            .as_array()
            .is_some_and(|errors| errors.iter().any(|entry| {
                entry
                    .as_str()
                    .is_some_and(|message| message.contains("missing integer field 'timestamp'"))
            })),
        "expected missing timestamp error, stdout: {}",
        stdout_text(&output)
    );
}

#[test]
fn object_verify_json_fails_for_revision_with_negative_timestamp() {
    let mut revision = signed_object(
        json!({
            "type": "revision",
            "version": "mycel/0.1",
            "doc_id": "doc:test",
            "parents": [],
            "patches": [],
            "state_hash": "hash:test-state",
            "timestamp": 1777778890u64
        }),
        "author",
        "revision_id",
        "rev",
    );
    revision["timestamp"] = json!(-1);
    revision["signature"] = Value::String(sign_value(&signing_key(), &revision));
    let object = write_object_file(
        "object-verify-revision-negative-timestamp",
        "revision.json",
        revision,
    );
    let path = path_arg(&object.path);
    let output = run_mycel(&["object", "verify", &path, "--json"]);

    assert_exit_code(&output, 1);
    let json = assert_json_status(&output, "failed");
    assert_eq!(json["object_type"], "revision");
    assert!(
        json["errors"]
            .as_array()
            .is_some_and(|errors| errors.iter().any(|entry| {
                entry.as_str().is_some_and(|message| {
                    message.contains("top-level 'timestamp' must be a non-negative integer")
                })
            })),
        "expected timestamp integer error, stdout: {}",
        stdout_text(&output)
    );
}

#[test]
fn object_verify_json_fails_for_invalid_revision_signature() {
    let mut revision = signed_object(
        json!({
            "type": "revision",
            "version": "mycel/0.1",
            "doc_id": "doc:test",
            "parents": [],
            "patches": [],
            "state_hash": "hash:test-state",
            "timestamp": 1777778890u64
        }),
        "author",
        "revision_id",
        "rev",
    );
    revision["signature"] = Value::String(
        "sig:ed25519:AAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAA=="
            .to_string(),
    );
    let object = write_object_file(
        "object-verify-revision-bad-signature",
        "revision.json",
        revision,
    );
    let path = path_arg(&object.path);
    let output = run_mycel(&["object", "verify", &path, "--json"]);

    assert_exit_code(&output, 1);
    let json = parse_json_stdout(&output);
    assert_eq!(json["status"], "failed");
    assert_eq!(json["signature_verification"], "failed");
    assert!(
        json["errors"]
            .as_array()
            .is_some_and(|errors| errors.iter().any(|entry| entry
                .as_str()
                .is_some_and(|message| message.contains("Ed25519 signature verification failed")))),
        "expected signature failure, stdout: {}",
        stdout_text(&output)
    );
}

#[test]
fn object_verify_json_fails_for_revision_with_wrong_signature_format() {
    let object = write_object_file(
        "object-verify-revision-wrong-signature-format",
        "revision.json",
        json!({
            "type": "revision",
            "version": "mycel/0.1",
            "revision_id": "rev:placeholder",
            "doc_id": "doc:test",
            "parents": [],
            "patches": [],
            "state_hash": "hash:test-state",
            "author": signer_id(&signing_key()),
            "timestamp": 1777778890u64,
            "signature": "sig:bad"
        }),
    );
    let path = path_arg(&object.path);
    let output = run_mycel(&["object", "verify", &path, "--json"]);

    assert_exit_code(&output, 1);
    let json = assert_json_status(&output, "failed");
    assert_eq!(json["object_type"], "revision");
    assert!(
        json["errors"].as_array().is_some_and(|errors| errors
            .iter()
            .any(|entry| entry.as_str().is_some_and(|message| message
                .contains("signature field must use format 'sig:ed25519:<base64>'")))),
        "expected signature format error, stdout: {}",
        stdout_text(&output)
    );
}

#[test]
fn object_verify_json_fails_for_revision_with_malformed_signature_bytes() {
    let object = write_object_file(
        "object-verify-revision-malformed-signature",
        "revision.json",
        json!({
            "type": "revision",
            "version": "mycel/0.1",
            "revision_id": "rev:placeholder",
            "doc_id": "doc:test",
            "parents": [],
            "patches": [],
            "state_hash": "hash:test-state",
            "author": signer_id(&signing_key()),
            "timestamp": 1777778890u64,
            "signature": "sig:ed25519:not-base64"
        }),
    );
    let path = path_arg(&object.path);
    let output = run_mycel(&["object", "verify", &path, "--json"]);

    assert_exit_code(&output, 1);
    let json = assert_json_status(&output, "failed");
    assert_eq!(json["object_type"], "revision");
    assert!(
        json["errors"]
            .as_array()
            .is_some_and(|errors| errors.iter().any(|entry| {
                entry
                    .as_str()
                    .is_some_and(|message| message.contains("failed to decode Ed25519 signature"))
            })),
        "expected malformed signature decode error, stdout: {}",
        stdout_text(&output)
    );
}

#[test]
fn object_verify_json_fails_for_revision_with_invalid_signature_bytes() {
    let object = write_object_file(
        "object-verify-revision-invalid-signature-bytes",
        "revision.json",
        json!({
            "type": "revision",
            "version": "mycel/0.1",
            "revision_id": "rev:placeholder",
            "doc_id": "doc:test",
            "parents": [],
            "patches": [],
            "state_hash": "hash:test-state",
            "author": signer_id(&signing_key()),
            "timestamp": 1777778890u64,
            "signature": "sig:ed25519:AA=="
        }),
    );
    let path = path_arg(&object.path);
    let output = run_mycel(&["object", "verify", &path, "--json"]);

    assert_exit_code(&output, 1);
    let json = assert_json_status(&output, "failed");
    assert_eq!(json["object_type"], "revision");
    assert!(
        json["errors"]
            .as_array()
            .is_some_and(|errors| errors.iter().any(|entry| {
                entry
                    .as_str()
                    .is_some_and(|message| message.contains("invalid Ed25519 signature bytes"))
            })),
        "expected invalid signature bytes error, stdout: {}",
        stdout_text(&output)
    );
}

#[test]
fn object_verify_json_fails_for_revision_with_non_string_signature() {
    let object = write_object_file(
        "object-verify-revision-non-string-signature",
        "revision.json",
        json!({
            "type": "revision",
            "version": "mycel/0.1",
            "revision_id": "rev:placeholder",
            "doc_id": "doc:test",
            "parents": [],
            "patches": [],
            "state_hash": "hash:test-state",
            "author": signer_id(&signing_key()),
            "timestamp": 1777778890u64,
            "signature": 7
        }),
    );
    let path = path_arg(&object.path);
    let output = run_mycel(&["object", "verify", &path, "--json"]);

    assert_exit_code(&output, 1);
    let json = assert_json_status(&output, "failed");
    assert_eq!(json["object_type"], "revision");
    assert!(
        json["errors"]
            .as_array()
            .is_some_and(|errors| errors.iter().any(|entry| {
                entry.as_str().is_some_and(|message| {
                    message.contains("top-level 'signature' must be a string")
                })
            })),
        "expected signature type error, stdout: {}",
        stdout_text(&output)
    );
}

#[test]
fn object_verify_json_fails_for_genesis_revision_with_wrong_patch_base_revision() {
    let dir = create_temp_dir("object-verify-revision-genesis-base-mismatch");
    let patch_path = dir.path().join("patch.json");
    let revision_path = dir.path().join("revision.json");
    let patch = signed_object(
        json!({
            "type": "patch",
            "version": "mycel/0.1",
            "doc_id": "doc:test",
            "base_revision": "rev:wrong-base",
            "timestamp": 1777778888u64,
            "ops": []
        }),
        "author",
        "patch_id",
        "patch",
    );
    fs::write(
        &patch_path,
        serde_json::to_string_pretty(&patch).expect("patch JSON should serialize"),
    )
    .expect("patch JSON should be written");
    let revision = signed_object(
        json!({
            "type": "revision",
            "version": "mycel/0.1",
            "doc_id": "doc:test",
            "parents": [],
            "patches": [patch["patch_id"].as_str().expect("patch id should exist")],
            "state_hash": "hash:test",
            "timestamp": 1777778890u64
        }),
        "author",
        "revision_id",
        "rev",
    );
    fs::write(
        &revision_path,
        serde_json::to_string_pretty(&revision).expect("revision JSON should serialize"),
    )
    .expect("revision JSON should be written");

    let output = run_mycel(&["object", "verify", &path_arg(&revision_path), "--json"]);

    assert_exit_code(&output, 1);
    let json = assert_json_status(&output, "failed");
    assert_eq!(json["object_type"], "revision");
    assert_eq!(json["state_hash_verification"], "failed");
    assert!(
        json["errors"]
            .as_array()
            .is_some_and(|errors| errors.iter().any(|entry| {
                entry.as_str().is_some_and(|message| {
                    message.contains("base_revision 'rev:wrong-base'")
                        && message.contains("expected 'rev:genesis-null'")
                })
            })),
        "expected genesis base_revision mismatch error, stdout: {}",
        stdout_text(&output)
    );
}

#[test]
fn object_verify_json_fails_for_non_genesis_revision_with_wrong_patch_base_revision() {
    let dir = create_temp_dir("object-verify-revision-parent-base-mismatch");
    let base_revision_path = dir.path().join("revision-base.json");
    let patch_path = dir.path().join("patch.json");
    let revision_path = dir.path().join("revision.json");
    let base_revision = signed_object(
        json!({
            "type": "revision",
            "version": "mycel/0.1",
            "doc_id": "doc:test",
            "parents": [],
            "patches": [],
            "state_hash": state_hash(&json!({
                "doc_id": "doc:test",
                "blocks": []
            })),
            "timestamp": 1777778887u64
        }),
        "author",
        "revision_id",
        "rev",
    );
    fs::write(
        &base_revision_path,
        serde_json::to_string_pretty(&base_revision).expect("base revision JSON should serialize"),
    )
    .expect("base revision JSON should be written");
    let patch = signed_object(
        json!({
            "type": "patch",
            "version": "mycel/0.1",
            "doc_id": "doc:test",
            "base_revision": "rev:wrong-base",
            "timestamp": 1777778888u64,
            "ops": []
        }),
        "author",
        "patch_id",
        "patch",
    );
    fs::write(
        &patch_path,
        serde_json::to_string_pretty(&patch).expect("patch JSON should serialize"),
    )
    .expect("patch JSON should be written");
    let revision = signed_object(
        json!({
            "type": "revision",
            "version": "mycel/0.1",
            "doc_id": "doc:test",
            "parents": [base_revision["revision_id"].as_str().expect("base revision id should exist")],
            "patches": [patch["patch_id"].as_str().expect("patch id should exist")],
            "state_hash": state_hash(&json!({
                "doc_id": "doc:test",
                "blocks": []
            })),
            "timestamp": 1777778890u64
        }),
        "author",
        "revision_id",
        "rev",
    );
    fs::write(
        &revision_path,
        serde_json::to_string_pretty(&revision).expect("revision JSON should serialize"),
    )
    .expect("revision JSON should be written");

    let output = run_mycel(&["object", "verify", &path_arg(&revision_path), "--json"]);

    assert_exit_code(&output, 1);
    let json = assert_json_status(&output, "failed");
    assert_eq!(json["object_type"], "revision");
    assert_eq!(json["state_hash_verification"], "failed");
    assert!(
        json["errors"]
            .as_array()
            .is_some_and(|errors| errors.iter().any(|entry| {
                entry.as_str().is_some_and(|message| {
                    message.contains("base_revision 'rev:wrong-base'")
                        && message.contains("expected '")
                        && message.contains("rev:")
                })
            })),
        "expected non-genesis base_revision mismatch error, stdout: {}",
        stdout_text(&output)
    );
}

#[test]
fn object_verify_json_fails_for_single_parent_revision_with_merge_strategy() {
    let dir = create_temp_dir("object-verify-revision-single-parent-merge-strategy");
    let revision_path = dir.path().join("revision.json");
    let revision = signed_object(
        json!({
            "type": "revision",
            "version": "mycel/0.1",
            "doc_id": "doc:test",
            "parents": ["rev:base"],
            "patches": [],
            "merge_strategy": "semantic-block-merge",
            "state_hash": "hash:test",
            "timestamp": 1777778890u64
        }),
        "author",
        "revision_id",
        "rev",
    );
    fs::write(
        &revision_path,
        serde_json::to_string_pretty(&revision).expect("revision JSON should serialize"),
    )
    .expect("revision JSON should be written");

    let output = run_mycel(&["object", "verify", &path_arg(&revision_path), "--json"]);

    assert_exit_code(&output, 1);
    let json = assert_json_status(&output, "failed");
    assert_eq!(json["object_type"], "revision");
    assert!(
        json["errors"]
            .as_array()
            .is_some_and(|errors| errors.iter().any(|entry| {
                entry.as_str().is_some_and(|message| {
                    message.contains("top-level 'merge_strategy' requires multiple parents")
                })
            })),
        "expected merge_strategy parent-count error, stdout: {}",
        stdout_text(&output)
    );
}

#[test]
fn object_verify_json_fails_for_multi_parent_revision_without_merge_strategy() {
    let dir = create_temp_dir("object-verify-revision-missing-merge-strategy");
    let revision_path = dir.path().join("revision.json");
    let revision = signed_object(
        json!({
            "type": "revision",
            "version": "mycel/0.1",
            "doc_id": "doc:test",
            "parents": ["rev:base", "rev:side"],
            "patches": [],
            "state_hash": "hash:test",
            "timestamp": 1777778890u64
        }),
        "author",
        "revision_id",
        "rev",
    );
    fs::write(
        &revision_path,
        serde_json::to_string_pretty(&revision).expect("revision JSON should serialize"),
    )
    .expect("revision JSON should be written");

    let output = run_mycel(&["object", "verify", &path_arg(&revision_path), "--json"]);

    assert_exit_code(&output, 1);
    let json = assert_json_status(&output, "failed");
    assert_eq!(json["object_type"], "revision");
    assert!(
        json["errors"]
            .as_array()
            .is_some_and(|errors| errors.iter().any(|entry| {
                entry.as_str().is_some_and(|message| {
                    message.contains(
                        "top-level 'merge_strategy' is required when 'parents' has multiple entries",
                    )
                })
            })),
        "expected missing merge_strategy error, stdout: {}",
        stdout_text(&output)
    );
}

#[test]
fn object_verify_json_reports_ok_for_valid_merge_revision() {
    let dir = create_temp_dir("object-verify-valid-merge-revision");
    let base_patch_path = dir.path().join("patch-base.json");
    let base_revision_path = dir.path().join("revision-base.json");
    let side_patch_path = dir.path().join("patch-side.json");
    let side_revision_path = dir.path().join("revision-side.json");
    let merge_patch_path = dir.path().join("patch-merge.json");
    let merge_revision_path = dir.path().join("revision-merge.json");

    let base_patch = signed_object(
        json!({
            "type": "patch",
            "version": "mycel/0.1",
            "doc_id": "doc:test",
            "base_revision": "rev:genesis-null",
            "timestamp": 1777778887u64,
            "ops": [
                {
                    "op": "insert_block",
                    "new_block": {
                        "block_id": "blk:001",
                        "block_type": "paragraph",
                        "content": "Base",
                        "attrs": {},
                        "children": []
                    }
                }
            ]
        }),
        "author",
        "patch_id",
        "patch",
    );
    fs::write(
        &base_patch_path,
        serde_json::to_string_pretty(&base_patch).expect("base patch JSON should serialize"),
    )
    .expect("base patch JSON should be written");

    let base_revision = signed_object(
        json!({
            "type": "revision",
            "version": "mycel/0.1",
            "doc_id": "doc:test",
            "parents": [],
            "patches": [base_patch["patch_id"].as_str().expect("base patch id should exist")],
            "state_hash": state_hash(&json!({
                "doc_id": "doc:test",
                "blocks": [
                    {
                        "block_id": "blk:001",
                        "block_type": "paragraph",
                        "content": "Base",
                        "attrs": {},
                        "children": []
                    }
                ]
            })),
            "timestamp": 1777778888u64
        }),
        "author",
        "revision_id",
        "rev",
    );
    fs::write(
        &base_revision_path,
        serde_json::to_string_pretty(&base_revision).expect("base revision JSON should serialize"),
    )
    .expect("base revision JSON should be written");

    let side_patch = signed_object(
        json!({
            "type": "patch",
            "version": "mycel/0.1",
            "doc_id": "doc:test",
            "base_revision": "rev:genesis-null",
            "timestamp": 1777778889u64,
            "ops": [
                {
                    "op": "insert_block",
                    "new_block": {
                        "block_id": "blk:002",
                        "block_type": "paragraph",
                        "content": "Side",
                        "attrs": {},
                        "children": []
                    }
                }
            ]
        }),
        "author",
        "patch_id",
        "patch",
    );
    fs::write(
        &side_patch_path,
        serde_json::to_string_pretty(&side_patch).expect("side patch JSON should serialize"),
    )
    .expect("side patch JSON should be written");

    let side_revision = signed_object(
        json!({
            "type": "revision",
            "version": "mycel/0.1",
            "doc_id": "doc:test",
            "parents": [],
            "patches": [side_patch["patch_id"].as_str().expect("side patch id should exist")],
            "state_hash": state_hash(&json!({
                "doc_id": "doc:test",
                "blocks": [
                    {
                        "block_id": "blk:002",
                        "block_type": "paragraph",
                        "content": "Side",
                        "attrs": {},
                        "children": []
                    }
                ]
            })),
            "timestamp": 1777778890u64
        }),
        "author",
        "revision_id",
        "rev",
    );
    fs::write(
        &side_revision_path,
        serde_json::to_string_pretty(&side_revision).expect("side revision JSON should serialize"),
    )
    .expect("side revision JSON should be written");

    let merge_patch = signed_object(
        json!({
            "type": "patch",
            "version": "mycel/0.1",
            "doc_id": "doc:test",
            "base_revision": base_revision["revision_id"].as_str().expect("base revision id should exist"),
            "timestamp": 1777778891u64,
            "ops": [
                {
                    "op": "replace_block",
                    "block_id": "blk:001",
                    "new_content": "Merged"
                }
            ]
        }),
        "author",
        "patch_id",
        "patch",
    );
    fs::write(
        &merge_patch_path,
        serde_json::to_string_pretty(&merge_patch).expect("merge patch JSON should serialize"),
    )
    .expect("merge patch JSON should be written");

    let merge_revision = signed_object(
        json!({
            "type": "revision",
            "version": "mycel/0.1",
            "doc_id": "doc:test",
            "parents": [
                base_revision["revision_id"].as_str().expect("base revision id should exist"),
                side_revision["revision_id"].as_str().expect("side revision id should exist")
            ],
            "patches": [merge_patch["patch_id"].as_str().expect("merge patch id should exist")],
            "merge_strategy": "semantic-block-merge",
            "state_hash": state_hash(&json!({
                "doc_id": "doc:test",
                "blocks": [
                    {
                        "block_id": "blk:001",
                        "block_type": "paragraph",
                        "content": "Merged",
                        "attrs": {},
                        "children": []
                    }
                ]
            })),
            "timestamp": 1777778892u64
        }),
        "author",
        "revision_id",
        "rev",
    );
    fs::write(
        &merge_revision_path,
        serde_json::to_string_pretty(&merge_revision)
            .expect("merge revision JSON should serialize"),
    )
    .expect("merge revision JSON should be written");

    let output = run_mycel(&[
        "object",
        "verify",
        &path_arg(&merge_revision_path),
        "--json",
    ]);

    assert_success(&output);
    let json = assert_json_status(&output, "ok");
    assert_eq!(json["object_type"], "revision");
    assert_eq!(json["state_hash_verification"], "verified");
    assert_eq!(json["declared_state_hash"], json["recomputed_state_hash"]);
}

#[test]
fn object_verify_json_fails_when_merge_revision_implicitly_includes_secondary_parent_content() {
    let dir = create_temp_dir("object-verify-merge-secondary-parent-content");
    let base_revision_path = dir.path().join("revision-base.json");
    let side_revision_path = dir.path().join("revision-side.json");
    let merge_revision_path = dir.path().join("revision-merge.json");

    let base_revision = signed_object(
        json!({
            "type": "revision",
            "version": "mycel/0.1",
            "doc_id": "doc:test",
            "parents": [],
            "patches": [],
            "state_hash": state_hash(&json!({
                "doc_id": "doc:test",
                "blocks": [
                    {
                        "block_id": "blk:001",
                        "block_type": "paragraph",
                        "content": "Base",
                        "attrs": {},
                        "children": []
                    }
                ]
            })),
            "timestamp": 1777778888u64
        }),
        "author",
        "revision_id",
        "rev",
    );
    fs::write(
        &base_revision_path,
        serde_json::to_string_pretty(&base_revision).expect("base revision JSON should serialize"),
    )
    .expect("base revision JSON should be written");

    let side_revision = signed_object(
        json!({
            "type": "revision",
            "version": "mycel/0.1",
            "doc_id": "doc:test",
            "parents": [],
            "patches": [],
            "state_hash": state_hash(&json!({
                "doc_id": "doc:test",
                "blocks": [
                    {
                        "block_id": "blk:002",
                        "block_type": "paragraph",
                        "content": "Side",
                        "attrs": {},
                        "children": []
                    }
                ]
            })),
            "timestamp": 1777778889u64
        }),
        "author",
        "revision_id",
        "rev",
    );
    fs::write(
        &side_revision_path,
        serde_json::to_string_pretty(&side_revision).expect("side revision JSON should serialize"),
    )
    .expect("side revision JSON should be written");

    let merge_revision = signed_object(
        json!({
            "type": "revision",
            "version": "mycel/0.1",
            "doc_id": "doc:test",
            "parents": [
                base_revision["revision_id"].as_str().expect("base revision id should exist"),
                side_revision["revision_id"].as_str().expect("side revision id should exist")
            ],
            "patches": [],
            "merge_strategy": "semantic-block-merge",
            "state_hash": state_hash(&json!({
                "doc_id": "doc:test",
                "blocks": [
                    {
                        "block_id": "blk:001",
                        "block_type": "paragraph",
                        "content": "Base",
                        "attrs": {},
                        "children": []
                    },
                    {
                        "block_id": "blk:002",
                        "block_type": "paragraph",
                        "content": "Side",
                        "attrs": {},
                        "children": []
                    }
                ]
            })),
            "timestamp": 1777778890u64
        }),
        "author",
        "revision_id",
        "rev",
    );
    fs::write(
        &merge_revision_path,
        serde_json::to_string_pretty(&merge_revision)
            .expect("merge revision JSON should serialize"),
    )
    .expect("merge revision JSON should be written");

    let output = run_mycel(&[
        "object",
        "verify",
        &path_arg(&merge_revision_path),
        "--json",
    ]);

    assert_exit_code(&output, 1);
    let json = assert_json_status(&output, "failed");
    assert_eq!(json["object_type"], "revision");
    assert_eq!(json["state_hash_verification"], "failed");
    assert!(
        json["errors"]
            .as_array()
            .is_some_and(|errors| errors.iter().any(|entry| {
                entry.as_str().is_some_and(|message| {
                    message.contains("declared state_hash does not match replayed state hash")
                })
            })),
        "expected ancestry-only state-hash mismatch, stdout: {}",
        stdout_text(&output)
    );
}

#[test]
fn object_verify_json_fails_for_swapped_merge_parent_order() {
    let dir = create_temp_dir("object-verify-swapped-merge-parent-order");
    let base_revision_path = dir.path().join("revision-base.json");
    let side_revision_path = dir.path().join("revision-side.json");
    let merge_patch_path = dir.path().join("patch-merge.json");
    let merge_revision_path = dir.path().join("revision-merge.json");

    let base_revision = signed_object(
        json!({
            "type": "revision",
            "version": "mycel/0.1",
            "doc_id": "doc:test",
            "parents": [],
            "patches": [],
            "state_hash": state_hash(&json!({
                "doc_id": "doc:test",
                "blocks": [
                    {
                        "block_id": "blk:001",
                        "block_type": "paragraph",
                        "content": "Base",
                        "attrs": {},
                        "children": []
                    }
                ]
            })),
            "timestamp": 1777778888u64
        }),
        "author",
        "revision_id",
        "rev",
    );
    fs::write(
        &base_revision_path,
        serde_json::to_string_pretty(&base_revision).expect("base revision JSON should serialize"),
    )
    .expect("base revision JSON should be written");

    let side_revision = signed_object(
        json!({
            "type": "revision",
            "version": "mycel/0.1",
            "doc_id": "doc:test",
            "parents": [],
            "patches": [],
            "state_hash": state_hash(&json!({
                "doc_id": "doc:test",
                "blocks": [
                    {
                        "block_id": "blk:002",
                        "block_type": "paragraph",
                        "content": "Side",
                        "attrs": {},
                        "children": []
                    }
                ]
            })),
            "timestamp": 1777778889u64
        }),
        "author",
        "revision_id",
        "rev",
    );
    fs::write(
        &side_revision_path,
        serde_json::to_string_pretty(&side_revision).expect("side revision JSON should serialize"),
    )
    .expect("side revision JSON should be written");

    let merge_patch = signed_object(
        json!({
            "type": "patch",
            "version": "mycel/0.1",
            "doc_id": "doc:test",
            "base_revision": base_revision["revision_id"].as_str().expect("base revision id should exist"),
            "timestamp": 1777778890u64,
            "ops": []
        }),
        "author",
        "patch_id",
        "patch",
    );
    fs::write(
        &merge_patch_path,
        serde_json::to_string_pretty(&merge_patch).expect("merge patch JSON should serialize"),
    )
    .expect("merge patch JSON should be written");

    let merge_revision = signed_object(
        json!({
            "type": "revision",
            "version": "mycel/0.1",
            "doc_id": "doc:test",
            "parents": [
                side_revision["revision_id"].as_str().expect("side revision id should exist"),
                base_revision["revision_id"].as_str().expect("base revision id should exist")
            ],
            "patches": [merge_patch["patch_id"].as_str().expect("merge patch id should exist")],
            "merge_strategy": "semantic-block-merge",
            "state_hash": state_hash(&json!({
                "doc_id": "doc:test",
                "blocks": [
                    {
                        "block_id": "blk:002",
                        "block_type": "paragraph",
                        "content": "Side",
                        "attrs": {},
                        "children": []
                    }
                ]
            })),
            "timestamp": 1777778891u64
        }),
        "author",
        "revision_id",
        "rev",
    );
    fs::write(
        &merge_revision_path,
        serde_json::to_string_pretty(&merge_revision)
            .expect("merge revision JSON should serialize"),
    )
    .expect("merge revision JSON should be written");

    let output = run_mycel(&[
        "object",
        "verify",
        &path_arg(&merge_revision_path),
        "--json",
    ]);

    assert_exit_code(&output, 1);
    let json = assert_json_status(&output, "failed");
    assert_eq!(json["object_type"], "revision");
    assert_eq!(json["state_hash_verification"], "failed");
    assert!(
        json["errors"]
            .as_array()
            .is_some_and(|errors| errors.iter().any(|entry| {
                entry.as_str().is_some_and(|message| {
                    message.contains("base_revision 'rev:")
                        && message.contains("does not match expected 'rev:")
                })
            })),
        "expected swapped parent-order replay error, stdout: {}",
        stdout_text(&output)
    );
}

#[test]
fn object_verify_json_fails_for_revision_with_patch_from_other_document() {
    let dir = create_temp_dir("object-verify-revision-cross-document-patch");
    let patch_path = dir.path().join("patch-other-doc.json");
    let revision_path = dir.path().join("revision.json");

    let patch = signed_object(
        json!({
            "type": "patch",
            "version": "mycel/0.1",
            "doc_id": "doc:other",
            "base_revision": "rev:genesis-null",
            "timestamp": 1777778888u64,
            "ops": [
                {
                    "op": "insert_block",
                    "new_block": {
                        "block_id": "blk:001",
                        "block_type": "paragraph",
                        "content": "Hello",
                        "attrs": {},
                        "children": []
                    }
                }
            ]
        }),
        "author",
        "patch_id",
        "patch",
    );
    fs::write(
        &patch_path,
        serde_json::to_string_pretty(&patch).expect("patch JSON should serialize"),
    )
    .expect("patch JSON should be written");

    let revision = signed_object(
        json!({
            "type": "revision",
            "version": "mycel/0.1",
            "doc_id": "doc:test",
            "parents": [],
            "patches": [patch["patch_id"].as_str().expect("patch id should exist")],
            "state_hash": "hash:wrong",
            "timestamp": 1777778889u64
        }),
        "author",
        "revision_id",
        "rev",
    );
    fs::write(
        &revision_path,
        serde_json::to_string_pretty(&revision).expect("revision JSON should serialize"),
    )
    .expect("revision JSON should be written");

    let output = run_mycel(&["object", "verify", &path_arg(&revision_path), "--json"]);

    assert_exit_code(&output, 1);
    let json = assert_json_status(&output, "failed");
    assert_eq!(json["object_type"], "revision");
    assert_eq!(json["state_hash_verification"], "failed");
    assert!(
        json["errors"]
            .as_array()
            .is_some_and(|errors| errors.iter().any(|entry| {
                entry.as_str().is_some_and(|message| {
                    message.contains("patch '")
                        && message.contains("belongs to 'doc:other' instead of 'doc:test'")
                })
            })),
        "expected cross-document replay error, stdout: {}",
        stdout_text(&output)
    );
}

#[test]
fn object_verify_json_fails_for_revision_with_missing_parent_revision() {
    let dir = create_temp_dir("object-verify-revision-missing-parent");
    let patch_path = dir.path().join("patch.json");
    let revision_path = dir.path().join("revision.json");

    let patch = signed_object(
        json!({
            "type": "patch",
            "version": "mycel/0.1",
            "doc_id": "doc:test",
            "base_revision": "rev:missing-parent",
            "timestamp": 1777778888u64,
            "ops": []
        }),
        "author",
        "patch_id",
        "patch",
    );
    fs::write(
        &patch_path,
        serde_json::to_string_pretty(&patch).expect("patch JSON should serialize"),
    )
    .expect("patch JSON should be written");

    let revision = signed_object(
        json!({
            "type": "revision",
            "version": "mycel/0.1",
            "doc_id": "doc:test",
            "parents": ["rev:missing-parent"],
            "patches": [patch["patch_id"].as_str().expect("patch id should exist")],
            "state_hash": "hash:placeholder",
            "timestamp": 1777778889u64
        }),
        "author",
        "revision_id",
        "rev",
    );
    fs::write(
        &revision_path,
        serde_json::to_string_pretty(&revision).expect("revision JSON should serialize"),
    )
    .expect("revision JSON should be written");

    let output = run_mycel(&["object", "verify", &path_arg(&revision_path), "--json"]);

    assert_exit_code(&output, 1);
    let json = assert_json_status(&output, "failed");
    assert_eq!(json["object_type"], "revision");
    assert_eq!(json["state_hash_verification"], "failed");
    assert!(
        json["errors"]
            .as_array()
            .is_some_and(|errors| errors.iter().any(|entry| {
                entry.as_str().is_some_and(|message| {
                    message.contains("missing parent revision 'rev:missing-parent' for replay")
                })
            })),
        "expected missing parent replay error, stdout: {}",
        stdout_text(&output)
    );
}

#[test]
fn object_verify_json_fails_for_revision_with_missing_patch_object() {
    let dir = create_temp_dir("object-verify-revision-missing-patch");
    let revision_path = dir.path().join("revision.json");

    let revision = signed_object(
        json!({
            "type": "revision",
            "version": "mycel/0.1",
            "doc_id": "doc:test",
            "parents": [],
            "patches": ["patch:missing"],
            "state_hash": "hash:placeholder",
            "timestamp": 1777778889u64
        }),
        "author",
        "revision_id",
        "rev",
    );
    fs::write(
        &revision_path,
        serde_json::to_string_pretty(&revision).expect("revision JSON should serialize"),
    )
    .expect("revision JSON should be written");

    let output = run_mycel(&["object", "verify", &path_arg(&revision_path), "--json"]);

    assert_exit_code(&output, 1);
    let json = assert_json_status(&output, "failed");
    assert_eq!(json["object_type"], "revision");
    assert_eq!(json["state_hash_verification"], "failed");
    assert!(
        json["errors"]
            .as_array()
            .is_some_and(|errors| errors.iter().any(|entry| {
                entry.as_str().is_some_and(|message| {
                    message.contains("missing patch 'patch:missing' for replay")
                })
            })),
        "expected missing patch replay error, stdout: {}",
        stdout_text(&output)
    );
}

#[test]
fn object_verify_json_fails_for_revision_with_parent_from_other_document() {
    let dir = create_temp_dir("object-verify-revision-cross-document-parent");
    let parent_revision_path = dir.path().join("revision-parent.json");
    let revision_path = dir.path().join("revision.json");

    let parent_state_hash = state_hash(&json!({
        "doc_id": "doc:other",
        "blocks": [],
        "metadata": {}
    }));
    let parent_revision = signed_object(
        json!({
            "type": "revision",
            "version": "mycel/0.1",
            "doc_id": "doc:other",
            "parents": [],
            "patches": [],
            "state_hash": parent_state_hash,
            "timestamp": 1777778888u64
        }),
        "author",
        "revision_id",
        "rev",
    );
    fs::write(
        &parent_revision_path,
        serde_json::to_string_pretty(&parent_revision)
            .expect("parent revision JSON should serialize"),
    )
    .expect("parent revision JSON should be written");

    let revision = signed_object(
        json!({
            "type": "revision",
            "version": "mycel/0.1",
            "doc_id": "doc:test",
            "parents": [parent_revision["revision_id"].as_str().expect("parent revision id should exist")],
            "patches": [],
            "state_hash": "hash:placeholder",
            "timestamp": 1777778889u64
        }),
        "author",
        "revision_id",
        "rev",
    );
    fs::write(
        &revision_path,
        serde_json::to_string_pretty(&revision).expect("revision JSON should serialize"),
    )
    .expect("revision JSON should be written");

    let output = run_mycel(&["object", "verify", &path_arg(&revision_path), "--json"]);

    assert_exit_code(&output, 1);
    let json = assert_json_status(&output, "failed");
    assert_eq!(json["object_type"], "revision");
    assert_eq!(json["state_hash_verification"], "failed");
    assert!(
        json["errors"]
            .as_array()
            .is_some_and(|errors| errors.iter().any(|entry| {
                entry.as_str().is_some_and(|message| {
                    message.contains("parent revision '")
                        && message.contains("belongs to 'doc:other' instead of 'doc:test'")
                })
            })),
        "expected cross-document parent replay error, stdout: {}",
        stdout_text(&output)
    );
}

#[test]
fn object_verify_json_fails_for_revision_with_unparseable_parent_revision() {
    let dir = create_temp_dir("object-verify-revision-unparseable-parent");
    let parent_revision_path = dir.path().join("revision-parent.json");
    let patch_path = dir.path().join("patch.json");
    let revision_path = dir.path().join("revision.json");

    let malformed_parent = json!({
        "type": "revision",
        "version": "mycel/0.1",
        "revision_id": "rev:bad-parent",
        "doc_id": "doc:test",
        "parents": [],
        "patches": [],
        "author": signer_id(&signing_key()),
        "timestamp": 1777778888u64
    });
    fs::write(
        &parent_revision_path,
        serde_json::to_string_pretty(&malformed_parent)
            .expect("parent revision JSON should serialize"),
    )
    .expect("parent revision JSON should be written");

    let patch = signed_object(
        json!({
            "type": "patch",
            "version": "mycel/0.1",
            "doc_id": "doc:test",
            "base_revision": "rev:bad-parent",
            "timestamp": 1777778889u64,
            "ops": []
        }),
        "author",
        "patch_id",
        "patch",
    );
    fs::write(
        &patch_path,
        serde_json::to_string_pretty(&patch).expect("patch JSON should serialize"),
    )
    .expect("patch JSON should be written");

    let revision = signed_object(
        json!({
            "type": "revision",
            "version": "mycel/0.1",
            "doc_id": "doc:test",
            "parents": ["rev:bad-parent"],
            "patches": [patch["patch_id"].as_str().expect("patch id should exist")],
            "state_hash": "hash:placeholder",
            "timestamp": 1777778890u64
        }),
        "author",
        "revision_id",
        "rev",
    );
    fs::write(
        &revision_path,
        serde_json::to_string_pretty(&revision).expect("revision JSON should serialize"),
    )
    .expect("revision JSON should be written");

    let output = run_mycel(&["object", "verify", &path_arg(&revision_path), "--json"]);

    assert_exit_code(&output, 1);
    let json = assert_json_status(&output, "failed");
    assert_eq!(json["object_type"], "revision");
    assert_eq!(json["state_hash_verification"], "failed");
    assert!(
        json["errors"]
            .as_array()
            .is_some_and(|errors| errors.iter().any(|entry| {
                entry.as_str().is_some_and(|message| {
                    message.contains("failed to parse parent revision 'rev:bad-parent'")
                        && message.contains("missing string field 'state_hash'")
                })
            })),
        "expected parent parse replay error, stdout: {}",
        stdout_text(&output)
    );
}

#[test]
fn object_verify_json_fails_for_revision_with_unparseable_patch_dependency() {
    let dir = create_temp_dir("object-verify-revision-unparseable-patch");
    let patch_path = dir.path().join("patch.json");
    let revision_path = dir.path().join("revision.json");

    let malformed_patch = json!({
        "type": "patch",
        "version": "mycel/0.1",
        "patch_id": "patch:bad",
        "doc_id": "doc:test",
        "author": signer_id(&signing_key()),
        "timestamp": 1777778888u64,
        "ops": []
    });
    fs::write(
        &patch_path,
        serde_json::to_string_pretty(&malformed_patch).expect("patch JSON should serialize"),
    )
    .expect("patch JSON should be written");

    let revision = signed_object(
        json!({
            "type": "revision",
            "version": "mycel/0.1",
            "doc_id": "doc:test",
            "parents": [],
            "patches": ["patch:bad"],
            "state_hash": "hash:placeholder",
            "timestamp": 1777778889u64
        }),
        "author",
        "revision_id",
        "rev",
    );
    fs::write(
        &revision_path,
        serde_json::to_string_pretty(&revision).expect("revision JSON should serialize"),
    )
    .expect("revision JSON should be written");

    let output = run_mycel(&["object", "verify", &path_arg(&revision_path), "--json"]);

    assert_exit_code(&output, 1);
    let json = assert_json_status(&output, "failed");
    assert_eq!(json["object_type"], "revision");
    assert_eq!(json["state_hash_verification"], "failed");
    assert!(
        json["errors"]
            .as_array()
            .is_some_and(|errors| errors.iter().any(|entry| {
                entry
                    .as_str()
                    .is_some_and(|message| message.contains("failed to parse patch 'patch:bad'"))
            })),
        "expected patch parse replay error, stdout: {}",
        stdout_text(&output)
    );
}

#[test]
fn object_verify_json_fails_when_sibling_object_json_is_unparseable() {
    let dir = create_temp_dir("object-verify-unparseable-sibling-json");
    let sibling_path = dir.path().join("patch-bad.json");
    let revision_path = dir.path().join("revision.json");

    fs::write(
        &sibling_path,
        r#"{
  "type": "patch",
  "version": "mycel/0.1",
  "patch_id": "patch:bad",
  "patch_id": "patch:duplicate"
}"#,
    )
    .expect("malformed sibling JSON should be written");

    let revision = signed_object(
        json!({
            "type": "revision",
            "version": "mycel/0.1",
            "doc_id": "doc:test",
            "parents": [],
            "patches": [],
            "state_hash": state_hash(&json!({
                "doc_id": "doc:test",
                "blocks": [],
                "metadata": {}
            })),
            "timestamp": 1777778891u64
        }),
        "author",
        "revision_id",
        "rev",
    );
    fs::write(
        &revision_path,
        serde_json::to_string_pretty(&revision).expect("revision JSON should serialize"),
    )
    .expect("revision JSON should be written");

    let output = run_mycel(&["object", "verify", &path_arg(&revision_path), "--json"]);

    assert_exit_code(&output, 1);
    let json = assert_json_status(&output, "failed");
    assert_eq!(json["object_type"], "revision");
    assert!(
        json["errors"]
            .as_array()
            .is_some_and(|errors| errors.iter().any(|entry| {
                entry.as_str().is_some_and(|message| {
                    message.contains("failed to parse sibling object")
                        && message.contains("patch-bad.json")
                })
            })),
        "expected sibling parse failure, stdout: {}",
        stdout_text(&output)
    );
}

#[test]
fn object_verify_json_fails_when_sibling_object_has_invalid_declared_id() {
    let dir = create_temp_dir("object-verify-invalid-sibling-id");
    let sibling_path = dir.path().join("patch-bad-id.json");
    let revision_path = dir.path().join("revision.json");

    fs::write(
        &sibling_path,
        serde_json::to_string_pretty(&json!({
            "type": "patch",
            "version": "mycel/0.1",
            "patch_id": 7,
            "doc_id": "doc:test",
            "author": signer_id(&signing_key()),
            "base_revision": "rev:genesis-null",
            "timestamp": 1777778888u64,
            "ops": []
        }))
        .expect("sibling JSON should serialize"),
    )
    .expect("sibling JSON should be written");

    let revision = signed_object(
        json!({
            "type": "revision",
            "version": "mycel/0.1",
            "doc_id": "doc:test",
            "parents": [],
            "patches": [],
            "state_hash": state_hash(&json!({
                "doc_id": "doc:test",
                "blocks": [],
                "metadata": {}
            })),
            "timestamp": 1777778891u64
        }),
        "author",
        "revision_id",
        "rev",
    );
    fs::write(
        &revision_path,
        serde_json::to_string_pretty(&revision).expect("revision JSON should serialize"),
    )
    .expect("revision JSON should be written");

    let output = run_mycel(&["object", "verify", &path_arg(&revision_path), "--json"]);

    assert_exit_code(&output, 1);
    let json = assert_json_status(&output, "failed");
    assert_eq!(json["object_type"], "revision");
    assert!(
        json["errors"]
            .as_array()
            .is_some_and(|errors| errors.iter().any(|entry| {
                entry.as_str().is_some_and(|message| {
                    message.contains("sibling object")
                        && message.contains("patch-bad-id.json has invalid declared ID")
                })
            })),
        "expected invalid sibling declared ID error, stdout: {}",
        stdout_text(&output)
    );
}

#[test]
fn object_verify_json_fails_when_sibling_objects_duplicate_a_declared_id() {
    let dir = create_temp_dir("object-verify-duplicate-sibling-id");
    let sibling_a_path = dir.path().join("patch-a.json");
    let sibling_b_path = dir.path().join("patch-b.json");
    let revision_path = dir.path().join("revision.json");

    let patch = signed_object(
        json!({
            "type": "patch",
            "version": "mycel/0.1",
            "doc_id": "doc:test",
            "base_revision": "rev:genesis-null",
            "timestamp": 1777778888u64,
            "ops": []
        }),
        "author",
        "patch_id",
        "patch",
    );
    let patch_id = patch["patch_id"]
        .as_str()
        .expect("patch id should exist")
        .to_string();
    let patch_json =
        serde_json::to_string_pretty(&patch).expect("sibling patch JSON should serialize");
    fs::write(&sibling_a_path, &patch_json).expect("first sibling patch JSON should be written");
    fs::write(&sibling_b_path, &patch_json).expect("second sibling patch JSON should be written");

    let revision = signed_object(
        json!({
            "type": "revision",
            "version": "mycel/0.1",
            "doc_id": "doc:test",
            "parents": [],
            "patches": [patch_id.clone()],
            "state_hash": state_hash(&json!({
                "doc_id": "doc:test",
                "blocks": [],
                "metadata": {}
            })),
            "timestamp": 1777778891u64
        }),
        "author",
        "revision_id",
        "rev",
    );
    fs::write(
        &revision_path,
        serde_json::to_string_pretty(&revision).expect("revision JSON should serialize"),
    )
    .expect("revision JSON should be written");

    let output = run_mycel(&["object", "verify", &path_arg(&revision_path), "--json"]);

    assert_exit_code(&output, 1);
    let json = assert_json_status(&output, "failed");
    assert_eq!(json["object_type"], "revision");
    assert!(
        json["errors"]
            .as_array()
            .is_some_and(|errors| errors.iter().any(|entry| {
                entry.as_str().is_some_and(|message| {
                    message.contains("duplicate sibling object ID") && message.contains(&patch_id)
                })
            })),
        "expected duplicate sibling ID error, stdout: {}",
        stdout_text(&output)
    );
}

#[test]
fn object_verify_json_fails_when_sibling_json_entry_is_unreadable() {
    let dir = create_temp_dir("object-verify-unreadable-sibling-entry");
    let sibling_dir_path = dir.path().join("patch-dir.json");
    let revision_path = dir.path().join("revision.json");

    fs::create_dir(&sibling_dir_path).expect("sibling .json directory should be created");

    let revision = signed_object(
        json!({
            "type": "revision",
            "version": "mycel/0.1",
            "doc_id": "doc:test",
            "parents": [],
            "patches": [],
            "state_hash": state_hash(&json!({
                "doc_id": "doc:test",
                "blocks": [],
                "metadata": {}
            })),
            "timestamp": 1777778891u64
        }),
        "author",
        "revision_id",
        "rev",
    );
    fs::write(
        &revision_path,
        serde_json::to_string_pretty(&revision).expect("revision JSON should serialize"),
    )
    .expect("revision JSON should be written");

    let output = run_mycel(&["object", "verify", &path_arg(&revision_path), "--json"]);

    assert_exit_code(&output, 1);
    let json = assert_json_status(&output, "failed");
    assert_eq!(json["object_type"], "revision");
    assert!(
        json["errors"]
            .as_array()
            .is_some_and(|errors| errors.iter().any(|entry| {
                entry.as_str().is_some_and(|message| {
                    message.contains("failed to read sibling object")
                        && message.contains("patch-dir.json")
                })
            })),
        "expected unreadable sibling object error, stdout: {}",
        stdout_text(&output)
    );
}

#[test]
fn object_verify_json_fails_for_revision_with_invalid_move_cycle() {
    let dir = create_temp_dir("object-verify-revision-move-cycle");
    let parent_patch_path = dir.path().join("patch-parent.json");
    let child_patch_path = dir.path().join("patch-child.json");
    let base_revision_path = dir.path().join("revision-base.json");
    let move_patch_path = dir.path().join("patch-move.json");
    let moved_revision_path = dir.path().join("revision-move.json");

    let parent_patch = signed_object(
        json!({
            "type": "patch",
            "version": "mycel/0.1",
            "doc_id": "doc:test",
            "base_revision": "rev:genesis-null",
            "timestamp": 1777778888u64,
            "ops": [
                {
                    "op": "insert_block",
                    "new_block": {
                        "block_id": "blk:001",
                        "block_type": "paragraph",
                        "content": "Parent",
                        "attrs": {},
                        "children": []
                    }
                }
            ]
        }),
        "author",
        "patch_id",
        "patch",
    );
    fs::write(
        &parent_patch_path,
        serde_json::to_string_pretty(&parent_patch).expect("parent patch JSON should serialize"),
    )
    .expect("parent patch JSON should be written");

    let child_patch = signed_object(
        json!({
            "type": "patch",
            "version": "mycel/0.1",
            "doc_id": "doc:test",
            "base_revision": "rev:genesis-null",
            "timestamp": 1777778889u64,
            "ops": [
                {
                    "op": "insert_block",
                    "parent_block_id": "blk:001",
                    "new_block": {
                        "block_id": "blk:002",
                        "block_type": "paragraph",
                        "content": "Child",
                        "attrs": {},
                        "children": []
                    }
                }
            ]
        }),
        "author",
        "patch_id",
        "patch",
    );
    fs::write(
        &child_patch_path,
        serde_json::to_string_pretty(&child_patch).expect("child patch JSON should serialize"),
    )
    .expect("child patch JSON should be written");

    let base_state_hash = state_hash(&json!({
        "doc_id": "doc:test",
        "blocks": [
            {
                "block_id": "blk:001",
                "block_type": "paragraph",
                "content": "Parent",
                "attrs": {},
                "children": [
                    {
                        "block_id": "blk:002",
                        "block_type": "paragraph",
                        "content": "Child",
                        "attrs": {},
                        "children": []
                    }
                ]
            }
        ]
    }));
    let base_revision = signed_object(
        json!({
            "type": "revision",
            "version": "mycel/0.1",
            "doc_id": "doc:test",
            "parents": [],
            "patches": [
                parent_patch["patch_id"].as_str().expect("parent patch id should exist"),
                child_patch["patch_id"].as_str().expect("child patch id should exist")
            ],
            "state_hash": base_state_hash,
            "timestamp": 1777778890u64
        }),
        "author",
        "revision_id",
        "rev",
    );
    fs::write(
        &base_revision_path,
        serde_json::to_string_pretty(&base_revision).expect("base revision JSON should serialize"),
    )
    .expect("base revision JSON should be written");

    let move_patch = signed_object(
        json!({
            "type": "patch",
            "version": "mycel/0.1",
            "doc_id": "doc:test",
            "base_revision": base_revision["revision_id"].as_str().expect("base revision id should exist"),
            "timestamp": 1777778891u64,
            "ops": [
                {
                    "op": "move_block",
                    "block_id": "blk:001",
                    "parent_block_id": "blk:002"
                }
            ]
        }),
        "author",
        "patch_id",
        "patch",
    );
    fs::write(
        &move_patch_path,
        serde_json::to_string_pretty(&move_patch).expect("move patch JSON should serialize"),
    )
    .expect("move patch JSON should be written");

    let moved_revision = signed_object(
        json!({
            "type": "revision",
            "version": "mycel/0.1",
            "doc_id": "doc:test",
            "parents": [base_revision["revision_id"].as_str().expect("base revision id should exist")],
            "patches": [move_patch["patch_id"].as_str().expect("move patch id should exist")],
            "state_hash": "hash:placeholder",
            "timestamp": 1777778892u64
        }),
        "author",
        "revision_id",
        "rev",
    );
    fs::write(
        &moved_revision_path,
        serde_json::to_string_pretty(&moved_revision)
            .expect("moved revision JSON should serialize"),
    )
    .expect("moved revision JSON should be written");

    let output = run_mycel(&[
        "object",
        "verify",
        &path_arg(&moved_revision_path),
        "--json",
    ]);

    assert_exit_code(&output, 1);
    let json = assert_json_status(&output, "failed");
    assert_eq!(json["object_type"], "revision");
    assert_eq!(json["state_hash_verification"], "failed");
    assert!(
        json["errors"]
            .as_array()
            .is_some_and(
                |errors| errors
                    .iter()
                    .any(|entry| entry.as_str().is_some_and(|message| {
                        message.contains("move_block destination parent cannot be the moved block")
                    }))
            ),
        "expected move-cycle replay error, stdout: {}",
        stdout_text(&output)
    );
}
