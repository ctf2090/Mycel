use super::*;

#[test]
fn parse_patch_object_reads_ops_and_new_block() {
    let patch = parse_patch_object(&json!({
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
                    "children": []
                }
            }
        ]
    }))
    .expect("patch should parse");

    assert_eq!(patch.patch_id, "patch:test");
    assert_eq!(patch.ops.len(), 1);
}

#[test]
fn parse_patch_object_rejects_move_without_destination() {
    let error = parse_patch_object(&json!({
        "type": "patch",
        "version": "mycel/0.1",
        "patch_id": "patch:test",
        "doc_id": "doc:test",
        "base_revision": "rev:base",
        "author": "pk:ed25519:test",
        "timestamp": 1u64,
        "ops": [
            {
                "op": "move_block",
                "block_id": "blk:001"
            }
        ]
    }))
    .unwrap_err();

    assert_eq!(
        error.to_string(),
        "top-level 'ops[0]': move_block requires at least one destination reference"
    );
}

#[test]
fn parse_patch_object_rejects_empty_metadata_entries() {
    let error = parse_patch_object(&json!({
        "type": "patch",
        "version": "mycel/0.1",
        "patch_id": "patch:test",
        "doc_id": "doc:test",
        "base_revision": "rev:base",
        "author": "pk:ed25519:test",
        "timestamp": 1u64,
        "ops": [
            {
                "op": "set_metadata",
                "metadata": {}
            }
        ]
    }))
    .unwrap_err();

    assert_eq!(
        error.to_string(),
        "top-level 'ops[0]': top-level 'metadata' must not be empty"
    );
}

#[test]
fn parse_patch_object_rejects_wrong_block_reference_prefix() {
    let error = parse_patch_object(&json!({
        "type": "patch",
        "version": "mycel/0.1",
        "patch_id": "patch:test",
        "doc_id": "doc:test",
        "base_revision": "rev:base",
        "author": "pk:ed25519:test",
        "timestamp": 1u64,
        "ops": [
            {
                "op": "replace_block",
                "block_id": "paragraph-1",
                "new_content": "Hello"
            }
        ]
    }))
    .unwrap_err();

    assert_eq!(
        error.to_string(),
        "top-level 'ops[0]': top-level 'block_id' must use 'blk:' prefix"
    );
}

#[test]
fn parse_patch_object_rejects_wrong_base_revision_prefix() {
    let error = parse_patch_object(&json!({
        "type": "patch",
        "version": "mycel/0.1",
        "patch_id": "patch:test",
        "doc_id": "doc:test",
        "base_revision": "hash:base",
        "author": "pk:ed25519:test",
        "timestamp": 1u64,
        "ops": []
    }))
    .unwrap_err();

    assert_eq!(
        error.to_string(),
        "top-level 'base_revision' must use 'rev:' prefix"
    );
}

#[test]
fn parse_patch_object_rejects_non_string_patch_id() {
    let error = parse_patch_object(&json!({
        "type": "patch",
        "version": "mycel/0.1",
        "patch_id": 7,
        "doc_id": "doc:test",
        "base_revision": "rev:base",
        "author": "pk:ed25519:test",
        "timestamp": 1u64,
        "ops": []
    }))
    .unwrap_err();

    assert_eq!(error.to_string(), "top-level 'patch_id' must be a string");
}

#[test]
fn parse_patch_object_rejects_wrong_patch_id_prefix() {
    let error = parse_patch_object(&json!({
        "type": "patch",
        "version": "mycel/0.1",
        "patch_id": "rev:test",
        "doc_id": "doc:test",
        "base_revision": "rev:base",
        "author": "pk:ed25519:test",
        "timestamp": 1u64,
        "ops": []
    }))
    .unwrap_err();

    assert_eq!(
        error.to_string(),
        "top-level 'patch_id' must use 'patch:' prefix"
    );
}

