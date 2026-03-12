use super::fixtures::*;
use super::*;

#[test]
pub(super) fn patch_id_recomputes_from_canonical_json() {
    let (signing_key, public_key) = signer_material();
    let mut value = json!({
        "type": "patch",
        "version": "mycel/0.1",
        "doc_id": "doc:test",
        "base_revision": "rev:genesis-null",
        "author": public_key,
        "timestamp": 1777778888u64,
        "ops": [],
    });
    let patch_id =
        recompute_object_id(&value, "patch_id", "patch").expect("patch ID should recompute");
    value["patch_id"] = Value::String(patch_id.clone());
    value["signature"] = Value::String(sign_value(&signing_key, &value));
    let path = write_test_file(
        "patch-valid",
        &serde_json::to_string_pretty(&value).expect("test JSON should serialize"),
    );

    let summary = verify_object_path(&path);

    assert!(summary.is_ok(), "expected success, got {summary:?}");
    assert_eq!(summary.signature_verification.as_deref(), Some("verified"));
    assert_eq!(summary.recomputed_id.as_deref(), Some(patch_id.as_str()));

    let _ = std::fs::remove_file(path);
}

#[test]
pub(super) fn patch_signature_is_reproducible_across_object_key_order() {
    let (signing_key, public_key) = signer_material();
    let left = json!({
        "type": "patch",
        "version": "mycel/0.1",
        "patch_id": "patch:test",
        "doc_id": "doc:test",
        "base_revision": "rev:genesis-null",
        "author": public_key,
        "timestamp": 1777778888u64,
        "ops": []
    });
    let right = json!({
        "ops": [],
        "timestamp": 1777778888u64,
        "author": public_key,
        "base_revision": "rev:genesis-null",
        "doc_id": "doc:test",
        "patch_id": "patch:test",
        "version": "mycel/0.1",
        "type": "patch"
    });

    let left_signature = sign_value(&signing_key, &left);
    let right_signature = sign_value(&signing_key, &right);

    assert_eq!(left_signature, right_signature);
}

#[test]
pub(super) fn null_values_are_rejected() {
    let path = write_test_file(
        "document-null",
        &serde_json::to_string_pretty(&json!({
            "type": "document",
            "version": "mycel/0.1",
            "doc_id": "doc:test",
            "title": null
        }))
        .expect("test JSON should serialize"),
    );

    let summary = verify_object_path(&path);

    assert!(!summary.is_ok(), "expected failure, got {summary:?}");
    assert!(
        summary
            .errors
            .iter()
            .any(|message| message.contains("$.title: null is not allowed")),
        "expected null validation error, got {summary:?}"
    );

    let _ = std::fs::remove_file(path);
}

#[test]
pub(super) fn block_nested_attrs_null_is_rejected_with_json_path() {
    let path = write_test_file(
        "block-nested-attrs-null",
        &serde_json::to_string_pretty(&json!({
            "type": "block",
            "version": "mycel/0.1",
            "block_id": "blk:test",
            "block_type": "paragraph",
            "content": "Hello",
            "attrs": {
                "style": {
                    "tone": null
                }
            },
            "children": []
        }))
        .expect("test JSON should serialize"),
    );

    let summary = verify_object_path(&path);

    assert!(!summary.is_ok(), "expected failure, got {summary:?}");
    assert!(
        summary
            .errors
            .iter()
            .any(|message| message.contains("$.attrs.style.tone: null is not allowed")),
        "expected nested null-path error, got {summary:?}"
    );

    let _ = std::fs::remove_file(path);
}

#[test]
pub(super) fn view_policy_nested_float_is_rejected_with_json_path() {
    let (_signing_key, public_key) = signer_material();
    let view = json!({
        "type": "view",
        "version": "mycel/0.1",
        "view_id": "view:test",
        "maintainer": public_key,
        "documents": {
            "doc:test": "rev:test"
        },
        "policy": {
            "threshold": 0.5
        },
        "timestamp": 12u64,
        "signature": "sig:placeholder"
    });
    let path = write_test_file(
        "view-policy-nested-float",
        &serde_json::to_string_pretty(&view).expect("test JSON should serialize"),
    );

    let summary = verify_object_path(&path);

    assert!(!summary.is_ok(), "expected failure, got {summary:?}");
    assert!(
        summary.errors.iter().any(|message| {
            message.contains(
                "$.policy.threshold: floating-point numbers are not allowed in canonical objects",
            )
        }),
        "expected nested float-path error, got {summary:?}"
    );

    let _ = std::fs::remove_file(path);
}

#[test]
pub(super) fn invalid_signature_is_rejected() {
    let (_signing_key, public_key) = signer_material();
    let mut value = json!({
        "type": "patch",
        "version": "mycel/0.1",
        "doc_id": "doc:test",
        "base_revision": "rev:genesis-null",
        "author": public_key,
        "timestamp": 1777778888u64,
        "ops": []
    });
    let patch_id =
        recompute_object_id(&value, "patch_id", "patch").expect("patch ID should recompute");
    value["patch_id"] = Value::String(patch_id);
    value["signature"] = Value::String(
        "sig:ed25519:AAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAA=="
            .to_string(),
    );
    let path = write_test_file(
        "patch-invalid-signature",
        &serde_json::to_string_pretty(&value).expect("test JSON should serialize"),
    );

    let summary = verify_object_path(&path);

    assert!(!summary.is_ok(), "expected failure, got {summary:?}");
    assert_eq!(summary.signature_verification.as_deref(), Some("failed"));
    assert!(
        summary
            .errors
            .iter()
            .any(|message| message.contains("Ed25519 signature verification failed")),
        "expected signature failure, got {summary:?}"
    );

    let _ = std::fs::remove_file(path);
}

#[rstest]
#[case("document", None, "missing string field 'version'")]
#[case("document", Some(json!("mycel/0.2")), "top-level 'version' must equal 'mycel/0.1'")]
#[case("patch", None, "missing string field 'version'")]
#[case("revision", Some(json!("mycel/0.2")), "top-level 'version' must equal 'mycel/0.1'")]
#[case("view", None, "missing string field 'version'")]
#[case("snapshot", Some(json!("mycel/0.2")), "top-level 'version' must equal 'mycel/0.1'")]
pub(super) fn core_protocol_version_must_match_for_typed_objects(
    #[case] kind: &str,
    #[case] version_value: Option<Value>,
    #[case] expected_error: &str,
) {
    let summary = verify_core_version_case_summary(kind, version_value);

    assert!(!summary.is_ok(), "expected failure, got {summary:?}");
    assert!(
        summary
            .errors
            .iter()
            .any(|message| message.contains(expected_error)),
        "expected core version strictness error, got {summary:?}"
    );
}

#[test]
pub(super) fn document_missing_logical_id_is_rejected() {
    let path = write_test_file(
        "document-missing-doc-id",
        &serde_json::to_string_pretty(&json!({
            "type": "document",
            "version": "mycel/0.1",
            "title": "Plain document"
        }))
        .expect("test JSON should serialize"),
    );

    let summary = verify_object_path(&path);

    assert!(!summary.is_ok(), "expected failure, got {summary:?}");
    assert!(
        summary.errors.iter().any(|message| {
            message.contains("document object is missing string field 'doc_id'")
        }),
        "expected missing logical ID error, got {summary:?}"
    );

    let _ = std::fs::remove_file(path);
}

#[rstest]
#[case("document", "doc_id", json!(7), "top-level 'doc_id' must be a string")]
#[case("block", "block_id", json!(7), "top-level 'block_id' must be a string")]
#[case("patch", "patch_id", json!(7), "top-level 'patch_id' must be a string")]
#[case("revision", "revision_id", json!(7), "top-level 'revision_id' must be a string")]
#[case("view", "view_id", json!(7), "top-level 'view_id' must be a string")]
#[case("snapshot", "snapshot_id", json!(7), "top-level 'snapshot_id' must be a string")]
pub(super) fn strict_id_type_errors_are_rejected(
    #[case] kind: &str,
    #[case] id_field: &str,
    #[case] invalid_value: Value,
    #[case] expected_error: &str,
) {
    let summary = verify_strict_id_case_summary(kind, id_field, invalid_value);

    assert!(!summary.is_ok(), "expected failure, got {summary:?}");
    assert!(
        summary
            .errors
            .iter()
            .any(|message| message.contains(expected_error)),
        "expected strict ID type error, got {summary:?}"
    );
}

