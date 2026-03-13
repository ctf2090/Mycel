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
    let decision_trace = json["decision_trace"]
        .as_array()
        .expect("decision_trace should be array");
    let trace_steps = decision_trace
        .iter()
        .map(|entry| {
            entry["step"]
                .as_str()
                .expect("decision_trace step should be a string")
        })
        .collect::<Vec<_>>();
    assert_eq!(
        trace_steps,
        vec![
            "selector_epoch",
            "verified_inputs",
            "critical_violations",
            "eligible_heads",
            "editor_admission",
            "effective_weight",
            "maintainer_support",
            "selector_scores",
            "selected_head"
        ]
    );
    assert!(decision_trace.iter().all(|entry| {
        entry["detail"]
            .as_str()
            .is_some_and(|detail| !detail.contains("pk:ed25519:"))
    }));
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
fn head_inspect_json_applies_fixture_backed_viewer_score_channels() {
    let output = run_mycel(&[
        "head",
        "inspect",
        "doc:sample",
        "--input",
        "viewer-score-channels",
        "--json",
    ]);

    assert_success(&output);
    let json = parse_json_stdout(&output);
    assert_eq!(
        json["input_path"],
        repo_root()
            .join("fixtures/head-inspect/viewer-score-channels/bundle.json")
            .to_string_lossy()
            .into_owned()
    );
    assert_eq!(json["status"], "ok");
    assert_eq!(json["doc_id"], "doc:sample");
    assert_eq!(json["viewer_signal_count"], Value::from(7));
    assert_eq!(
        json["selected_head"],
        "rev:b98e3dca59291ebab04e88eadafaf30d52fcc78dd18df41568e5689c2be300ad"
    );
    assert_eq!(json["tie_break_reason"], "higher_selector_score");

    let eligible_heads = json["eligible_heads"]
        .as_array()
        .expect("eligible_heads should be array");
    assert_eq!(eligible_heads.len(), 2);
    let selected = eligible_heads
        .iter()
        .find(|entry| {
            entry["revision_id"]
                == Value::String(
                    "rev:b98e3dca59291ebab04e88eadafaf30d52fcc78dd18df41568e5689c2be300ad"
                        .to_string(),
                )
        })
        .expect("selected viewer fixture head should exist");
    let alternative = eligible_heads
        .iter()
        .find(|entry| {
            entry["revision_id"]
                == Value::String(
                    "rev:552fce487de89e2e8c7a002249b200440f4c24bfed735d1e7f730ea774f06430"
                        .to_string(),
                )
        })
        .expect("alternative viewer fixture head should exist");
    assert_eq!(selected["maintainer_score"], Value::from(2));
    assert_eq!(selected["viewer_bonus"], Value::from(2));
    assert_eq!(selected["viewer_penalty"], Value::from(0));
    assert_eq!(selected["selector_score"], Value::from(4));
    assert_eq!(alternative["maintainer_score"], Value::from(1));
    assert_eq!(alternative["viewer_bonus"], Value::from(0));
    assert_eq!(alternative["viewer_penalty"], Value::from(2));
    assert_eq!(alternative["selector_score"], Value::from(0));

    let viewer_signals = json["viewer_signals"]
        .as_array()
        .expect("viewer_signals should be array");
    assert_eq!(viewer_signals.len(), 7);
    assert!(
        viewer_signals.iter().any(|entry| {
            entry["signal_id"] == Value::String("signal-expired-approval".to_string())
                && entry["selector_eligible"] == Value::Bool(false)
                && entry["effective_signal_weight"] == Value::from(0)
        }),
        "expected expired approval signal to stay inactive, stdout: {}",
        stdout_text(&output)
    );
    assert!(
        viewer_signals.iter().any(|entry| {
            entry["signal_id"] == Value::String("signal-pending-admission".to_string())
                && entry["selector_eligible"] == Value::Bool(false)
                && entry["effective_signal_weight"] == Value::from(0)
        }),
        "expected pending admission signal to stay gated, stdout: {}",
        stdout_text(&output)
    );
    assert!(
        viewer_signals.iter().any(|entry| {
            entry["signal_id"] == Value::String("signal-no-identity".to_string())
                && entry["selector_eligible"] == Value::Bool(false)
                && entry["effective_signal_weight"] == Value::from(0)
        }),
        "expected none-tier objection signal to stay gated, stdout: {}",
        stdout_text(&output)
    );
    assert!(
        viewer_signals.iter().any(|entry| {
            entry["signal_id"] == Value::String("signal-challenge-alternative-high".to_string())
                && entry["selector_eligible"] == Value::Bool(true)
                && entry["effective_signal_weight"] == Value::from(0)
                && entry["evidence_ref"] == Value::String("evidence:challenge-alt-1".to_string())
        }),
        "expected evidenced challenge to contribute review pressure only, stdout: {}",
        stdout_text(&output)
    );
    assert!(
        viewer_signals.iter().any(|entry| {
            entry["signal_id"] == Value::String("signal-challenge-without-evidence".to_string())
                && entry["selector_eligible"] == Value::Bool(true)
                && entry["effective_signal_weight"] == Value::from(0)
                && entry["evidence_ref"] == Value::Null
        }),
        "expected unevidenced challenge to avoid score contribution, stdout: {}",
        stdout_text(&output)
    );

    let viewer_score_channels = json["viewer_score_channels"]
        .as_array()
        .expect("viewer_score_channels should be array");
    let selected_channel = viewer_score_channels
        .iter()
        .find(|entry| entry["revision_id"] == selected["revision_id"])
        .expect("selected viewer fixture channel should exist");
    let alternative_channel = viewer_score_channels
        .iter()
        .find(|entry| entry["revision_id"] == alternative["revision_id"])
        .expect("alternative viewer fixture channel should exist");
    assert_eq!(selected_channel["viewer_bonus"], Value::from(2));
    assert_eq!(selected_channel["approval_signal_count"], Value::from(1));
    assert_eq!(selected_channel["challenge_signal_count"], Value::from(0));
    assert_eq!(
        selected_channel["viewer_review_state"],
        Value::String("none".to_string())
    );
    assert_eq!(alternative_channel["viewer_penalty"], Value::from(2));
    assert_eq!(
        alternative_channel["objection_signal_count"],
        Value::from(1)
    );
    assert_eq!(
        alternative_channel["challenge_signal_count"],
        Value::from(2)
    );
    assert_eq!(
        alternative_channel["challenge_review_pressure"],
        Value::from(2)
    );
    assert_eq!(
        alternative_channel["challenge_freeze_pressure"],
        Value::from(2)
    );
    assert_eq!(
        alternative_channel["viewer_review_state"],
        Value::String("freeze-pressure".to_string())
    );

    assert!(
        json["decision_trace"]
            .as_array()
            .is_some_and(|trace| trace.iter().any(|entry| {
                entry["step"].as_str() == Some("viewer_score_channels")
                    && entry["detail"].as_str().is_some_and(|detail| {
                        detail.contains("mode=bounded-bonus-penalty")
                            && detail.contains("signals=7")
                            && detail.contains("eligible=4")
                            && detail.contains("contributing=2")
                            && detail.contains("bonus_cap=2")
                            && detail.contains("penalty_cap=2")
                    })
            })),
        "expected fixture-backed viewer_score_channels trace entry, stdout: {}",
        stdout_text(&output)
    );
}

#[test]
fn head_inspect_viewer_score_fixture_is_deterministic_across_repeated_runs() {
    let first = run_mycel(&[
        "head",
        "inspect",
        "doc:sample",
        "--input",
        "viewer-score-channels",
        "--json",
    ]);
    let second = run_mycel(&[
        "head",
        "inspect",
        "doc:sample",
        "--input",
        "viewer-score-channels",
        "--json",
    ]);

    assert_success(&first);
    assert_success(&second);
    assert_eq!(parse_json_stdout(&first), parse_json_stdout(&second));
}