#[test]
fn parse_patch_object_rejects_wrong_author_prefix() {
    let error = parse_patch_object(&json!({
        "type": "patch",
        "version": "mycel/0.1",
        "patch_id": "patch:test",
        "doc_id": "doc:test",
        "base_revision": "rev:base",
        "author": "author:test",
        "timestamp": 1u64,
        "ops": []
    }))
    .unwrap_err();

    assert_eq!(
        error.to_string(),
        "top-level 'author' must use 'pk:' prefix"
    );
}

#[test]
fn parse_patch_object_rejects_unknown_top_level_field() {
    let error = parse_patch_object(&json!({
        "type": "patch",
        "version": "mycel/0.1",
        "patch_id": "patch:test",
        "doc_id": "doc:test",
        "base_revision": "rev:base",
        "author": "pk:ed25519:test",
        "timestamp": 1u64,
        "ops": [],
        "unexpected": true
    }))
    .unwrap_err();

    assert_eq!(
        error.to_string(),
        "top-level contains unexpected field 'unexpected'"
    );
}

#[test]
fn parse_patch_object_rejects_unknown_patch_op_field() {
    let error = parse_patch_object(&json!({
        "type": "patch",
        "version": "mycel/0.1",
        "patch_id": "patch:test",
        "doc_id": "doc:test",
        "base_revision": "rev:base",
        "author": "pk:ed25519:test",
        "timestamp": 1u64,
        "ops": [
            {
                "op": "delete_block",
                "block_id": "blk:001",
                "new_content": "unexpected"
            }
        ]
    }))
    .unwrap_err();

    assert_eq!(
        error.to_string(),
        "top-level 'ops[0]': patch op contains unexpected field 'new_content'"
    );
}

#[test]
fn parse_patch_object_rejects_nested_new_block_missing_attrs_with_path() {
    let error = parse_patch_object(&json!({
        "type": "patch",
        "version": "mycel/0.1",
        "patch_id": "patch:test",
        "doc_id": "doc:test",
        "base_revision": "rev:base",
        "author": "pk:ed25519:test",
        "timestamp": 1u64,
        "ops": [
            {
                "op": "insert_block",
                "new_block": {
                    "block_id": "blk:001",
                    "block_type": "paragraph",
                    "content": "Hello",
                    "children": []
                }
            }
        ]
    }))
    .unwrap_err();

    assert_eq!(
        error.to_string(),
        "top-level 'ops[0]': top-level 'new_block': missing object field 'attrs'"
    );
}

#[test]
fn parse_patch_object_rejects_mixed_set_metadata_forms() {
    let error = parse_patch_object(&json!({
        "type": "patch",
        "version": "mycel/0.1",
        "patch_id": "patch:test",
        "doc_id": "doc:test",
        "base_revision": "rev:base",
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
        ]
    }))
    .unwrap_err();

    assert_eq!(
        error.to_string(),
        "top-level 'ops[0]': patch op contains unexpected field 'key'"
    );
}

#[test]
fn parse_patch_object_rejects_unknown_nested_new_block_field_with_path() {
    let error = parse_patch_object(&json!({
        "type": "patch",
        "version": "mycel/0.1",
        "patch_id": "patch:test",
        "doc_id": "doc:test",
        "base_revision": "rev:base",
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
        ]
    }))
    .unwrap_err();

    assert_eq!(
        error.to_string(),
        "top-level 'ops[0]': top-level 'new_block': top-level contains unexpected field 'unexpected'"
    );
}

#[test]
fn parse_document_object_reads_identity_and_baseline_fields() {
    let document = parse_document_object(&json!({
        "type": "document",
        "version": "mycel/0.1",
        "doc_id": "doc:test",
        "title": "Origin Text",
        "language": "zh-Hant",
        "content_model": "block-tree",
        "created_at": 1u64,
        "created_by": "pk:ed25519:test",
        "genesis_revision": "rev:test"
    }))
    .expect("document should parse");

    assert_eq!(document.doc_id, "doc:test");
    assert_eq!(document.content_model, "block-tree");
    assert_eq!(document.genesis_revision, "rev:test");
}

