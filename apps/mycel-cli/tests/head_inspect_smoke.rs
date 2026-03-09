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

#[test]
fn head_inspect_json_selects_highest_supported_head() {
    let doc_id = "doc:sample";
    let path = repo_root()
        .join("fixtures/head-inspect/minimal-head-selection")
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
    assert_eq!(
        json["critical_violations"]
            .as_array()
            .expect("critical_violations should be array")
            .len(),
        0
    );
    let effective_weights = json["effective_weights"]
        .as_array()
        .expect("effective_weights should be array");
    assert_eq!(effective_weights.len(), 3);
    assert!(effective_weights.iter().all(|entry| {
        entry["admitted"] == Value::Bool(true)
            && entry["effective_weight"] == Value::from(1)
            && entry["critical_violation_counts"]
                .as_array()
                .is_some_and(|counts| counts.is_empty())
    }));
    let maintainer_support = json["maintainer_support"]
        .as_array()
        .expect("maintainer_support should be array");
    assert_eq!(maintainer_support.len(), 3);
    assert!(maintainer_support.iter().all(|entry| {
        entry["effective_weight"] == Value::from(1)
            && entry["maintainer"].as_str().is_some()
            && entry["revision_id"].as_str().is_some()
    }));
    assert!(
        json["decision_trace"]
            .as_array()
            .is_some_and(|trace| trace.iter().any(|entry| {
                entry["step"].as_str() == Some("selector_epoch")
                    && entry["detail"]
                        .as_str()
                        .is_some_and(|detail| detail.contains("selector_epoch=0"))
            })),
        "expected selector_epoch trace entry, stdout: {}",
        stdout_text(&output)
    );
    assert!(
        json["decision_trace"]
            .as_array()
            .is_some_and(|trace| trace.iter().any(|entry| {
                entry["step"].as_str() == Some("maintainer_support")
                    && entry["detail"]
                        .as_str()
                        .is_some_and(|detail| detail.contains("pk:ed25519:"))
            })),
        "expected maintainer_support trace entry, stdout: {}",
        stdout_text(&output)
    );
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
fn head_inspect_json_resolves_repo_native_fixture_name() {
    let output = run_mycel(&[
        "head",
        "inspect",
        "doc:sample",
        "--input",
        "minimal-head-selection",
        "--json",
    ]);

    assert_success(&output);
    let json = parse_json_stdout(&output);
    assert_eq!(
        json["input_path"],
        repo_root()
            .join("fixtures/head-inspect/minimal-head-selection/bundle.json")
            .to_string_lossy()
            .into_owned()
    );
    assert_eq!(
        json["selected_head"],
        "rev:b98e3dca59291ebab04e88eadafaf30d52fcc78dd18df41568e5689c2be300ad"
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
        "profile": head_profile(hash_json(&policy), 1200),
        "revisions": [revision],
        "views": [],
        "critical_violations": []
    });
    let input = write_input_file("head-inspect-missing-doc", "input.json", bundle);
    let path = path_arg(&input.path);
    let output = run_mycel(&["head", "inspect", "doc:missing", "--input", &path]);

    assert_exit_code(&output, 1);
    assert_stdout_contains(&output, "head inspection: failed");
    assert_stderr_contains(&output, "NO_ELIGIBLE_HEAD");
}

#[test]
fn head_inspect_directory_resolves_input_json() {
    let author_key = signing_key(11);
    let revision = signed_revision(&author_key, "doc:sample", vec![], 1000, "hash:state-a");
    let policy = json!({
        "accept_keys": [signer_id(&signing_key(12))],
        "merge_rule": "manual-reviewed",
        "preferred_branches": ["main"]
    });
    let bundle = json!({
        "profile": head_profile(hash_json(&policy), 1200),
        "revisions": [revision],
        "views": [],
        "critical_violations": []
    });
    let dir = create_temp_dir("head-inspect-directory");
    let path = dir.path().join("input.json");
    fs::write(
        &path,
        serde_json::to_string_pretty(&bundle).expect("bundle JSON should serialize"),
    )
    .expect("bundle JSON should be written");
    let output = run_mycel(&[
        "head",
        "inspect",
        "doc:sample",
        "--input",
        &path_arg(&dir.path().to_path_buf()),
        "--json",
    ]);

    assert_success(&output);
    let json = parse_json_stdout(&output);
    assert_eq!(json["input_path"], path.to_string_lossy().into_owned());
}

#[test]
fn head_inspect_text_reports_decision_trace() {
    let output = run_mycel(&[
        "head",
        "inspect",
        "doc:sample",
        "--input",
        "minimal-head-selection",
    ]);

    assert_success(&output);
    assert_stdout_contains(&output, "trace: selector_epoch:");
    assert_stdout_contains(&output, "trace: maintainer_support:");
    assert_stdout_contains(&output, "trace: selected_head:");
}

