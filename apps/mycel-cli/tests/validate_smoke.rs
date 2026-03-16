use std::fs;
use std::path::PathBuf;
use std::sync::{Mutex, OnceLock};

use serde_json::Value;

mod common;

use common::{
    assert_exit_code, assert_json_error_contains, assert_json_status, assert_json_warning_contains,
    assert_stderr_contains, assert_stdout_contains, assert_success, create_temp_dir,
    parse_json_stdout, repo_root, run_mycel_in_dir, run_validate, stdout_text,
};

struct TempRepoJsonFile {
    path: PathBuf,
}

impl TempRepoJsonFile {
    fn new(prefix: &str, content: &str) -> Self {
        let unique = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .expect("system time should be after unix epoch")
            .as_nanos();
        let temp_dir = repo_root().join("sim/peers/.tmp");
        fs::create_dir_all(&temp_dir).expect("temporary peer fixture directory should exist");
        let path = temp_dir.join(format!("{prefix}-{unique}.json"));
        fs::write(&path, content).expect("temporary validate fixture should write");
        Self { path }
    }
}

impl Drop for TempRepoJsonFile {
    fn drop(&mut self) {
        let _ = fs::remove_file(&self.path);
        if let Some(parent) = self.path.parent() {
            let _ = fs::remove_dir(parent);
        }
    }
}

fn validate_peer_fixture_lock() -> &'static Mutex<()> {
    static LOCK: OnceLock<Mutex<()>> = OnceLock::new();
    LOCK.get_or_init(|| Mutex::new(()))
}

#[test]
fn repo_validate_json_reports_ok_status() {
    let output = run_validate(&["validate", "--json"]);

    assert_success(&output);
    let json = assert_json_status(&output, "ok");
    assert_eq!(json["errors"], Value::Array(Vec::new()));
}

#[test]
fn validate_outside_repo_reports_root_detection_failure_json() {
    let temp_dir = create_temp_dir("validate-outside-repo");
    let output = run_mycel_in_dir(temp_dir.path(), &["validate", "--json"]);

    assert_exit_code(&output, 1);
    let json = assert_json_status(&output, "failed");
    assert_eq!(json["root"], Value::Null);
    assert_eq!(json["errors"].as_array().map(Vec::len), Some(1));
    assert!(
        json["errors"][0]["message"]
            .as_str()
            .is_some_and(|message| message.contains("could not find repository root")),
        "expected missing repository root error, stdout: {}",
        stdout_text(&output)
    );
}

#[test]
fn fixture_file_validate_json_scopes_related_artifacts() {
    let output = run_validate(&[
        "validate",
        "fixtures/object-sets/minimal-valid/fixture.json",
        "--json",
    ]);

    assert_success(&output);
    let json = assert_json_status(&output, "ok");
    assert_eq!(json["fixture_count"], 1);
    assert_eq!(json["peer_count"], 1);
    assert_eq!(json["topology_count"], 1);
    assert_eq!(json["test_case_count"], 1);
    assert!(
        json["report_count"]
            .as_u64()
            .expect("report_count should be numeric")
            >= 1
    );
}

#[test]
fn fixture_directory_validate_json_scopes_related_artifacts() {
    let output = run_validate(&["validate", "fixtures/object-sets/minimal-valid", "--json"]);

    assert_success(&output);
    let json = assert_json_status(&output, "ok");
    assert_eq!(json["fixture_count"], 1);
    assert_eq!(json["peer_count"], 1);
    assert_eq!(json["topology_count"], 1);
    assert_eq!(json["test_case_count"], 1);
    assert!(
        json["report_count"]
            .as_u64()
            .expect("report_count should be numeric")
            >= 1
    );
}

#[test]
fn repo_validate_text_reports_ok_summary() {
    let output = run_validate(&["validate"]);

    assert_success(&output);
    assert_stdout_contains(&output, "repo root:");
    assert_stdout_contains(&output, "validated target:");
    assert_stdout_contains(&output, "status: ok");
    assert_stdout_contains(&output, "validation: ok");
}

