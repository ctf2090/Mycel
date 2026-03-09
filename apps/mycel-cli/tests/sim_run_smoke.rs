use std::sync::{Mutex, MutexGuard, OnceLock};

use serde_json::Value;

mod common;

use common::{
    assert_exit_code, assert_stderr_contains, load_report, parse_json_stdout, run_sim,
    stderr_text, validate_generated_report,
};

fn sim_run_lock() -> MutexGuard<'static, ()> {
    static LOCK: OnceLock<Mutex<()>> = OnceLock::new();
    LOCK.get_or_init(|| Mutex::new(()))
        .lock()
        .expect("sim run test lock should not be poisoned")
}

fn assert_runtime_seed_mode(summary: &Value, report: &Value, expected_source: &str) {
    let deterministic_seed = summary["deterministic_seed"]
        .as_str()
        .expect("deterministic_seed should be a string");
    assert_eq!(summary["seed_source"], expected_source);
    assert!(
        deterministic_seed.starts_with(&format!("{expected_source}:")),
        "expected seed '{deterministic_seed}' to start with '{expected_source}:'"
    );

    let metadata = report["metadata"]
        .as_object()
        .expect("report metadata should be an object");
    assert_eq!(metadata["seed_source"], expected_source);
    assert_eq!(metadata["deterministic_seed"], deterministic_seed);
}

#[test]
fn three_peer_consistency_run_produces_pass_report() {
    let _guard = sim_run_lock();
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

    let validation = validate_generated_report(&summary);
    assert_eq!(validation["report_count"], 1);
}

#[test]
fn hash_mismatch_run_produces_fault_plan_and_fail_result() {
    let _guard = sim_run_lock();
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

    let validation = validate_generated_report(&summary);
    assert_eq!(validation["report_count"], 1);
}

#[test]
fn hash_mismatch_run_supports_random_seed_mode() {
    let _guard = sim_run_lock();
    let output = run_sim(&[
        "sim",
        "run",
        "sim/tests/hash-mismatch.example.json",
        "--json",
        "--seed",
        "random",
    ]);

    assert!(
        output.status.success(),
        "expected success, stderr: {}",
        String::from_utf8_lossy(&output.stderr)
    );

    let summary = parse_json_stdout(&output);
    assert_eq!(summary["result"], "fail");
    assert_eq!(summary["validation_status"], "ok");

    let report = load_report(&summary);
    assert_runtime_seed_mode(&summary, &report, "random");

    let validation = validate_generated_report(&summary);
    assert_eq!(validation["status"], "ok");
}

#[test]
fn hash_mismatch_run_supports_auto_seed_mode() {
    let _guard = sim_run_lock();
    let output = run_sim(&[
        "sim",
        "run",
        "sim/tests/hash-mismatch.example.json",
        "--json",
        "--seed",
        "auto",
    ]);

    assert!(
        output.status.success(),
        "expected success, stderr: {}",
        String::from_utf8_lossy(&output.stderr)
    );

    let summary = parse_json_stdout(&output);
    assert_eq!(summary["result"], "fail");
    assert_eq!(summary["validation_status"], "ok");

    let report = load_report(&summary);
    assert_runtime_seed_mode(&summary, &report, "auto");

    let validation = validate_generated_report(&summary);
    assert_eq!(validation["status"], "ok");
}

#[test]
fn partial_want_recovery_run_records_recovery_flow() {
    let _guard = sim_run_lock();
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

    let validation = validate_generated_report(&summary);
    assert_eq!(validation["report_count"], 1);
}

#[test]
fn sim_run_rejects_schema_file_targets() {
    let _guard = sim_run_lock();
    let output = run_sim(&["sim", "run", "sim/tests/test-case.schema.json"]);

    assert_exit_code(&output, 1);
    assert_stderr_contains(&output, "schema files are not");
}

#[test]
fn sim_run_requires_seed_value_after_flag() {
    let _guard = sim_run_lock();
    let output = run_sim(&[
        "sim",
        "run",
        "sim/tests/hash-mismatch.example.json",
        "--seed",
    ]);

    assert_exit_code(&output, 2);
    assert_stderr_contains(&output, "missing value for --seed");
}

#[test]
fn sim_run_rejects_unexpected_extra_arguments() {
    let _guard = sim_run_lock();
    let output = run_sim(&[
        "sim",
        "run",
        "sim/tests/hash-mismatch.example.json",
        "--json",
        "unexpected",
    ]);

    assert_exit_code(&output, 2);
    assert_stderr_contains(&output, "unexpected sim run argument: unexpected");
}

#[test]
fn sim_requires_subcommand() {
    let _guard = sim_run_lock();
    let output = run_sim(&["sim"]);

    assert_exit_code(&output, 2);
    assert_stderr_contains(&output, "missing sim subcommand");
}

#[test]
fn sim_rejects_unknown_subcommand() {
    let _guard = sim_run_lock();
    let output = run_sim(&["sim", "bogus"]);

    assert_exit_code(&output, 2);
    assert_stderr_contains(&output, "unknown sim subcommand: bogus");
}

#[test]
fn sim_run_requires_target_path() {
    let _guard = sim_run_lock();
    let output = run_sim(&["sim", "run"]);

    assert_exit_code(&output, 2);
    assert_stderr_contains(&output, "missing sim run target");
}

#[test]
fn sim_run_rejects_directory_targets() {
    let _guard = sim_run_lock();
    let output = run_sim(&["sim", "run", "sim/tests"]);

    assert_exit_code(&output, 1);
    let stderr = stderr_text(&output);
    assert!(
        stderr.contains("failed to read") && stderr.contains("Is a directory"),
        "expected directory target read failure, stderr: {stderr}"
    );
}
