use std::fs;
use std::path::{Path, PathBuf};
use std::process::{Command, Output};

use serde_json::Value;

fn repo_root() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("../..")
        .canonicalize()
        .expect("repo root should resolve")
}

fn mycel_bin() -> PathBuf {
    PathBuf::from(env!("CARGO_BIN_EXE_mycel"))
}

fn run_sim(args: &[&str]) -> Output {
    Command::new(mycel_bin())
        .current_dir(repo_root())
        .args(args)
        .output()
        .expect("sim command should run")
}

fn parse_json_stdout(output: &Output) -> Value {
    serde_json::from_slice(&output.stdout).expect("stdout should contain valid JSON")
}

fn load_report(summary: &Value) -> Value {
    let report_path = summary["report_path"]
        .as_str()
        .expect("report_path should be a string");
    let content = fs::read_to_string(report_path).expect("report file should exist");
    serde_json::from_str(&content).expect("report file should contain valid JSON")
}

#[test]
fn three_peer_consistency_run_produces_pass_report() {
    let output = run_sim(&[
        "sim",
        "run",
        "sim/tests/three-peer-consistency.example.json",
        "--json",
    ]);

    assert!(
        output.status.success(),
        "expected success, stderr: {}",
        String::from_utf8_lossy(&output.stderr)
    );

    let summary = parse_json_stdout(&output);
    assert_eq!(summary["result"], "pass");
    assert_eq!(summary["seed_source"], "derived");
    assert_eq!(summary["validation_status"], "ok");
    assert_eq!(summary["peer_count"], 3);
    assert!(
        summary["event_count"]
            .as_u64()
            .expect("event_count should be numeric")
            >= 1,
        "expected events in run summary"
    );

    let outcomes = summary["matched_expected_outcomes"]
        .as_array()
        .expect("matched_expected_outcomes should be an array");
    assert!(
        outcomes.iter().any(|entry| entry == "sync-success"),
        "expected sync-success outcome, stdout: {}",
        String::from_utf8_lossy(&output.stdout)
    );

    let report = load_report(&summary);
    assert_eq!(report["result"], "pass");
    let events = report["events"]
        .as_array()
        .expect("events should be an array");
    assert!(
        events
            .iter()
            .any(|entry| entry["action"] == "seed-advertise"),
        "expected seed-advertise event in report"
    );
}

#[test]
fn hash_mismatch_run_produces_fault_plan_and_fail_result() {
    let output = run_sim(&[
        "sim",
        "run",
        "sim/tests/hash-mismatch.example.json",
        "--json",
        "--seed",
        "custom-seed",
    ]);

    assert!(
        output.status.success(),
        "expected success, stderr: {}",
        String::from_utf8_lossy(&output.stderr)
    );

    let summary = parse_json_stdout(&output);
    assert_eq!(summary["result"], "fail");
    assert_eq!(summary["deterministic_seed"], "custom-seed");
    assert_eq!(summary["seed_source"], "override");
    assert_eq!(summary["rejected_object_count"], 1);

    let fault_plan = summary["fault_plan"]
        .as_array()
        .expect("fault_plan should be an array");
    assert_eq!(fault_plan.len(), 1);
    assert_eq!(fault_plan[0]["fault"], "hash-mismatch");

    let report = load_report(&summary);
    let events = report["events"]
        .as_array()
        .expect("events should be an array");
    assert!(
        events.iter().any(|entry| entry["action"] == "inject-fault"),
        "expected inject-fault event in report"
    );
    assert!(
        events
            .iter()
            .any(|entry| entry["action"] == "reject-object-set"),
        "expected reject-object-set event in report"
    );
}

#[test]
fn partial_want_recovery_run_records_recovery_flow() {
    let output = run_sim(&[
        "sim",
        "run",
        "sim/tests/partial-want-recovery.example.json",
        "--json",
    ]);

    assert!(
        output.status.success(),
        "expected success, stderr: {}",
        String::from_utf8_lossy(&output.stderr)
    );

    let summary = parse_json_stdout(&output);
    assert_eq!(summary["result"], "pass");
    assert_eq!(summary["seed_source"], "derived");
    assert_eq!(summary["fault_plan"], Value::Array(Vec::new()));

    let outcomes = summary["matched_expected_outcomes"]
        .as_array()
        .expect("matched_expected_outcomes should be an array");
    assert!(
        outcomes.iter().any(|entry| entry == "recovery-success"),
        "expected recovery-success outcome, stdout: {}",
        String::from_utf8_lossy(&output.stdout)
    );

    let report = load_report(&summary);
    let events = report["events"]
        .as_array()
        .expect("events should be an array");
    assert!(
        events
            .iter()
            .any(|entry| entry["action"] == "request-missing-objects"),
        "expected request-missing-objects event in report"
    );
}