#[test]
fn parse_document_object_rejects_wrong_content_model() {
    let error = parse_document_object(&json!({
        "type": "document",
        "version": "mycel/0.1",
        "doc_id": "doc:test",
        "title": "Origin Text",
        "language": "zh-Hant",
        "content_model": "markdown",
        "created_at": 1u64,
        "created_by": "pk:ed25519:test",
        "genesis_revision": "rev:test"
    }))
    .unwrap_err();

    assert_eq!(
        error.to_string(),
        "top-level 'content_model' must equal 'block-tree'"
    );
}

#[test]
fn parse_document_object_rejects_unknown_top_level_field() {
    let error = parse_document_object(&json!({
        "type": "document",
        "version": "mycel/0.1",
        "doc_id": "doc:test",
        "title": "Origin Text",
        "language": "zh-Hant",
        "content_model": "block-tree",
        "created_at": 1u64,
        "created_by": "pk:ed25519:test",
        "genesis_revision": "rev:test",
        "unexpected": true
    }))
    .unwrap_err();

    assert_eq!(
        error.to_string(),
        "top-level contains unexpected field 'unexpected'"
    );
}

#[test]
fn parse_document_object_rejects_missing_version() {
    let error = parse_document_object(&json!({
        "type": "document",
        "doc_id": "doc:test",
        "title": "Origin Text",
        "language": "zh-Hant",
        "content_model": "block-tree",
        "created_at": 1u64,
        "created_by": "pk:ed25519:test",
        "genesis_revision": "rev:test"
    }))
    .unwrap_err();

    assert_eq!(error.to_string(), "missing string field 'version'");
}

#[test]
fn parse_document_object_rejects_wrong_version() {
    let error = parse_document_object(&json!({
        "type": "document",
        "version": "mycel/0.2",
        "doc_id": "doc:test",
        "title": "Origin Text",
        "language": "zh-Hant",
        "content_model": "block-tree",
        "created_at": 1u64,
        "created_by": "pk:ed25519:test",
        "genesis_revision": "rev:test"
    }))
    .unwrap_err();

    assert_eq!(
        error.to_string(),
        "top-level 'version' must equal 'mycel/0.1'"
    );
}

#[test]
fn parse_revision_object_reads_parent_and_patch_ids() {
    let revision = parse_revision_object(&json!({
        "type": "revision",
        "version": "mycel/0.1",
        "revision_id": "rev:test",
        "doc_id": "doc:test",
        "parents": ["rev:base"],
        "patches": ["patch:test"],
        "state_hash": "hash:test",
        "author": "pk:ed25519:test",
        "timestamp": 2u64
    }))
    .expect("revision should parse");

    assert_eq!(revision.parents, vec!["rev:base"]);
    assert_eq!(revision.patches, vec!["patch:test"]);
    assert_eq!(revision.merge_strategy, None);
}

#[test]
fn parse_revision_object_reads_merge_strategy_for_multi_parent_revision() {
    let revision = parse_revision_object(&json!({
        "type": "revision",
        "version": "mycel/0.1",
        "revision_id": "rev:test",
        "doc_id": "doc:test",
        "parents": ["rev:base", "rev:side"],
        "patches": ["patch:test"],
        "merge_strategy": "semantic-block-merge",
        "state_hash": "hash:test",
        "author": "pk:ed25519:test",
        "timestamp": 2u64
    }))
    .expect("merge revision should parse");

    assert_eq!(revision.parents, vec!["rev:base", "rev:side"]);
    assert_eq!(
        revision.merge_strategy.as_deref(),
        Some("semantic-block-merge")
    );
}