#[rstest]
#[case("document", "doc_id", json!("revision:test"), "top-level 'doc_id' must use 'doc:' prefix")]
#[case("block", "block_id", json!("paragraph-1"), "top-level 'block_id' must use 'blk:' prefix")]
#[case("patch", "patch_id", json!("rev:test"), "top-level 'patch_id' must use 'patch:' prefix")]
#[case("revision", "revision_id", json!("patch:test"), "top-level 'revision_id' must use 'rev:' prefix")]
#[case("view", "view_id", json!("snap:test"), "top-level 'view_id' must use 'view:' prefix")]
#[case("snapshot", "snapshot_id", json!("view:test"), "top-level 'snapshot_id' must use 'snap:' prefix")]
pub(super) fn strict_id_prefix_errors_are_rejected(
    #[case] kind: &str,
    #[case] id_field: &str,
    #[case] invalid_value: Value,
    #[case] expected_error: &str,
) {
    let summary = verify_strict_id_case_summary(kind, id_field, invalid_value);

    assert!(!summary.is_ok(), "expected failure, got {summary:?}");
    assert!(
        summary
            .errors
            .iter()
            .any(|message| message.contains(expected_error)),
        "expected strict ID prefix error, got {summary:?}"
    );
}

#[test]
pub(super) fn document_missing_baseline_fields_is_rejected_by_typed_validation() {
    let path = write_test_file(
        "document-missing-title",
        &serde_json::to_string_pretty(&json!({
            "type": "document",
            "version": "mycel/0.1",
            "doc_id": "doc:test",
            "language": "zh-Hant",
            "content_model": "block-tree",
            "created_at": 1u64,
            "created_by": "pk:ed25519:test",
            "genesis_revision": "rev:test"
        }))
        .expect("test JSON should serialize"),
    );

    let summary = verify_object_path(&path);

    assert!(!summary.is_ok(), "expected failure, got {summary:?}");
    assert!(
        summary
            .errors
            .iter()
            .any(|message| message.contains("missing string field 'title'")),
        "expected typed document parse error, got {summary:?}"
    );

    let _ = std::fs::remove_file(path);
}

#[test]
pub(super) fn document_wrong_content_model_is_rejected() {
    let path = write_test_file(
        "document-wrong-content-model",
        &serde_json::to_string_pretty(&json!({
            "type": "document",
            "version": "mycel/0.1",
            "doc_id": "doc:test",
            "title": "Plain document",
            "language": "zh-Hant",
            "content_model": "markdown",
            "created_at": 1u64,
            "created_by": "pk:ed25519:test",
            "genesis_revision": "rev:test"
        }))
        .expect("test JSON should serialize"),
    );

    let summary = verify_object_path(&path);

    assert!(!summary.is_ok(), "expected failure, got {summary:?}");
    assert!(
        summary.errors.iter().any(|message| {
            message.contains("top-level 'content_model' must equal 'block-tree'")
        }),
        "expected content_model validation error, got {summary:?}"
    );

    let _ = std::fs::remove_file(path);
}

#[test]
pub(super) fn document_wrong_created_by_prefix_is_rejected() {
    let path = write_test_file(
        "document-wrong-created-by-prefix",
        &serde_json::to_string_pretty(&json!({
            "type": "document",
            "version": "mycel/0.1",
            "doc_id": "doc:test",
            "title": "Plain document",
            "language": "zh-Hant",
            "content_model": "block-tree",
            "created_at": 1u64,
            "created_by": "sig:bad",
            "genesis_revision": "rev:test"
        }))
        .expect("test JSON should serialize"),
    );

    let summary = verify_object_path(&path);

    assert!(!summary.is_ok(), "expected failure, got {summary:?}");
    assert!(
        summary
            .errors
            .iter()
            .any(|message| { message.contains("top-level 'created_by' must use 'pk:' prefix") }),
        "expected created_by prefix error, got {summary:?}"
    );

    let _ = std::fs::remove_file(path);
}

#[test]
pub(super) fn document_wrong_genesis_revision_prefix_is_rejected() {
    let path = write_test_file(
        "document-wrong-genesis-revision-prefix",
        &serde_json::to_string_pretty(&json!({
            "type": "document",
            "version": "mycel/0.1",
            "doc_id": "doc:test",
            "title": "Plain document",
            "language": "zh-Hant",
            "content_model": "block-tree",
            "created_at": 1u64,
            "created_by": "pk:ed25519:test",
            "genesis_revision": "hash:test"
        }))
        .expect("test JSON should serialize"),
    );

    let summary = verify_object_path(&path);

    assert!(!summary.is_ok(), "expected failure, got {summary:?}");
    assert!(
        summary.errors.iter().any(|message| {
            message.contains("top-level 'genesis_revision' must use 'rev:' prefix")
        }),
        "expected genesis_revision prefix error, got {summary:?}"
    );

    let _ = std::fs::remove_file(path);
}

#[test]
pub(super) fn revision_duplicate_parent_ids_are_rejected_by_typed_validation() {
    let (signing_key, public_key) = signer_material();
    let mut revision = json!({
        "type": "revision",
        "version": "mycel/0.1",
        "doc_id": "doc:test",
        "parents": ["rev:base", "rev:base"],
        "patches": [],
        "state_hash": "hash:test",
        "author": public_key,
        "timestamp": 11u64
    });
    let revision_id =
        recompute_object_id(&revision, "revision_id", "rev").expect("revision ID should recompute");
    revision["revision_id"] = Value::String(revision_id);
    revision["signature"] = Value::String(sign_value(&signing_key, &revision));
    let path = write_test_file(
        "revision-duplicate-parents",
        &serde_json::to_string_pretty(&revision).expect("test JSON should serialize"),
    );

    let summary = verify_object_path(&path);

    assert!(!summary.is_ok(), "expected failure, got {summary:?}");
    assert!(
        summary
            .errors
            .iter()
            .any(|message| message.contains("top-level 'parents[1]' duplicates 'parents[0]'")),
        "expected duplicate parent error, got {summary:?}"
    );

    let _ = std::fs::remove_file(path);
}

#[test]
pub(super) fn revision_wrong_parent_prefix_is_rejected() {
    let (signing_key, public_key) = signer_material();
    let mut revision = json!({
        "type": "revision",
        "version": "mycel/0.1",
        "doc_id": "doc:test",
        "parents": ["hash:base"],
        "patches": [],
        "state_hash": "hash:test",
        "author": public_key,
        "timestamp": 11u64
    });
    let revision_id =
        recompute_object_id(&revision, "revision_id", "rev").expect("revision ID should recompute");
    revision["revision_id"] = Value::String(revision_id);
    revision["signature"] = Value::String(sign_value(&signing_key, &revision));
    let path = write_test_file(
        "revision-wrong-parent-prefix",
        &serde_json::to_string_pretty(&revision).expect("test JSON should serialize"),
    );

    let summary = verify_object_path(&path);

    assert!(!summary.is_ok(), "expected failure, got {summary:?}");
    assert!(
        summary
            .errors
            .iter()
            .any(|message| message.contains("top-level 'parents[0]' must use 'rev:' prefix")),
        "expected parent prefix error, got {summary:?}"
    );

    let _ = std::fs::remove_file(path);
}

#[test]
pub(super) fn patch_wrong_base_revision_prefix_is_rejected() {
    let (signing_key, public_key) = signer_material();
    let mut patch = json!({
        "type": "patch",
        "version": "mycel/0.1",
        "doc_id": "doc:test",
        "base_revision": "hash:base",
        "author": public_key,
        "timestamp": 11u64,
        "ops": []
    });
    let patch_id =
        recompute_object_id(&patch, "patch_id", "patch").expect("patch ID should recompute");
    patch["patch_id"] = Value::String(patch_id);
    patch["signature"] = Value::String(sign_value(&signing_key, &patch));
    let path = write_test_file(
        "patch-wrong-base-revision-prefix",
        &serde_json::to_string_pretty(&patch).expect("test JSON should serialize"),
    );

    let summary = verify_object_path(&path);

    assert!(!summary.is_ok(), "expected failure, got {summary:?}");
    assert!(
        summary.errors.iter().any(|message| {
            message.contains("top-level 'base_revision' must use 'rev:' prefix")
        }),
        "expected base_revision prefix error, got {summary:?}"
    );

    let _ = std::fs::remove_file(path);
}

