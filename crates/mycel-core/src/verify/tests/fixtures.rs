use super::*;

pub(super) fn write_test_file(name: &str, content: &str) -> std::path::PathBuf {
    let unique = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .expect("system time should be after unix epoch")
        .as_nanos();
    let path = std::env::temp_dir().join(format!("mycel-core-{name}-{unique}.json"));
    std::fs::write(&path, content).expect("test JSON should be written");
    path
}

pub(super) fn write_test_dir(name: &str) -> std::path::PathBuf {
    let unique = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .expect("system time should be after unix epoch")
        .as_nanos();
    let path = std::env::temp_dir().join(format!("mycel-core-{name}-{unique}"));
    std::fs::create_dir_all(&path).expect("test directory should be created");
    path
}

pub(super) fn signer_material() -> (SigningKey, String) {
    let signing_key = SigningKey::from_bytes(&[7u8; 32]);
    let public_key = format!(
        "pk:ed25519:{}",
        base64::engine::general_purpose::STANDARD.encode(signing_key.verifying_key().as_bytes())
    );
    (signing_key, public_key)
}

pub(super) fn sign_value(signing_key: &SigningKey, value: &Value) -> String {
    let payload = signed_payload_bytes(value).expect("payload should canonicalize");
    let signature = signing_key.sign(&payload);
    format!(
        "sig:ed25519:{}",
        base64::engine::general_purpose::STANDARD.encode(signature.to_bytes())
    )
}

pub(super) fn state_hash_for_blocks(doc_id: &str, blocks: Vec<BlockObject>) -> String {
    compute_state_hash(&DocumentState {
        doc_id: doc_id.to_string(),
        blocks,
        metadata: Map::new(),
    })
    .expect("state hash should compute")
}

pub(super) fn verify_strict_id_case_summary(
    kind: &str,
    id_field: &str,
    id_value: Value,
) -> ObjectVerificationSummary {
    let (signing_key, public_key) = signer_material();
    let mut value = match kind {
        "document" => json!({
            "type": "document",
            "version": "mycel/0.1",
            "doc_id": "doc:test",
            "title": "Plain document",
            "language": "zh-Hant",
            "content_model": "block-tree",
            "created_at": 1u64,
            "created_by": "pk:ed25519:test",
            "genesis_revision": "rev:test"
        }),
        "block" => json!({
            "type": "block",
            "version": "mycel/0.1",
            "block_id": "blk:001",
            "block_type": "paragraph",
            "content": "Hello",
            "attrs": {},
            "children": []
        }),
        "patch" => json!({
            "type": "patch",
            "version": "mycel/0.1",
            "doc_id": "doc:test",
            "base_revision": "rev:genesis-null",
            "author": public_key,
            "timestamp": 11u64,
            "ops": []
        }),
        "revision" => json!({
            "type": "revision",
            "version": "mycel/0.1",
            "doc_id": "doc:test",
            "parents": ["rev:base"],
            "patches": [],
            "state_hash": "hash:test",
            "author": public_key,
            "timestamp": 11u64
        }),
        "view" => json!({
            "type": "view",
            "version": "mycel/0.1",
            "maintainer": public_key,
            "documents": {
                "doc:test": "rev:test"
            },
            "policy": {
                "merge_rule": "manual-reviewed"
            },
            "timestamp": 12u64
        }),
        "snapshot" => json!({
            "type": "snapshot",
            "version": "mycel/0.1",
            "documents": {
                "doc:test": "rev:test"
            },
            "included_objects": ["rev:test", "patch:test"],
            "root_hash": "hash:test",
            "created_by": public_key,
            "timestamp": 9u64
        }),
        _ => panic!("unknown strict ID verify case: {kind}"),
    };

    match kind {
        "patch" => {
            let patch_id = recompute_object_id(&value, "patch_id", "patch")
                .expect("patch ID should recompute");
            value["patch_id"] = Value::String(patch_id);
            value[id_field] = id_value;
            value["signature"] = Value::String(sign_value(&signing_key, &value));
        }
        "revision" => {
            let revision_id = recompute_object_id(&value, "revision_id", "rev")
                .expect("revision ID should recompute");
            value["revision_id"] = Value::String(revision_id);
            value[id_field] = id_value;
            value["signature"] = Value::String(sign_value(&signing_key, &value));
        }
        "view" => {
            let view_id =
                recompute_object_id(&value, "view_id", "view").expect("view ID should recompute");
            value["view_id"] = Value::String(view_id);
            value[id_field] = id_value;
            value["signature"] = Value::String(sign_value(&signing_key, &value));
        }
        "snapshot" => {
            let snapshot_id = recompute_object_id(&value, "snapshot_id", "snap")
                .expect("snapshot ID should recompute");
            value["snapshot_id"] = Value::String(snapshot_id);
            value[id_field] = id_value;
            value["signature"] = Value::String(sign_value(&signing_key, &value));
        }
        _ => {
            value[id_field] = id_value;
        }
    }

    let path = write_test_file(
        &format!("{kind}-{id_field}-strictness"),
        &serde_json::to_string_pretty(&value).expect("test JSON should serialize"),
    );

    let summary = verify_object_path(&path);
    let _ = std::fs::remove_file(path);
    summary
}