#[test]
fn parse_revision_object_rejects_wrong_parent_prefix() {
    let error = parse_revision_object(&json!({
        "type": "revision",
        "version": "mycel/0.1",
        "revision_id": "rev:test",
        "doc_id": "doc:test",
        "parents": ["patch:base"],
        "patches": ["patch:test"],
        "state_hash": "hash:test",
        "author": "pk:ed25519:test",
        "timestamp": 2u64
    }))
    .unwrap_err();

    assert_eq!(
        error.to_string(),
        "top-level 'parents[0]' must use 'rev:' prefix"
    );
}

#[test]
fn parse_revision_object_rejects_wrong_state_hash_prefix() {
    let error = parse_revision_object(&json!({
        "type": "revision",
        "version": "mycel/0.1",
        "revision_id": "rev:test",
        "doc_id": "doc:test",
        "parents": ["rev:base"],
        "patches": ["patch:test"],
        "state_hash": "rev:test",
        "author": "pk:ed25519:test",
        "timestamp": 2u64
    }))
    .unwrap_err();

    assert_eq!(
        error.to_string(),
        "top-level 'state_hash' must use 'hash:' prefix"
    );
}

#[test]
fn parse_revision_object_rejects_wrong_author_prefix() {
    let error = parse_revision_object(&json!({
        "type": "revision",
        "version": "mycel/0.1",
        "revision_id": "rev:test",
        "doc_id": "doc:test",
        "parents": ["rev:base"],
        "patches": ["patch:test"],
        "state_hash": "hash:test",
        "author": "author:test",
        "timestamp": 2u64
    }))
    .unwrap_err();

    assert_eq!(
        error.to_string(),
        "top-level 'author' must use 'pk:' prefix"
    );
}

#[test]
fn parse_revision_object_rejects_duplicate_parent_ids() {
    let error = parse_revision_object(&json!({
        "type": "revision",
        "version": "mycel/0.1",
        "revision_id": "rev:test",
        "doc_id": "doc:test",
        "parents": ["rev:base", "rev:base"],
        "patches": ["patch:test"],
        "state_hash": "hash:test",
        "author": "pk:ed25519:test",
        "timestamp": 2u64
    }))
    .unwrap_err();

    assert_eq!(
        error.to_string(),
        "top-level 'parents[1]' duplicates 'parents[0]'"
    );
}

#[test]
fn parse_revision_object_rejects_duplicate_patch_ids() {
    let error = parse_revision_object(&json!({
        "type": "revision",
        "version": "mycel/0.1",
        "revision_id": "rev:test",
        "doc_id": "doc:test",
        "parents": ["rev:base"],
        "patches": ["patch:test", "patch:test"],
        "state_hash": "hash:test",
        "author": "pk:ed25519:test",
        "timestamp": 2u64
    }))
    .unwrap_err();

    assert_eq!(
        error.to_string(),
        "top-level 'patches[1]' duplicates 'patches[0]'"
    );
}

#[test]
fn parse_revision_object_rejects_merge_strategy_on_genesis_revision() {
    let error = parse_revision_object(&json!({
        "type": "revision",
        "version": "mycel/0.1",
        "revision_id": "rev:test",
        "doc_id": "doc:test",
        "parents": [],
        "patches": ["patch:test"],
        "merge_strategy": "semantic-block-merge",
        "state_hash": "hash:test",
        "author": "pk:ed25519:test",
        "timestamp": 2u64
    }))
    .unwrap_err();

    assert_eq!(
        error.to_string(),
        "top-level 'merge_strategy' is not allowed when 'parents' is empty"
    );
}

#[test]
fn parse_revision_object_rejects_merge_strategy_on_single_parent_revision() {
    let error = parse_revision_object(&json!({
        "type": "revision",
        "version": "mycel/0.1",
        "revision_id": "rev:test",
        "doc_id": "doc:test",
        "parents": ["rev:base"],
        "patches": ["patch:test"],
        "merge_strategy": "semantic-block-merge",
        "state_hash": "hash:test",
        "author": "pk:ed25519:test",
        "timestamp": 2u64
    }))
    .unwrap_err();

    assert_eq!(
        error.to_string(),
        "top-level 'merge_strategy' requires multiple parents"
    );
}