#[test]
fn head_inspect_requires_profile_id_for_multi_profile_bundle() {
    let doc_id = "doc:multi-profile";
    let revision_author = signing_key(62);
    let maintainer = signing_key(74);
    let policy = json!({
        "accept_keys": [signer_id(&maintainer)],
        "merge_rule": "manual-reviewed",
        "preferred_branches": ["main"]
    });
    let revision_a = signed_revision(
        &revision_author,
        doc_id,
        vec![],
        10,
        &empty_document_state_hash(doc_id),
    );
    let revision_b = signed_revision(
        &revision_author,
        doc_id,
        vec![revision_a["revision_id"]
            .as_str()
            .expect("revision id should exist")
            .to_string()],
        20,
        &empty_document_state_hash(doc_id),
    );
    let bundle = json!({
        "profiles": named_profiles(&[
            ("stable", head_profile(hash_json(&policy), 18)),
            ("preview", head_profile(hash_json(&policy), 30))
        ]),
        "revisions": [revision_a, revision_b.clone()],
        "views": [
            signed_view(
                &maintainer,
                &policy,
                documents_value(doc_id, &revision_b["revision_id"]),
                25
            )
        ],
        "critical_violations": []
    });
    let input = write_input_file("head-inspect-multi-profile", "input.json", bundle);
    let output = run_mycel(&["head", "inspect", doc_id, "--input", &path_arg(&input.path)]);

    assert_exit_code(&output, 1);
    assert_stdout_contains(&output, "head inspection: failed");
    assert_stderr_contains(
        &output,
        "multiple named profiles; pass --profile-id (preview, stable)",
    );
}

#[test]
fn head_inspect_json_can_source_objects_from_store_index() {
    let doc_id = "doc:sample";
    let revision_author = signing_key(61);
    let maintainer_a = signing_key(71);
    let maintainer_b = signing_key(72);
    let maintainer_c = signing_key(73);
    let policy = json!({
        "accept_keys": [
            signer_id(&maintainer_a),
            signer_id(&maintainer_b),
            signer_id(&maintainer_c)
        ],
        "merge_rule": "manual-reviewed",
        "preferred_branches": ["main"]
    });
    let state_hash = empty_document_state_hash(doc_id);
    let revision_a = signed_revision(&revision_author, doc_id, vec![], 1000, &state_hash);
    let revision_b = signed_revision(
        &revision_author,
        doc_id,
        vec![revision_a["revision_id"]
            .as_str()
            .expect("revision id should exist")
            .to_string()],
        1010,
        &state_hash,
    );
    let revision_c = signed_revision(
        &revision_author,
        doc_id,
        vec![revision_a["revision_id"]
            .as_str()
            .expect("revision id should exist")
            .to_string()],
        1020,
        &state_hash,
    );
    let view_a = signed_view(
        &maintainer_a,
        &policy,
        documents_value(doc_id, &revision_b["revision_id"]),
        1100,
    );
    let view_b = signed_view(
        &maintainer_b,
        &policy,
        documents_value(doc_id, &revision_c["revision_id"]),
        1110,
    );
    let view_c = signed_view(
        &maintainer_c,
        &policy,
        documents_value(doc_id, &revision_b["revision_id"]),
        1120,
    );
    let store_dir = build_store_from_objects(&[
        revision_a.clone(),
        revision_b.clone(),
        revision_c.clone(),
        view_a,
        view_b,
        view_c,
    ]);
    let input = write_input_file(
        "head-inspect-store-backed",
        "input.json",
        json!({
            "profile": head_profile(hash_json(&policy), 1200),
            "revisions": [],
            "views": [],
            "critical_violations": []
        }),
    );
    let output = run_mycel(&[
        "head",
        "inspect",
        doc_id,
        "--input",
        &path_arg(&input.path),
        "--store-root",
        &path_arg(&store_dir.path().to_path_buf()),
        "--json",
    ]);

    assert_success(&output);
    let json = parse_json_stdout(&output);
    assert_eq!(json["selected_head"], revision_b["revision_id"]);
    assert_eq!(json["verified_revision_count"], 3);
    assert_eq!(json["verified_view_count"], 3);
    assert!(
        json["notes"]
            .as_array()
            .is_some_and(|notes| notes.iter().any(|entry| {
                entry.as_str().is_some_and(|message| {
                    message.contains("store-backed selector inputs loaded from")
                })
            })),
        "expected store-backed note, stdout: {}",
        stdout_text(&output)
    );
}

#[test]
fn head_inspect_store_backed_applies_editor_admission_from_profile() {
    let doc_id = "doc:store-backed-editor-admission";
    let admitted_author = signing_key(64);
    let non_admitted_author = signing_key(65);
    let policy = json!({
        "merge_rule": "manual-reviewed",
        "preferred_branches": ["main"]
    });
    let state_hash = empty_document_state_hash(doc_id);
    let admitted_revision = signed_revision(&admitted_author, doc_id, vec![], 1000, &state_hash);
    let non_admitted_revision =
        signed_revision(&non_admitted_author, doc_id, vec![], 1010, &state_hash);
    let store_dir =
        build_store_from_objects(&[admitted_revision.clone(), non_admitted_revision.clone()]);
    let mut profile = head_profile(hash_json(&policy), 1200);
    profile["editor_admission"] = json!({
        "mode": "admitted-only",
        "admitted_keys": [signer_id(&admitted_author)]
    });
    let input = write_input_file(
        "head-inspect-store-backed-editor-admission",
        "input.json",
        json!({
            "profile": profile,
            "revisions": [],
            "views": [],
            "critical_violations": []
        }),
    );
    let output = run_mycel(&[
        "head",
        "inspect",
        doc_id,
        "--input",
        &path_arg(&input.path),
        "--store-root",
        &path_arg(&store_dir.path().to_path_buf()),
        "--json",
    ]);

    assert_success(&output);
    let json = parse_json_stdout(&output);
    assert_eq!(json["selected_head"], admitted_revision["revision_id"]);
    let editor_candidates = json["editor_candidates"]
        .as_array()
        .expect("editor_candidates should be array");
    assert!(
        editor_candidates.iter().any(|entry| {
            entry["revision_id"] == admitted_revision["revision_id"]
                && entry["editor_admitted"] == Value::Bool(true)
                && entry["candidate_eligible"] == Value::Bool(true)
        }),
        "expected admitted store-backed candidate summary, stdout: {}",
        stdout_text(&output)
    );
    assert!(
        editor_candidates.iter().any(|entry| {
            entry["revision_id"] == non_admitted_revision["revision_id"]
                && entry["editor_admitted"] == Value::Bool(false)
                && entry["candidate_eligible"] == Value::Bool(false)
        }),
        "expected filtered store-backed candidate summary, stdout: {}",
        stdout_text(&output)
    );
    assert!(
        json["decision_trace"]
            .as_array()
            .is_some_and(|trace| trace.iter().any(|entry| {
                entry["step"].as_str() == Some("editor_admission")
                    && entry["detail"].as_str().is_some_and(|detail| {
                        detail.contains("mode=admitted-only")
                            && detail.contains("structural_heads=2")
                            && detail.contains("eligible=1")
                    })
            })),
        "expected store-backed editor_admission trace, stdout: {}",
        stdout_text(&output)
    );
}

