use super::fixtures::*;
use super::*;

#[rstest]
#[case("patch", "patch_id", json!(7), "top-level 'patch_id' should be a string")]
#[case("revision", "revision_id", json!(7), "top-level 'revision_id' should be a string")]
#[case("view", "view_id", json!(7), "top-level 'view_id' should be a string")]
#[case("snapshot", "snapshot_id", json!(7), "top-level 'snapshot_id' should be a string")]
fn inspect_warns_when_derived_id_has_wrong_type(
    #[case] kind: &str,
    #[case] id_field: &str,
    #[case] invalid_value: Value,
    #[case] expected_note: &str,
) {
    let summary = inspect_strict_id_case_summary(kind, id_field, invalid_value);

    assert_eq!(summary.status, "warning");
    assert!(
        summary
            .notes
            .iter()
            .any(|message| message.contains(expected_note)),
        "expected strict derived ID type warning, got {summary:?}"
    );
}

#[rstest]
#[case("patch", "patch_id", json!("rev:test"), "top-level 'patch_id' must use 'patch:' prefix")]
#[case("revision", "revision_id", json!("patch:test"), "top-level 'revision_id' must use 'rev:' prefix")]
#[case("view", "view_id", json!("snap:test"), "top-level 'view_id' must use 'view:' prefix")]
#[case("snapshot", "snapshot_id", json!("view:test"), "top-level 'snapshot_id' must use 'snap:' prefix")]
fn inspect_warns_when_derived_id_prefix_is_wrong(
    #[case] kind: &str,
    #[case] id_field: &str,
    #[case] invalid_value: Value,
    #[case] expected_note: &str,
) {
    let summary = inspect_strict_id_case_summary(kind, id_field, invalid_value);

    assert_eq!(summary.status, "warning");
    assert!(
        summary
            .notes
            .iter()
            .any(|message| message.contains(expected_note)),
        "expected strict derived ID prefix warning, got {summary:?}"
    );
}

#[rstest]
#[case("document", None, "missing string field 'version'")]
#[case("document", Some(json!(7)), "top-level 'version' should be a string")]
#[case("patch", Some(json!("mycel/0.2")), "top-level 'version' must equal 'mycel/0.1'")]
#[case("revision", None, "missing string field 'version'")]
#[case("view", Some(json!("mycel/0.2")), "top-level 'version' must equal 'mycel/0.1'")]
#[case("snapshot", None, "missing string field 'version'")]
fn inspect_warns_when_core_protocol_version_is_missing_or_wrong(
    #[case] kind: &str,
    #[case] version_value: Option<Value>,
    #[case] expected_note: &str,
) {
    let summary = inspect_core_version_case_summary(kind, version_value);

    assert_eq!(summary.status, "warning");
    assert!(
        summary
            .notes
            .iter()
            .any(|message| message.contains(expected_note)),
        "expected core version warning, got {summary:?}"
    );
}

#[test]
fn inspect_warns_when_document_logical_id_has_wrong_type() {
    let path = write_test_file(
        "document-wrong-doc-id-type",
        &serde_json::to_string_pretty(&json!({
            "type": "document",
            "version": "mycel/0.1",
            "doc_id": 7,
            "title": "Plain document",
            "language": "zh-Hant",
            "content_model": "block-tree",
            "created_at": 1u64,
            "created_by": "pk:ed25519:test",
            "genesis_revision": "rev:test"
        }))
        .expect("test JSON should serialize"),
    );

    let summary = inspect_object_path(&path);

    assert_eq!(summary.status, "warning");
    assert!(
        summary
            .notes
            .iter()
            .any(|message| message.contains("top-level 'doc_id' should be a string")),
        "expected logical ID warning, got {summary:?}"
    );

    let _ = std::fs::remove_file(path);
}

#[test]
fn inspect_warns_when_document_content_model_is_wrong() {
    let path = write_test_file(
        "document-wrong-content-model-inspect",
        &serde_json::to_string_pretty(&json!({
            "type": "document",
            "version": "mycel/0.1",
            "doc_id": "doc:test",
            "content_model": "rich-text",
            "title": "Plain document",
            "language": "zh-Hant",
            "created_at": 1u64,
            "created_by": "pk:ed25519:test",
            "genesis_revision": "rev:test"
        }))
        .expect("test JSON should serialize"),
    );

    let summary = inspect_object_path(&path);

    assert_eq!(summary.status, "warning");
    assert!(
        summary.notes.iter().any(|message| {
            message.contains("top-level 'content_model' must equal 'block-tree'")
        }),
        "expected content-model warning, got {summary:?}"
    );

    let _ = std::fs::remove_file(path);
}

#[test]
fn inspect_warns_when_document_created_by_prefix_is_wrong() {
    let path = write_test_file(
        "document-wrong-created-by-prefix-inspect",
        &serde_json::to_string_pretty(&json!({
            "type": "document",
            "version": "mycel/0.1",
            "doc_id": "doc:test",
            "created_by": "author:test",
            "title": "Plain document",
            "language": "zh-Hant",
            "content_model": "block-tree",
            "created_at": 1u64,
            "genesis_revision": "rev:test"
        }))
        .expect("test JSON should serialize"),
    );

    let summary = inspect_object_path(&path);

    assert_eq!(summary.status, "warning");
    assert!(
        summary
            .notes
            .iter()
            .any(|message| message.contains("top-level 'created_by' must use 'pk:' prefix")),
        "expected created_by warning, got {summary:?}"
    );

    let _ = std::fs::remove_file(path);
}

#[test]
fn inspect_warns_when_document_genesis_revision_prefix_is_wrong() {
    let path = write_test_file(
        "document-wrong-genesis-revision-prefix-inspect",
        &serde_json::to_string_pretty(&json!({
            "type": "document",
            "version": "mycel/0.1",
            "doc_id": "doc:test",
            "genesis_revision": "hash:test",
            "title": "Plain document",
            "language": "zh-Hant",
            "content_model": "block-tree",
            "created_at": 1u64,
            "created_by": "pk:ed25519:test"
        }))
        .expect("test JSON should serialize"),
    );

    let summary = inspect_object_path(&path);

    assert_eq!(summary.status, "warning");
    assert!(
        summary.notes.iter().any(|message| {
            message.contains("top-level 'genesis_revision' must use 'rev:' prefix")
        }),
        "expected genesis revision warning, got {summary:?}"
    );

    let _ = std::fs::remove_file(path);
}