#[test]
pub(super) fn patch_wrong_block_reference_prefix_is_rejected() {
    let (signing_key, public_key) = signer_material();
    let mut patch = json!({
        "type": "patch",
        "version": "mycel/0.1",
        "doc_id": "doc:test",
        "base_revision": "rev:genesis-null",
        "author": public_key,
        "timestamp": 11u64,
        "ops": [
            {
                "op": "replace_block",
                "block_id": "paragraph-1",
                "new_content": "Hello"
            }
        ]
    });
    let patch_id =
        recompute_object_id(&patch, "patch_id", "patch").expect("patch ID should recompute");
    patch["patch_id"] = Value::String(patch_id);
    patch["signature"] = Value::String(sign_value(&signing_key, &patch));
    let path = write_test_file(
        "patch-wrong-block-reference-prefix",
        &serde_json::to_string_pretty(&patch).expect("test JSON should serialize"),
    );

    let summary = verify_object_path(&path);

    assert!(!summary.is_ok(), "expected failure, got {summary:?}");
    assert!(
        summary.errors.iter().any(|message| {
            message.contains("top-level 'ops[0]': top-level 'block_id' must use 'blk:' prefix")
        }),
        "expected block reference prefix error, got {summary:?}"
    );

    let _ = std::fs::remove_file(path);
}

#[test]
pub(super) fn patch_wrong_author_prefix_is_rejected() {
    let (signing_key, public_key) = signer_material();
    let mut patch = json!({
        "type": "patch",
        "version": "mycel/0.1",
        "doc_id": "doc:test",
        "base_revision": "rev:genesis-null",
        "author": public_key,
        "timestamp": 11u64,
        "ops": []
    });
    let patch_id =
        recompute_object_id(&patch, "patch_id", "patch").expect("patch ID should recompute");
    patch["patch_id"] = Value::String(patch_id);
    patch["signature"] = Value::String(sign_value(&signing_key, &patch));
    patch["author"] = Value::String("author:test".to_string());
    let path = write_test_file(
        "patch-wrong-author-prefix",
        &serde_json::to_string_pretty(&patch).expect("test JSON should serialize"),
    );

    let summary = verify_object_path(&path);

    assert!(!summary.is_ok(), "expected failure, got {summary:?}");
    assert!(
        summary.errors.iter().any(|message| {
            message.contains("signer field must use format 'pk:ed25519:<base64>'")
        }),
        "expected signer format error, got {summary:?}"
    );

    let _ = std::fs::remove_file(path);
}

#[test]
pub(super) fn patch_unknown_top_level_field_is_rejected() {
    let (signing_key, public_key) = signer_material();
    let mut patch = json!({
        "type": "patch",
        "version": "mycel/0.1",
        "doc_id": "doc:test",
        "base_revision": "rev:genesis-null",
        "author": public_key,
        "timestamp": 11u64,
        "ops": [],
        "unexpected": true
    });
    let patch_id =
        recompute_object_id(&patch, "patch_id", "patch").expect("patch ID should recompute");
    patch["patch_id"] = Value::String(patch_id);
    patch["signature"] = Value::String(sign_value(&signing_key, &patch));
    let path = write_test_file(
        "patch-unknown-top-level-field",
        &serde_json::to_string_pretty(&patch).expect("test JSON should serialize"),
    );

    let summary = verify_object_path(&path);

    assert!(!summary.is_ok(), "expected failure, got {summary:?}");
    assert!(
        summary
            .errors
            .iter()
            .any(|message| message.contains("top-level contains unexpected field 'unexpected'")),
        "expected unknown top-level field error, got {summary:?}"
    );

    let _ = std::fs::remove_file(path);
}

#[test]
pub(super) fn patch_move_without_destination_is_rejected() {
    let (signing_key, public_key) = signer_material();
    let mut patch = json!({
        "type": "patch",
        "version": "mycel/0.1",
        "doc_id": "doc:test",
        "base_revision": "rev:genesis-null",
        "author": public_key,
        "timestamp": 11u64,
        "ops": [
            {
                "op": "move_block",
                "block_id": "blk:001"
            }
        ]
    });
    let patch_id =
        recompute_object_id(&patch, "patch_id", "patch").expect("patch ID should recompute");
    patch["patch_id"] = Value::String(patch_id);
    patch["signature"] = Value::String(sign_value(&signing_key, &patch));
    let path = write_test_file(
        "patch-move-without-destination",
        &serde_json::to_string_pretty(&patch).expect("test JSON should serialize"),
    );

    let summary = verify_object_path(&path);

    assert!(!summary.is_ok(), "expected failure, got {summary:?}");
    assert!(
        summary.errors.iter().any(|message| {
            message.contains(
                "top-level 'ops[0]': move_block requires at least one destination reference",
            )
        }),
        "expected move_block destination error, got {summary:?}"
    );

    let _ = std::fs::remove_file(path);
}

#[test]
pub(super) fn patch_nested_new_block_unknown_field_is_rejected() {
    let (signing_key, public_key) = signer_material();
    let mut patch = json!({
        "type": "patch",
        "version": "mycel/0.1",
        "doc_id": "doc:test",
        "base_revision": "rev:genesis-null",
        "author": public_key,
        "timestamp": 11u64,
        "ops": [
            {
                "op": "insert_block",
                "new_block": {
                    "block_id": "blk:001",
                    "block_type": "paragraph",
                    "content": "Hello",
                    "attrs": {},
                    "children": [],
                    "unexpected": true
                }
            }
        ]
    });
    let patch_id =
        recompute_object_id(&patch, "patch_id", "patch").expect("patch ID should recompute");
    patch["patch_id"] = Value::String(patch_id);
    patch["signature"] = Value::String(sign_value(&signing_key, &patch));
    let path = write_test_file(
        "patch-nested-new-block-unknown-field",
        &serde_json::to_string_pretty(&patch).expect("test JSON should serialize"),
    );

    let summary = verify_object_path(&path);

    assert!(!summary.is_ok(), "expected failure, got {summary:?}");
    assert!(summary.errors.iter().any(|message| {
        message.contains(
            "top-level 'ops[0]': top-level 'new_block': top-level contains unexpected field 'unexpected'",
        )
    }), "expected nested new_block unknown-field error, got {summary:?}");

    let _ = std::fs::remove_file(path);
}

#[test]
pub(super) fn patch_nested_new_block_non_object_attrs_is_rejected() {
    let (signing_key, public_key) = signer_material();
    let mut patch = json!({
        "type": "patch",
        "version": "mycel/0.1",
        "doc_id": "doc:test",
        "base_revision": "rev:genesis-null",
        "author": public_key,
        "timestamp": 11u64,
        "ops": [
            {
                "op": "insert_block",
                "new_block": {
                    "block_id": "blk:001",
                    "block_type": "paragraph",
                    "content": "Hello",
                    "attrs": "not-an-object",
                    "children": []
                }
            }
        ]
    });
    let patch_id =
        recompute_object_id(&patch, "patch_id", "patch").expect("patch ID should recompute");
    patch["patch_id"] = Value::String(patch_id);
    patch["signature"] = Value::String(sign_value(&signing_key, &patch));
    let path = write_test_file(
        "patch-nested-new-block-non-object-attrs",
        &serde_json::to_string_pretty(&patch).expect("test JSON should serialize"),
    );

    let summary = verify_object_path(&path);

    assert!(!summary.is_ok(), "expected failure, got {summary:?}");
    assert!(
        summary.errors.iter().any(|message| {
            message.contains(
                "top-level 'ops[0]': top-level 'new_block': top-level 'attrs' must be an object",
            )
        }),
        "expected nested new_block attrs type error, got {summary:?}"
    );

    let _ = std::fs::remove_file(path);
}