pub(super) fn inspect_strict_id_case_summary(
    kind: &str,
    id_field: &str,
    id_value: Value,
) -> ObjectInspectionSummary {
    let (signing_key, public_key) = signer_material();
    let mut value = match kind {
        "document" => json!({
            "type": "document",
            "version": "mycel/0.1",
            "doc_id": "doc:test",
            "title": "Plain document",
            "language": "zh-Hant",
            "content_model": "block-tree",
            "created_at": 1u64,
            "created_by": "pk:ed25519:test",
            "genesis_revision": "rev:test"
        }),
        "block" => json!({
            "type": "block",
            "version": "mycel/0.1",
            "block_id": "blk:001",
            "block_type": "paragraph",
            "content": "Hello",
            "attrs": {},
            "children": []
        }),
        "patch" => json!({
            "type": "patch",
            "version": "mycel/0.1",
            "doc_id": "doc:test",
            "base_revision": "rev:genesis-null",
            "author": public_key,
            "timestamp": 11u64,
            "ops": []
        }),
        "revision" => json!({
            "type": "revision",
            "version": "mycel/0.1",
            "doc_id": "doc:test",
            "parents": ["rev:base"],
            "patches": [],
            "state_hash": "hash:test",
            "author": public_key,
            "timestamp": 11u64
        }),
        "view" => json!({
            "type": "view",
            "version": "mycel/0.1",
            "maintainer": public_key,
            "documents": {
                "doc:test": "rev:test"
            },
            "policy": {
                "merge_rule": "manual-reviewed"
            },
            "timestamp": 12u64
        }),
        "snapshot" => json!({
            "type": "snapshot",
            "version": "mycel/0.1",
            "documents": {
                "doc:test": "rev:test"
            },
            "included_objects": ["rev:test", "patch:test"],
            "root_hash": "hash:test",
            "created_by": public_key,
            "timestamp": 9u64
        }),
        _ => panic!("unknown strict ID inspect case: {kind}"),
    };

    match kind {
        "patch" => {
            let patch_id = recompute_object_id(&value, "patch_id", "patch")
                .expect("patch ID should recompute");
            value["patch_id"] = Value::String(patch_id);
            value[id_field] = id_value;
            value["signature"] = Value::String(sign_value(&signing_key, &value));
        }
        "revision" => {
            let revision_id = recompute_object_id(&value, "revision_id", "rev")
                .expect("revision ID should recompute");
            value["revision_id"] = Value::String(revision_id);
            value[id_field] = id_value;
            value["signature"] = Value::String(sign_value(&signing_key, &value));
        }
        "view" => {
            let view_id =
                recompute_object_id(&value, "view_id", "view").expect("view ID should recompute");
            value["view_id"] = Value::String(view_id);
            value[id_field] = id_value;
            value["signature"] = Value::String(sign_value(&signing_key, &value));
        }
        "snapshot" => {
            let snapshot_id = recompute_object_id(&value, "snapshot_id", "snap")
                .expect("snapshot ID should recompute");
            value["snapshot_id"] = Value::String(snapshot_id);
            value[id_field] = id_value;
            value["signature"] = Value::String(sign_value(&signing_key, &value));
        }
        _ => {
            value[id_field] = id_value;
        }
    }

    let path = write_test_file(
        &format!("{kind}-{id_field}-inspect-strictness"),
        &serde_json::to_string_pretty(&value).expect("test JSON should serialize"),
    );

    let summary = inspect_object_path(&path);
    let _ = std::fs::remove_file(path);
    summary
}

