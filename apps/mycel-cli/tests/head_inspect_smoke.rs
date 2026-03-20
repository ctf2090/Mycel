use std::fs;
use std::path::PathBuf;

use base64::Engine;
use ed25519_dalek::{Signer, SigningKey};
use serde_json::{json, Value};
use sha2::{Digest, Sha256};

mod common;

use common::{
    assert_exit_code, assert_stderr_contains, assert_stderr_starts_with, assert_stdout_contains,
    assert_success, assert_top_level_help, create_temp_dir, parse_json_stdout, repo_root,
    run_mycel, stdout_text,
};

struct TempInputFile {
    _dir: common::TempDir,
    path: PathBuf,
}

fn write_input_file(prefix: &str, name: &str, value: Value) -> TempInputFile {
    let dir = create_temp_dir(prefix);
    let path = dir.path().join(name);
    let content = serde_json::to_string_pretty(&value).expect("bundle JSON should serialize");
    fs::write(&path, content).expect("bundle JSON should be written");
    TempInputFile { _dir: dir, path }
}

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

fn signed_revision(
    signing_key: &SigningKey,
    doc_id: &str,
    parents: Vec<String>,
    timestamp: u64,
    state_hash: &str,
) -> Value {
    signed_revision_with_patches(
        signing_key,
        doc_id,
        parents,
        Vec::new(),
        timestamp,
        state_hash,
    )
}

fn signed_revision_with_patches(
    signing_key: &SigningKey,
    doc_id: &str,
    parents: Vec<String>,
    patches: Vec<String>,
    timestamp: u64,
    state_hash: &str,
) -> Value {
    let mut value = json!({
        "type": "revision",
        "version": "mycel/0.1",
        "doc_id": doc_id,
        "parents": parents,
        "patches": patches,
        "state_hash": state_hash,
        "author": signer_id(signing_key),
        "timestamp": timestamp
    });
    let id = recompute_id(&value, "revision_id", "rev");
    value["revision_id"] = Value::String(id);
    value["signature"] = Value::String(sign_value(signing_key, &value));
    value
}

fn signed_patch(
    signing_key: &SigningKey,
    doc_id: &str,
    base_revision: &str,
    timestamp: u64,
    ops: Value,
) -> Value {
    let mut value = json!({
        "type": "patch",
        "version": "mycel/0.1",
        "doc_id": doc_id,
        "base_revision": base_revision,
        "author": signer_id(signing_key),
        "timestamp": timestamp,
        "ops": ops
    });
    let id = recompute_id(&value, "patch_id", "patch");
    value["patch_id"] = Value::String(id);
    value["signature"] = Value::String(sign_value(signing_key, &value));
    value
}

fn hash_json(value: &Value) -> String {
    let canonical = canonical_json(value);
    let mut hasher = Sha256::new();
    hasher.update(canonical.as_bytes());
    format!("hash:{:x}", hasher.finalize())
}

fn head_profile(policy_hash: String, effective_selection_time: u64) -> Value {
    json!({
        "policy_hash": policy_hash,
        "effective_selection_time": effective_selection_time,
        "epoch_seconds": 3600,
        "epoch_zero_timestamp": 0,
        "admission_window_epochs": 0,
        "min_valid_views_for_admission": 0,
        "min_valid_views_per_epoch": 1,
        "weight_cap_per_key": 3
    })
}

fn bounded_viewer_score_profile() -> Value {
    json!({
        "mode": "bounded-bonus-penalty",
        "bonus_cap": 2,
        "penalty_cap": 2,
        "signal_weight_cap": 2,
        "admission_required": true,
        "min_identity_tier": "basic",
        "min_reputation_band": "new"
    })
}

fn named_profiles(entries: &[(&str, Value)]) -> Value {
    let mut profiles = serde_json::Map::new();
    for (profile_id, profile) in entries {
        profiles.insert((*profile_id).to_string(), profile.clone());
    }
    Value::Object(profiles)
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
        "policy": policy,
        "documents": documents,
        "timestamp": timestamp
    });
    let id = recompute_id(&value, "view_id", "view");
    value["view_id"] = Value::String(id);
    value["signature"] = Value::String(sign_value(signing_key, &value));
    value
}

fn documents_value(doc_id: &str, revision_id: &Value) -> Value {
    let mut documents = serde_json::Map::new();
    documents.insert(doc_id.to_string(), revision_id.clone());
    Value::Object(documents)
}

fn critical_violation(maintainer: &SigningKey, timestamp: u64, reason: &str) -> Value {
    json!({
        "maintainer": signer_id(maintainer),
        "timestamp": timestamp,
        "reason": reason
    })
}

fn viewer_signal(
    signal_id: &str,
    viewer_seed: u8,
    revision_id: &Value,
    signal_type: &str,
    confidence_level: &str,
    created_at: u64,
    expires_at: u64,
) -> Value {
    json!({
        "signal_id": signal_id,
        "viewer_id": format!("viewer:{viewer_seed}"),
        "candidate_revision_id": revision_id.as_str().expect("revision id should be string"),
        "signal_type": signal_type,
        "reason_code": format!("reason-{signal_id}"),
        "confidence_level": confidence_level,
        "created_at": created_at,
        "expires_at": expires_at,
        "signal_status": "active",
        "viewer_identity_tier": "basic",
        "viewer_admission_status": "admitted",
        "viewer_reputation_band": "established"
    })
}

fn empty_document_state_hash(doc_id: &str) -> String {
    hash_json(&json!({
        "doc_id": doc_id,
        "blocks": []
    }))
}

fn document_state_hash(doc_id: &str, blocks: Vec<Value>) -> String {
    hash_json(&json!({
        "doc_id": doc_id,
        "blocks": blocks
    }))
}

fn write_store_source_objects(prefix: &str, objects: &[Value]) -> common::TempDir {
    let dir = create_temp_dir(prefix);
    for (index, object) in objects.iter().enumerate() {
        let path = dir.path().join(format!("object-{index}.json"));
        fs::write(
            path,
            serde_json::to_string_pretty(object).expect("object JSON should serialize"),
        )
        .expect("object JSON should be written");
    }
    dir
}

fn build_store_from_objects(objects: &[Value]) -> common::TempDir {
    let source_dir = write_store_source_objects("head-inspect-store-source", &objects);
    let store_dir = create_temp_dir("head-inspect-store-root");
    let ingest = run_mycel(&[
        "store",
        "ingest",
        &path_arg(&source_dir.path().to_path_buf()),
        "--into",
        &path_arg(&store_dir.path().to_path_buf()),
    ]);
    assert_success(&ingest);
    store_dir
}

#[path = "head_inspect_smoke/cli.rs"]
mod cli;
#[path = "head_inspect_smoke/inspect.rs"]
mod inspect;
#[path = "head_inspect_smoke/render.rs"]
mod render;
#[path = "head_inspect_smoke/selector.rs"]
mod selector;