#[test]
pub(super) fn patch_mixed_set_metadata_forms_are_rejected() {
    let (signing_key, public_key) = signer_material();
    let mut patch = json!({
        "type": "patch",
        "version": "mycel/0.1",
        "doc_id": "doc:test",
        "base_revision": "rev:genesis-null",
        "author": public_key,
        "timestamp": 11u64,
        "ops": [
            {
                "op": "set_metadata",
                "metadata": {
                    "title": "Hello"
                },
                "key": "extra"
            }
        ]
    });
    let patch_id =
        recompute_object_id(&patch, "patch_id", "patch").expect("patch ID should recompute");
    patch["patch_id"] = Value::String(patch_id);
    patch["signature"] = Value::String(sign_value(&signing_key, &patch));
    let path = write_test_file(
        "patch-mixed-set-metadata-forms",
        &serde_json::to_string_pretty(&patch).expect("test JSON should serialize"),
    );

    let summary = verify_object_path(&path);

    assert!(!summary.is_ok(), "expected failure, got {summary:?}");
    assert!(
        summary.errors.iter().any(|message| {
            message.contains("top-level 'ops[0]': patch op contains unexpected field 'key'")
        }),
        "expected mixed set_metadata forms error, got {summary:?}"
    );

    let _ = std::fs::remove_file(path);
}

#[test]
pub(super) fn patch_empty_metadata_entries_are_rejected() {
    let (signing_key, public_key) = signer_material();
    let mut patch = json!({
        "type": "patch",
        "version": "mycel/0.1",
        "doc_id": "doc:test",
        "base_revision": "rev:genesis-null",
        "author": public_key,
        "timestamp": 11u64,
        "ops": [
            {
                "op": "set_metadata",
                "metadata": {}
            }
        ]
    });
    let patch_id =
        recompute_object_id(&patch, "patch_id", "patch").expect("patch ID should recompute");
    patch["patch_id"] = Value::String(patch_id);
    patch["signature"] = Value::String(sign_value(&signing_key, &patch));
    let path = write_test_file(
        "patch-empty-metadata-entries",
        &serde_json::to_string_pretty(&patch).expect("test JSON should serialize"),
    );

    let summary = verify_object_path(&path);

    assert!(!summary.is_ok(), "expected failure, got {summary:?}");
    assert!(
        summary.errors.iter().any(|message| {
            message.contains("top-level 'ops[0]': top-level 'metadata' must not be empty")
        }),
        "expected empty metadata error, got {summary:?}"
    );

    let _ = std::fs::remove_file(path);
}

#[test]
pub(super) fn patch_set_metadata_single_entry_missing_value_is_rejected() {
    let (signing_key, public_key) = signer_material();
    let mut patch = json!({
        "type": "patch",
        "version": "mycel/0.1",
        "doc_id": "doc:test",
        "base_revision": "rev:genesis-null",
        "author": public_key,
        "timestamp": 11u64,
        "ops": [
            {
                "op": "set_metadata",
                "key": "title"
            }
        ]
    });
    let patch_id =
        recompute_object_id(&patch, "patch_id", "patch").expect("patch ID should recompute");
    patch["patch_id"] = Value::String(patch_id);
    patch["signature"] = Value::String(sign_value(&signing_key, &patch));
    let path = write_test_file(
        "patch-set-metadata-missing-value",
        &serde_json::to_string_pretty(&patch).expect("test JSON should serialize"),
    );

    let summary = verify_object_path(&path);

    assert!(!summary.is_ok(), "expected failure, got {summary:?}");
    assert!(
        summary
            .errors
            .iter()
            .any(|message| message.contains("top-level 'ops[0]': missing object field 'value'")),
        "expected missing value error, got {summary:?}"
    );

    let _ = std::fs::remove_file(path);
}

#[test]
pub(super) fn patch_set_metadata_single_entry_empty_key_is_rejected() {
    let (signing_key, public_key) = signer_material();
    let mut patch = json!({
        "type": "patch",
        "version": "mycel/0.1",
        "doc_id": "doc:test",
        "base_revision": "rev:genesis-null",
        "author": public_key,
        "timestamp": 11u64,
        "ops": [
            {
                "op": "set_metadata",
                "key": "",
                "value": "Hello"
            }
        ]
    });
    let patch_id =
        recompute_object_id(&patch, "patch_id", "patch").expect("patch ID should recompute");
    patch["patch_id"] = Value::String(patch_id);
    patch["signature"] = Value::String(sign_value(&signing_key, &patch));
    let path = write_test_file(
        "patch-set-metadata-empty-key",
        &serde_json::to_string_pretty(&patch).expect("test JSON should serialize"),
    );

    let summary = verify_object_path(&path);

    assert!(!summary.is_ok(), "expected failure, got {summary:?}");
    assert!(
        summary.errors.iter().any(|message| {
            message.contains("top-level 'ops[0]': top-level 'key' must not be an empty string")
        }),
        "expected empty key error, got {summary:?}"
    );

    let _ = std::fs::remove_file(path);
}

#[test]
pub(super) fn revision_duplicate_patch_ids_are_rejected_by_typed_validation() {
    let (signing_key, public_key) = signer_material();
    let mut revision = json!({
        "type": "revision",
        "version": "mycel/0.1",
        "doc_id": "doc:test",
        "parents": ["rev:base"],
        "patches": ["patch:test", "patch:test"],
        "state_hash": "hash:test",
        "author": public_key,
        "timestamp": 11u64
    });
    let revision_id =
        recompute_object_id(&revision, "revision_id", "rev").expect("revision ID should recompute");
    revision["revision_id"] = Value::String(revision_id);
    revision["signature"] = Value::String(sign_value(&signing_key, &revision));
    let path = write_test_file(
        "revision-duplicate-patches",
        &serde_json::to_string_pretty(&revision).expect("test JSON should serialize"),
    );

    let summary = verify_object_path(&path);

    assert!(!summary.is_ok(), "expected failure, got {summary:?}");
    assert!(
        summary
            .errors
            .iter()
            .any(|message| message.contains("top-level 'patches[1]' duplicates 'patches[0]'")),
        "expected duplicate patch error, got {summary:?}"
    );

    let _ = std::fs::remove_file(path);
}

#[test]
pub(super) fn revision_merge_strategy_requires_multiple_parents() {
    let (signing_key, public_key) = signer_material();
    let mut revision = json!({
        "type": "revision",
        "version": "mycel/0.1",
        "doc_id": "doc:test",
        "parents": ["rev:base"],
        "patches": [],
        "merge_strategy": "semantic-block-merge",
        "state_hash": "hash:test",
        "author": public_key,
        "timestamp": 11u64
    });
    let revision_id =
        recompute_object_id(&revision, "revision_id", "rev").expect("revision ID should recompute");
    revision["revision_id"] = Value::String(revision_id);
    revision["signature"] = Value::String(sign_value(&signing_key, &revision));
    let path = write_test_file(
        "revision-merge-strategy-single-parent",
        &serde_json::to_string_pretty(&revision).expect("test JSON should serialize"),
    );

    let summary = verify_object_path(&path);

    assert!(!summary.is_ok(), "expected failure, got {summary:?}");
    assert!(
        summary.errors.iter().any(|message| {
            message.contains("top-level 'merge_strategy' requires multiple parents")
        }),
        "expected merge_strategy parent-count error, got {summary:?}"
    );

    let _ = std::fs::remove_file(path);
}

#[test]
pub(super) fn genesis_revision_rejects_merge_strategy() {
    let (signing_key, public_key) = signer_material();
    let mut revision = json!({
        "type": "revision",
        "version": "mycel/0.1",
        "doc_id": "doc:test",
        "parents": [],
        "patches": [],
        "merge_strategy": "semantic-block-merge",
        "state_hash": "hash:test",
        "author": public_key,
        "timestamp": 11u64
    });
    let revision_id =
        recompute_object_id(&revision, "revision_id", "rev").expect("revision ID should recompute");
    revision["revision_id"] = Value::String(revision_id);
    revision["signature"] = Value::String(sign_value(&signing_key, &revision));
    let path = write_test_file(
        "revision-merge-strategy-genesis",
        &serde_json::to_string_pretty(&revision).expect("test JSON should serialize"),
    );

    let summary = verify_object_path(&path);

    assert!(!summary.is_ok(), "expected failure, got {summary:?}");
    assert!(
        summary.errors.iter().any(|message| {
            message.contains("top-level 'merge_strategy' is not allowed when 'parents' is empty")
        }),
        "expected genesis merge_strategy error, got {summary:?}"
    );

    let _ = std::fs::remove_file(path);
}