#[test]
fn head_inspect_json_selects_requested_named_profile() {
    let doc_id = "doc:selected-profile";
    let revision_author = signing_key(63);
    let maintainer = signing_key(75);
    let policy = json!({
        "accept_keys": [signer_id(&maintainer)],
        "merge_rule": "manual-reviewed",
        "preferred_branches": ["main"]
    });
    let state_hash = empty_document_state_hash(doc_id);
    let revision_a = signed_revision(&revision_author, doc_id, vec![], 10, &state_hash);
    let revision_b = signed_revision(
        &revision_author,
        doc_id,
        vec![revision_a["revision_id"]
            .as_str()
            .expect("revision id should exist")
            .to_string()],
        20,
        &state_hash,
    );
    let bundle = json!({
        "profiles": named_profiles(&[
            ("stable", head_profile(hash_json(&policy), 18)),
            ("preview", head_profile(hash_json(&policy), 30))
        ]),
        "revisions": [revision_a.clone(), revision_b.clone()],
        "views": [
            signed_view(
                &maintainer,
                &policy,
                documents_value(doc_id, &revision_a["revision_id"]),
                15
            ),
            signed_view(
                &maintainer,
                &policy,
                documents_value(doc_id, &revision_b["revision_id"]),
                25
            )
        ],
        "critical_violations": []
    });
    let input = write_input_file("head-inspect-selected-profile", "input.json", bundle);
    let output = run_mycel(&[
        "head",
        "inspect",
        doc_id,
        "--input",
        &path_arg(&input.path),
        "--profile-id",
        "preview",
        "--json",
    ]);

    assert_success(&output);
    let json = parse_json_stdout(&output);
    assert_eq!(json["profile_id"], "preview");
    assert_eq!(json["selected_head"], revision_b["revision_id"]);
    assert_eq!(json["effective_selection_time"], 30);
}

#[test]
fn head_inspect_named_profile_applies_requested_editor_admission_mode() {
    let doc_id = "doc:selected-editor-profile";
    let admitted_author = signing_key(66);
    let non_admitted_author = signing_key(67);
    let policy = json!({
        "merge_rule": "manual-reviewed",
        "preferred_branches": ["main"]
    });
    let state_hash = empty_document_state_hash(doc_id);
    let admitted_revision = signed_revision(&admitted_author, doc_id, vec![], 10, &state_hash);
    let non_admitted_revision =
        signed_revision(&non_admitted_author, doc_id, vec![], 20, &state_hash);
    let stable = head_profile(hash_json(&policy), 30);
    let mut preview = head_profile(hash_json(&policy), 30);
    preview["editor_admission"] = json!({
        "mode": "admitted-only",
        "admitted_keys": [signer_id(&admitted_author)]
    });
    let bundle = json!({
        "profiles": named_profiles(&[
            ("stable", stable),
            ("preview", preview)
        ]),
        "revisions": [admitted_revision.clone(), non_admitted_revision.clone()],
        "views": [],
        "critical_violations": []
    });
    let input = write_input_file("head-inspect-selected-editor-profile", "input.json", bundle);
    let output = run_mycel(&[
        "head",
        "inspect",
        doc_id,
        "--input",
        &path_arg(&input.path),
        "--profile-id",
        "preview",
        "--json",
    ]);

    assert_success(&output);
    let json = parse_json_stdout(&output);
    assert_eq!(json["profile_id"], "preview");
    assert_eq!(json["selected_head"], admitted_revision["revision_id"]);
    let eligible_heads = json["eligible_heads"]
        .as_array()
        .expect("eligible_heads should be array");
    assert_eq!(eligible_heads.len(), 1);
    assert_eq!(
        eligible_heads[0]["revision_id"],
        admitted_revision["revision_id"]
    );
    assert_eq!(eligible_heads[0]["editor_admitted"], Value::Bool(true));
    assert!(
        json["decision_trace"]
            .as_array()
            .is_some_and(|trace| trace.iter().any(|entry| {
                entry["step"].as_str() == Some("editor_admission")
                    && entry["detail"].as_str().is_some_and(|detail| {
                        detail.contains("mode=admitted-only")
                            && detail.contains("structural_heads=2")
                            && detail.contains("eligible=1")
                    })
            })),
        "expected named-profile editor_admission trace, stdout: {}",
        stdout_text(&output)
    );
}

#[test]
fn head_inspect_store_root_reports_missing_manifest() {
    let input = write_input_file(
        "head-inspect-store-missing-manifest",
        "input.json",
        json!({
            "profile": head_profile("hash:missing".to_string(), 1200),
            "revisions": [],
            "views": [],
            "critical_violations": []
        }),
    );
    let store_dir = create_temp_dir("head-inspect-missing-store-root");
    let output = run_mycel(&[
        "head",
        "inspect",
        "doc:sample",
        "--input",
        &path_arg(&input.path),
        "--store-root",
        &path_arg(&store_dir.path().to_path_buf()),
        "--json",
    ]);

    assert_exit_code(&output, 1);
    let json = parse_json_stdout(&output);
    assert_eq!(json["status"], "failed");
    assert!(
        json["errors"]
            .as_array()
            .is_some_and(|errors| errors.iter().any(|entry| {
                entry
                    .as_str()
                    .is_some_and(|message| message.contains("failed to read store index manifest"))
            })),
        "expected missing manifest error, stdout: {}",
        stdout_text(&output)
    );
}

#[test]
fn head_render_json_replays_selected_head_from_store() {
    let doc_id = "doc:render";
    let revision_author = signing_key(81);
    let maintainer_a = signing_key(91);
    let maintainer_b = signing_key(92);
    let policy = json!({
        "accept_keys": [
            signer_id(&maintainer_a),
            signer_id(&maintainer_b)
        ],
        "merge_rule": "manual-reviewed",
        "preferred_branches": ["main"]
    });
    let genesis_hash = empty_document_state_hash(doc_id);
    let revision_a = signed_revision(&revision_author, doc_id, vec![], 1000, &genesis_hash);
    let patch = signed_patch(
        &revision_author,
        doc_id,
        revision_a["revision_id"]
            .as_str()
            .expect("revision id should exist"),
        1010,
        json!([
            {
                "op": "insert_block",
                "new_block": {
                    "block_id": "blk:render-001",
                    "block_type": "paragraph",
                    "content": "Hello render",
                    "attrs": {},
                    "children": [
                        {
                            "block_id": "blk:render-002",
                            "block_type": "paragraph",
                            "content": "Nested reply",
                            "attrs": {},
                            "children": []
                        }
                    ]
                }
            }
        ]),
    );
    let revision_b = signed_revision_with_patches(
        &revision_author,
        doc_id,
        vec![revision_a["revision_id"]
            .as_str()
            .expect("revision id should exist")
            .to_string()],
        vec![patch["patch_id"]
            .as_str()
            .expect("patch id should exist")
            .to_string()],
        1020,
        &document_state_hash(
            doc_id,
            vec![json!({
                "block_id": "blk:render-001",
                "block_type": "paragraph",
                "content": "Hello render",
                "attrs": {},
                "children": [
                    {
                        "block_id": "blk:render-002",
                        "block_type": "paragraph",
                        "content": "Nested reply",
                        "attrs": {},
                        "children": []
                    }
                ]
            })],
        ),
    );
    let view_a = signed_view(
        &maintainer_a,
        &policy,
        documents_value(doc_id, &revision_b["revision_id"]),
        1100,
    );
    let view_b = signed_view(
        &maintainer_b,
        &policy,
        documents_value(doc_id, &revision_b["revision_id"]),
        1110,
    );
    let store_dir =
        build_store_from_objects(&[revision_a, patch, revision_b.clone(), view_a, view_b]);
    let input = write_input_file(
        "head-render-store-backed",
        "input.json",
        json!({
            "profile": head_profile(hash_json(&policy), 1200),
            "revisions": [],
            "views": [],
            "critical_violations": []
        }),
    );

    let output = run_mycel(&[
        "head",
        "render",
        doc_id,
        "--input",
        &path_arg(&input.path),
        "--store-root",
        &path_arg(&store_dir.path().to_path_buf()),
        "--json",
    ]);

    assert_success(&output);
    let json = parse_json_stdout(&output);
    assert_eq!(json["status"], "ok");
    assert_eq!(json["selected_head"], revision_b["revision_id"]);
    assert_eq!(json["rendered_block_count"], 2);
    assert_eq!(json["rendered_text"], "Hello render\n  Nested reply");
    assert!(json["recomputed_state_hash"]
        .as_str()
        .is_some_and(|value| value.starts_with("hash:")));
    assert_eq!(
        json["rendered_blocks"]
            .as_array()
            .and_then(|blocks| blocks.first())
            .and_then(|block| block["content"].as_str()),
        Some("Hello render")
    );
}