pub(super) fn verify_core_version_case_summary(
    kind: &str,
    version_value: Option<Value>,
) -> ObjectVerificationSummary {
    let (signing_key, public_key) = signer_material();
    let mut value = match kind {
        "document" => json!({
            "type": "document",
            "version": "mycel/0.1",
            "doc_id": "doc:test",
            "title": "Plain document",
            "language": "zh-Hant",
            "content_model": "block-tree",
            "created_at": 1u64,
            "created_by": "pk:ed25519:test",
            "genesis_revision": "rev:test"
        }),
        "patch" => json!({
            "type": "patch",
            "version": "mycel/0.1",
            "doc_id": "doc:test",
            "base_revision": "rev:genesis-null",
            "author": public_key,
            "timestamp": 11u64,
            "ops": []
        }),
        "revision" => json!({
            "type": "revision",
            "version": "mycel/0.1",
            "doc_id": "doc:test",
            "parents": ["rev:base"],
            "patches": [],
            "state_hash": "hash:test",
            "author": public_key,
            "timestamp": 11u64
        }),
        "view" => json!({
            "type": "view",
            "version": "mycel/0.1",
            "maintainer": public_key,
            "documents": {
                "doc:test": "rev:test"
            },
            "policy": {
                "merge_rule": "manual-reviewed"
            },
            "timestamp": 12u64
        }),
        "snapshot" => json!({
            "type": "snapshot",
            "version": "mycel/0.1",
            "documents": {
                "doc:test": "rev:test"
            },
            "included_objects": ["rev:test", "patch:test"],
            "root_hash": "hash:test",
            "created_by": public_key,
            "timestamp": 9u64
        }),
        _ => panic!("unknown core-version verify case: {kind}"),
    };

    match version_value {
        Some(version) => value["version"] = version,
        None => {
            value
                .as_object_mut()
                .expect("object value")
                .remove("version");
        }
    }

    match kind {
        "patch" => {
            let patch_id = recompute_object_id(&value, "patch_id", "patch")
                .expect("patch ID should recompute");
            value["patch_id"] = Value::String(patch_id);
            value["signature"] = Value::String(sign_value(&signing_key, &value));
        }
        "revision" => {
            let revision_id = recompute_object_id(&value, "revision_id", "rev")
                .expect("revision ID should recompute");
            value["revision_id"] = Value::String(revision_id);
            value["signature"] = Value::String(sign_value(&signing_key, &value));
        }
        "view" => {
            let view_id =
                recompute_object_id(&value, "view_id", "view").expect("view ID should recompute");
            value["view_id"] = Value::String(view_id);
            value["signature"] = Value::String(sign_value(&signing_key, &value));
        }
        "snapshot" => {
            let snapshot_id = recompute_object_id(&value, "snapshot_id", "snap")
                .expect("snapshot ID should recompute");
            value["snapshot_id"] = Value::String(snapshot_id);
            value["signature"] = Value::String(sign_value(&signing_key, &value));
        }
        _ => {}
    }

    let path = write_test_file(
        &format!("{kind}-core-version-strictness"),
        &serde_json::to_string_pretty(&value).expect("test JSON should serialize"),
    );

    let summary = verify_object_path(&path);
    let _ = std::fs::remove_file(path);
    summary
}