#[test]
pub(super) fn multi_parent_revision_requires_merge_strategy() {
    let (signing_key, public_key) = signer_material();
    let mut revision = json!({
        "type": "revision",
        "version": "mycel/0.1",
        "doc_id": "doc:test",
        "parents": ["rev:base", "rev:side"],
        "patches": [],
        "state_hash": "hash:test",
        "author": public_key,
        "timestamp": 11u64
    });
    let revision_id =
        recompute_object_id(&revision, "revision_id", "rev").expect("revision ID should recompute");
    revision["revision_id"] = Value::String(revision_id);
    revision["signature"] = Value::String(sign_value(&signing_key, &revision));
    let path = write_test_file(
        "revision-missing-merge-strategy",
        &serde_json::to_string_pretty(&revision).expect("test JSON should serialize"),
    );

    let summary = verify_object_path(&path);

    assert!(!summary.is_ok(), "expected failure, got {summary:?}");
    assert!(
        summary.errors.iter().any(|message| {
            message.contains(
                "top-level 'merge_strategy' is required when 'parents' has multiple entries",
            )
        }),
        "expected missing merge_strategy error, got {summary:?}"
    );

    let _ = std::fs::remove_file(path);
}

#[test]
pub(super) fn revision_non_string_merge_strategy_is_rejected() {
    let (signing_key, public_key) = signer_material();
    let mut revision = json!({
        "type": "revision",
        "version": "mycel/0.1",
        "doc_id": "doc:test",
        "parents": ["rev:base", "rev:side"],
        "patches": [],
        "merge_strategy": 7,
        "state_hash": "hash:test",
        "author": public_key,
        "timestamp": 11u64
    });
    let revision_id =
        recompute_object_id(&revision, "revision_id", "rev").expect("revision ID should recompute");
    revision["revision_id"] = Value::String(revision_id);
    revision["signature"] = Value::String(sign_value(&signing_key, &revision));
    let path = write_test_file(
        "revision-non-string-merge-strategy",
        &serde_json::to_string_pretty(&revision).expect("test JSON should serialize"),
    );

    let summary = verify_object_path(&path);

    assert!(!summary.is_ok(), "expected failure, got {summary:?}");
    assert!(
        summary
            .errors
            .iter()
            .any(|message| message.contains("top-level 'merge_strategy' must be a string")),
        "expected merge_strategy type error, got {summary:?}"
    );

    let _ = std::fs::remove_file(path);
}

#[test]
pub(super) fn revision_wrong_state_hash_prefix_is_rejected() {
    let (signing_key, public_key) = signer_material();
    let mut revision = json!({
        "type": "revision",
        "version": "mycel/0.1",
        "doc_id": "doc:test",
        "parents": ["rev:base"],
        "patches": [],
        "state_hash": "rev:test",
        "author": public_key,
        "timestamp": 11u64
    });
    let revision_id =
        recompute_object_id(&revision, "revision_id", "rev").expect("revision ID should recompute");
    revision["revision_id"] = Value::String(revision_id);
    revision["signature"] = Value::String(sign_value(&signing_key, &revision));
    let path = write_test_file(
        "revision-wrong-state-hash-prefix",
        &serde_json::to_string_pretty(&revision).expect("test JSON should serialize"),
    );

    let summary = verify_object_path(&path);

    assert!(!summary.is_ok(), "expected failure, got {summary:?}");
    assert!(
        summary
            .errors
            .iter()
            .any(|message| message.contains("top-level 'state_hash' must use 'hash:' prefix")),
        "expected state_hash prefix error, got {summary:?}"
    );

    let _ = std::fs::remove_file(path);
}

#[test]
pub(super) fn revision_wrong_author_prefix_is_rejected() {
    let (signing_key, public_key) = signer_material();
    let mut revision = json!({
        "type": "revision",
        "version": "mycel/0.1",
        "doc_id": "doc:test",
        "parents": ["rev:base"],
        "patches": [],
        "state_hash": "hash:test",
        "author": public_key,
        "timestamp": 11u64
    });
    let revision_id =
        recompute_object_id(&revision, "revision_id", "rev").expect("revision ID should recompute");
    revision["revision_id"] = Value::String(revision_id);
    revision["signature"] = Value::String(sign_value(&signing_key, &revision));
    revision["author"] = Value::String("author:test".to_string());
    let path = write_test_file(
        "revision-wrong-author-prefix",
        &serde_json::to_string_pretty(&revision).expect("test JSON should serialize"),
    );

    let summary = verify_object_path(&path);

    assert!(!summary.is_ok(), "expected failure, got {summary:?}");
    assert!(
        summary.errors.iter().any(|message| {
            message.contains("signer field must use format 'pk:ed25519:<base64>'")
        }),
        "expected signer format error, got {summary:?}"
    );

    let _ = std::fs::remove_file(path);
}

#[test]
pub(super) fn block_missing_logical_id_is_rejected() {
    let path = write_test_file(
        "block-missing-block-id",
        &serde_json::to_string_pretty(&json!({
            "type": "block",
            "version": "mycel/0.1",
            "block_type": "paragraph",
            "content": "Hello",
            "attrs": {},
            "children": []
        }))
        .expect("test JSON should serialize"),
    );

    let summary = verify_object_path(&path);

    assert!(!summary.is_ok(), "expected failure, got {summary:?}");
    assert!(
        summary
            .errors
            .iter()
            .any(|message| message.contains("block object is missing string field 'block_id'")),
        "expected missing logical ID error, got {summary:?}"
    );

    let _ = std::fs::remove_file(path);
}

#[test]
pub(super) fn block_unknown_top_level_field_is_rejected() {
    let path = write_test_file(
        "block-unknown-top-level-field",
        &serde_json::to_string_pretty(&json!({
            "type": "block",
            "version": "mycel/0.1",
            "block_id": "blk:001",
            "block_type": "paragraph",
            "content": "Hello",
            "attrs": {},
            "children": [],
            "unexpected": true
        }))
        .expect("test JSON should serialize"),
    );

    let summary = verify_object_path(&path);

    assert!(!summary.is_ok(), "expected failure, got {summary:?}");
    assert!(
        summary
            .errors
            .iter()
            .any(|message| message.contains("top-level contains unexpected field 'unexpected'")),
        "expected unknown-field error, got {summary:?}"
    );

    let _ = std::fs::remove_file(path);
}

#[test]
pub(super) fn block_unknown_nested_child_field_is_rejected() {
    let path = write_test_file(
        "block-unknown-nested-child-field",
        &serde_json::to_string_pretty(&json!({
            "type": "block",
            "version": "mycel/0.1",
            "block_id": "blk:001",
            "block_type": "paragraph",
            "content": "Hello",
            "attrs": {},
            "children": [
                {
                    "block_id": "blk:002",
                    "block_type": "paragraph",
                    "content": "Child",
                    "attrs": {},
                    "children": [],
                    "unexpected": true
                }
            ]
        }))
        .expect("test JSON should serialize"),
    );

    let summary = verify_object_path(&path);

    assert!(!summary.is_ok(), "expected failure, got {summary:?}");
    assert!(
        summary.errors.iter().any(|message| {
            message.contains("top-level 'children[0]' contains unexpected field 'unexpected'")
        }),
        "expected nested child unknown-field error, got {summary:?}"
    );

    let _ = std::fs::remove_file(path);
}

#[test]
pub(super) fn block_non_object_nested_child_is_rejected() {
    let path = write_test_file(
        "block-non-object-nested-child",
        &serde_json::to_string_pretty(&json!({
            "type": "block",
            "version": "mycel/0.1",
            "block_id": "blk:001",
            "block_type": "paragraph",
            "content": "Hello",
            "attrs": {},
            "children": ["not-an-object"]
        }))
        .expect("test JSON should serialize"),
    );

    let summary = verify_object_path(&path);

    assert!(!summary.is_ok(), "expected failure, got {summary:?}");
    assert!(
        summary
            .errors
            .iter()
            .any(|message| message.contains("top-level 'children[0]' must be a JSON object")),
        "expected nested child object-shape error, got {summary:?}"
    );

    let _ = std::fs::remove_file(path);
}

