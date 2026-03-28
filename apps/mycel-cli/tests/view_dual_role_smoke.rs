use std::fs;
use std::path::{Path, PathBuf};

use ed25519_dalek::SigningKey;
use mycel_core::author::signer_id;
use serde_json::{json, Value};

mod common;

use common::{
    assert_exit_code, assert_success, create_temp_dir, parse_json_stdout, recompute_test_object_id,
    run_mycel, sign_test_value,
};

fn path_arg(path: &Path) -> String {
    path.to_string_lossy().into_owned()
}

fn signing_key(seed: u8) -> SigningKey {
    SigningKey::from_bytes(&[seed; 32])
}

fn signed_view(
    signing_key: &SigningKey,
    policy: &Value,
    documents: Value,
    timestamp: u64,
) -> Value {
    let mut value = json!({
        "type": "view",
        "version": "mycel/0.1",
        "maintainer": signer_id(signing_key),
        "documents": documents,
        "policy": policy,
        "timestamp": timestamp
    });
    let id = recompute_test_object_id(&value, "view_id", "view");
    value["view_id"] = Value::String(id);
    value["signature"] = Value::String(sign_test_value(signing_key, &value));
    value
}

fn documents_value(doc_id: &str, revision_id: &str) -> Value {
    json!({
        doc_id: revision_id
    })
}

fn write_json_file(prefix: &str, name: &str, value: &Value) -> (common::TempDir, PathBuf) {
    let dir = create_temp_dir(prefix);
    let path = dir.path().join(name);
    fs::write(
        &path,
        serde_json::to_string_pretty(value).expect("value should serialize"),
    )
    .expect("value should write");
    (dir, path)
}

fn publish_view(source_path: &Path, store_root: &str) -> Value {
    let output = run_mycel(&[
        "view",
        "publish",
        &path_arg(source_path),
        "--into",
        store_root,
        "--json",
    ]);
    assert_success(&output);
    parse_json_stdout(&output)
}