#[test]
fn head_inspect_uses_effective_weight_in_selector_score() {
    let doc_id = "doc:weighted";
    let revision_author = signing_key(21);
    let maintainer_a = signing_key(31);
    let maintainer_b = signing_key(32);
    let maintainer_c = signing_key(33);
    let policy = json!({
        "accept_keys": [
            signer_id(&maintainer_a),
            signer_id(&maintainer_b),
            signer_id(&maintainer_c)
        ],
        "merge_rule": "manual-reviewed",
        "preferred_branches": ["main"]
    });
    let revision_a = signed_revision(&revision_author, doc_id, vec![], 10, "hash:weighted-a");
    let revision_b = signed_revision(&revision_author, doc_id, vec![], 20, "hash:weighted-b");
    let bundle = json!({
        "profile": {
            "policy_hash": hash_json(&policy),
            "effective_selection_time": 250,
            "epoch_seconds": 100,
            "epoch_zero_timestamp": 0,
            "admission_window_epochs": 2,
            "min_valid_views_for_admission": 1,
            "min_valid_views_per_epoch": 2,
            "weight_cap_per_key": 3
        },
        "revisions": [revision_a.clone(), revision_b.clone()],
        "views": [
            signed_view(
                &maintainer_a,
                &policy,
                documents_value(doc_id, &revision_a["revision_id"]),
                10
            ),
            signed_view(
                &maintainer_b,
                &policy,
                documents_value(doc_id, &revision_b["revision_id"]),
                12
            ),
            signed_view(
                &maintainer_a,
                &policy,
                documents_value(doc_id, &revision_a["revision_id"]),
                110
            ),
            signed_view(
                &maintainer_a,
                &policy,
                documents_value(doc_id, &revision_a["revision_id"]),
                120
            ),
            signed_view(
                &maintainer_c,
                &policy,
                documents_value(doc_id, &revision_b["revision_id"]),
                220
            ),
            signed_view(
                &maintainer_a,
                &policy,
                documents_value(doc_id, &revision_a["revision_id"]),
                230
            ),
            signed_view(
                &maintainer_b,
                &policy,
                documents_value(doc_id, &revision_b["revision_id"]),
                240
            )
        ],
        "critical_violations": []
    });
    let input = write_input_file("head-inspect-weighted", "input.json", bundle);
    let output = run_mycel(&[
        "head",
        "inspect",
        doc_id,
        "--input",
        &path_arg(&input.path),
        "--json",
    ]);

    assert_success(&output);
    let json = parse_json_stdout(&output);
    assert_eq!(json["selected_head"], revision_a["revision_id"]);
    let eligible_heads = json["eligible_heads"]
        .as_array()
        .expect("eligible_heads should be array");
    let selected = eligible_heads
        .iter()
        .find(|entry| entry["revision_id"] == revision_a["revision_id"])
        .expect("selected head summary should exist");
    let alternative = eligible_heads
        .iter()
        .find(|entry| entry["revision_id"] == revision_b["revision_id"])
        .expect("alternative head summary should exist");
    assert_eq!(selected["weighted_support"], 2);
    assert_eq!(selected["supporter_count"], 1);
    assert_eq!(alternative["weighted_support"], 1);
    let effective_weights = json["effective_weights"]
        .as_array()
        .expect("effective_weights should be array");
    let promoted = effective_weights
        .iter()
        .find(|entry| entry["effective_weight"] == Value::from(2))
        .expect("expected promoted effective weight entry");
    assert_eq!(promoted["admitted"], Value::Bool(true));
    assert!(
        promoted["valid_view_counts"]
            .as_array()
            .is_some_and(|counts| counts.iter().any(|entry| {
                entry["epoch"] == Value::from(1) && entry["count"] == Value::from(2)
            })),
        "expected epoch 1 valid_view_counts entry, stdout: {}",
        stdout_text(&output)
    );
    let maintainer_support = json["maintainer_support"]
        .as_array()
        .expect("maintainer_support should be array");
    assert!(
        maintainer_support.iter().any(|entry| {
            entry["revision_id"] == revision_a["revision_id"]
                && entry["effective_weight"] == Value::from(2)
        }),
        "expected weighted maintainer_support entry, stdout: {}",
        stdout_text(&output)
    );
    assert!(
        json["decision_trace"]
            .as_array()
            .is_some_and(|trace| trace.iter().any(|entry| {
                entry["step"].as_str() == Some("effective_weight")
                    && entry["detail"]
                        .as_str()
                        .is_some_and(|detail| detail.contains("weight=2"))
            })),
        "expected effective_weight trace entry, stdout: {}",
        stdout_text(&output)
    );
}

