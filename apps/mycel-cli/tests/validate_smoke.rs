use std::path::{Path, PathBuf};
use std::process::Command;

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

fn run_validate(args: &[&str]) -> std::process::Output {
    Command::new(mycel_bin())
        .current_dir(repo_root())
        .args(args)
        .output()
        .expect("validate command should run")
}

fn parse_json_stdout(output: &std::process::Output) -> Value {
    serde_json::from_slice(&output.stdout).expect("stdout should contain valid JSON")
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
fn invalid_random_seed_prefix_report_fails_validation() {
    let output = run_validate(&[
        "validate",
        "sim/reports/invalid/random-seed-prefix-mismatch.example.json",
        "--json",
    ]);

    assert!(
        !output.status.success(),
        "expected failure, stdout: {}",
        String::from_utf8_lossy(&output.stdout)
    );

    let json = parse_json_stdout(&output);
    assert_eq!(json["status"], "failed");
    let errors = json["errors"]
        .as_array()
        .expect("errors should be an array");
    assert!(
        errors.iter().any(|entry| {
            entry["message"]
                .as_str()
                .is_some_and(|message| message.contains("seed_source 'random'"))
        }),
        "expected random seed prefix error, stdout: {}",
        String::from_utf8_lossy(&output.stdout)
    );
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
