use std::fs;
use std::path::{Path, PathBuf};

use ed25519_dalek::SigningKey;
use mycel_core::author::signer_id;
use mycel_core::canonical::prefixed_canonical_hash;
use mycel_core::protocol::{recompute_object_id, CORE_PROTOCOL_VERSION};
use serde_json::{json, Value};

mod common;

use common::{
    assert_exit_code, assert_stderr_contains, assert_stderr_starts_with, assert_stdout_contains,
    assert_success, assert_top_level_help, create_temp_dir, parse_json_stdout, repo_root,
    run_mycel, sign_test_value as sign_value, stdout_text,
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

fn path_arg(path: &Path) -> String {
    path.to_string_lossy().into_owned()
}

fn signing_key(seed: u8) -> SigningKey {
    SigningKey::from_bytes(&[seed; 32])
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
        "version": CORE_PROTOCOL_VERSION,
        "doc_id": doc_id,
        "parents": parents,
        "patches": patches,
        "state_hash": state_hash,
        "author": signer_id(signing_key),
        "timestamp": timestamp
    });
    let id =
        recompute_object_id(&value, "revision_id", "rev").expect("revision id should recompute");
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
        "version": CORE_PROTOCOL_VERSION,
        "doc_id": doc_id,
        "base_revision": base_revision,
        "author": signer_id(signing_key),
        "timestamp": timestamp,
        "ops": ops
    });
    let id = recompute_object_id(&value, "patch_id", "patch").expect("patch id should recompute");
    value["patch_id"] = Value::String(id);
    value["signature"] = Value::String(sign_value(signing_key, &value));
    value
}

fn hash_json(value: &Value) -> String {
    prefixed_canonical_hash(value, "hash").expect("JSON hash should compute")
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
        "version": CORE_PROTOCOL_VERSION,
        "maintainer": signer_id(signing_key),
        "policy": policy,
        "documents": documents,
        "timestamp": timestamp
    });
    let id = recompute_object_id(&value, "view_id", "view").expect("view id should recompute");
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
    let source_dir = write_store_source_objects("head-inspect-store-source", objects);
    let store_dir = create_temp_dir("head-inspect-store-root");
    let ingest = run_mycel(&[
        "store",
        "ingest",
        &path_arg(source_dir.path()),
        "--into",
        &path_arg(store_dir.path()),
    ]);
    assert_success(&ingest);
    store_dir
}

#[path = "head_inspect_smoke/cli.rs"]
mod cli;
#[path = "head_inspect_smoke/dual_role.rs"]
mod dual_role;
#[path = "head_inspect_smoke/inspect.rs"]
mod inspect;
#[path = "head_inspect_smoke/render.rs"]
mod render;
#[path = "head_inspect_smoke/selector.rs"]
mod selector;