#[test]
fn persisted_governance_keeps_editor_and_view_roles_independent() {
    let store_dir = create_temp_dir("view-dual-role-store");
    let store_root = path_arg(store_dir.path());
    let init = run_mycel(&["store", "init", &store_root, "--json"]);
    assert_success(&init);

    let shared_dual_role = signing_key(151);
    let maintainer_only = signing_key(152);
    let editor_only = signing_key(153);
    let shared_policy = json!({
        "accept_keys": [signer_id(&shared_dual_role), signer_id(&editor_only)],
        "merge_rule": "manual-reviewed",
        "preferred_branches": ["main"]
    });
    let mixed_policy = json!({
        "accept_keys": [signer_id(&editor_only)],
        "merge_rule": "manual-reviewed",
        "preferred_branches": ["preview"]
    });

    let shared_view = signed_view(
        &shared_dual_role,
        &shared_policy,
        documents_value(
            "doc:shared-dual-role",
            "rev:1111111111111111111111111111111111111111111111111111111111111111",
        ),
        10,
    );
    let mixed_view = signed_view(
        &maintainer_only,
        &mixed_policy,
        documents_value(
            "doc:mixed-role",
            "rev:2222222222222222222222222222222222222222222222222222222222222222",
        ),
        20,
    );

    let (_shared_dir, shared_path) =
        write_json_file("view-shared-dual-role", "shared.json", &shared_view);
    let (_mixed_dir, mixed_path) = write_json_file("view-mixed-role", "mixed.json", &mixed_view);

    let shared_publish = publish_view(&shared_path, &store_root);
    let mixed_publish = publish_view(&mixed_path, &store_root);

    let shared_current = run_mycel(&[
        "view",
        "current",
        "--store-root",
        &store_root,
        "--profile-id",
        shared_publish["profile_id"]
            .as_str()
            .expect("shared profile id should exist"),
        "--doc-id",
        "doc:shared-dual-role",
        "--json",
    ]);
    assert_success(&shared_current);
    let shared_current_json = parse_json_stdout(&shared_current);
    assert_eq!(
        shared_current_json["maintainer"],
        Value::String(signer_id(&shared_dual_role))
    );

    let mixed_current = run_mycel(&[
        "view",
        "current",
        "--store-root",
        &store_root,
        "--profile-id",
        mixed_publish["profile_id"]
            .as_str()
            .expect("mixed profile id should exist"),
        "--doc-id",
        "doc:mixed-role",
        "--json",
    ]);
    assert_success(&mixed_current);
    let mixed_current_json = parse_json_stdout(&mixed_current);
    assert_eq!(
        mixed_current_json["maintainer"],
        Value::String(signer_id(&maintainer_only))
    );

    let shared_maintainer = run_mycel(&[
        "view",
        "maintainer",
        "--store-root",
        &store_root,
        "--maintainer",
        &signer_id(&shared_dual_role),
        "--profile-id",
        shared_publish["profile_id"]
            .as_str()
            .expect("shared profile id should exist"),
        "--doc-id",
        "doc:shared-dual-role",
        "--json",
    ]);
    assert_success(&shared_maintainer);
    let shared_maintainer_json = parse_json_stdout(&shared_maintainer);
    assert_eq!(
        shared_maintainer_json["source"],
        Value::String("persisted".to_string())
    );
    assert_eq!(
        shared_maintainer_json["current_profiles"][0]["profile_id"],
        shared_publish["profile_id"]
    );
    assert_eq!(
        shared_maintainer_json["current_documents"][0]["profiles"][0]["maintainer"],
        Value::String(signer_id(&shared_dual_role))
    );

    let maintainer_only_output = run_mycel(&[
        "view",
        "maintainer",
        "--store-root",
        &store_root,
        "--maintainer",
        &signer_id(&maintainer_only),
        "--profile-id",
        mixed_publish["profile_id"]
            .as_str()
            .expect("mixed profile id should exist"),
        "--doc-id",
        "doc:mixed-role",
        "--json",
    ]);
    assert_success(&maintainer_only_output);
    let maintainer_only_json = parse_json_stdout(&maintainer_only_output);
    assert_eq!(
        maintainer_only_json["current_documents"][0]["profiles"][0]["maintainer"],
        Value::String(signer_id(&maintainer_only))
    );

    let editor_only_output = run_mycel(&[
        "view",
        "maintainer",
        "--store-root",
        &store_root,
        "--maintainer",
        &signer_id(&editor_only),
        "--json",
    ]);
    assert_exit_code(&editor_only_output, 1);
    let editor_only_json = parse_json_stdout(&editor_only_output);
    assert_eq!(
        editor_only_json["status"],
        Value::String("failed".to_string())
    );
    assert!(
        editor_only_json["errors"]
            .as_array()
            .is_some_and(|errors| errors.iter().any(|error| {
                error.as_str().is_some_and(|message| {
                    message
                        .contains("was not found in persisted current maintainer governance state")
                })
            })),
        "expected persisted maintainer-governance miss in JSON error list: {editor_only_json}",
    );

    let store_index = run_mycel(&["store", "index", &store_root, "--governance-only", "--json"]);
    assert_success(&store_index);
    let store_index_json = parse_json_stdout(&store_index);
    let shared_profile_id = shared_publish["profile_id"]
        .as_str()
        .expect("shared profile id should exist");
    let mixed_profile_id = mixed_publish["profile_id"]
        .as_str()
        .expect("mixed profile id should exist");
    let editor_only_id = signer_id(&editor_only);

    assert_eq!(
        store_index_json["current_governance"][shared_profile_id]["maintainer"],
        Value::String(signer_id(&shared_dual_role))
    );
    assert_eq!(
        store_index_json["current_governance"][mixed_profile_id]["maintainer"],
        Value::String(signer_id(&maintainer_only))
    );
    assert!(
        store_index_json["current_maintainer_governance"]
            .get(&editor_only_id)
            .is_none(),
        "editor-only admitted key must not appear as current maintainer governance: {store_index_json}",
    );
    assert!(
        store_index_json["current_maintainer_governance"]
            .get(signer_id(&shared_dual_role))
            .is_some(),
        "shared dual-role maintainer should remain queryable in current maintainer governance",
    );
    assert!(
        store_index_json["current_maintainer_governance"]
            .get(signer_id(&maintainer_only))
            .is_some(),
        "maintainer-only key should remain queryable in current maintainer governance",
    );
}
