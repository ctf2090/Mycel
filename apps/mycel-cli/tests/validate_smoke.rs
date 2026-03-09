use serde_json::Value;

mod common;

use common::{parse_json_stdout, run_validate, stderr_text, stdout_text};

fn assert_failed_with_message(output: &std::process::Output, expected_text: &str) {
    assert!(
        !output.status.success(),
        "expected failure, stdout: {}",
        String::from_utf8_lossy(&output.stdout)
    );

    let json = parse_json_stdout(output);
    assert_eq!(json["status"], "failed");
    let errors = json["errors"]
        .as_array()
        .expect("errors should be an array");
    assert!(
        errors.iter().any(|entry| {
            entry["message"]
                .as_str()
                .is_some_and(|message| message.contains(expected_text))
        }),
        "expected error containing '{expected_text}', stdout: {}",
        String::from_utf8_lossy(&output.stdout)
    );
}

#[test]
fn repo_validate_json_reports_ok_status() {
    let output = run_validate(&["validate", "--json"]);

    assert!(
        output.status.success(),
        "expected success, stderr: {}",
        String::from_utf8_lossy(&output.stderr)
    );

    let json = parse_json_stdout(&output);
    assert_eq!(json["status"], "ok");
    assert_eq!(json["errors"], Value::Array(Vec::new()));
}

#[test]
fn tests_directory_validate_json_reports_ok_status() {
    let output = run_validate(&["validate", "sim/tests", "--json"]);

    assert!(
        output.status.success(),
        "expected success, stderr: {}",
        String::from_utf8_lossy(&output.stderr)
    );

    let json = parse_json_stdout(&output);
    assert_eq!(json["status"], "ok");
    assert_eq!(json["test_case_count"], 4);
    assert_eq!(json["topology_count"], 4);
}

#[test]
fn reports_out_directory_validate_json_reports_ok_status() {
    let output = run_validate(&["validate", "sim/reports/out", "--json"]);

    assert!(
        output.status.success(),
        "expected success, stderr: {}",
        String::from_utf8_lossy(&output.stderr)
    );

    let json = parse_json_stdout(&output);
    assert_eq!(json["status"], "ok");
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

    assert_failed_with_message(&output, "schema files are not validate targets");
}

#[test]
fn missing_validate_target_path_fails_cleanly() {
    let output = run_validate(&["validate", "does-not-exist.json", "--json"]);

    assert_failed_with_message(&output, "path does not exist");
}

#[test]
fn invalid_random_seed_prefix_report_fails_validation() {
    let output = run_validate(&[
        "validate",
        "sim/reports/invalid/random-seed-prefix-mismatch.example.json",
        "--json",
    ]);

    assert_failed_with_message(&output, "seed_source 'random'");
}

#[test]
fn invalid_auto_seed_prefix_report_fails_validation() {
    let output = run_validate(&[
        "validate",
        "sim/reports/invalid/auto-seed-prefix-mismatch.example.json",
        "--json",
    ]);

    assert_failed_with_message(&output, "seed_source 'auto'");
}

#[test]
fn unknown_topology_reference_report_fails_validation() {
    let output = run_validate(&[
        "validate",
        "sim/reports/invalid/unknown-topology-reference.example.json",
        "--json",
    ]);

    assert_failed_with_message(&output, "does not match any loaded topology");
}

#[test]
fn unknown_fixture_reference_report_fails_validation() {
    let output = run_validate(&[
        "validate",
        "sim/reports/invalid/unknown-fixture-reference.example.json",
        "--json",
    ]);

    assert_failed_with_message(&output, "does not match any loaded fixture");
}

#[test]
fn missing_seed_source_warns_and_strict_fails() {
    let normal_output = run_validate(&[
        "validate",
        "sim/reports/invalid/missing-seed-source.example.json",
        "--json",
    ]);

    assert!(
        normal_output.status.success(),
        "expected warning-only success, stderr: {}",
        String::from_utf8_lossy(&normal_output.stderr)
    );

    let normal_json = parse_json_stdout(&normal_output);
    assert_eq!(normal_json["status"], "warning");
    let warnings = normal_json["warnings"]
        .as_array()
        .expect("warnings should be an array");
    assert!(
        warnings.iter().any(|entry| {
            entry["message"]
                .as_str()
                .is_some_and(|message| message.contains("does not include seed_source"))
        }),
        "expected missing seed_source warning, stdout: {}",
        String::from_utf8_lossy(&normal_output.stdout)
    );

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

    let strict_json = parse_json_stdout(&strict_output);
    assert_eq!(strict_json["status"], "warning");
}

#[test]
fn validate_rejects_unexpected_extra_argument_after_target() {
    let output = run_validate(&["validate", "sim/tests", "--json", "unexpected"]);

    assert_eq!(output.status.code(), Some(2));
    let stderr = stderr_text(&output);
    assert!(
        stderr.contains("unexpected validate argument: unexpected"),
        "expected unexpected argument error, stderr: {stderr}"
    );
}

#[test]
fn validate_rejects_unknown_flag_after_target() {
    let output = run_validate(&["validate", "sim/tests", "--bogus"]);

    assert_eq!(output.status.code(), Some(2));
    let stderr = stderr_text(&output);
    assert!(
        stderr.contains("unexpected validate argument: --bogus"),
        "expected unknown flag error, stderr: {stderr}"
    );
}

#[test]
fn validate_treats_positional_after_known_flags_as_target() {
    let output = run_validate(&["validate", "--json", "--strict", "unexpected"]);

    assert_eq!(output.status.code(), Some(1));
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
