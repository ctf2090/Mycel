use std::fs;
use std::path::PathBuf;

use base64::Engine;
use ed25519_dalek::{Signer, SigningKey};
use serde_json::{json, Value};
use sha2::{Digest, Sha256};

mod common;

use common::{
    assert_exit_code, assert_stderr_contains, assert_stdout_contains, assert_success,
    create_temp_dir, parse_json_stdout, run_mycel, stdout_text,
};

fn path_arg(path: &PathBuf) -> String {
    path.to_string_lossy().into_owned()
}

fn signing_key(seed: u8) -> SigningKey {
    SigningKey::from_bytes(&[seed; 32])
}

fn signer_id(signing_key: &SigningKey) -> String {
    format!(
        "pk:ed25519:{}",
        base64::engine::general_purpose::STANDARD.encode(signing_key.verifying_key().as_bytes())
    )
}

fn canonical_json(value: &Value) -> String {
    match value {
        Value::Null => panic!("test objects should not use null"),
        Value::Bool(boolean) => boolean.to_string(),
        Value::Number(number) => number.to_string(),
        Value::String(string) => serde_json::to_string(string).expect("string should encode"),
        Value::Array(values) => format!(
            "[{}]",
            values
                .iter()
                .map(canonical_json)
                .collect::<Vec<_>>()
                .join(",")
        ),
        Value::Object(entries) => {
            let mut keys: Vec<&String> = entries.keys().collect();
            keys.sort_unstable();
            let parts = keys
                .into_iter()
                .map(|key| {
                    format!(
                        "{}:{}",
                        serde_json::to_string(key).expect("key should encode"),
                        canonical_json(&entries[key])
                    )
                })
                .collect::<Vec<_>>();
            format!("{{{}}}", parts.join(","))
        }
    }
}

fn recompute_id(value: &Value, id_field: &str, prefix: &str) -> String {
    let mut object = value
        .as_object()
        .cloned()
        .expect("test object should be JSON object");
    object.remove(id_field);
    object.remove("signature");
    let canonical = canonical_json(&Value::Object(object));
    let mut hasher = Sha256::new();
    hasher.update(canonical.as_bytes());
    format!("{prefix}:{:x}", hasher.finalize())
}

fn sign_value(signing_key: &SigningKey, value: &Value) -> String {
    let mut object = value
        .as_object()
        .cloned()
        .expect("test object should be JSON object");
    object.remove("signature");
    let canonical = canonical_json(&Value::Object(object));
    let signature = signing_key.sign(canonical.as_bytes());
    format!(
        "sig:ed25519:{}",
        base64::engine::general_purpose::STANDARD.encode(signature.to_bytes())
    )
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
    let id = recompute_id(&value, "view_id", "view");
    value["view_id"] = Value::String(id);
    value["signature"] = Value::String(sign_value(signing_key, &value));
    value
}