#[test]
fn tests_directory_validate_json_reports_ok_status() {
    let output = run_validate(&["validate", "sim/tests", "--json"]);

    assert_success(&output);
    let json = assert_json_status(&output, "ok");
    assert_eq!(json["peer_count"], 1);
    assert_eq!(json["test_case_count"], 13);
    assert_eq!(json["topology_count"], 13);
}

#[test]
fn peer_file_validate_json_scopes_related_artifacts() {
    let output = run_validate(&["validate", "sim/peers/peer.example.json", "--json"]);

    assert_success(&output);
    let json = assert_json_status(&output, "ok");
    assert_eq!(json["peer_count"], 1);
    assert_eq!(json["topology_count"], 13);
    assert_eq!(json["test_case_count"], 13);
    assert!(
        json["report_count"]
            .as_u64()
            .expect("report_count should be numeric")
            >= 4
    );
}

#[test]
fn validate_json_fails_for_duplicate_keys_in_peer_file() {
    let _guard = validate_peer_fixture_lock()
        .lock()
        .expect("validate peer fixture lock should not be poisoned");
    let peer_file = TempRepoJsonFile::new(
        "duplicate-keys",
        r#"{
  "node_id": "node:dup-a",
  "node_id": "node:dup-b",
  "role": "peer",
  "display_name": "Duplicate peer",
  "transport": {
    "kind": "memory"
  }
}"#,
    );

    let output = run_validate(&["validate", &peer_file.path.to_string_lossy(), "--json"]);

    assert_exit_code(&output, 1);
    let json = assert_json_status(&output, "failed");
    assert!(
        json["errors"]
            .as_array()
            .is_some_and(|errors| errors.iter().any(|entry| {
                entry["message"].as_str().is_some_and(|message| {
                    message.contains("invalid JSON content: duplicate object key 'node_id'")
                })
            })),
        "expected duplicate-key validation error, stdout: {}",
        stdout_text(&output)
    );
}

#[test]
fn validate_json_fails_for_null_values_in_peer_file() {
    let _guard = validate_peer_fixture_lock()
        .lock()
        .expect("validate peer fixture lock should not be poisoned");
    let peer_file = TempRepoJsonFile::new(
        "null-value",
        r#"{
  "node_id": "node:null",
  "role": "peer",
  "display_name": null,
  "transport": {
    "kind": "memory"
  }
}"#,
    );

    let output = run_validate(&["validate", &peer_file.path.to_string_lossy(), "--json"]);

    assert_exit_code(&output, 1);
    let json = assert_json_status(&output, "failed");
    assert!(
        json["errors"]
            .as_array()
            .is_some_and(|errors| errors.iter().any(|entry| {
                entry["message"].as_str().is_some_and(|message| {
                    message.contains("invalid JSON content: $.display_name: null is not allowed")
                })
            })),
        "expected null-value validation error, stdout: {}",
        stdout_text(&output)
    );
}

#[test]
fn validate_json_fails_for_floating_point_values_in_peer_file() {
    let _guard = validate_peer_fixture_lock()
        .lock()
        .expect("validate peer fixture lock should not be poisoned");
    let peer_file = TempRepoJsonFile::new(
        "floating-point",
        r#"{
  "node_id": "node:float",
  "role": "peer",
  "display_name": "Floating peer",
  "transport": {
    "kind": "memory",
    "latency_ms": 1.5
  }
}"#,
    );

    let output = run_validate(&["validate", &peer_file.path.to_string_lossy(), "--json"]);

    assert_exit_code(&output, 1);
    let json = assert_json_status(&output, "failed");
    assert!(
        json["errors"].as_array().is_some_and(|errors| errors.iter().any(|entry| {
            entry["message"].as_str().is_some_and(|message| {
                message.contains(
                    "invalid JSON content: $.transport.latency_ms: floating-point numbers are not allowed",
                )
            })
        })),
        "expected floating-point validation error, stdout: {}",
        stdout_text(&output)
    );
}