#[test]
fn inspect_warns_when_block_logical_id_has_wrong_type() {
    let path = write_test_file(
        "block-wrong-block-id-type",
        &serde_json::to_string_pretty(&json!({
            "type": "block",
            "version": "mycel/0.1",
            "block_id": 7,
            "block_type": "paragraph",
            "content": "Hello",
            "attrs": {},
            "children": []
        }))
        .expect("test JSON should serialize"),
    );

    let summary = inspect_object_path(&path);

    assert_eq!(summary.status, "warning");
    assert!(
        summary
            .notes
            .iter()
            .any(|message| message.contains("top-level 'block_id' should be a string")),
        "expected logical ID warning, got {summary:?}"
    );

    let _ = std::fs::remove_file(path);
}

#[test]
fn inspect_fails_when_block_nested_attrs_contains_null() {
    let path = write_test_file(
        "block-nested-attrs-null-inspect",
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

    let summary = inspect_object_path(&path);

    assert_eq!(summary.status, "warning");
    assert!(
        summary
            .notes
            .iter()
            .any(|message| { message.contains("$.attrs.style.tone: null is not allowed") }),
        "expected nested null warning, got {summary:?}"
    );

    let _ = std::fs::remove_file(path);
}

#[test]
fn inspect_warns_when_block_logical_id_prefix_is_wrong() {
    let path = write_test_file(
        "block-wrong-block-id-prefix-inspect",
        &serde_json::to_string_pretty(&json!({
            "type": "block",
            "version": "mycel/0.1",
            "block_id": "paragraph-1",
            "block_type": "paragraph",
            "content": "Hello",
            "attrs": {},
            "children": []
        }))
        .expect("test JSON should serialize"),
    );

    let summary = inspect_object_path(&path);

    assert_eq!(summary.status, "warning");
    assert!(
        summary
            .notes
            .iter()
            .any(|message| message.contains("top-level 'block_id' must use 'blk:' prefix")),
        "expected block_id prefix warning, got {summary:?}"
    );

    let _ = std::fs::remove_file(path);
}

#[test]
fn inspect_warns_when_block_contains_unknown_top_level_field() {
    let path = write_test_file(
        "block-unknown-top-level-field-inspect",
        &serde_json::to_string_pretty(&json!({
            "type": "block",
            "version": "mycel/0.1",
            "block_id": "blk:test",
            "block_type": "paragraph",
            "content": "Hello",
            "attrs": {},
            "children": [],
            "unexpected": true
        }))
        .expect("test JSON should serialize"),
    );

    let summary = inspect_object_path(&path);

    assert_eq!(summary.status, "warning");
    assert!(
        summary.notes.iter().any(|message| {
            message.contains("top-level contains unexpected field 'unexpected'")
        }),
        "expected unexpected-field warning, got {summary:?}"
    );

    let _ = std::fs::remove_file(path);
}

#[test]
fn inspect_warns_when_block_contains_unknown_nested_child_field() {
    let path = write_test_file(
        "block-unknown-nested-child-field-inspect",
        &serde_json::to_string_pretty(&json!({
            "type": "block",
            "version": "mycel/0.1",
            "block_id": "blk:test",
            "block_type": "paragraph",
            "content": "Hello",
            "attrs": {},
            "children": [
                {
                    "block_id": "blk:child",
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

    let summary = inspect_object_path(&path);

    assert_eq!(summary.status, "warning");
    assert!(
        summary.notes.iter().any(|message| {
            message.contains("top-level 'children[0]' contains unexpected field 'unexpected'")
        }),
        "expected nested child warning, got {summary:?}"
    );

    let _ = std::fs::remove_file(path);
}

#[test]
fn inspect_warns_when_block_nested_child_is_not_an_object() {
    let path = write_test_file(
        "block-non-object-nested-child-inspect",
        &serde_json::to_string_pretty(&json!({
            "type": "block",
            "version": "mycel/0.1",
            "block_id": "blk:test",
            "block_type": "paragraph",
            "content": "Hello",
            "attrs": {},
            "children": ["not-an-object"]
        }))
        .expect("test JSON should serialize"),
    );

    let summary = inspect_object_path(&path);

    assert_eq!(summary.status, "warning");
    assert!(
        summary
            .notes
            .iter()
            .any(|message| message.contains("top-level 'children[0]' must be a JSON object")),
        "expected nested child object-shape warning, got {summary:?}"
    );

    let _ = std::fs::remove_file(path);
}

#[test]
fn inspect_warns_when_block_nested_child_children_is_not_an_array() {
    let path = write_test_file(
        "block-nested-child-non-array-children-inspect",
        &serde_json::to_string_pretty(&json!({
            "type": "block",
            "version": "mycel/0.1",
            "block_id": "blk:test",
            "block_type": "paragraph",
            "content": "Hello",
            "attrs": {},
            "children": [
                {
                    "block_id": "blk:child",
                    "block_type": "paragraph",
                    "content": "Child",
                    "attrs": {},
                    "children": "not-an-array"
                }
            ]
        }))
        .expect("test JSON should serialize"),
    );

    let summary = inspect_object_path(&path);

    assert_eq!(summary.status, "warning");
    assert!(
        summary.notes.iter().any(|message| {
            message.contains("top-level 'children[0]': top-level 'children' must be an array")
        }),
        "expected nested child children-array warning, got {summary:?}"
    );

    let _ = std::fs::remove_file(path);
}

#[test]
fn inspect_warns_when_patch_base_revision_prefix_is_wrong() {
    let path = write_test_file(
        "patch-wrong-base-revision-prefix-inspect",
        &serde_json::to_string_pretty(&json!({
            "type": "patch",
            "version": "mycel/0.1",
            "patch_id": "patch:test",
            "doc_id": "doc:test",
            "base_revision": "hash:base",
            "author": "pk:ed25519:test",
            "timestamp": 1u64,
            "ops": [],
            "signature": "sig:ed25519:test"
        }))
        .expect("test JSON should serialize"),
    );

    let summary = inspect_object_path(&path);

    assert_eq!(summary.status, "warning");
    assert!(
        summary
            .notes
            .iter()
            .any(|message| message.contains("top-level 'base_revision' must use 'rev:' prefix")),
        "expected base_revision prefix warning, got {summary:?}"
    );

    let _ = std::fs::remove_file(path);
}

#[test]
fn inspect_warns_when_patch_block_reference_prefix_is_wrong() {
    let path = write_test_file(
        "patch-wrong-block-reference-prefix-inspect",
        &serde_json::to_string_pretty(&json!({
            "type": "patch",
            "version": "mycel/0.1",
            "patch_id": "patch:test",
            "doc_id": "doc:test",
            "base_revision": "rev:genesis-null",
            "author": "pk:ed25519:test",
            "timestamp": 1u64,
            "ops": [
                {
                    "op": "replace_block",
                    "block_id": "paragraph-1",
                    "new_content": "Hello"
                }
            ],
            "signature": "sig:ed25519:test"
        }))
        .expect("test JSON should serialize"),
    );

    let summary = inspect_object_path(&path);

    assert_eq!(summary.status, "warning");
    assert!(
        summary.notes.iter().any(|message| {
            message.contains("top-level 'ops[0]': top-level 'block_id' must use 'blk:' prefix")
        }),
        "expected block reference prefix warning, got {summary:?}"
    );

    let _ = std::fs::remove_file(path);
}

#[test]
fn inspect_warns_when_patch_author_prefix_is_wrong() {
    let path = write_test_file(
        "patch-wrong-author-prefix-inspect",
        &serde_json::to_string_pretty(&json!({
            "type": "patch",
            "version": "mycel/0.1",
            "patch_id": "patch:test",
            "doc_id": "doc:test",
            "base_revision": "rev:genesis-null",
            "author": "author:test",
            "timestamp": 1u64,
            "ops": [],
            "signature": "sig:ed25519:test"
        }))
        .expect("test JSON should serialize"),
    );

    let summary = inspect_object_path(&path);

    assert_eq!(summary.status, "warning");
    assert!(
        summary
            .notes
            .iter()
            .any(|message| message.contains("top-level 'author' must use 'pk:' prefix")),
        "expected author prefix warning, got {summary:?}"
    );

    let _ = std::fs::remove_file(path);
}

#[test]
fn inspect_warns_when_patch_contains_unknown_top_level_field() {
    let path = write_test_file(
        "patch-unknown-top-level-field-inspect",
        &serde_json::to_string_pretty(&json!({
            "type": "patch",
            "version": "mycel/0.1",
            "patch_id": "patch:test",
            "doc_id": "doc:test",
            "base_revision": "rev:genesis-null",
            "author": "pk:ed25519:test",
            "unexpected": true,
            "timestamp": 1u64,
            "ops": [],
            "signature": "sig:ed25519:test"
        }))
        .expect("test JSON should serialize"),
    );

    let summary = inspect_object_path(&path);

    assert_eq!(summary.status, "warning");
    assert!(
        summary
            .notes
            .iter()
            .any(|message| message.contains("top-level contains unexpected field 'unexpected'")),
        "expected unknown top-level field warning, got {summary:?}"
    );

    let _ = std::fs::remove_file(path);
}

#[test]
fn inspect_warns_when_patch_move_block_has_no_destination() {
    let path = write_test_file(
        "patch-move-without-destination-inspect",
        &serde_json::to_string_pretty(&json!({
            "type": "patch",
            "version": "mycel/0.1",
            "patch_id": "patch:test",
            "doc_id": "doc:test",
            "base_revision": "rev:genesis-null",
            "author": "pk:ed25519:test",
            "timestamp": 1u64,
            "ops": [
                {
                    "op": "move_block",
                    "block_id": "blk:001"
                }
            ],
            "signature": "sig:ed25519:test"
        }))
        .expect("test JSON should serialize"),
    );

    let summary = inspect_object_path(&path);

    assert_eq!(summary.status, "warning");
    assert!(
        summary.notes.iter().any(|message| {
            message.contains(
                "top-level 'ops[0]': move_block requires at least one destination reference",
            )
        }),
        "expected move_block destination warning, got {summary:?}"
    );

    let _ = std::fs::remove_file(path);
}

#[test]
fn inspect_warns_when_patch_metadata_entries_are_empty() {
    let path = write_test_file(
        "patch-empty-metadata-inspect",
        &serde_json::to_string_pretty(&json!({
            "type": "patch",
            "version": "mycel/0.1",
            "patch_id": "patch:test",
            "doc_id": "doc:test",
            "base_revision": "rev:genesis-null",
            "author": "pk:ed25519:test",
            "timestamp": 1u64,
            "ops": [
                {
                    "op": "set_metadata",
                    "metadata": {}
                }
            ],
            "signature": "sig:ed25519:test"
        }))
        .expect("test JSON should serialize"),
    );

    let summary = inspect_object_path(&path);

    assert_eq!(summary.status, "warning");
    assert!(
        summary.notes.iter().any(|message| {
            message.contains("top-level 'ops[0]': top-level 'metadata' must not be empty")
        }),
        "expected empty metadata warning, got {summary:?}"
    );

    let _ = std::fs::remove_file(path);
}

#[test]
fn inspect_warns_when_patch_set_metadata_single_entry_is_missing_value() {
    let path = write_test_file(
        "patch-set-metadata-missing-value-inspect",
        &serde_json::to_string_pretty(&json!({
            "type": "patch",
            "version": "mycel/0.1",
            "patch_id": "patch:test",
            "doc_id": "doc:test",
            "base_revision": "rev:genesis-null",
            "author": "pk:ed25519:test",
            "timestamp": 1u64,
            "ops": [
                {
                    "op": "set_metadata",
                    "key": "title"
                }
            ],
            "signature": "sig:ed25519:test"
        }))
        .expect("test JSON should serialize"),
    );

    let summary = inspect_object_path(&path);

    assert_eq!(summary.status, "warning");
    assert!(
        summary
            .notes
            .iter()
            .any(|message| message.contains("top-level 'ops[0]': missing object field 'value'")),
        "expected missing value warning, got {summary:?}"
    );

    let _ = std::fs::remove_file(path);
}

#[test]
fn inspect_warns_when_patch_set_metadata_single_entry_has_empty_key() {
    let path = write_test_file(
        "patch-set-metadata-empty-key-inspect",
        &serde_json::to_string_pretty(&json!({
            "type": "patch",
            "version": "mycel/0.1",
            "patch_id": "patch:test",
            "doc_id": "doc:test",
            "base_revision": "rev:genesis-null",
            "author": "pk:ed25519:test",
            "timestamp": 1u64,
            "ops": [
                {
                    "op": "set_metadata",
                    "key": "",
                    "value": "Hello"
                }
            ],
            "signature": "sig:ed25519:test"
        }))
        .expect("test JSON should serialize"),
    );

    let summary = inspect_object_path(&path);

    assert_eq!(summary.status, "warning");
    assert!(
        summary.notes.iter().any(|message| {
            message.contains("top-level 'ops[0]': top-level 'key' must not be an empty string")
        }),
        "expected empty key warning, got {summary:?}"
    );

    let _ = std::fs::remove_file(path);
}

#[test]
fn inspect_warns_when_patch_mixes_set_metadata_forms() {
    let path = write_test_file(
        "patch-mixed-set-metadata-forms-inspect",
        &serde_json::to_string_pretty(&json!({
            "type": "patch",
            "version": "mycel/0.1",
            "patch_id": "patch:test",
            "doc_id": "doc:test",
            "base_revision": "rev:genesis-null",
            "author": "pk:ed25519:test",
            "timestamp": 1u64,
            "ops": [
                {
                    "op": "set_metadata",
                    "metadata": {
                        "title": "Hello"
                    },
                    "key": "extra"
                }
            ],
            "signature": "sig:ed25519:test"
        }))
        .expect("test JSON should serialize"),
    );

    let summary = inspect_object_path(&path);

    assert_eq!(summary.status, "warning");
    assert!(
        summary.notes.iter().any(|message| {
            message.contains("top-level 'ops[0]': patch op contains unexpected field 'key'")
        }),
        "expected mixed set_metadata warning, got {summary:?}"
    );

    let _ = std::fs::remove_file(path);
}

#[test]
fn inspect_warns_when_patch_nested_new_block_contains_unknown_field() {
    let path = write_test_file(
        "patch-nested-new-block-unknown-field-inspect",
        &serde_json::to_string_pretty(&json!({
            "type": "patch",
            "version": "mycel/0.1",
            "patch_id": "patch:test",
            "doc_id": "doc:test",
            "base_revision": "rev:genesis-null",
            "author": "pk:ed25519:test",
            "timestamp": 1u64,
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
            ],
            "signature": "sig:ed25519:test"
        }))
        .expect("test JSON should serialize"),
    );

    let summary = inspect_object_path(&path);

    assert_eq!(summary.status, "warning");
    assert!(summary.notes.iter().any(|message| {
        message.contains(
            "top-level 'ops[0]': top-level 'new_block': top-level contains unexpected field 'unexpected'",
        )
    }), "expected nested new_block warning, got {summary:?}");

    let _ = std::fs::remove_file(path);
}

#[test]
fn inspect_warns_when_revision_state_hash_prefix_is_wrong() {
    let path = write_test_file(
        "revision-wrong-state-hash-prefix-inspect",
        &serde_json::to_string_pretty(&json!({
            "type": "revision",
            "version": "mycel/0.1",
            "revision_id": "rev:test",
            "doc_id": "doc:test",
            "parents": ["rev:base"],
            "patches": [],
            "state_hash": "rev:test",
            "author": "pk:ed25519:test",
            "timestamp": 1u64,
            "signature": "sig:ed25519:test"
        }))
        .expect("test JSON should serialize"),
    );

    let summary = inspect_object_path(&path);

    assert_eq!(summary.status, "warning");
    assert!(
        summary
            .notes
            .iter()
            .any(|message| message.contains("top-level 'state_hash' must use 'hash:' prefix")),
        "expected state_hash prefix warning, got {summary:?}"
    );

    let _ = std::fs::remove_file(path);
}

#[test]
fn inspect_warns_when_genesis_revision_has_merge_strategy() {
    let path = write_test_file(
        "revision-genesis-merge-strategy-inspect",
        &serde_json::to_string_pretty(&json!({
            "type": "revision",
            "version": "mycel/0.1",
            "revision_id": "rev:test",
            "doc_id": "doc:test",
            "parents": [],
            "patches": [],
            "merge_strategy": "semantic-block-merge",
            "state_hash": "hash:test",
            "author": "pk:ed25519:test",
            "timestamp": 1u64,
            "signature": "sig:ed25519:test"
        }))
        .expect("test JSON should serialize"),
    );

    let summary = inspect_object_path(&path);

    assert_eq!(summary.status, "warning");
    assert!(
        summary.notes.iter().any(|message| {
            message.contains("top-level 'merge_strategy' is not allowed when 'parents' is empty")
        }),
        "expected genesis merge_strategy warning, got {summary:?}"
    );

    let _ = std::fs::remove_file(path);
}

#[test]
fn inspect_warns_when_patch_nested_new_block_attrs_is_not_an_object() {
    let path = write_test_file(
        "patch-nested-new-block-non-object-attrs-inspect",
        &serde_json::to_string_pretty(&json!({
            "type": "patch",
            "version": "mycel/0.1",
            "patch_id": "patch:test",
            "doc_id": "doc:test",
            "base_revision": "rev:genesis-null",
            "author": "pk:ed25519:test",
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
            ],
            "signature": "sig:placeholder"
        }))
        .expect("test JSON should serialize"),
    );

    let summary = inspect_object_path(&path);

    assert_eq!(summary.status, "warning");
    assert!(
        summary.notes.iter().any(|message| {
            message.contains(
                "top-level 'ops[0]': top-level 'new_block': top-level 'attrs' must be an object",
            )
        }),
        "expected nested new_block attrs warning, got {summary:?}"
    );

    let _ = std::fs::remove_file(path);
}

#[test]
fn inspect_warns_when_revision_contains_duplicate_parent_ids() {
    let path = write_test_file(
        "revision-duplicate-parents-inspect",
        &serde_json::to_string_pretty(&json!({
            "type": "revision",
            "version": "mycel/0.1",
            "revision_id": "rev:test",
            "doc_id": "doc:test",
            "parents": ["rev:base", "rev:base"],
            "patches": [],
            "state_hash": "hash:test",
            "author": "pk:ed25519:test",
            "timestamp": 1u64,
            "signature": "sig:ed25519:test"
        }))
        .expect("test JSON should serialize"),
    );

    let summary = inspect_object_path(&path);

    assert_eq!(summary.status, "warning");
    assert!(
        summary
            .notes
            .iter()
            .any(|message| message.contains("top-level 'parents[1]' duplicates 'parents[0]'")),
        "expected duplicate parents warning, got {summary:?}"
    );

    let _ = std::fs::remove_file(path);
}

#[test]
fn inspect_warns_when_revision_contains_unknown_top_level_field() {
    let path = write_test_file(
        "revision-unknown-top-level-field-inspect",
        &serde_json::to_string_pretty(&json!({
            "type": "revision",
            "version": "mycel/0.1",
            "revision_id": "rev:test",
            "doc_id": "doc:test",
            "parents": ["rev:base"],
            "patches": [],
            "state_hash": "hash:test",
            "author": "pk:ed25519:test",
            "unexpected": true,
            "timestamp": 1u64,
            "signature": "sig:ed25519:test"
        }))
        .expect("test JSON should serialize"),
    );

    let summary = inspect_object_path(&path);

    assert_eq!(summary.status, "warning");
    assert!(
        summary
            .notes
            .iter()
            .any(|message| message.contains("top-level contains unexpected field 'unexpected'")),
        "expected unknown top-level field warning, got {summary:?}"
    );

    let _ = std::fs::remove_file(path);
}

#[test]
fn inspect_warns_when_revision_parent_prefix_is_wrong() {
    let path = write_test_file(
        "revision-wrong-parent-prefix-inspect",
        &serde_json::to_string_pretty(&json!({
            "type": "revision",
            "version": "mycel/0.1",
            "revision_id": "rev:test",
            "doc_id": "doc:test",
            "parents": ["hash:base"],
            "patches": [],
            "state_hash": "hash:test",
            "author": "pk:ed25519:test",
            "timestamp": 1u64,
            "signature": "sig:ed25519:test"
        }))
        .expect("test JSON should serialize"),
    );

    let summary = inspect_object_path(&path);

    assert_eq!(summary.status, "warning");
    assert!(
        summary
            .notes
            .iter()
            .any(|message| message.contains("top-level 'parents[0]' must use 'rev:' prefix")),
        "expected parent prefix warning, got {summary:?}"
    );

    let _ = std::fs::remove_file(path);
}

#[test]
fn inspect_warns_when_revision_contains_duplicate_patch_ids() {
    let path = write_test_file(
        "revision-duplicate-patches-inspect",
        &serde_json::to_string_pretty(&json!({
            "type": "revision",
            "version": "mycel/0.1",
            "revision_id": "rev:test",
            "doc_id": "doc:test",
            "parents": ["rev:base"],
            "patches": ["patch:test", "patch:test"],
            "state_hash": "hash:test",
            "author": "pk:ed25519:test",
            "timestamp": 1u64,
            "signature": "sig:ed25519:test"
        }))
        .expect("test JSON should serialize"),
    );

    let summary = inspect_object_path(&path);

    assert_eq!(summary.status, "warning");
    assert!(
        summary
            .notes
            .iter()
            .any(|message| message.contains("top-level 'patches[1]' duplicates 'patches[0]'")),
        "expected duplicate patches warning, got {summary:?}"
    );

    let _ = std::fs::remove_file(path);
}

#[test]
fn inspect_warns_when_revision_author_prefix_is_wrong() {
    let path = write_test_file(
        "revision-wrong-author-prefix-inspect",
        &serde_json::to_string_pretty(&json!({
            "type": "revision",
            "version": "mycel/0.1",
            "revision_id": "rev:test",
            "doc_id": "doc:test",
            "parents": ["rev:base"],
            "patches": [],
            "state_hash": "hash:test",
            "author": "author:test",
            "timestamp": 1u64,
            "signature": "sig:ed25519:test"
        }))
        .expect("test JSON should serialize"),
    );

    let summary = inspect_object_path(&path);

    assert_eq!(summary.status, "warning");
    assert!(
        summary
            .notes
            .iter()
            .any(|message| message.contains("top-level 'author' must use 'pk:' prefix")),
        "expected author prefix warning, got {summary:?}"
    );

    let _ = std::fs::remove_file(path);
}

#[test]
fn inspect_warns_when_revision_merge_strategy_requires_multiple_parents() {
    let path = write_test_file(
        "revision-merge-strategy-single-parent-inspect",
        &serde_json::to_string_pretty(&json!({
            "type": "revision",
            "version": "mycel/0.1",
            "revision_id": "rev:test",
            "doc_id": "doc:test",
            "parents": ["rev:base"],
            "patches": [],
            "merge_strategy": "semantic-block-merge",
            "state_hash": "hash:test",
            "author": "pk:ed25519:test",
            "timestamp": 1u64,
            "signature": "sig:ed25519:test"
        }))
        .expect("test JSON should serialize"),
    );

    let summary = inspect_object_path(&path);

    assert_eq!(summary.status, "warning");
    assert!(
        summary.notes.iter().any(|message| {
            message.contains("top-level 'merge_strategy' requires multiple parents")
        }),
        "expected merge_strategy warning, got {summary:?}"
    );

    let _ = std::fs::remove_file(path);
}

#[test]
fn inspect_warns_when_multi_parent_revision_is_missing_merge_strategy() {
    let path = write_test_file(
        "revision-missing-merge-strategy-inspect",
        &serde_json::to_string_pretty(&json!({
            "type": "revision",
            "version": "mycel/0.1",
            "revision_id": "rev:test",
            "doc_id": "doc:test",
            "parents": ["rev:base", "rev:side"],
            "patches": [],
            "state_hash": "hash:test",
            "author": "pk:ed25519:test",
            "timestamp": 1u64,
            "signature": "sig:ed25519:test"
        }))
        .expect("test JSON should serialize"),
    );

    let summary = inspect_object_path(&path);

    assert_eq!(summary.status, "warning");
    assert!(
        summary.notes.iter().any(|message| {
            message.contains(
                "top-level 'merge_strategy' is required when 'parents' has multiple entries",
            )
        }),
        "expected missing merge_strategy warning, got {summary:?}"
    );

    let _ = std::fs::remove_file(path);
}

#[test]
fn inspect_warns_when_revision_merge_strategy_is_not_a_string() {
    let path = write_test_file(
        "revision-non-string-merge-strategy-inspect",
        &serde_json::to_string_pretty(&json!({
            "type": "revision",
            "version": "mycel/0.1",
            "revision_id": "rev:test",
            "doc_id": "doc:test",
            "parents": ["rev:base", "rev:side"],
            "patches": [],
            "merge_strategy": 7,
            "state_hash": "hash:test",
            "author": "pk:ed25519:test",
            "timestamp": 1u64,
            "signature": "sig:ed25519:test"
        }))
        .expect("test JSON should serialize"),
    );

    let summary = inspect_object_path(&path);

    assert_eq!(summary.status, "warning");
    assert!(
        summary
            .notes
            .iter()
            .any(|message| message.contains("top-level 'merge_strategy' must be a string")),
        "expected merge_strategy type warning, got {summary:?}"
    );

    let _ = std::fs::remove_file(path);
}

#[test]
fn inspect_warns_when_view_policy_is_not_object() {
    let path = write_test_file(
        "view-policy-non-object-inspect",
        &serde_json::to_string_pretty(&json!({
            "type": "view",
            "version": "mycel/0.1",
            "view_id": "view:test",
            "maintainer": "pk:ed25519:test",
            "documents": {
                "doc:test": "rev:test"
            },
            "policy": "manual-reviewed",
            "timestamp": 12u64,
            "signature": "sig:ed25519:test"
        }))
        .expect("test JSON should serialize"),
    );

    let summary = inspect_object_path(&path);

    assert_eq!(summary.status, "warning");
    assert!(
        summary
            .notes
            .iter()
            .any(|message| message.contains("top-level 'policy' must be an object")),
        "expected non-object policy warning, got {summary:?}"
    );

    let _ = std::fs::remove_file(path);
}

#[test]
fn inspect_fails_when_view_policy_contains_floating_point_value() {
    let path = write_test_file(
        "view-policy-nested-float-inspect",
        &serde_json::to_string_pretty(&json!({
            "type": "view",
            "version": "mycel/0.1",
            "view_id": "view:test",
            "maintainer": "pk:ed25519:test",
            "documents": {
                "doc:test": "rev:test"
            },
            "policy": {
                "threshold": 0.5
            },
            "timestamp": 12u64,
            "signature": "sig:placeholder"
        }))
        .expect("test JSON should serialize"),
    );

    let summary = inspect_object_path(&path);

    assert_eq!(summary.status, "warning");
    assert!(
        summary.notes.iter().any(|message| {
            message.contains(
                "$.policy.threshold: floating-point numbers are not allowed in canonical objects",
            )
        }),
        "expected nested float warning, got {summary:?}"
    );

    let _ = std::fs::remove_file(path);
}

#[test]
fn inspect_warns_when_view_is_missing_policy() {
    let path = write_test_file(
        "view-missing-policy-inspect",
        &serde_json::to_string_pretty(&json!({
            "type": "view",
            "version": "mycel/0.1",
            "view_id": "view:test",
            "maintainer": "pk:ed25519:test",
            "documents": {
                "doc:test": "rev:test"
            },
            "timestamp": 12u64,
            "signature": "sig:ed25519:test"
        }))
        .expect("test JSON should serialize"),
    );

    let summary = inspect_object_path(&path);

    assert_eq!(summary.status, "warning");
    assert!(
        summary
            .notes
            .iter()
            .any(|message| message.contains("missing object field 'policy'")),
        "expected missing policy warning, got {summary:?}"
    );

    let _ = std::fs::remove_file(path);
}

#[test]
fn inspect_warns_when_view_maintainer_prefix_is_wrong() {
    let path = write_test_file(
        "view-wrong-maintainer-prefix-inspect",
        &serde_json::to_string_pretty(&json!({
            "type": "view",
            "version": "mycel/0.1",
            "view_id": "view:test",
            "maintainer": "maintainer:test",
            "documents": {
                "doc:test": "rev:test"
            },
            "policy": {
                "merge_rule": "manual-reviewed"
            },
            "timestamp": 12u64,
            "signature": "sig:ed25519:test"
        }))
        .expect("test JSON should serialize"),
    );

    let summary = inspect_object_path(&path);

    assert_eq!(summary.status, "warning");
    assert!(
        summary
            .notes
            .iter()
            .any(|message| message.contains("top-level 'maintainer' must use 'pk:' prefix")),
        "expected maintainer prefix warning, got {summary:?}"
    );

    let _ = std::fs::remove_file(path);
}

#[test]
fn inspect_warns_when_view_documents_is_empty() {
    let path = write_test_file(
        "view-empty-documents-inspect",
        &serde_json::to_string_pretty(&json!({
            "type": "view",
            "version": "mycel/0.1",
            "view_id": "view:test",
            "maintainer": "pk:ed25519:test",
            "documents": {},
            "policy": {
                "merge_rule": "manual-reviewed"
            },
            "timestamp": 12u64,
            "signature": "sig:ed25519:test"
        }))
        .expect("test JSON should serialize"),
    );

    let summary = inspect_object_path(&path);

    assert_eq!(summary.status, "warning");
    assert!(
        summary
            .notes
            .iter()
            .any(|message| message.contains("top-level 'documents' must not be empty")),
        "expected empty documents warning, got {summary:?}"
    );

    let _ = std::fs::remove_file(path);
}

#[test]
fn inspect_warns_when_view_document_value_prefix_is_wrong() {
    let path = write_test_file(
        "view-wrong-document-value-prefix-inspect",
        &serde_json::to_string_pretty(&json!({
            "type": "view",
            "version": "mycel/0.1",
            "view_id": "view:test",
            "maintainer": "pk:ed25519:test",
            "documents": {
                "doc:test": "patch:test"
            },
            "policy": {
                "merge_rule": "manual-reviewed"
            },
            "timestamp": 12u64,
            "signature": "sig:ed25519:test"
        }))
        .expect("test JSON should serialize"),
    );

    let summary = inspect_object_path(&path);

    assert_eq!(summary.status, "warning");
    assert!(
        summary.notes.iter().any(
            |message| message.contains("top-level 'documents.doc:test' must use 'rev:' prefix")
        ),
        "expected document value prefix warning, got {summary:?}"
    );

    let _ = std::fs::remove_file(path);
}

#[test]
fn inspect_warns_when_view_document_value_is_not_a_string() {
    let path = write_test_file(
        "view-non-string-document-value-inspect",
        &serde_json::to_string_pretty(&json!({
            "type": "view",
            "version": "mycel/0.1",
            "view_id": "view:test",
            "maintainer": "pk:ed25519:test",
            "documents": {
                "doc:test": 7
            },
            "policy": {
                "merge_rule": "manual-reviewed"
            },
            "timestamp": 12u64,
            "signature": "sig:ed25519:test"
        }))
        .expect("test JSON should serialize"),
    );

    let summary = inspect_object_path(&path);

    assert_eq!(summary.status, "warning");
    assert!(
        summary
            .notes
            .iter()
            .any(|message| message.contains("top-level 'documents.doc:test' must be a string")),
        "expected document value type warning, got {summary:?}"
    );

    let _ = std::fs::remove_file(path);
}

#[test]
fn inspect_warns_when_view_contains_unknown_top_level_field() {
    let path = write_test_file(
        "view-unknown-top-level-field-inspect",
        &serde_json::to_string_pretty(&json!({
            "type": "view",
            "version": "mycel/0.1",
            "view_id": "view:test",
            "maintainer": "pk:ed25519:test",
            "documents": {
                "doc:test": "rev:test"
            },
            "policy": {
                "merge_rule": "manual-reviewed"
            },
            "unexpected": true,
            "timestamp": 12u64,
            "signature": "sig:ed25519:test"
        }))
        .expect("test JSON should serialize"),
    );

    let summary = inspect_object_path(&path);

    assert_eq!(summary.status, "warning");
    assert!(
        summary
            .notes
            .iter()
            .any(|message| message.contains("top-level contains unexpected field 'unexpected'")),
        "expected unknown top-level field warning, got {summary:?}"
    );

    let _ = std::fs::remove_file(path);
}

#[test]
fn inspect_warns_when_snapshot_missing_declared_revision_in_included_objects() {
    let path = write_test_file(
        "snapshot-missing-declared-revision-inspect",
        &serde_json::to_string_pretty(&json!({
            "type": "snapshot",
            "version": "mycel/0.1",
            "snapshot_id": "snap:test",
            "documents": {
                "doc:test": "rev:test"
            },
            "included_objects": ["patch:test"],
            "root_hash": "hash:test",
            "created_by": "pk:ed25519:test",
            "timestamp": 9u64,
            "signature": "sig:ed25519:test"
        }))
        .expect("test JSON should serialize"),
    );

    let summary = inspect_object_path(&path);

    assert_eq!(summary.status, "warning");
    assert!(summary.notes.iter().any(|message| {
        message.contains(
            "top-level 'included_objects' must include revision 'rev:test' declared by 'documents.doc:test'",
        )
    }), "expected missing declared revision warning, got {summary:?}");

    let _ = std::fs::remove_file(path);
}

#[test]
fn inspect_warns_when_snapshot_document_value_is_not_a_string() {
    let path = write_test_file(
        "snapshot-non-string-document-value-inspect",
        &serde_json::to_string_pretty(&json!({
            "type": "snapshot",
            "version": "mycel/0.1",
            "snapshot_id": "snap:test",
            "documents": {
                "doc:test": 9
            },
            "included_objects": ["rev:test"],
            "root_hash": "hash:test",
            "created_by": "pk:ed25519:test",
            "timestamp": 9u64,
            "signature": "sig:ed25519:test"
        }))
        .expect("test JSON should serialize"),
    );

    let summary = inspect_object_path(&path);

    assert_eq!(summary.status, "warning");
    assert!(
        summary
            .notes
            .iter()
            .any(|message| message.contains("top-level 'documents.doc:test' must be a string")),
        "expected snapshot document value type warning, got {summary:?}"
    );

    let _ = std::fs::remove_file(path);
}

#[test]
fn inspect_warns_when_snapshot_document_key_prefix_is_wrong() {
    let path = write_test_file(
        "snapshot-wrong-document-key-prefix-inspect",
        &serde_json::to_string_pretty(&json!({
            "type": "snapshot",
            "version": "mycel/0.1",
            "snapshot_id": "snap:test",
            "documents": {
                "patch:test": "rev:test"
            },
            "included_objects": ["rev:test", "patch:test"],
            "root_hash": "hash:test",
            "created_by": "pk:ed25519:test",
            "timestamp": 9u64,
            "signature": "sig:ed25519:test"
        }))
        .expect("test JSON should serialize"),
    );

    let summary = inspect_object_path(&path);

    assert_eq!(summary.status, "warning");
    assert!(
        summary.notes.iter().any(|message| message
            .contains("top-level 'documents.patch:test key' must use 'doc:' prefix")),
        "expected document key prefix warning, got {summary:?}"
    );

    let _ = std::fs::remove_file(path);
}

#[test]
fn inspect_warns_when_snapshot_missing_documents() {
    let path = write_test_file(
        "snapshot-missing-documents-inspect",
        &serde_json::to_string_pretty(&json!({
            "type": "snapshot",
            "version": "mycel/0.1",
            "snapshot_id": "snap:test",
            "included_objects": ["rev:test"],
            "root_hash": "hash:test",
            "created_by": "pk:ed25519:test",
            "timestamp": 9u64,
            "signature": "sig:ed25519:test"
        }))
        .expect("test JSON should serialize"),
    );

    let summary = inspect_object_path(&path);

    assert_eq!(summary.status, "warning");
    assert!(
        summary
            .notes
            .iter()
            .any(|message| message.contains("missing object field 'documents'")),
        "expected missing documents warning, got {summary:?}"
    );

    let _ = std::fs::remove_file(path);
}

#[test]
fn inspect_warns_when_snapshot_contains_unknown_top_level_field() {
    let path = write_test_file(
        "snapshot-unknown-top-level-field-inspect",
        &serde_json::to_string_pretty(&json!({
            "type": "snapshot",
            "version": "mycel/0.1",
            "snapshot_id": "snap:test",
            "documents": {
                "doc:test": "rev:test"
            },
            "included_objects": ["rev:test", "patch:test"],
            "root_hash": "hash:test",
            "created_by": "pk:ed25519:test",
            "unexpected": true,
            "timestamp": 9u64,
            "signature": "sig:ed25519:test"
        }))
        .expect("test JSON should serialize"),
    );

    let summary = inspect_object_path(&path);

    assert_eq!(summary.status, "warning");
    assert!(
        summary
            .notes
            .iter()
            .any(|message| message.contains("top-level contains unexpected field 'unexpected'")),
        "expected unknown top-level field warning, got {summary:?}"
    );

    let _ = std::fs::remove_file(path);
}

#[test]
fn inspect_warns_when_snapshot_included_objects_duplicates_entry() {
    let path = write_test_file(
        "snapshot-duplicate-included-objects-inspect",
        &serde_json::to_string_pretty(&json!({
            "type": "snapshot",
            "version": "mycel/0.1",
            "snapshot_id": "snap:test",
            "documents": {
                "doc:test": "rev:test"
            },
            "included_objects": ["rev:test", "rev:test"],
            "root_hash": "hash:test",
            "created_by": "pk:ed25519:test",
            "timestamp": 9u64,
            "signature": "sig:ed25519:test"
        }))
        .expect("test JSON should serialize"),
    );

    let summary = inspect_object_path(&path);

    assert_eq!(summary.status, "warning");
    assert!(
        summary.notes.iter().any(|message| {
            message.contains("top-level 'included_objects[1]' duplicates 'included_objects[0]'")
        }),
        "expected duplicate included_objects warning, got {summary:?}"
    );

    let _ = std::fs::remove_file(path);
}

#[test]
fn inspect_warns_when_snapshot_included_objects_has_empty_entry() {
    let path = write_test_file(
        "snapshot-empty-included-object-entry-inspect",
        &serde_json::to_string_pretty(&json!({
            "type": "snapshot",
            "version": "mycel/0.1",
            "snapshot_id": "snap:test",
            "documents": {
                "doc:test": "rev:test"
            },
            "included_objects": ["rev:test", ""],
            "root_hash": "hash:test",
            "created_by": "pk:ed25519:test",
            "timestamp": 9u64,
            "signature": "sig:ed25519:test"
        }))
        .expect("test JSON should serialize"),
    );

    let summary = inspect_object_path(&path);

    assert_eq!(summary.status, "warning");
    assert!(
        summary.notes.iter().any(|message| {
            message.contains("top-level 'included_objects[1]' must not be an empty string")
        }),
        "expected empty included_objects entry warning, got {summary:?}"
    );

    let _ = std::fs::remove_file(path);
}

#[test]
fn inspect_warns_when_snapshot_included_objects_entry_is_not_a_string() {
    let path = write_test_file(
        "snapshot-non-string-included-object-entry-inspect",
        &serde_json::to_string_pretty(&json!({
            "type": "snapshot",
            "version": "mycel/0.1",
            "snapshot_id": "snap:test",
            "documents": {
                "doc:test": "rev:test"
            },
            "included_objects": ["rev:test", 7],
            "root_hash": "hash:test",
            "created_by": "pk:ed25519:test",
            "timestamp": 9u64,
            "signature": "sig:ed25519:test"
        }))
        .expect("test JSON should serialize"),
    );

    let summary = inspect_object_path(&path);

    assert_eq!(summary.status, "warning");
    assert!(
        summary
            .notes
            .iter()
            .any(|message| message.contains("top-level 'included_objects[1]' must be a string")),
        "expected included_objects type warning, got {summary:?}"
    );

    let _ = std::fs::remove_file(path);
}

#[test]
fn inspect_warns_when_snapshot_included_objects_has_non_canonical_id() {
    let path = write_test_file(
        "snapshot-non-canonical-included-object-inspect",
        &serde_json::to_string_pretty(&json!({
            "type": "snapshot",
            "version": "mycel/0.1",
            "snapshot_id": "snap:test",
            "documents": {
                "doc:test": "rev:test"
            },
            "included_objects": ["doc:test"],
            "root_hash": "hash:test",
            "created_by": "pk:ed25519:test",
            "timestamp": 9u64,
            "signature": "sig:ed25519:test"
        }))
        .expect("test JSON should serialize"),
    );

    let summary = inspect_object_path(&path);

    assert_eq!(summary.status, "warning");
    assert!(
        summary.notes.iter().any(|message| {
            message
                .contains("top-level 'included_objects[0]' must use a canonical object ID prefix")
        }),
        "expected canonical included_objects warning, got {summary:?}"
    );

    let _ = std::fs::remove_file(path);
}

#[test]
fn inspect_warns_when_snapshot_root_hash_prefix_is_wrong() {
    let path = write_test_file(
        "snapshot-wrong-root-hash-prefix-inspect",
        &serde_json::to_string_pretty(&json!({
            "type": "snapshot",
            "version": "mycel/0.1",
            "snapshot_id": "snap:test",
            "documents": {
                "doc:test": "rev:test"
            },
            "included_objects": ["rev:test", "patch:test"],
            "root_hash": "rev:test",
            "created_by": "pk:ed25519:test",
            "timestamp": 9u64,
            "signature": "sig:ed25519:test"
        }))
        .expect("test JSON should serialize"),
    );

    let summary = inspect_object_path(&path);

    assert_eq!(summary.status, "warning");
    assert!(
        summary
            .notes
            .iter()
            .any(|message| message.contains("top-level 'root_hash' must use 'hash:' prefix")),
        "expected root_hash prefix warning, got {summary:?}"
    );

    let _ = std::fs::remove_file(path);
}

#[test]
fn inspect_warns_when_snapshot_created_by_prefix_is_wrong() {
    let path = write_test_file(
        "snapshot-wrong-created-by-prefix-inspect",
        &serde_json::to_string_pretty(&json!({
            "type": "snapshot",
            "version": "mycel/0.1",
            "snapshot_id": "snap:test",
            "documents": {
                "doc:test": "rev:test"
            },
            "included_objects": ["rev:test", "patch:test"],
            "root_hash": "hash:test",
            "created_by": "creator:test",
            "timestamp": 9u64,
            "signature": "sig:ed25519:test"
        }))
        .expect("test JSON should serialize"),
    );

    let summary = inspect_object_path(&path);

    assert_eq!(summary.status, "warning");
    assert!(
        summary
            .notes
            .iter()
            .any(|message| message.contains("top-level 'created_by' must use 'pk:' prefix")),
        "expected created_by prefix warning, got {summary:?}"
    );

    let _ = std::fs::remove_file(path);
}