#[test]
fn head_render_store_backed_reports_multi_hop_ancestry_context() {
    let doc_id = "doc:render-store-ancestry";
    let revision_author = signing_key(86);
    let maintainer = signing_key(97);
    let policy = json!({
        "accept_keys": [signer_id(&maintainer)],
        "merge_rule": "manual-reviewed",
        "preferred_branches": ["main"]
    });
    let genesis_hash = empty_document_state_hash(doc_id);
    let parent_revision = signed_revision_with_patches(
        &revision_author,
        doc_id,
        vec![],
        vec!["patch:missing-ancestor".to_string()],
        1000,
        &genesis_hash,
    );
    let revision_b = signed_revision(
        &revision_author,
        doc_id,
        vec![parent_revision["revision_id"]
            .as_str()
            .expect("parent revision id should exist")
            .to_string()],
        1010,
        &genesis_hash,
    );
    let view = signed_view(
        &maintainer,
        &policy,
        documents_value(doc_id, &revision_b["revision_id"]),
        1100,
    );
    let store_dir = create_temp_dir("head-render-store-ancestry-root");
    let objects_dir = store_dir.path().join("objects");
    let indexes_dir = store_dir.path().join("indexes");
    fs::create_dir_all(objects_dir.join("revision")).expect("revision store dir should exist");
    fs::create_dir_all(objects_dir.join("view")).expect("view store dir should exist");
    fs::create_dir_all(&indexes_dir).expect("index dir should exist");

    let write_store_object = |object_type: &str, object_id: &str, value: &Value| -> PathBuf {
        let (_, object_hash) = object_id
            .split_once(':')
            .expect("object id should contain a type prefix");
        let path = objects_dir
            .join(object_type)
            .join(format!("{object_hash}.json"));
        fs::write(
            &path,
            serde_json::to_string_pretty(value).expect("store object should serialize"),
        )
        .expect("store object should write");
        path
    };
    let parent_revision_id = parent_revision["revision_id"]
        .as_str()
        .expect("parent revision id should exist")
        .to_string();
    let revision_b_id = revision_b["revision_id"]
        .as_str()
        .expect("selected head id should exist")
        .to_string();
    let view_id = view["view_id"]
        .as_str()
        .expect("view id should exist")
        .to_string();
    write_store_object("revision", &parent_revision_id, &parent_revision);
    write_store_object("revision", &revision_b_id, &revision_b);
    write_store_object("view", &view_id, &view);
    fs::write(
        indexes_dir.join("manifest.json"),
        serde_json::to_string_pretty(&json!({
            "version": "mycel-store-index/0.1",
            "stored_object_count": 3,
            "object_ids_by_type": {
                "revision": [parent_revision_id.clone(), revision_b_id.clone()],
                "view": [view_id.clone()]
            },
            "doc_revisions": {
                doc_id: [parent_revision_id.clone(), revision_b_id.clone()]
            },
            "revision_parents": {
                revision_b_id.clone(): [parent_revision_id.clone()]
            },
            "author_patches": {},
            "view_governance": [
                {
                    "view_id": view_id,
                    "maintainer": signer_id(&maintainer),
                    "profile_id": hash_json(&policy),
                    "documents": {
                        doc_id: revision_b_id.clone()
                    }
                }
            ],
            "maintainer_views": {},
            "profile_views": {},
            "document_views": {},
            "profile_heads": {}
        }))
        .expect("manifest should serialize"),
    )
    .expect("manifest should write");
    let input = write_input_file(
        "head-render-store-ancestry",
        "input.json",
        json!({
            "profile": head_profile(hash_json(&policy), 1200),
            "revisions": [],
            "views": [],
            "critical_violations": []
        }),
    );

    let output = run_mycel(&[
        "head",
        "render",
        doc_id,
        "--input",
        &path_arg(&input.path),
        "--store-root",
        &path_arg(&store_dir.path().to_path_buf()),
        "--json",
    ]);

    assert_exit_code(&output, 1);
    let json = parse_json_stdout(&output);
    assert_eq!(json["status"], "failed");
    assert!(
        json["errors"]
            .as_array()
            .is_some_and(|errors| errors.iter().any(|entry| {
                entry.as_str().is_some_and(|message| {
                    message.contains("failed verification before render replay")
                        && message.contains(&format!(
                            "while verifying ancestry through parent revision '{parent_revision_id}'"
                        ))
                        && message.contains("missing patch 'patch:missing-ancestor' for replay")
                })
            })),
        "expected nested ancestry-context render error, stdout: {}",
        stdout_text(&output)
    );
}

#[test]
fn head_render_store_backed_applies_editor_admission_from_profile() {
    let doc_id = "doc:render-store-editor-admission";
    let admitted_author = signing_key(87);
    let non_admitted_author = signing_key(88);
    let policy = json!({
        "merge_rule": "manual-reviewed",
        "preferred_branches": ["main"]
    });
    let admitted_patch = signed_patch(
        &admitted_author,
        doc_id,
        "rev:genesis-null",
        1000,
        json!([
            {
                "op": "insert_block",
                "new_block": {
                    "block_id": "blk:render-admitted-001",
                    "block_type": "paragraph",
                    "content": "Admitted render line",
                    "attrs": {},
                    "children": []
                }
            }
        ]),
    );
    let admitted_revision = signed_revision_with_patches(
        &admitted_author,
        doc_id,
        vec![],
        vec![admitted_patch["patch_id"]
            .as_str()
            .expect("patch id should exist")
            .to_string()],
        1010,
        &document_state_hash(
            doc_id,
            vec![json!({
                "block_id": "blk:render-admitted-001",
                "block_type": "paragraph",
                "content": "Admitted render line",
                "attrs": {},
                "children": []
            })],
        ),
    );
    let non_admitted_patch = signed_patch(
        &non_admitted_author,
        doc_id,
        "rev:genesis-null",
        1020,
        json!([
            {
                "op": "insert_block",
                "new_block": {
                    "block_id": "blk:render-non-admitted-001",
                    "block_type": "paragraph",
                    "content": "Non-admitted render line",
                    "attrs": {},
                    "children": []
                }
            }
        ]),
    );
    let non_admitted_revision = signed_revision_with_patches(
        &non_admitted_author,
        doc_id,
        vec![],
        vec![non_admitted_patch["patch_id"]
            .as_str()
            .expect("patch id should exist")
            .to_string()],
        1030,
        &document_state_hash(
            doc_id,
            vec![json!({
                "block_id": "blk:render-non-admitted-001",
                "block_type": "paragraph",
                "content": "Non-admitted render line",
                "attrs": {},
                "children": []
            })],
        ),
    );
    let store_dir = build_store_from_objects(&[
        admitted_patch,
        admitted_revision.clone(),
        non_admitted_patch,
        non_admitted_revision.clone(),
    ]);
    let mut profile = head_profile(hash_json(&policy), 1200);
    profile["editor_admission"] = json!({
        "mode": "admitted-only",
        "admitted_keys": [signer_id(&admitted_author)]
    });
    let input = write_input_file(
        "head-render-store-editor-admission",
        "input.json",
        json!({
            "profile": profile,
            "revisions": [],
            "views": [],
            "critical_violations": []
        }),
    );

    let output = run_mycel(&[
        "head",
        "render",
        doc_id,
        "--input",
        &path_arg(&input.path),
        "--store-root",
        &path_arg(&store_dir.path().to_path_buf()),
        "--json",
    ]);

    assert_success(&output);
    let json = parse_json_stdout(&output);
    assert_eq!(json["selected_head"], admitted_revision["revision_id"]);
    assert_eq!(json["rendered_text"], "Admitted render line");
    assert!(
        json["notes"]
            .as_array()
            .is_some_and(|notes| notes.iter().any(|entry| entry
                .as_str()
                .is_some_and(|message| message.contains("store-backed replay")))),
        "expected store-backed render note, stdout: {}",
        stdout_text(&output)
    );
}