#[test]
fn peers_directory_validate_json_scopes_related_artifacts() {
    let output = run_validate(&["validate", "sim/peers", "--json"]);

    assert_success(&output);
    let json = assert_json_status(&output, "ok");
    assert_eq!(json["peer_count"], 1);
    assert_eq!(json["topology_count"], 13);
    assert_eq!(json["test_case_count"], 13);
    assert!(
        json["report_count"]
            .as_u64()
            .expect("report_count should be numeric")
            >= 4
    );
}

#[test]
fn topology_file_validate_json_scopes_related_artifacts() {
    let output = run_validate(&[
        "validate",
        "sim/topologies/three-peer-consistency.example.json",
        "--json",
    ]);

    assert_success(&output);
    let json = assert_json_status(&output, "ok");
    assert_eq!(json["fixture_count"], 1);
    assert_eq!(json["peer_count"], 1);
    assert_eq!(json["topology_count"], 1);
    assert_eq!(json["test_case_count"], 1);
    assert!(
        json["report_count"]
            .as_u64()
            .expect("report_count should be numeric")
            >= 1
    );
}

#[test]
fn topologies_directory_validate_json_reports_ok_status() {
    let output = run_validate(&["validate", "sim/topologies", "--json"]);

    assert_success(&output);
    let json = assert_json_status(&output, "ok");
    assert_eq!(json["fixture_count"], 12);
    assert_eq!(json["peer_count"], 1);
    assert_eq!(json["topology_count"], 13);
    assert_eq!(json["test_case_count"], 13);
}

#[test]
fn test_case_file_validate_json_scopes_related_artifacts() {
    let output = run_validate(&[
        "validate",
        "sim/tests/three-peer-consistency.example.json",
        "--json",
    ]);

    assert_success(&output);
    let json = assert_json_status(&output, "ok");
    assert_eq!(json["fixture_count"], 1);
    assert_eq!(json["peer_count"], 1);
    assert_eq!(json["topology_count"], 1);
    assert_eq!(json["test_case_count"], 1);
    assert!(
        json["report_count"]
            .as_u64()
            .expect("report_count should be numeric")
            >= 1
    );
}

#[test]
fn report_file_validate_json_scopes_report_only() {
    let output = run_validate(&["validate", "sim/reports/report.example.json", "--json"]);

    assert_success(&output);
    let json = assert_json_status(&output, "ok");
    assert_eq!(json["fixture_count"], 1);
    assert_eq!(json["peer_count"], 1);
    assert_eq!(json["topology_count"], 1);
    assert_eq!(json["test_case_count"], 1);
    assert_eq!(json["report_count"], 1);
}

#[test]
fn reports_directory_validate_json_scopes_report_only() {
    let output = run_validate(&["validate", "sim/reports", "--json"]);

    assert_success(&output);
    let json = assert_json_status(&output, "ok");
    assert_eq!(json["fixture_count"], 1);
    assert_eq!(json["peer_count"], 1);
    assert_eq!(json["topology_count"], 1);
    assert_eq!(json["test_case_count"], 1);
    assert_eq!(json["report_count"], 1);
}

#[test]
fn reports_out_directory_validate_json_reports_ok_status() {
    let output = run_validate(&["validate", "sim/reports/out", "--json"]);

    assert_success(&output);
    let json = assert_json_status(&output, "ok");
    assert!(
        json["report_count"]
            .as_u64()
            .expect("report_count should be numeric")
            >= 1,
        "expected at least one generated report, stdout: {}",
        String::from_utf8_lossy(&output.stdout)
    );
}

#[test]
fn schema_file_is_not_a_valid_validate_target() {
    let output = run_validate(&["validate", "sim/tests/test-case.schema.json", "--json"]);

    assert_json_error_contains(&output, "schema files are not validate targets");
}

#[test]
fn missing_validate_target_path_fails_cleanly() {
    let output = run_validate(&["validate", "does-not-exist.json", "--json"]);

    assert_json_error_contains(&output, "path does not exist");
}