#[test]
fn parse_revision_object_rejects_multi_parent_revision_without_merge_strategy() {
    let error = parse_revision_object(&json!({
        "type": "revision",
        "version": "mycel/0.1",
        "revision_id": "rev:test",
        "doc_id": "doc:test",
        "parents": ["rev:base", "rev:side"],
        "patches": ["patch:test"],
        "state_hash": "hash:test",
        "author": "pk:ed25519:test",
        "timestamp": 2u64
    }))
    .unwrap_err();

    assert_eq!(
        error.to_string(),
        "top-level 'merge_strategy' is required when 'parents' has multiple entries"
    );
}

#[test]
fn parse_block_object_requires_attrs_and_children() {
    let error = parse_block_object(&json!({
        "block_id": "blk:001",
        "block_type": "paragraph",
        "content": "Hello"
    }))
    .unwrap_err();

    assert_eq!(error.to_string(), "missing object field 'attrs'");
}

#[test]
fn parse_block_object_rejects_unknown_block_type() {
    let error = parse_block_object(&json!({
        "block_id": "blk:001",
        "block_type": "table",
        "content": "Hello",
        "attrs": {},
        "children": []
    }))
    .unwrap_err();

    assert_eq!(
        error.to_string(),
        "top-level 'block_type' must be one of: title, heading, paragraph, quote, verse, list, annotation, metadata"
    );
}

#[test]
fn parse_block_object_rejects_unknown_top_level_field() {
    let error = parse_block_object(&json!({
        "block_id": "blk:001",
        "block_type": "paragraph",
        "content": "Hello",
        "attrs": {},
        "children": [],
        "unexpected": true
    }))
    .unwrap_err();

    assert_eq!(
        error.to_string(),
        "top-level contains unexpected field 'unexpected'"
    );
}