fn signed_patch(signing_key: &SigningKey, doc_id: &str) -> Value {
    let mut value = json!({
        "type": "patch",
        "version": "mycel/0.1",
        "doc_id": doc_id,
        "base_revision": "rev:bbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbb",
        "author": signer_id(signing_key),
        "timestamp": 1u64,
        "ops": []
    });
    let id = recompute_id(&value, "patch_id", "patch");
    value["patch_id"] = Value::String(id);
    value["signature"] = Value::String(sign_value(signing_key, &value));
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

#[test]
fn view_publish_json_writes_verified_view_into_store() {
    let store_dir = create_temp_dir("view-publish-store");
    let store_root = path_arg(&store_dir.path().to_path_buf());
    let init = run_mycel(&["store", "init", &store_root, "--json"]);
    assert_success(&init);

    let maintainer = signing_key(41);
    let policy = json!({
        "accept_keys": [signer_id(&maintainer)],
        "merge_rule": "manual-reviewed",
        "preferred_branches": ["main"]
    });
    let view = signed_view(
        &maintainer,
        &policy,
        documents_value(
            "doc:view-publish",
            "rev:aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa",
        ),
        10,
    );
    let (_source_dir, source_path) = write_json_file("view-publish-source", "view.json", &view);

    let output = run_mycel(&[
        "view",
        "publish",
        &path_arg(&source_path),
        "--into",
        &store_root,
        "--json",
    ]);

    assert_success(&output);
    let json = parse_json_stdout(&output);
    assert_eq!(json["status"], "ok");
    assert_eq!(json["view_id"], view["view_id"]);
    assert_eq!(json["maintainer"], view["maintainer"]);
    assert_eq!(
        json["documents"]["doc:view-publish"],
        "rev:aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa"
    );
    assert_eq!(json["created"], Value::Bool(true));
    assert!(
        json["profile_id"]
            .as_str()
            .is_some_and(|value| value.starts_with("hash:")),
        "expected hashed profile id, stdout: {}",
        stdout_text(&output)
    );

    let inspect = run_mycel(&[
        "view",
        "inspect",
        view["view_id"].as_str().expect("view id should exist"),
        "--store-root",
        &store_root,
        "--json",
    ]);
    assert_success(&inspect);
    let inspect_json = parse_json_stdout(&inspect);
    assert_eq!(inspect_json["status"], "ok");
    assert_eq!(inspect_json["view_id"], view["view_id"]);
    assert_eq!(
        inspect_json["profile_heads"]["doc:view-publish"],
        json!(["rev:aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa"])
    );
}

#[test]
fn view_publish_reports_existing_view_on_repeat_publish() {
    let store_dir = create_temp_dir("view-publish-repeat-store");
    let store_root = path_arg(&store_dir.path().to_path_buf());
    let init = run_mycel(&["store", "init", &store_root, "--json"]);
    assert_success(&init);

    let maintainer = signing_key(42);
    let policy = json!({
        "accept_keys": [signer_id(&maintainer)],
        "merge_rule": "manual-reviewed",
        "preferred_branches": ["main"]
    });
    let view = signed_view(
        &maintainer,
        &policy,
        documents_value(
            "doc:view-repeat",
            "rev:cccccccccccccccccccccccccccccccccccccccccccccccccccccccccccccccc",
        ),
        11,
    );
    let (_source_dir, source_path) = write_json_file("view-publish-repeat", "view.json", &view);

    let first = run_mycel(&[
        "view",
        "publish",
        &path_arg(&source_path),
        "--into",
        &store_root,
        "--json",
    ]);
    assert_success(&first);

    let second = run_mycel(&[
        "view",
        "publish",
        &path_arg(&source_path),
        "--into",
        &store_root,
        "--json",
    ]);
    assert_success(&second);
    let json = parse_json_stdout(&second);
    assert_eq!(json["status"], "ok");
    assert_eq!(json["created"], Value::Bool(false));
}

#[test]
fn view_publish_rejects_non_view_object() {
    let store_dir = create_temp_dir("view-publish-invalid-store");
    let store_root = path_arg(&store_dir.path().to_path_buf());
    let init = run_mycel(&["store", "init", &store_root, "--json"]);
    assert_success(&init);

    let patch = signed_patch(&signing_key(43), "doc:not-view");
    let (_source_dir, source_path) =
        write_json_file("view-publish-invalid-source", "patch.json", &patch);

    let output = run_mycel(&[
        "view",
        "publish",
        &path_arg(&source_path),
        "--into",
        &store_root,
    ]);

    assert_exit_code(&output, 1);
    assert_stdout_contains(&output, "view publish: failed");
    assert_stderr_contains(&output, "view publish source is not a valid view object");
}

#[test]
fn view_inspect_reports_missing_view_id() {
    let store_dir = create_temp_dir("view-inspect-missing-store");
    let store_root = path_arg(&store_dir.path().to_path_buf());
    let init = run_mycel(&["store", "init", &store_root, "--json"]);
    assert_success(&init);

    let output = run_mycel(&[
        "view",
        "inspect",
        "view:missing",
        "--store-root",
        &store_root,
    ]);

    assert_exit_code(&output, 1);
    assert_stdout_contains(&output, "view inspection: failed");
    assert_stderr_contains(&output, "was not found in persisted governance indexes");
}