#[test]
fn head_render_text_reports_rendered_text() {
    let doc_id = "doc:render-text";
    let revision_author = signing_key(82);
    let maintainer = signing_key(93);
    let policy = json!({
        "accept_keys": [signer_id(&maintainer)],
        "merge_rule": "manual-reviewed",
        "preferred_branches": ["main"]
    });
    let genesis_hash = empty_document_state_hash(doc_id);
    let revision_a = signed_revision(&revision_author, doc_id, vec![], 1000, &genesis_hash);
    let patch = signed_patch(
        &revision_author,
        doc_id,
        revision_a["revision_id"]
            .as_str()
            .expect("revision id should exist"),
        1010,
        json!([
            {
                "op": "insert_block",
                "new_block": {
                    "block_id": "blk:render-text-001",
                    "block_type": "paragraph",
                    "content": "Rendered line",
                    "attrs": {},
                    "children": []
                }
            }
        ]),
    );
    let revision_b = signed_revision_with_patches(
        &revision_author,
        doc_id,
        vec![revision_a["revision_id"]
            .as_str()
            .expect("revision id should exist")
            .to_string()],
        vec![patch["patch_id"]
            .as_str()
            .expect("patch id should exist")
            .to_string()],
        1020,
        &document_state_hash(
            doc_id,
            vec![json!({
                "block_id": "blk:render-text-001",
                "block_type": "paragraph",
                "content": "Rendered line",
                "attrs": {},
                "children": []
            })],
        ),
    );
    let view = signed_view(
        &maintainer,
        &policy,
        documents_value(doc_id, &revision_b["revision_id"]),
        1100,
    );
    let store_dir = build_store_from_objects(&[revision_a, patch, revision_b, view]);
    let input = write_input_file(
        "head-render-store-backed-text",
        "input.json",
        json!({
            "profile": head_profile(hash_json(&policy), 1200),
            "revisions": [],
            "views": [],
            "critical_violations": []
        }),
    );

    let output = run_mycel(&[
        "head",
        "render",
        doc_id,
        "--input",
        &path_arg(&input.path),
        "--store-root",
        &path_arg(&store_dir.path().to_path_buf()),
    ]);

    assert_success(&output);
    assert_stdout_contains(&output, "head render: ok");
    assert_stdout_contains(&output, "rendered text:");
    assert_stdout_contains(&output, "Rendered line");
}

#[test]
fn head_render_json_replays_selected_head_from_bundle_objects() {
    let doc_id = "doc:render-bundle";
    let revision_author = signing_key(83);
    let maintainer = signing_key(94);
    let policy = json!({
        "accept_keys": [signer_id(&maintainer)],
        "merge_rule": "manual-reviewed",
        "preferred_branches": ["main"]
    });
    let genesis_hash = empty_document_state_hash(doc_id);
    let revision_a = signed_revision(&revision_author, doc_id, vec![], 1000, &genesis_hash);
    let patch = signed_patch(
        &revision_author,
        doc_id,
        revision_a["revision_id"]
            .as_str()
            .expect("revision id should exist"),
        1010,
        json!([
            {
                "op": "insert_block",
                "new_block": {
                    "block_id": "blk:render-bundle-001",
                    "block_type": "paragraph",
                    "content": "Bundle line",
                    "attrs": {},
                    "children": []
                }
            }
        ]),
    );
    let revision_b = signed_revision_with_patches(
        &revision_author,
        doc_id,
        vec![revision_a["revision_id"]
            .as_str()
            .expect("revision id should exist")
            .to_string()],
        vec![patch["patch_id"]
            .as_str()
            .expect("patch id should exist")
            .to_string()],
        1020,
        &document_state_hash(
            doc_id,
            vec![json!({
                "block_id": "blk:render-bundle-001",
                "block_type": "paragraph",
                "content": "Bundle line",
                "attrs": {},
                "children": []
            })],
        ),
    );
    let view = signed_view(
        &maintainer,
        &policy,
        documents_value(doc_id, &revision_b["revision_id"]),
        1100,
    );
    let input = write_input_file(
        "head-render-bundle-backed",
        "input.json",
        json!({
            "profile": head_profile(hash_json(&policy), 1200),
            "revisions": [revision_a, revision_b.clone()],
            "objects": [patch],
            "views": [view],
            "critical_violations": []
        }),
    );

    let output = run_mycel(&[
        "head",
        "render",
        doc_id,
        "--input",
        &path_arg(&input.path),
        "--json",
    ]);

    assert_success(&output);
    let json = parse_json_stdout(&output);
    assert_eq!(json["status"], "ok");
    assert_eq!(json["selected_head"], revision_b["revision_id"]);
    assert_eq!(json["rendered_text"], "Bundle line");
    assert_eq!(json["store_root"], Value::Null);
    assert!(
        json["notes"]
            .as_array()
            .is_some_and(|notes| notes.iter().any(|entry| entry
                .as_str()
                .is_some_and(|message| message.contains("bundle-backed replay objects")))),
        "expected bundle-backed render note, stdout: {}",
        stdout_text(&output)
    );
}

#[test]
fn head_render_json_uses_requested_named_profile_from_bundle() {
    let doc_id = "doc:render-named-profile";
    let revision_author = signing_key(85);
    let maintainer = signing_key(96);
    let policy = json!({
        "accept_keys": [signer_id(&maintainer)],
        "merge_rule": "manual-reviewed",
        "preferred_branches": ["main"]
    });
    let genesis_hash = empty_document_state_hash(doc_id);
    let revision_a = signed_revision(&revision_author, doc_id, vec![], 1000, &genesis_hash);
    let patch = signed_patch(
        &revision_author,
        doc_id,
        revision_a["revision_id"]
            .as_str()
            .expect("revision id should exist"),
        1010,
        json!([
            {
                "op": "insert_block",
                "new_block": {
                    "block_id": "blk:render-profile-001",
                    "block_type": "paragraph",
                    "content": "Preview line",
                    "attrs": {},
                    "children": []
                }
            }
        ]),
    );
    let revision_b = signed_revision_with_patches(
        &revision_author,
        doc_id,
        vec![revision_a["revision_id"]
            .as_str()
            .expect("revision id should exist")
            .to_string()],
        vec![patch["patch_id"]
            .as_str()
            .expect("patch id should exist")
            .to_string()],
        1020,
        &document_state_hash(
            doc_id,
            vec![json!({
                "block_id": "blk:render-profile-001",
                "block_type": "paragraph",
                "content": "Preview line",
                "attrs": {},
                "children": []
            })],
        ),
    );
    let bundle = json!({
        "profiles": named_profiles(&[
            ("stable", head_profile(hash_json(&policy), 1005)),
            ("preview", head_profile(hash_json(&policy), 1200))
        ]),
        "revisions": [revision_a.clone(), revision_b.clone()],
        "objects": [patch],
        "views": [
            signed_view(
                &maintainer,
                &policy,
                documents_value(doc_id, &revision_a["revision_id"]),
                1002
            ),
            signed_view(
                &maintainer,
                &policy,
                documents_value(doc_id, &revision_b["revision_id"]),
                1100
            )
        ],
        "critical_violations": []
    });
    let input = write_input_file("head-render-named-profile", "input.json", bundle);
    let output = run_mycel(&[
        "head",
        "render",
        doc_id,
        "--input",
        &path_arg(&input.path),
        "--profile-id",
        "preview",
        "--json",
    ]);

    assert_success(&output);
    let json = parse_json_stdout(&output);
    assert_eq!(json["profile_id"], "preview");
    assert_eq!(json["selected_head"], revision_b["revision_id"]);
    assert_eq!(json["rendered_text"], "Preview line");
}