#[test]
pub(super) fn block_nested_child_non_array_children_is_rejected() {
    let path = write_test_file(
        "block-nested-child-non-array-children",
        &serde_json::to_string_pretty(&json!({
            "type": "block",
            "version": "mycel/0.1",
            "block_id": "blk:001",
            "block_type": "paragraph",
            "content": "Hello",
            "attrs": {},
            "children": [
                {
                    "block_id": "blk:002",
                    "block_type": "paragraph",
                    "content": "Child",
                    "attrs": {},
                    "children": "not-an-array"
                }
            ]
        }))
        .expect("test JSON should serialize"),
    );

    let summary = verify_object_path(&path);

    assert!(!summary.is_ok(), "expected failure, got {summary:?}");
    assert!(
        summary.errors.iter().any(|message| {
            message.contains("top-level 'children[0]': top-level 'children' must be an array")
        }),
        "expected nested child children array-shape error, got {summary:?}"
    );

    let _ = std::fs::remove_file(path);
}

#[test]
pub(super) fn snapshot_missing_documents_is_rejected_by_typed_validation() {
    let (signing_key, public_key) = signer_material();
    let mut snapshot = json!({
        "type": "snapshot",
        "version": "mycel/0.1",
        "included_objects": ["rev:test"],
        "root_hash": "hash:test",
        "created_by": public_key,
        "timestamp": 9u64
    });
    let snapshot_id = recompute_object_id(&snapshot, "snapshot_id", "snap")
        .expect("snapshot ID should recompute");
    snapshot["snapshot_id"] = Value::String(snapshot_id);
    snapshot["signature"] = Value::String(sign_value(&signing_key, &snapshot));
    let path = write_test_file(
        "snapshot-missing-documents",
        &serde_json::to_string_pretty(&snapshot).expect("test JSON should serialize"),
    );

    let summary = verify_object_path(&path);

    assert!(!summary.is_ok(), "expected failure, got {summary:?}");
    assert!(
        summary
            .errors
            .iter()
            .any(|message| message.contains("missing object field 'documents'")),
        "expected typed snapshot parse error, got {summary:?}"
    );

    let _ = std::fs::remove_file(path);
}

#[test]
pub(super) fn snapshot_empty_documents_is_rejected_by_typed_validation() {
    let (signing_key, public_key) = signer_material();
    let mut snapshot = json!({
        "type": "snapshot",
        "version": "mycel/0.1",
        "documents": {},
        "included_objects": ["rev:test"],
        "root_hash": "hash:test",
        "created_by": public_key,
        "timestamp": 9u64
    });
    let snapshot_id = recompute_object_id(&snapshot, "snapshot_id", "snap")
        .expect("snapshot ID should recompute");
    snapshot["snapshot_id"] = Value::String(snapshot_id);
    snapshot["signature"] = Value::String(sign_value(&signing_key, &snapshot));
    let path = write_test_file(
        "snapshot-empty-documents",
        &serde_json::to_string_pretty(&snapshot).expect("test JSON should serialize"),
    );

    let summary = verify_object_path(&path);

    assert!(!summary.is_ok(), "expected failure, got {summary:?}");
    assert!(
        summary
            .errors
            .iter()
            .any(|message| message.contains("top-level 'documents' must not be empty")),
        "expected empty documents error, got {summary:?}"
    );

    let _ = std::fs::remove_file(path);
}

#[test]
pub(super) fn snapshot_duplicate_included_objects_are_rejected() {
    let (signing_key, public_key) = signer_material();
    let mut snapshot = json!({
        "type": "snapshot",
        "version": "mycel/0.1",
        "documents": {
            "doc:test": "rev:test"
        },
        "included_objects": ["rev:test", "rev:test"],
        "root_hash": "hash:test",
        "created_by": public_key,
        "timestamp": 9u64
    });
    let snapshot_id = recompute_object_id(&snapshot, "snapshot_id", "snap")
        .expect("snapshot ID should recompute");
    snapshot["snapshot_id"] = Value::String(snapshot_id);
    snapshot["signature"] = Value::String(sign_value(&signing_key, &snapshot));
    let path = write_test_file(
        "snapshot-duplicate-included-objects",
        &serde_json::to_string_pretty(&snapshot).expect("test JSON should serialize"),
    );

    let summary = verify_object_path(&path);

    assert!(!summary.is_ok(), "expected failure, got {summary:?}");
    assert!(
        summary.errors.iter().any(|message| {
            message.contains("top-level 'included_objects[1]' duplicates 'included_objects[0]'")
        }),
        "expected duplicate included_objects error, got {summary:?}"
    );

    let _ = std::fs::remove_file(path);
}

#[test]
pub(super) fn snapshot_empty_included_object_entry_is_rejected() {
    let (signing_key, public_key) = signer_material();
    let mut snapshot = json!({
        "type": "snapshot",
        "version": "mycel/0.1",
        "documents": {
            "doc:test": "rev:test"
        },
        "included_objects": ["rev:test", ""],
        "root_hash": "hash:test",
        "created_by": public_key,
        "timestamp": 9u64
    });
    let snapshot_id = recompute_object_id(&snapshot, "snapshot_id", "snap")
        .expect("snapshot ID should recompute");
    snapshot["snapshot_id"] = Value::String(snapshot_id);
    snapshot["signature"] = Value::String(sign_value(&signing_key, &snapshot));
    let path = write_test_file(
        "snapshot-empty-included-object-entry",
        &serde_json::to_string_pretty(&snapshot).expect("test JSON should serialize"),
    );

    let summary = verify_object_path(&path);

    assert!(!summary.is_ok(), "expected failure, got {summary:?}");
    assert!(
        summary.errors.iter().any(|message| {
            message.contains("top-level 'included_objects[1]' must not be an empty string")
        }),
        "expected empty included_objects entry error, got {summary:?}"
    );

    let _ = std::fs::remove_file(path);
}

#[test]
pub(super) fn snapshot_non_canonical_included_object_id_is_rejected() {
    let (signing_key, public_key) = signer_material();
    let mut snapshot = json!({
        "type": "snapshot",
        "version": "mycel/0.1",
        "documents": {
            "doc:test": "rev:test"
        },
        "included_objects": ["doc:test"],
        "root_hash": "hash:test",
        "created_by": public_key,
        "timestamp": 9u64
    });
    let snapshot_id = recompute_object_id(&snapshot, "snapshot_id", "snap")
        .expect("snapshot ID should recompute");
    snapshot["snapshot_id"] = Value::String(snapshot_id);
    snapshot["signature"] = Value::String(sign_value(&signing_key, &snapshot));
    let path = write_test_file(
        "snapshot-non-canonical-included-object-id",
        &serde_json::to_string_pretty(&snapshot).expect("test JSON should serialize"),
    );

    let summary = verify_object_path(&path);

    assert!(!summary.is_ok(), "expected failure, got {summary:?}");
    assert!(
        summary.errors.iter().any(|message| {
            message
                .contains("top-level 'included_objects[0]' must use a canonical object ID prefix")
        }),
        "expected canonical included_objects ID error, got {summary:?}"
    );

    let _ = std::fs::remove_file(path);
}

#[test]
pub(super) fn snapshot_non_string_document_value_is_rejected() {
    let (signing_key, public_key) = signer_material();
    let mut snapshot = json!({
        "type": "snapshot",
        "version": "mycel/0.1",
        "documents": {
            "doc:test": 9
        },
        "included_objects": ["rev:test"],
        "root_hash": "hash:test",
        "created_by": public_key,
        "timestamp": 9u64
    });
    let snapshot_id = recompute_object_id(&snapshot, "snapshot_id", "snap")
        .expect("snapshot ID should recompute");
    snapshot["snapshot_id"] = Value::String(snapshot_id);
    snapshot["signature"] = Value::String(sign_value(&signing_key, &snapshot));
    let path = write_test_file(
        "snapshot-non-string-document-value",
        &serde_json::to_string_pretty(&snapshot).expect("test JSON should serialize"),
    );

    let summary = verify_object_path(&path);

    assert!(!summary.is_ok(), "expected failure, got {summary:?}");
    assert!(
        summary
            .errors
            .iter()
            .any(|message| message.contains("top-level 'documents.doc:test' must be a string")),
        "expected snapshot document value type error, got {summary:?}"
    );

    let _ = std::fs::remove_file(path);
}

