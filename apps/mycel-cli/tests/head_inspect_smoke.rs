use std::fs;
use std::path::PathBuf;

use base64::Engine;
use ed25519_dalek::{Signer, SigningKey};
use serde_json::{json, Value};
use sha2::{Digest, Sha256};

mod common;

use common::{
    assert_exit_code, assert_stderr_contains, assert_stdout_contains, assert_success,
    create_temp_dir, parse_json_stdout, repo_root, run_mycel, stdout_text,
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
    let mut value = json!({
        "type": "revision",
        "version": "mycel/0.1",
        "doc_id": doc_id,
        "parents": parents,
        "patches": [],
        "state_hash": state_hash,
        "author": signer_id(signing_key),
        "timestamp": timestamp
    });
    let id = recompute_id(&value, "revision_id", "rev");
    value["revision_id"] = Value::String(id);
    value["signature"] = Value::String(sign_value(signing_key, &value));
    value
}

fn hash_json(value: &Value) -> String {
    let canonical = canonical_json(value);
    let mut hasher = Sha256::new();
    hasher.update(canonical.as_bytes());
    format!("hash:{:x}", hasher.finalize())
}

#[test]
fn head_inspect_json_selects_highest_supported_head() {
    let doc_id = "doc:sample";
    let path = repo_root()
        .join("fixtures/head-inspect/minimal-head-selection.example.json")
        .to_string_lossy()
        .into_owned();
    let output = run_mycel(&["head", "inspect", doc_id, "--input", &path, "--json"]);

    assert_success(&output);
    let json = parse_json_stdout(&output);
    assert_eq!(json["status"], "ok");
    assert_eq!(json["doc_id"], doc_id);
    assert_eq!(
        json["selected_head"],
        "rev:b98e3dca59291ebab04e88eadafaf30d52fcc78dd18df41568e5689c2be300ad"
    );
    assert_eq!(json["tie_break_reason"], "higher_selector_score");
    assert_eq!(json["verified_revision_count"], 3);
    assert_eq!(json["verified_view_count"], 3);
    assert!(
        json["eligible_heads"]
            .as_array()
            .is_some_and(|heads| heads.len() == 2),
        "expected two eligible heads, stdout: {}",
        stdout_text(&output)
    );
    assert!(
        json["notes"]
            .as_array()
            .is_some_and(|notes| notes.iter().any(|entry| {
                entry
                    .as_str()
                    .is_some_and(|message| message.contains("minimal selector mode"))
            })),
        "expected minimal selector note, stdout: {}",
        stdout_text(&output)
    );
}

#[test]
fn head_inspect_text_fails_when_no_eligible_head_exists() {
    let author_key = signing_key(11);
    let revision = signed_revision(&author_key, "doc:other", vec![], 1000, "hash:state-a");
    let policy = json!({
        "accept_keys": [signer_id(&signing_key(12))],
        "merge_rule": "manual-reviewed",
        "preferred_branches": ["main"]
    });
    let bundle = json!({
        "profile": {
            "policy_hash": hash_json(&policy),
            "effective_selection_time": 1200
        },
        "revisions": [revision],
        "views": []
    });
    let input = write_input_file("head-inspect-missing-doc", "input.json", bundle);
    let path = path_arg(&input.path);
    let output = run_mycel(&["head", "inspect", "doc:missing", "--input", &path]);

    assert_exit_code(&output, 1);
    assert_stdout_contains(&output, "head inspection: failed");
    assert_stderr_contains(&output, "NO_ELIGIBLE_HEAD");
}

#[test]
fn head_inspect_requires_input_path() {
    let output = run_mycel(&["head", "inspect", "doc:sample"]);

    assert_exit_code(&output, 2);
    assert_stderr_contains(&output, "missing --input for head inspect");
    assert_stdout_contains(&output, "Head options:");
}

#[test]
fn head_rejects_unknown_subcommand() {
    let output = run_mycel(&["head", "bogus"]);

    assert_exit_code(&output, 2);
    assert_stderr_contains(&output, "unknown head subcommand: bogus");
    assert_stdout_contains(&output, "Head options:");
}