#[test]
fn head_render_named_profile_applies_requested_editor_admission_mode() {
    let doc_id = "doc:render-editor-profile";
    let admitted_author = signing_key(89);
    let non_admitted_author = signing_key(90);
    let policy = json!({
        "merge_rule": "manual-reviewed",
        "preferred_branches": ["main"]
    });
    let admitted_patch = signed_patch(
        &admitted_author,
        doc_id,
        "rev:genesis-null",
        1000,
        json!([
            {
                "op": "insert_block",
                "new_block": {
                    "block_id": "blk:render-profile-admitted-001",
                    "block_type": "paragraph",
                    "content": "Named admitted line",
                    "attrs": {},
                    "children": []
                }
            }
        ]),
    );
    let admitted_revision = signed_revision_with_patches(
        &admitted_author,
        doc_id,
        vec![],
        vec![admitted_patch["patch_id"]
            .as_str()
            .expect("patch id should exist")
            .to_string()],
        1010,
        &document_state_hash(
            doc_id,
            vec![json!({
                "block_id": "blk:render-profile-admitted-001",
                "block_type": "paragraph",
                "content": "Named admitted line",
                "attrs": {},
                "children": []
            })],
        ),
    );
    let non_admitted_patch = signed_patch(
        &non_admitted_author,
        doc_id,
        "rev:genesis-null",
        1020,
        json!([
            {
                "op": "insert_block",
                "new_block": {
                    "block_id": "blk:render-profile-non-admitted-001",
                    "block_type": "paragraph",
                    "content": "Named non-admitted line",
                    "attrs": {},
                    "children": []
                }
            }
        ]),
    );
    let non_admitted_revision = signed_revision_with_patches(
        &non_admitted_author,
        doc_id,
        vec![],
        vec![non_admitted_patch["patch_id"]
            .as_str()
            .expect("patch id should exist")
            .to_string()],
        1030,
        &document_state_hash(
            doc_id,
            vec![json!({
                "block_id": "blk:render-profile-non-admitted-001",
                "block_type": "paragraph",
                "content": "Named non-admitted line",
                "attrs": {},
                "children": []
            })],
        ),
    );
    let stable = head_profile(hash_json(&policy), 1200);
    let mut preview = head_profile(hash_json(&policy), 1200);
    preview["editor_admission"] = json!({
        "mode": "admitted-only",
        "admitted_keys": [signer_id(&admitted_author)]
    });
    let bundle = json!({
        "profiles": named_profiles(&[
            ("stable", stable),
            ("preview", preview)
        ]),
        "revisions": [admitted_revision.clone(), non_admitted_revision],
        "objects": [admitted_patch, non_admitted_patch],
        "views": [],
        "critical_violations": []
    });
    let input = write_input_file("head-render-editor-profile", "input.json", bundle);
    let output = run_mycel(&[
        "head",
        "render",
        doc_id,
        "--input",
        &path_arg(&input.path),
        "--profile-id",
        "preview",
        "--json",
    ]);

    assert_success(&output);
    let json = parse_json_stdout(&output);
    assert_eq!(json["profile_id"], "preview");
    assert_eq!(json["selected_head"], admitted_revision["revision_id"]);
    assert_eq!(json["rendered_text"], "Named admitted line");
}

#[test]
fn head_render_bundle_reports_missing_replay_objects() {
    let doc_id = "doc:render-missing-objects";
    let revision_author = signing_key(84);
    let maintainer = signing_key(95);
    let policy = json!({
        "accept_keys": [signer_id(&maintainer)],
        "merge_rule": "manual-reviewed",
        "preferred_branches": ["main"]
    });
    let genesis_hash = empty_document_state_hash(doc_id);
    let revision_a = signed_revision(&revision_author, doc_id, vec![], 1000, &genesis_hash);
    let patch = signed_patch(
        &revision_author,
        doc_id,
        revision_a["revision_id"]
            .as_str()
            .expect("revision id should exist"),
        1010,
        json!([
            {
                "op": "insert_block",
                "new_block": {
                    "block_id": "blk:render-missing-001",
                    "block_type": "paragraph",
                    "content": "Missing patch payload",
                    "attrs": {},
                    "children": []
                }
            }
        ]),
    );
    let revision_b = signed_revision_with_patches(
        &revision_author,
        doc_id,
        vec![revision_a["revision_id"]
            .as_str()
            .expect("revision id should exist")
            .to_string()],
        vec![patch["patch_id"]
            .as_str()
            .expect("patch id should exist")
            .to_string()],
        1020,
        &document_state_hash(
            doc_id,
            vec![json!({
                "block_id": "blk:render-missing-001",
                "block_type": "paragraph",
                "content": "Missing patch payload",
                "attrs": {},
                "children": []
            })],
        ),
    );
    let view = signed_view(
        &maintainer,
        &policy,
        documents_value(doc_id, &revision_b["revision_id"]),
        1100,
    );
    let input = write_input_file(
        "head-render-bundle-missing-object",
        "input.json",
        json!({
            "profile": head_profile(hash_json(&policy), 1200),
            "revisions": [revision_a, revision_b],
            "views": [view],
            "critical_violations": []
        }),
    );

    let output = run_mycel(&[
        "head",
        "render",
        doc_id,
        "--input",
        &path_arg(&input.path),
        "--json",
    ]);

    assert_exit_code(&output, 1);
    let json = parse_json_stdout(&output);
    assert_eq!(json["status"], "failed");
    assert!(
        json["errors"]
            .as_array()
            .is_some_and(|errors| errors.iter().any(|entry| entry
                .as_str()
                .is_some_and(|message| message.contains("missing patch")))),
        "expected missing patch error, stdout: {}",
        stdout_text(&output)
    );
}

#[test]
fn head_render_bundle_reports_multi_hop_ancestry_context() {
    let doc_id = "doc:render-bundle-ancestry";
    let revision_author = signing_key(85);
    let maintainer = signing_key(96);
    let policy = json!({
        "accept_keys": [signer_id(&maintainer)],
        "merge_rule": "manual-reviewed",
        "preferred_branches": ["main"]
    });
    let genesis_hash = empty_document_state_hash(doc_id);
    let parent_revision = signed_revision_with_patches(
        &revision_author,
        doc_id,
        vec![],
        vec!["patch:missing-ancestor".to_string()],
        1000,
        &genesis_hash,
    );
    let revision_b = signed_revision(
        &revision_author,
        doc_id,
        vec![parent_revision["revision_id"]
            .as_str()
            .expect("parent revision id should exist")
            .to_string()],
        1010,
        &genesis_hash,
    );
    let view = signed_view(
        &maintainer,
        &policy,
        documents_value(doc_id, &revision_b["revision_id"]),
        1100,
    );
    let input = write_input_file(
        "head-render-bundle-ancestry",
        "input.json",
        json!({
            "profile": head_profile(hash_json(&policy), 1200),
            "revisions": [parent_revision.clone(), revision_b],
            "views": [view],
            "critical_violations": []
        }),
    );

    let output = run_mycel(&[
        "head",
        "render",
        doc_id,
        "--input",
        &path_arg(&input.path),
        "--json",
    ]);

    assert_exit_code(&output, 1);
    let json = parse_json_stdout(&output);
    assert_eq!(json["status"], "failed");
    let parent_revision_id = parent_revision["revision_id"]
        .as_str()
        .expect("parent revision id should exist");
    assert!(
        json["errors"]
            .as_array()
            .is_some_and(|errors| errors.iter().any(|entry| {
                entry.as_str().is_some_and(|message| {
                    message.contains("failed verification before render replay")
                        && message.contains(&format!(
                            "while verifying ancestry through parent revision '{parent_revision_id}'"
                        ))
                        && message.contains("missing patch 'patch:missing-ancestor' for replay")
                })
            })),
        "expected nested ancestry-context render error, stdout: {}",
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
    assert_stderr_starts_with(&output, "error: ");
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
    assert!(
        !stdout_text(&output).contains("pk:ed25519:"),
        "expected high-level decision trace only, stdout: {}",
        stdout_text(&output)
    );
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
                    && entry["detail"].as_str().is_some_and(|detail| {
                        detail.contains("admitted=2")
                            && detail.contains("zero_weight=1")
                            && detail.contains("max_effective_weight=2")
                    })
            })),
        "expected effective_weight trace entry, stdout: {}",
        stdout_text(&output)
    );
}