#[test]
fn parse_block_object_rejects_unknown_nested_child_field_with_path() {
    let error = parse_block_object(&json!({
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
    .unwrap_err();

    assert_eq!(
        error.to_string(),
        "top-level 'children[0]' contains unexpected field 'unexpected'"
    );
}

#[test]
fn parse_block_object_rejects_non_object_nested_child_with_path() {
    let error = parse_block_object(&json!({
        "block_id": "blk:001",
        "block_type": "paragraph",
        "content": "Hello",
        "attrs": {},
        "children": ["not-an-object"]
    }))
    .unwrap_err();

    assert_eq!(
        error.to_string(),
        "top-level 'children[0]' must be a JSON object"
    );
}

#[test]
fn parse_block_object_rejects_nested_child_missing_attrs_with_path() {
    let error = parse_block_object(&json!({
        "block_id": "blk:001",
        "block_type": "paragraph",
        "content": "Hello",
        "attrs": {},
        "children": [
            {
                "block_id": "blk:002",
                "block_type": "paragraph",
                "content": "Child",
                "children": []
            }
        ]
    }))
    .unwrap_err();

    assert_eq!(
        error.to_string(),
        "top-level 'children[0]': missing object field 'attrs'"
    );
}

#[test]
fn parse_view_object_reads_documents_and_policy() {
    let view = parse_view_object(&json!({
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
        "timestamp": 7u64
    }))
    .expect("view should parse");

    assert_eq!(view.view_id, "view:test");
    assert_eq!(
        view.documents.get("doc:test").map(String::as_str),
        Some("rev:test")
    );
    assert!(view.policy.is_object());
}

#[test]
fn parse_view_object_rejects_empty_documents() {
    let error = parse_view_object(&json!({
        "type": "view",
        "version": "mycel/0.1",
        "view_id": "view:test",
        "maintainer": "pk:ed25519:test",
        "documents": {},
        "policy": {
            "merge_rule": "manual-reviewed"
        },
        "timestamp": 7u64
    }))
    .unwrap_err();

    assert_eq!(error.to_string(), "top-level 'documents' must not be empty");
}

#[test]
fn parse_view_object_rejects_non_object_policy() {
    let error = parse_view_object(&json!({
        "type": "view",
        "version": "mycel/0.1",
        "view_id": "view:test",
        "maintainer": "pk:ed25519:test",
        "documents": {
            "doc:test": "rev:test"
        },
        "policy": "manual-reviewed",
        "timestamp": 7u64
    }))
    .unwrap_err();

    assert_eq!(error.to_string(), "top-level 'policy' must be an object");
}

#[test]
fn parse_view_object_rejects_missing_policy() {
    let error = parse_view_object(&json!({
        "type": "view",
        "version": "mycel/0.1",
        "view_id": "view:test",
        "maintainer": "pk:ed25519:test",
        "documents": {
            "doc:test": "rev:test"
        },
        "timestamp": 7u64
    }))
    .unwrap_err();

    assert_eq!(error.to_string(), "missing object field 'policy'");
}

#[test]
fn parse_view_object_rejects_non_string_document_value() {
    let error = parse_view_object(&json!({
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
        "timestamp": 7u64
    }))
    .unwrap_err();

    assert_eq!(
        error.to_string(),
        "top-level 'documents.doc:test' must be a string"
    );
}

#[test]
fn parse_view_object_rejects_wrong_document_value_prefix() {
    let error = parse_view_object(&json!({
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
        "timestamp": 7u64
    }))
    .unwrap_err();

    assert_eq!(
        error.to_string(),
        "top-level 'documents.doc:test' must use 'rev:' prefix"
    );
}

#[test]
fn parse_view_object_rejects_wrong_maintainer_prefix() {
    let error = parse_view_object(&json!({
        "type": "view",
        "version": "mycel/0.1",
        "view_id": "view:test",
        "maintainer": "sig:test",
        "documents": {
            "doc:test": "rev:test"
        },
        "policy": {
            "merge_rule": "manual-reviewed"
        },
        "timestamp": 7u64
    }))
    .unwrap_err();

    assert_eq!(
        error.to_string(),
        "top-level 'maintainer' must use 'pk:' prefix"
    );
}

#[test]
fn parse_view_object_rejects_wrong_document_key_prefix() {
    let error = parse_view_object(&json!({
        "type": "view",
        "version": "mycel/0.1",
        "view_id": "view:test",
        "maintainer": "pk:ed25519:test",
        "documents": {
            "patch:test": "rev:test"
        },
        "policy": {
            "merge_rule": "manual-reviewed"
        },
        "timestamp": 7u64
    }))
    .unwrap_err();

    assert_eq!(
        error.to_string(),
        "top-level 'documents.patch:test key' must use 'doc:' prefix"
    );
}

#[test]
fn parse_view_object_rejects_unknown_top_level_field() {
    let error = parse_view_object(&json!({
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
        "timestamp": 7u64,
        "unexpected": true
    }))
    .unwrap_err();

    assert_eq!(
        error.to_string(),
        "top-level contains unexpected field 'unexpected'"
    );
}

#[test]
fn parse_snapshot_object_reads_documents_and_included_objects() {
    let snapshot = parse_snapshot_object(&json!({
        "type": "snapshot",
        "version": "mycel/0.1",
        "snapshot_id": "snap:test",
        "documents": {
            "doc:test": "rev:test"
        },
        "included_objects": ["rev:test", "patch:test"],
        "root_hash": "hash:test",
        "created_by": "pk:ed25519:test",
        "timestamp": 9u64
    }))
    .expect("snapshot should parse");

    assert_eq!(snapshot.snapshot_id, "snap:test");
    assert_eq!(snapshot.included_objects, vec!["rev:test", "patch:test"]);
    assert_eq!(
        snapshot.documents.get("doc:test").map(String::as_str),
        Some("rev:test")
    );
}

#[test]
fn parse_snapshot_object_rejects_empty_documents() {
    let error = parse_snapshot_object(&json!({
        "type": "snapshot",
        "version": "mycel/0.1",
        "snapshot_id": "snap:test",
        "documents": {},
        "included_objects": ["rev:test", "patch:test"],
        "root_hash": "hash:test",
        "created_by": "pk:ed25519:test",
        "timestamp": 9u64
    }))
    .unwrap_err();

    assert_eq!(error.to_string(), "top-level 'documents' must not be empty");
}

#[test]
fn parse_snapshot_object_rejects_non_string_document_value() {
    let error = parse_snapshot_object(&json!({
        "type": "snapshot",
        "version": "mycel/0.1",
        "snapshot_id": "snap:test",
        "documents": {
            "doc:test": 9
        },
        "included_objects": ["rev:test"],
        "root_hash": "hash:test",
        "created_by": "pk:ed25519:test",
        "timestamp": 9u64
    }))
    .unwrap_err();

    assert_eq!(
        error.to_string(),
        "top-level 'documents.doc:test' must be a string"
    );
}

#[test]
fn parse_snapshot_object_rejects_empty_included_object_entry() {
    let error = parse_snapshot_object(&json!({
        "type": "snapshot",
        "version": "mycel/0.1",
        "snapshot_id": "snap:test",
        "documents": {
            "doc:test": "rev:test"
        },
        "included_objects": ["rev:test", ""],
        "root_hash": "hash:test",
        "created_by": "pk:ed25519:test",
        "timestamp": 9u64
    }))
    .unwrap_err();

    assert_eq!(
        error.to_string(),
        "top-level 'included_objects[1]' must not be an empty string"
    );
}

#[test]
fn parse_snapshot_object_rejects_non_string_included_object_entry() {
    let error = parse_snapshot_object(&json!({
        "type": "snapshot",
        "version": "mycel/0.1",
        "snapshot_id": "snap:test",
        "documents": {
            "doc:test": "rev:test"
        },
        "included_objects": ["rev:test", 7],
        "root_hash": "hash:test",
        "created_by": "pk:ed25519:test",
        "timestamp": 9u64
    }))
    .unwrap_err();

    assert_eq!(
        error.to_string(),
        "top-level 'included_objects[1]' must be a string"
    );
}

#[test]
fn parse_snapshot_object_rejects_non_canonical_included_object_id() {
    let error = parse_snapshot_object(&json!({
        "type": "snapshot",
        "version": "mycel/0.1",
        "snapshot_id": "snap:test",
        "documents": {
            "doc:test": "rev:test"
        },
        "included_objects": ["doc:test"],
        "root_hash": "hash:test",
        "created_by": "pk:ed25519:test",
        "timestamp": 9u64
    }))
    .unwrap_err();

    assert_eq!(
        error.to_string(),
        "top-level 'included_objects[0]' must use a canonical object ID prefix"
    );
}

#[test]
fn parse_snapshot_object_rejects_duplicate_included_object_ids() {
    let error = parse_snapshot_object(&json!({
        "type": "snapshot",
        "version": "mycel/0.1",
        "snapshot_id": "snap:test",
        "documents": {
            "doc:test": "rev:test"
        },
        "included_objects": ["rev:test", "rev:test"],
        "root_hash": "hash:test",
        "created_by": "pk:ed25519:test",
        "timestamp": 9u64
    }))
    .unwrap_err();

    assert_eq!(
        error.to_string(),
        "top-level 'included_objects[1]' duplicates 'included_objects[0]'"
    );
}

#[test]
fn parse_snapshot_object_requires_declared_document_revision_in_included_objects() {
    let error = parse_snapshot_object(&json!({
        "type": "snapshot",
        "version": "mycel/0.1",
        "snapshot_id": "snap:test",
        "documents": {
            "doc:test": "rev:test"
        },
        "included_objects": ["patch:test"],
        "root_hash": "hash:test",
        "created_by": "pk:ed25519:test",
        "timestamp": 9u64
    }))
    .unwrap_err();

    assert_eq!(
        error.to_string(),
        "top-level 'included_objects' must include revision 'rev:test' declared by 'documents.doc:test'"
    );
}

#[test]
fn parse_snapshot_object_rejects_wrong_document_value_prefix() {
    let error = parse_snapshot_object(&json!({
        "type": "snapshot",
        "version": "mycel/0.1",
        "snapshot_id": "snap:test",
        "documents": {
            "doc:test": "patch:test"
        },
        "included_objects": ["patch:test"],
        "root_hash": "hash:test",
        "created_by": "pk:ed25519:test",
        "timestamp": 9u64
    }))
    .unwrap_err();

    assert_eq!(
        error.to_string(),
        "top-level 'documents.doc:test' must use 'rev:' prefix"
    );
}

#[test]
fn parse_snapshot_object_rejects_wrong_document_key_prefix() {
    let error = parse_snapshot_object(&json!({
        "type": "snapshot",
        "version": "mycel/0.1",
        "snapshot_id": "snap:test",
        "documents": {
            "patch:test": "rev:test"
        },
        "included_objects": ["rev:test", "patch:test"],
        "root_hash": "hash:test",
        "created_by": "pk:ed25519:test",
        "timestamp": 9u64
    }))
    .unwrap_err();

    assert_eq!(
        error.to_string(),
        "top-level 'documents.patch:test key' must use 'doc:' prefix"
    );
}

#[test]
fn parse_snapshot_object_rejects_wrong_root_hash_prefix() {
    let error = parse_snapshot_object(&json!({
        "type": "snapshot",
        "version": "mycel/0.1",
        "snapshot_id": "snap:test",
        "documents": {
            "doc:test": "rev:test"
        },
        "included_objects": ["rev:test", "patch:test"],
        "root_hash": "rev:test",
        "created_by": "pk:ed25519:test",
        "timestamp": 9u64
    }))
    .unwrap_err();

    assert_eq!(
        error.to_string(),
        "top-level 'root_hash' must use 'hash:' prefix"
    );
}

#[test]
fn parse_revision_object_rejects_non_string_merge_strategy() {
    let error = parse_revision_object(&json!({
        "type": "revision",
        "version": "mycel/0.1",
        "revision_id": "rev:test",
        "doc_id": "doc:test",
        "parents": ["rev:base", "rev:side"],
        "patches": ["patch:test"],
        "merge_strategy": 7,
        "state_hash": "hash:test",
        "author": "pk:ed25519:test",
        "timestamp": 2u64
    }))
    .unwrap_err();

    assert_eq!(
        error.to_string(),
        "top-level 'merge_strategy' must be a string"
    );
}

#[test]
fn parse_snapshot_object_rejects_wrong_created_by_prefix() {
    let error = parse_snapshot_object(&json!({
        "type": "snapshot",
        "version": "mycel/0.1",
        "snapshot_id": "snap:test",
        "documents": {
            "doc:test": "rev:test"
        },
        "included_objects": ["rev:test", "patch:test"],
        "root_hash": "hash:test",
        "created_by": "sig:test",
        "timestamp": 9u64
    }))
    .unwrap_err();

    assert_eq!(
        error.to_string(),
        "top-level 'created_by' must use 'pk:' prefix"
    );
}

#[test]
fn parse_snapshot_object_rejects_unknown_top_level_field() {
    let error = parse_snapshot_object(&json!({
        "type": "snapshot",
        "version": "mycel/0.1",
        "snapshot_id": "snap:test",
        "documents": {
            "doc:test": "rev:test"
        },
        "included_objects": ["rev:test", "patch:test"],
        "root_hash": "hash:test",
        "created_by": "pk:ed25519:test",
        "timestamp": 9u64,
        "unexpected": true
    }))
    .unwrap_err();

    assert_eq!(
        error.to_string(),
        "top-level contains unexpected field 'unexpected'"
    );
}