#[test]
fn invalid_random_seed_prefix_report_fails_validation() {
    let output = run_validate(&[
        "validate",
        "sim/reports/invalid/random-seed-prefix-mismatch.example.json",
        "--json",
    ]);

    assert_json_error_contains(&output, "seed_source 'random'");
}

#[test]
fn invalid_auto_seed_prefix_report_fails_validation() {
    let output = run_validate(&[
        "validate",
        "sim/reports/invalid/auto-seed-prefix-mismatch.example.json",
        "--json",
    ]);

    assert_json_error_contains(&output, "seed_source 'auto'");
}

#[test]
fn unknown_topology_reference_report_fails_validation() {
    let output = run_validate(&[
        "validate",
        "sim/reports/invalid/unknown-topology-reference.example.json",
        "--json",
    ]);

    assert_json_error_contains(&output, "does not match any loaded topology");
}

#[test]
fn unknown_fixture_reference_report_fails_validation() {
    let output = run_validate(&[
        "validate",
        "sim/reports/invalid/unknown-fixture-reference.example.json",
        "--json",
    ]);

    assert_json_error_contains(&output, "does not match any loaded fixture");
}

#[test]
fn missing_seed_source_warns_and_strict_fails() {
    let normal_output = run_validate(&[
        "validate",
        "sim/reports/invalid/missing-seed-source.example.json",
        "--json",
    ]);

    assert_success(&normal_output);
    let _normal_json = assert_json_warning_contains(&normal_output, "does not include seed_source");

    let strict_output = run_validate(&[
        "validate",
        "sim/reports/invalid/missing-seed-source.example.json",
        "--json",
        "--strict",
    ]);

    assert!(
        !strict_output.status.success(),
        "expected strict warning failure, stdout: {}",
        String::from_utf8_lossy(&strict_output.stdout)
    );

    let _strict_json = assert_json_warning_contains(&strict_output, "does not include seed_source");
}

#[test]
fn missing_seed_source_text_reports_warning_summary() {
    let output = run_validate(&[
        "validate",
        "sim/reports/invalid/missing-seed-source.example.json",
    ]);

    assert_success(&output);
    assert_stdout_contains(&output, "status: warning");
    assert_stdout_contains(&output, "validation: warning");
    assert_stderr_contains(&output, "warning:");
    assert_stderr_contains(&output, "does not include seed_source");
}

#[test]
fn invalid_random_seed_prefix_text_reports_failure_summary() {
    let output = run_validate(&[
        "validate",
        "sim/reports/invalid/random-seed-prefix-mismatch.example.json",
    ]);

    assert_exit_code(&output, 1);
    assert_stdout_contains(&output, "status: failed");
    assert_stdout_contains(&output, "validation: failed");
    assert_stderr_contains(&output, "error:");
    assert_stderr_contains(&output, "seed_source 'random'");
}

#[test]
fn validate_rejects_unexpected_extra_argument_after_target() {
    let output = run_validate(&["validate", "sim/tests", "--json", "unexpected"]);

    assert_exit_code(&output, 2);
    assert_stderr_contains(&output, "unexpected validate argument: unexpected");
}

#[test]
fn validate_rejects_unknown_flag_after_target() {
    let output = run_validate(&["validate", "sim/tests", "--bogus"]);

    assert_exit_code(&output, 2);
    assert_stderr_contains(&output, "unexpected validate argument: --bogus");
}

#[test]
fn validate_treats_positional_after_known_flags_as_target() {
    let output = run_validate(&["validate", "--json", "--strict", "unexpected"]);

    assert_exit_code(&output, 1);
    let json = parse_json_stdout(&output);
    assert_eq!(json["status"], "failed");
    let errors = json["errors"]
        .as_array()
        .expect("errors should be an array");
    assert!(
        errors.iter().any(|entry| {
            entry["message"]
                .as_str()
                .is_some_and(|message| message.contains("path does not exist"))
        }),
        "expected missing target path failure, stdout: {}",
        stdout_text(&output)
    );
}