#[test]
fn head_inspect_applies_bounded_viewer_score_channels() {
    let doc_id = "doc:viewer-score";
    let revision_author = signing_key(91);
    let maintainer_a = signing_key(92);
    let maintainer_b = signing_key(93);
    let policy = json!({
        "accept_keys": [
            signer_id(&maintainer_a),
            signer_id(&maintainer_b)
        ],
        "merge_rule": "manual-reviewed",
        "preferred_branches": ["main"]
    });
    let revision_a = signed_revision(&revision_author, doc_id, vec![], 10, "hash:viewer-score-a");
    let revision_b = signed_revision(&revision_author, doc_id, vec![], 20, "hash:viewer-score-b");
    let mut profile = head_profile(hash_json(&policy), 250);
    profile["viewer_score"] = bounded_viewer_score_profile();
    let mut challenge = viewer_signal(
        "signal-challenge",
        104,
        &revision_b["revision_id"],
        "challenge",
        "high",
        100,
        400,
    );
    challenge["evidence_ref"] = Value::String("evidence:challenge-1".to_string());
    let bundle = json!({
        "profile": profile,
        "revisions": [revision_a.clone(), revision_b.clone()],
        "views": [
            signed_view(
                &maintainer_a,
                &policy,
                documents_value(doc_id, &revision_a["revision_id"]),
                100
            ),
            signed_view(
                &maintainer_b,
                &policy,
                documents_value(doc_id, &revision_b["revision_id"]),
                110
            )
        ],
        "viewer_signals": [
            viewer_signal(
                "signal-approval-low",
                101,
                &revision_a["revision_id"],
                "approval",
                "low",
                100,
                400
            ),
            viewer_signal(
                "signal-approval-high",
                102,
                &revision_a["revision_id"],
                "approval",
                "high",
                100,
                400
            ),
            viewer_signal(
                "signal-objection-medium",
                103,
                &revision_b["revision_id"],
                "objection",
                "medium",
                100,
                400
            ),
            challenge
        ],
        "critical_violations": []
    });
    let input = write_input_file("head-inspect-viewer-score", "input.json", bundle);
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
    assert_eq!(json["viewer_signal_count"], Value::from(4));
    let eligible_heads = json["eligible_heads"]
        .as_array()
        .expect("eligible_heads should be array");
    let selected = eligible_heads
        .iter()
        .find(|entry| entry["revision_id"] == revision_a["revision_id"])
        .expect("selected viewer-scored head should exist");
    let alternative = eligible_heads
        .iter()
        .find(|entry| entry["revision_id"] == revision_b["revision_id"])
        .expect("alternative viewer-scored head should exist");
    assert_eq!(selected["maintainer_score"], Value::from(1));
    assert_eq!(selected["weighted_support"], Value::from(1));
    assert_eq!(selected["viewer_bonus"], Value::from(2));
    assert_eq!(selected["viewer_penalty"], Value::from(0));
    assert_eq!(selected["selector_score"], Value::from(3));
    assert_eq!(alternative["maintainer_score"], Value::from(1));
    assert_eq!(alternative["viewer_bonus"], Value::from(0));
    assert_eq!(alternative["viewer_penalty"], Value::from(2));
    assert_eq!(alternative["selector_score"], Value::from(0));

    let viewer_signals = json["viewer_signals"]
        .as_array()
        .expect("viewer_signals should be array");
    assert_eq!(viewer_signals.len(), 4);
    let challenge_entry = viewer_signals
        .iter()
        .find(|entry| entry["signal_type"] == Value::String("challenge".to_string()))
        .expect("challenge signal summary should exist");
    assert_eq!(challenge_entry["selector_eligible"], Value::Bool(true));
    assert_eq!(challenge_entry["effective_signal_weight"], Value::from(0));
    assert_eq!(
        challenge_entry["signal_status"],
        Value::String("active".to_string())
    );

    let viewer_score_channels = json["viewer_score_channels"]
        .as_array()
        .expect("viewer_score_channels should be array");
    let selected_channel = viewer_score_channels
        .iter()
        .find(|entry| entry["revision_id"] == revision_a["revision_id"])
        .expect("selected viewer score channel should exist");
    assert_eq!(selected_channel["maintainer_score"], Value::from(1));
    assert_eq!(selected_channel["viewer_bonus"], Value::from(2));
    assert_eq!(selected_channel["viewer_penalty"], Value::from(0));
    assert_eq!(selected_channel["approval_signal_count"], Value::from(2));
    assert_eq!(selected_channel["challenge_signal_count"], Value::from(0));
    assert_eq!(
        selected_channel["challenge_review_pressure"],
        Value::from(0)
    );
    assert_eq!(
        selected_channel["challenge_freeze_pressure"],
        Value::from(0)
    );
    assert_eq!(
        selected_channel["viewer_review_state"],
        Value::String("none".to_string())
    );
    assert_eq!(selected_channel["selector_score"], Value::from(3));
    let alternative_channel = viewer_score_channels
        .iter()
        .find(|entry| entry["revision_id"] == revision_b["revision_id"])
        .expect("alternative viewer score channel should exist");
    assert_eq!(alternative_channel["viewer_bonus"], Value::from(0));
    assert_eq!(alternative_channel["viewer_penalty"], Value::from(2));
    assert_eq!(
        alternative_channel["objection_signal_count"],
        Value::from(1)
    );
    assert_eq!(
        alternative_channel["challenge_signal_count"],
        Value::from(1)
    );
    assert_eq!(
        alternative_channel["challenge_review_pressure"],
        Value::from(2)
    );
    assert_eq!(
        alternative_channel["challenge_freeze_pressure"],
        Value::from(2)
    );
    assert_eq!(
        alternative_channel["viewer_review_state"],
        Value::String("freeze-pressure".to_string())
    );
    assert_eq!(alternative_channel["selector_score"], Value::from(0));

    assert!(
        json["decision_trace"]
            .as_array()
            .is_some_and(|trace| trace.iter().any(|entry| {
                entry["step"].as_str() == Some("viewer_score_channels")
                    && entry["detail"].as_str().is_some_and(|detail| {
                        detail.contains("mode=bounded-bonus-penalty")
                            && detail.contains("signals=4")
                            && detail.contains("contributing=3")
                            && detail.contains("bonus_cap=2")
                            && detail.contains("penalty_cap=2")
                    })
            })),
        "expected viewer_score_channels trace entry, stdout: {}",
        stdout_text(&output)
    );
}