#[test]
fn head_inspect_penalizes_critical_violations() {
    let doc_id = "doc:penalty";
    let revision_author = signing_key(41);
    let maintainer_a = signing_key(51);
    let maintainer_b = signing_key(52);
    let policy = json!({
        "accept_keys": [
            signer_id(&maintainer_a),
            signer_id(&maintainer_b)
        ],
        "merge_rule": "manual-reviewed",
        "preferred_branches": ["main"]
    });
    let revision_a = signed_revision(&revision_author, doc_id, vec![], 10, "hash:penalty-a");
    let revision_b = signed_revision(&revision_author, doc_id, vec![], 20, "hash:penalty-b");
    let bundle = json!({
        "profile": {
            "policy_hash": hash_json(&policy),
            "effective_selection_time": 250,
            "epoch_seconds": 100,
            "epoch_zero_timestamp": 0,
            "admission_window_epochs": 2,
            "min_valid_views_for_admission": 1,
            "min_valid_views_per_epoch": 2,
            "weight_cap_per_key": 3
        },
        "revisions": [revision_a.clone(), revision_b.clone()],
        "views": [
            signed_view(
                &maintainer_a,
                &policy,
                documents_value(doc_id, &revision_a["revision_id"]),
                10
            ),
            signed_view(
                &maintainer_b,
                &policy,
                documents_value(doc_id, &revision_b["revision_id"]),
                12
            ),
            signed_view(
                &maintainer_a,
                &policy,
                documents_value(doc_id, &revision_a["revision_id"]),
                110
            ),
            signed_view(
                &maintainer_a,
                &policy,
                documents_value(doc_id, &revision_a["revision_id"]),
                120
            ),
            signed_view(
                &maintainer_b,
                &policy,
                documents_value(doc_id, &revision_b["revision_id"]),
                210
            ),
            signed_view(
                &maintainer_b,
                &policy,
                documents_value(doc_id, &revision_b["revision_id"]),
                220
            )
        ],
        "critical_violations": [
            critical_violation(&maintainer_a, 150, "equivocated view publication")
        ]
    });
    let input = write_input_file("head-inspect-penalty", "input.json", bundle);
    let output = run_mycel(&[
        "head",
        "inspect",
        doc_id,
        "--input",
        &path_arg(&input.path),
        "--json",
    ]);

    assert_success(&output);
    let json = parse_json_stdout(&output);
    assert_eq!(json["selected_head"], revision_b["revision_id"]);
    let eligible_heads = json["eligible_heads"]
        .as_array()
        .expect("eligible_heads should be array");
    let penalized = eligible_heads
        .iter()
        .find(|entry| entry["revision_id"] == revision_a["revision_id"])
        .expect("penalized head summary should exist");
    let surviving = eligible_heads
        .iter()
        .find(|entry| entry["revision_id"] == revision_b["revision_id"])
        .expect("surviving head summary should exist");
    assert_eq!(penalized["weighted_support"], 0);
    assert_eq!(surviving["weighted_support"], 1);
    let critical_violations = json["critical_violations"]
        .as_array()
        .expect("critical_violations should be array");
    assert_eq!(critical_violations.len(), 1);
    assert_eq!(critical_violations[0]["selector_epoch"], Value::from(1));
    assert_eq!(
        critical_violations[0]["reason"],
        Value::String("equivocated view publication".to_string())
    );
    let effective_weights = json["effective_weights"]
        .as_array()
        .expect("effective_weights should be array");
    let penalized_weight = effective_weights
        .iter()
        .find(|entry| {
            entry["critical_violation_counts"]
                .as_array()
                .is_some_and(|counts| {
                    counts.iter().any(|count| {
                        count["epoch"] == Value::from(1) && count["count"] == Value::from(1)
                    })
                })
        })
        .expect("expected penalized effective weight entry");
    assert_eq!(penalized_weight["admitted"], Value::Bool(false));
    assert_eq!(penalized_weight["effective_weight"], Value::from(0));
    let maintainer_support = json["maintainer_support"]
        .as_array()
        .expect("maintainer_support should be array");
    assert!(
        maintainer_support
            .iter()
            .all(|entry| entry["effective_weight"] != Value::from(0)),
        "expected penalized maintainer to be absent from maintainer_support, stdout: {}",
        stdout_text(&output)
    );
    assert!(
        json["decision_trace"]
            .as_array()
            .is_some_and(|trace| trace.iter().any(|entry| {
                entry["step"].as_str() == Some("effective_weight")
                    && entry["detail"].as_str().is_some_and(|detail| {
                        detail.contains("violations=[epoch1=1]") && detail.contains("weight=0")
                    })
            })),
        "expected penalty trace entry, stdout: {}",
        stdout_text(&output)
    );
}

#[test]
fn head_inspect_requires_input_path() {
    let output = run_mycel(&["head", "inspect", "doc:sample"]);

    assert_exit_code(&output, 2);
    assert_stderr_contains(&output, "missing --input for head inspect");
    assert_stdout_contains(&output, "Head options:");
}

#[test]
fn head_inspect_reports_unknown_repo_native_fixture() {
    let output = run_mycel(&["head", "inspect", "doc:sample", "--input", "does-not-exist"]);

    assert_exit_code(&output, 1);
    assert_stdout_contains(&output, "head inspection: failed");
    assert_stderr_contains(
        &output,
        "could not resolve head-inspect input 'does-not-exist'",
    );
}

#[test]
fn head_rejects_unknown_subcommand() {
    let output = run_mycel(&["head", "bogus"]);

    assert_exit_code(&output, 2);
    assert_stderr_contains(&output, "unknown head subcommand: bogus");
    assert_stdout_contains(&output, "Head options:");
}