#[test]
pub(super) fn snapshot_non_string_included_object_entry_is_rejected() {
    let (signing_key, public_key) = signer_material();
    let mut snapshot = json!({
        "type": "snapshot",
        "version": "mycel/0.1",
        "documents": {
            "doc:test": "rev:test"
        },
        "included_objects": ["rev:test", 7],
        "root_hash": "hash:test",
        "created_by": public_key,
        "timestamp": 9u64
    });
    let snapshot_id = recompute_object_id(&snapshot, "snapshot_id", "snap")
        .expect("snapshot ID should recompute");
    snapshot["snapshot_id"] = Value::String(snapshot_id);
    snapshot["signature"] = Value::String(sign_value(&signing_key, &snapshot));
    let path = write_test_file(
        "snapshot-non-string-included-object-entry",
        &serde_json::to_string_pretty(&snapshot).expect("test JSON should serialize"),
    );

    let summary = verify_object_path(&path);

    assert!(!summary.is_ok(), "expected failure, got {summary:?}");
    assert!(
        summary
            .errors
            .iter()
            .any(|message| message.contains("top-level 'included_objects[1]' must be a string")),
        "expected included_objects type error, got {summary:?}"
    );

    let _ = std::fs::remove_file(path);
}

#[test]
pub(super) fn snapshot_missing_declared_revision_in_included_objects_is_rejected() {
    let (signing_key, public_key) = signer_material();
    let mut snapshot = json!({
        "type": "snapshot",
        "version": "mycel/0.1",
        "documents": {
            "doc:test": "rev:test"
        },
        "included_objects": ["patch:test"],
        "root_hash": "hash:test",
        "created_by": public_key,
        "timestamp": 9u64
    });
    let snapshot_id = recompute_object_id(&snapshot, "snapshot_id", "snap")
        .expect("snapshot ID should recompute");
    snapshot["snapshot_id"] = Value::String(snapshot_id);
    snapshot["signature"] = Value::String(sign_value(&signing_key, &snapshot));
    let path = write_test_file(
        "snapshot-missing-declared-revision",
        &serde_json::to_string_pretty(&snapshot).expect("test JSON should serialize"),
    );

    let summary = verify_object_path(&path);

    assert!(!summary.is_ok(), "expected failure, got {summary:?}");
    assert!(summary.errors.iter().any(|message| {
        message.contains(
            "top-level 'included_objects' must include revision 'rev:test' declared by 'documents.doc:test'",
        )
    }), "expected missing declared revision error, got {summary:?}");

    let _ = std::fs::remove_file(path);
}

#[test]
pub(super) fn snapshot_wrong_root_hash_prefix_is_rejected() {
    let (signing_key, public_key) = signer_material();
    let mut snapshot = json!({
        "type": "snapshot",
        "version": "mycel/0.1",
        "documents": {
            "doc:test": "rev:test"
        },
        "included_objects": ["rev:test", "patch:test"],
        "root_hash": "rev:test",
        "created_by": public_key,
        "timestamp": 9u64
    });
    let snapshot_id = recompute_object_id(&snapshot, "snapshot_id", "snap")
        .expect("snapshot ID should recompute");
    snapshot["snapshot_id"] = Value::String(snapshot_id);
    snapshot["signature"] = Value::String(sign_value(&signing_key, &snapshot));
    let path = write_test_file(
        "snapshot-wrong-root-hash-prefix",
        &serde_json::to_string_pretty(&snapshot).expect("test JSON should serialize"),
    );

    let summary = verify_object_path(&path);

    assert!(!summary.is_ok(), "expected failure, got {summary:?}");
    assert!(
        summary
            .errors
            .iter()
            .any(|message| message.contains("top-level 'root_hash' must use 'hash:' prefix")),
        "expected root_hash prefix error, got {summary:?}"
    );

    let _ = std::fs::remove_file(path);
}

#[test]
pub(super) fn snapshot_wrong_created_by_prefix_is_rejected() {
    let (signing_key, public_key) = signer_material();
    let mut snapshot = json!({
        "type": "snapshot",
        "version": "mycel/0.1",
        "documents": {
            "doc:test": "rev:test"
        },
        "included_objects": ["rev:test", "patch:test"],
        "root_hash": "hash:test",
        "created_by": public_key.replacen("pk:", "sig:", 1),
        "timestamp": 9u64
    });
    let snapshot_id = recompute_object_id(&snapshot, "snapshot_id", "snap")
        .expect("snapshot ID should recompute");
    snapshot["snapshot_id"] = Value::String(snapshot_id);
    snapshot["signature"] = Value::String(sign_value(&signing_key, &snapshot));
    let path = write_test_file(
        "snapshot-wrong-created-by-prefix",
        &serde_json::to_string_pretty(&snapshot).expect("test JSON should serialize"),
    );

    let summary = verify_object_path(&path);

    assert!(!summary.is_ok(), "expected failure, got {summary:?}");
    assert!(
        summary
            .errors
            .iter()
            .any(|message| message.contains("signer field must use format 'pk:ed25519:<base64>'")),
        "expected created_by signer-format error, got {summary:?}"
    );

    let _ = std::fs::remove_file(path);
}

#[test]
pub(super) fn snapshot_unknown_top_level_field_is_rejected() {
    let (signing_key, public_key) = signer_material();
    let mut snapshot = json!({
        "type": "snapshot",
        "version": "mycel/0.1",
        "documents": {
            "doc:test": "rev:test"
        },
        "included_objects": ["rev:test", "patch:test"],
        "root_hash": "hash:test",
        "created_by": public_key,
        "timestamp": 9u64,
        "unexpected": true
    });
    let snapshot_id = recompute_object_id(&snapshot, "snapshot_id", "snap")
        .expect("snapshot ID should recompute");
    snapshot["snapshot_id"] = Value::String(snapshot_id);
    snapshot["signature"] = Value::String(sign_value(&signing_key, &snapshot));
    let path = write_test_file(
        "snapshot-unknown-top-level-field",
        &serde_json::to_string_pretty(&snapshot).expect("test JSON should serialize"),
    );

    let summary = verify_object_path(&path);

    assert!(!summary.is_ok(), "expected failure, got {summary:?}");
    assert!(
        summary
            .errors
            .iter()
            .any(|message| message.contains("top-level contains unexpected field 'unexpected'")),
        "expected unknown-field error, got {summary:?}"
    );

    let _ = std::fs::remove_file(path);
}

#[test]
pub(super) fn snapshot_mismatched_derived_id_is_rejected() {
    let (signing_key, public_key) = signer_material();
    let mut snapshot = json!({
        "type": "snapshot",
        "version": "mycel/0.1",
        "documents": {
            "doc:test": "rev:test"
        },
        "included_objects": ["rev:test", "patch:test"],
        "root_hash": "hash:test",
        "created_by": public_key,
        "timestamp": 9u64
    });
    let snapshot_id = recompute_object_id(&snapshot, "snapshot_id", "snap")
        .expect("snapshot ID should recompute");
    snapshot["snapshot_id"] = Value::String("snap:wrong".to_string());
    snapshot["signature"] = Value::String(sign_value(&signing_key, &snapshot));
    let path = write_test_file(
        "snapshot-mismatched-id",
        &serde_json::to_string_pretty(&snapshot).expect("test JSON should serialize"),
    );

    let summary = verify_object_path(&path);

    assert!(!summary.is_ok(), "expected failure, got {summary:?}");
    assert_eq!(summary.recomputed_id.as_deref(), Some(snapshot_id.as_str()));
    assert!(
        summary
            .errors
            .iter()
            .any(|message| message.contains("declared snapshot_id does not match")),
        "expected snapshot derived ID mismatch error, got {summary:?}"
    );

    let _ = std::fs::remove_file(path);
}

#[test]
pub(super) fn valid_snapshot_verifies_signature_and_typed_shape() {
    let (signing_key, public_key) = signer_material();
    let mut snapshot = json!({
        "type": "snapshot",
        "version": "mycel/0.1",
        "documents": {
            "doc:test": "rev:test"
        },
        "included_objects": ["rev:test", "patch:test"],
        "root_hash": "hash:test",
        "created_by": public_key,
        "timestamp": 9u64
    });
    let snapshot_id = recompute_object_id(&snapshot, "snapshot_id", "snap")
        .expect("snapshot ID should recompute");
    snapshot["snapshot_id"] = Value::String(snapshot_id.clone());
    snapshot["signature"] = Value::String(sign_value(&signing_key, &snapshot));
    let path = write_test_file(
        "snapshot-valid",
        &serde_json::to_string_pretty(&snapshot).expect("test JSON should serialize"),
    );

    let summary = verify_object_path(&path);

    assert!(summary.is_ok(), "expected success, got {summary:?}");
    assert_eq!(summary.signature_verification.as_deref(), Some("verified"));
    assert_eq!(summary.recomputed_id.as_deref(), Some(snapshot_id.as_str()));

    let _ = std::fs::remove_file(path);
}