pub(super) fn inspect_core_version_case_summary(
    kind: &str,
    version_value: Option<Value>,
) -> ObjectInspectionSummary {
    let (signing_key, public_key) = signer_material();
    let mut value = match kind {
        "document" => json!({
            "type": "document",
            "version": "mycel/0.1",
            "doc_id": "doc:test",
            "title": "Plain document",
            "language": "zh-Hant",
            "content_model": "block-tree",
            "created_at": 1u64,
            "created_by": "pk:ed25519:test",
            "genesis_revision": "rev:test"
        }),
        "patch" => json!({
            "type": "patch",
            "version": "mycel/0.1",
            "doc_id": "doc:test",
            "base_revision": "rev:genesis-null",
            "author": public_key,
            "timestamp": 11u64,
            "ops": []
        }),
        "revision" => json!({
            "type": "revision",
            "version": "mycel/0.1",
            "doc_id": "doc:test",
            "parents": ["rev:base"],
            "patches": [],
            "state_hash": "hash:test",
            "author": public_key,
            "timestamp": 11u64
        }),
        "view" => json!({
            "type": "view",
            "version": "mycel/0.1",
            "maintainer": public_key,
            "documents": {
                "doc:test": "rev:test"
            },
            "policy": {
                "merge_rule": "manual-reviewed"
            },
            "timestamp": 12u64
        }),
        "snapshot" => json!({
            "type": "snapshot",
            "version": "mycel/0.1",
            "documents": {
                "doc:test": "rev:test"
            },
            "included_objects": ["rev:test", "patch:test"],
            "root_hash": "hash:test",
            "created_by": public_key,
            "timestamp": 9u64
        }),
        _ => panic!("unknown core-version inspect case: {kind}"),
    };

    match version_value {
        Some(version) => value["version"] = version,
        None => {
            value
                .as_object_mut()
                .expect("object value")
                .remove("version");
        }
    }

    match kind {
        "patch" => {
            let patch_id = recompute_object_id(&value, "patch_id", "patch")
                .expect("patch ID should recompute");
            value["patch_id"] = Value::String(patch_id);
            value["signature"] = Value::String(sign_value(&signing_key, &value));
        }
        "revision" => {
            let revision_id = recompute_object_id(&value, "revision_id", "rev")
                .expect("revision ID should recompute");
            value["revision_id"] = Value::String(revision_id);
            value["signature"] = Value::String(sign_value(&signing_key, &value));
        }
        "view" => {
            let view_id =
                recompute_object_id(&value, "view_id", "view").expect("view ID should recompute");
            value["view_id"] = Value::String(view_id);
            value["signature"] = Value::String(sign_value(&signing_key, &value));
        }
        "snapshot" => {
            let snapshot_id = recompute_object_id(&value, "snapshot_id", "snap")
                .expect("snapshot ID should recompute");
            value["snapshot_id"] = Value::String(snapshot_id);
            value["signature"] = Value::String(sign_value(&signing_key, &value));
        }
        _ => {}
    }

    let path = write_test_file(
        &format!("{kind}-core-version-inspect"),
        &serde_json::to_string_pretty(&value).expect("test JSON should serialize"),
    );

    let summary = inspect_object_path(&path);
    let _ = std::fs::remove_file(path);
    summary
}

pub(super) fn verify_documents_map_prefix_summary(
    kind: &str,
    documents: Value,
) -> ObjectVerificationSummary {
    let (signing_key, public_key) = signer_material();
    let mut value = match kind {
        "view" => json!({
            "type": "view",
            "version": "mycel/0.1",
            "maintainer": public_key,
            "documents": documents,
            "policy": { "merge_rule": "manual-reviewed" },
            "timestamp": 12u64
        }),
        "snapshot" => json!({
            "type": "snapshot",
            "version": "mycel/0.1",
            "documents": documents,
            "included_objects": ["rev:test", "patch:test"],
            "root_hash": "hash:test",
            "created_by": public_key,
            "timestamp": 9u64
        }),
        _ => panic!("unknown documents-map case: {kind}"),
    };

    match kind {
        "view" => {
            let view_id =
                recompute_object_id(&value, "view_id", "view").expect("view ID should recompute");
            value["view_id"] = Value::String(view_id);
        }
        "snapshot" => {
            if value["documents"]["doc:test"] == json!("patch:test") {
                value["included_objects"] = json!(["patch:test"]);
            }
            let snapshot_id = recompute_object_id(&value, "snapshot_id", "snap")
                .expect("snapshot ID should recompute");
            value["snapshot_id"] = Value::String(snapshot_id);
        }
        _ => unreachable!(),
    }

    value["signature"] = Value::String(sign_value(&signing_key, &value));
    let path = write_test_file(
        &format!("{kind}-documents-map-prefix"),
        &serde_json::to_string_pretty(&value).expect("test JSON should serialize"),
    );

    let summary = verify_object_path(&path);
    let _ = std::fs::remove_file(path);
    summary
}