#[test]
fn head_inspect_text_reports_viewer_channels_without_overloading_trace() {
    let doc_id = "doc:viewer-score-text";
    let revision_author = signing_key(111);
    let maintainer_a = signing_key(112);
    let maintainer_b = signing_key(113);
    let policy = json!({
        "accept_keys": [
            signer_id(&maintainer_a),
            signer_id(&maintainer_b)
        ],
        "merge_rule": "manual-reviewed",
        "preferred_branches": ["main"]
    });
    let revision_a = signed_revision(
        &revision_author,
        doc_id,
        vec![],
        10,
        "hash:viewer-score-text-a",
    );
    let revision_b = signed_revision(
        &revision_author,
        doc_id,
        vec![],
        20,
        "hash:viewer-score-text-b",
    );
    let mut profile = head_profile(hash_json(&policy), 250);
    profile["viewer_score"] = bounded_viewer_score_profile();
    let mut challenge = viewer_signal(
        "signal-challenge-text",
        114,
        &revision_b["revision_id"],
        "challenge",
        "high",
        100,
        400,
    );
    challenge["evidence_ref"] = Value::String("evidence:challenge-text".to_string());
    let bundle = json!({
        "profile": profile,
        "revisions": [revision_a.clone(), revision_b.clone()],
        "views": [
            signed_view(
                &maintainer_a,
                &policy,
                documents_value(doc_id, &revision_a["revision_id"]),
                100
            ),
            signed_view(
                &maintainer_b,
                &policy,
                documents_value(doc_id, &revision_b["revision_id"]),
                110
            )
        ],
        "viewer_signals": [
            viewer_signal(
                "signal-approval-text",
                115,
                &revision_a["revision_id"],
                "approval",
                "medium",
                100,
                400
            ),
            challenge
        ],
        "critical_violations": []
    });
    let input = write_input_file("head-inspect-viewer-score-text", "input.json", bundle);
    let output = run_mycel(&["head", "inspect", doc_id, "--input", &path_arg(&input.path)]);

    assert_success(&output);
    assert_stdout_contains(&output, "viewer signals: 2");
    assert_stdout_contains(&output, "viewer channel: ");
    assert_stdout_contains(&output, "review_pressure=2");
    assert_stdout_contains(&output, "freeze_pressure=2");
    assert_stdout_contains(&output, "review_state=freeze-pressure");
    assert!(
        !stdout_text(&output).contains("trace: viewer_signal_id"),
        "expected trace to stay high-level, stdout: {}",
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
                        detail.contains("penalized=1")
                            && detail.contains("zero_weight=1")
                            && detail.contains("max_effective_weight=1")
                    })
            })),
        "expected penalty trace entry, stdout: {}",
        stdout_text(&output)
    );
    assert!(
        json["decision_trace"]
            .as_array()
            .is_some_and(|trace| trace.iter().any(|entry| {
                entry["step"].as_str() == Some("critical_violations")
                    && entry["detail"]
                        .as_str()
                        .is_some_and(|detail| detail == "count=1 affected_maintainers=1")
            })),
        "expected critical_violations trace summary, stdout: {}",
        stdout_text(&output)
    );
}

#[test]
fn head_inspect_admitted_only_editor_policy_filters_non_admitted_candidate_heads() {
    let doc_id = "doc:editor-admitted-only";
    let admitted_author = signing_key(61);
    let non_admitted_author = signing_key(62);
    let policy = json!({
        "merge_rule": "manual-reviewed",
        "preferred_branches": ["main"]
    });
    let admitted_revision =
        signed_revision(&admitted_author, doc_id, vec![], 10, "hash:editor-admitted");
    let non_admitted_revision = signed_revision(
        &non_admitted_author,
        doc_id,
        vec![],
        20,
        "hash:editor-non-admitted",
    );
    let mut profile = head_profile(hash_json(&policy), 1200);
    profile["editor_admission"] = json!({
        "mode": "admitted-only",
        "admitted_keys": [signer_id(&admitted_author)]
    });
    let bundle = json!({
        "profile": profile,
        "revisions": [admitted_revision.clone(), non_admitted_revision.clone()],
        "views": [],
        "critical_violations": []
    });
    let input = write_input_file("head-inspect-editor-admitted-only", "input.json", bundle);
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
    assert_eq!(json["selected_head"], admitted_revision["revision_id"]);
    let editor_candidates = json["editor_candidates"]
        .as_array()
        .expect("editor_candidates should be array");
    assert!(
        editor_candidates.iter().any(|entry| {
            entry["revision_id"] == admitted_revision["revision_id"]
                && entry["editor_admitted"] == Value::Bool(true)
                && entry["candidate_eligible"] == Value::Bool(true)
                && entry["formal_candidate"] == Value::Bool(true)
        }),
        "expected admitted editor candidate summary, stdout: {}",
        stdout_text(&output)
    );
    assert!(
        editor_candidates.iter().any(|entry| {
            entry["revision_id"] == non_admitted_revision["revision_id"]
                && entry["editor_admitted"] == Value::Bool(false)
                && entry["candidate_eligible"] == Value::Bool(false)
                && entry["formal_candidate"] == Value::Bool(false)
        }),
        "expected filtered editor candidate summary, stdout: {}",
        stdout_text(&output)
    );
    let eligible_heads = json["eligible_heads"]
        .as_array()
        .expect("eligible_heads should be array");
    assert_eq!(eligible_heads.len(), 1);
    assert_eq!(
        eligible_heads[0]["revision_id"],
        admitted_revision["revision_id"]
    );
    assert_eq!(
        eligible_heads[0]["author"],
        Value::String(signer_id(&admitted_author))
    );
    assert_eq!(eligible_heads[0]["editor_admitted"], Value::Bool(true));
    assert_eq!(eligible_heads[0]["formal_candidate"], Value::Bool(true));
    assert!(
        json["decision_trace"]
            .as_array()
            .is_some_and(|trace| trace.iter().any(|entry| {
                entry["step"].as_str() == Some("editor_admission")
                    && entry["detail"].as_str().is_some_and(|detail| {
                        detail.contains("mode=admitted-only")
                            && detail.contains("structural_heads=2")
                            && detail.contains("eligible=1")
                            && detail.contains("formal=1")
                    })
            })),
        "expected editor_admission trace entry, stdout: {}",
        stdout_text(&output)
    );
}

#[test]
fn head_inspect_mixed_editor_policy_marks_formal_candidates_without_filtering_selection() {
    let doc_id = "doc:editor-mixed";
    let admitted_author = signing_key(71);
    let non_admitted_author = signing_key(72);
    let policy = json!({
        "merge_rule": "manual-reviewed",
        "preferred_branches": ["main"]
    });
    let admitted_revision = signed_revision(
        &admitted_author,
        doc_id,
        vec![],
        10,
        "hash:editor-mixed-admitted",
    );
    let non_admitted_revision = signed_revision(
        &non_admitted_author,
        doc_id,
        vec![],
        20,
        "hash:editor-mixed-non-admitted",
    );
    let mut profile = head_profile(hash_json(&policy), 1200);
    profile["editor_admission"] = json!({
        "mode": "mixed",
        "admitted_keys": [signer_id(&admitted_author)]
    });
    let bundle = json!({
        "profile": profile,
        "revisions": [admitted_revision.clone(), non_admitted_revision.clone()],
        "views": [],
        "critical_violations": []
    });
    let input = write_input_file("head-inspect-editor-mixed", "input.json", bundle);
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
    assert_eq!(json["selected_head"], non_admitted_revision["revision_id"]);
    let eligible_heads = json["eligible_heads"]
        .as_array()
        .expect("eligible_heads should be array");
    assert_eq!(eligible_heads.len(), 2);
    assert!(
        eligible_heads.iter().any(|entry| {
            entry["revision_id"] == admitted_revision["revision_id"]
                && entry["formal_candidate"] == Value::Bool(true)
        }),
        "expected admitted formal candidate head, stdout: {}",
        stdout_text(&output)
    );
    assert!(
        eligible_heads.iter().any(|entry| {
            entry["revision_id"] == non_admitted_revision["revision_id"]
                && entry["editor_admitted"] == Value::Bool(false)
                && entry["formal_candidate"] == Value::Bool(false)
        }),
        "expected mixed-mode informal candidate head, stdout: {}",
        stdout_text(&output)
    );
    assert!(
        json["decision_trace"]
            .as_array()
            .is_some_and(|trace| trace.iter().any(|entry| {
                entry["step"].as_str() == Some("editor_admission")
                    && entry["detail"].as_str().is_some_and(|detail| {
                        detail.contains("mode=mixed")
                            && detail.contains("structural_heads=2")
                            && detail.contains("eligible=2")
                            && detail.contains("formal=1")
                    })
            })),
        "expected mixed editor_admission trace entry, stdout: {}",
        stdout_text(&output)
    );
}

#[test]
fn head_inspect_requires_input_path() {
    let output = run_mycel(&["head", "inspect", "doc:sample"]);

    assert_exit_code(&output, 2);
    assert_stderr_contains(&output, "required arguments were not provided");
    assert_stderr_contains(&output, "--input <PATH_OR_FIXTURE>");
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
    assert_top_level_help(&stdout_text(&output));
}