#[test]
pub(super) fn view_wrong_maintainer_prefix_is_rejected_by_typed_validation() {
    let (signing_key, public_key) = signer_material();
    let mut view = json!({
        "type": "view",
        "version": "mycel/0.1",
        "maintainer": public_key.replacen("pk:", "sig:", 1),
        "documents": {
            "doc:test": "rev:test"
        },
        "policy": {
            "merge_rule": "manual-reviewed"
        },
        "timestamp": 12u64
    });
    let view_id = recompute_object_id(&view, "view_id", "view").expect("view ID should recompute");
    view["view_id"] = Value::String(view_id);
    view["signature"] = Value::String(sign_value(&signing_key, &view));
    let path = write_test_file(
        "view-wrong-maintainer-prefix",
        &serde_json::to_string_pretty(&view).expect("test JSON should serialize"),
    );

    let summary = verify_object_path(&path);

    assert!(!summary.is_ok(), "expected failure, got {summary:?}");
    assert!(
        summary
            .errors
            .iter()
            .any(|message| message.contains("signer field must use format 'pk:ed25519:<base64>'")),
        "expected maintainer signer-format error, got {summary:?}"
    );

    let _ = std::fs::remove_file(path);
}

#[rstest]
#[case(
    "snapshot",
    json!({"doc:test": "patch:test"}),
    "top-level 'documents.doc:test' must use 'rev:' prefix"
)]
#[case(
    "snapshot",
    json!({"patch:test": "rev:test"}),
    "top-level 'documents.patch:test key' must use 'doc:' prefix"
)]
#[case(
    "view",
    json!({"doc:test": "patch:test"}),
    "top-level 'documents.doc:test' must use 'rev:' prefix"
)]
#[case(
    "view",
    json!({"patch:test": "rev:test"}),
    "top-level 'documents.patch:test key' must use 'doc:' prefix"
)]

pub(super) fn documents_map_prefix_errors_are_rejected(
    #[case] kind: &str,
    #[case] documents: Value,
    #[case] expected_error: &str,
) {
    let summary = verify_documents_map_prefix_summary(kind, documents);

    assert!(!summary.is_ok(), "expected failure, got {summary:?}");
    assert!(
        summary
            .errors
            .iter()
            .any(|message| message.contains(expected_error)),
        "expected documents map prefix error, got {summary:?}"
    );
}

#[test]
pub(super) fn view_non_object_policy_is_rejected_by_typed_validation() {
    let (signing_key, public_key) = signer_material();
    let mut view = json!({
        "type": "view",
        "version": "mycel/0.1",
        "maintainer": public_key,
        "documents": {
            "doc:test": "rev:test"
        },
        "policy": "manual-reviewed",
        "timestamp": 12u64
    });
    let view_id = recompute_object_id(&view, "view_id", "view").expect("view ID should recompute");
    view["view_id"] = Value::String(view_id);
    view["signature"] = Value::String(sign_value(&signing_key, &view));
    let path = write_test_file(
        "view-policy-non-object",
        &serde_json::to_string_pretty(&view).expect("test JSON should serialize"),
    );

    let summary = verify_object_path(&path);

    assert!(!summary.is_ok(), "expected failure, got {summary:?}");
    assert!(
        summary
            .errors
            .iter()
            .any(|message| message.contains("top-level 'policy' must be an object")),
        "expected non-object policy error, got {summary:?}"
    );

    let _ = std::fs::remove_file(path);
}

#[test]
pub(super) fn view_missing_policy_is_rejected_by_typed_validation() {
    let (signing_key, public_key) = signer_material();
    let mut view = json!({
        "type": "view",
        "version": "mycel/0.1",
        "maintainer": public_key,
        "documents": {
            "doc:test": "rev:test"
        },
        "timestamp": 12u64
    });
    let view_id = recompute_object_id(&view, "view_id", "view").expect("view ID should recompute");
    view["view_id"] = Value::String(view_id);
    view["signature"] = Value::String(sign_value(&signing_key, &view));
    let path = write_test_file(
        "view-missing-policy",
        &serde_json::to_string_pretty(&view).expect("test JSON should serialize"),
    );

    let summary = verify_object_path(&path);

    assert!(!summary.is_ok(), "expected failure, got {summary:?}");
    assert!(
        summary
            .errors
            .iter()
            .any(|message| message.contains("missing object field 'policy'")),
        "expected missing policy error, got {summary:?}"
    );

    let _ = std::fs::remove_file(path);
}

#[test]
pub(super) fn view_non_string_document_value_is_rejected_by_typed_validation() {
    let (signing_key, public_key) = signer_material();
    let mut view = json!({
        "type": "view",
        "version": "mycel/0.1",
        "maintainer": public_key,
        "documents": {
            "doc:test": 7
        },
        "policy": {
            "merge_rule": "manual-reviewed"
        },
        "timestamp": 12u64
    });
    let view_id = recompute_object_id(&view, "view_id", "view").expect("view ID should recompute");
    view["view_id"] = Value::String(view_id);
    view["signature"] = Value::String(sign_value(&signing_key, &view));
    let path = write_test_file(
        "view-non-string-document-value",
        &serde_json::to_string_pretty(&view).expect("test JSON should serialize"),
    );

    let summary = verify_object_path(&path);

    assert!(!summary.is_ok(), "expected failure, got {summary:?}");
    assert!(
        summary
            .errors
            .iter()
            .any(|message| message.contains("top-level 'documents.doc:test' must be a string")),
        "expected document value type error, got {summary:?}"
    );

    let _ = std::fs::remove_file(path);
}

#[test]
pub(super) fn view_with_empty_documents_is_rejected_by_typed_validation() {
    let (signing_key, public_key) = signer_material();
    let mut view = json!({
        "type": "view",
        "version": "mycel/0.1",
        "maintainer": public_key,
        "documents": {},
        "policy": {
            "merge_rule": "manual-reviewed"
        },
        "timestamp": 12u64
    });
    let view_id = recompute_object_id(&view, "view_id", "view").expect("view ID should recompute");
    view["view_id"] = Value::String(view_id);
    view["signature"] = Value::String(sign_value(&signing_key, &view));
    let path = write_test_file(
        "view-empty-documents",
        &serde_json::to_string_pretty(&view).expect("test JSON should serialize"),
    );

    let summary = verify_object_path(&path);

    assert!(!summary.is_ok(), "expected failure, got {summary:?}");
    assert!(
        summary
            .errors
            .iter()
            .any(|message| message.contains("top-level 'documents' must not be empty")),
        "expected typed view parse error, got {summary:?}"
    );

    let _ = std::fs::remove_file(path);
}

#[test]
pub(super) fn view_unknown_top_level_field_is_rejected_by_typed_validation() {
    let (signing_key, public_key) = signer_material();
    let mut view = json!({
        "type": "view",
        "version": "mycel/0.1",
        "maintainer": public_key,
        "documents": {
            "doc:test": "rev:test"
        },
        "policy": {
            "merge_rule": "manual-reviewed"
        },
        "timestamp": 12u64,
        "unexpected": true
    });
    let view_id = recompute_object_id(&view, "view_id", "view").expect("view ID should recompute");
    view["view_id"] = Value::String(view_id);
    view["signature"] = Value::String(sign_value(&signing_key, &view));
    let path = write_test_file(
        "view-unknown-top-level-field",
        &serde_json::to_string_pretty(&view).expect("test JSON should serialize"),
    );

    let summary = verify_object_path(&path);

    assert!(!summary.is_ok(), "expected failure, got {summary:?}");
    assert!(
        summary
            .errors
            .iter()
            .any(|message| message.contains("top-level contains unexpected field 'unexpected'")),
        "expected unknown-field error, got {summary:?}"
    );

    let _ = std::fs::remove_file(path);
}
