#![allow(dead_code)]

use std::fs;
use std::path::{Path, PathBuf};
use std::process::{Command, Output};

use serde_json::Value;

pub fn repo_root() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("../..")
        .canonicalize()
        .expect("repo root should resolve")
}

pub fn mycel_bin() -> PathBuf {
    PathBuf::from(env!("CARGO_BIN_EXE_mycel"))
}

pub fn run_mycel(args: &[&str]) -> Output {
    Command::new(mycel_bin())
        .current_dir(repo_root())
        .args(args)
        .output()
        .expect("mycel command should run")
}

pub fn run_validate(args: &[&str]) -> Output {
    run_mycel(args)
}

pub fn run_sim(args: &[&str]) -> Output {
    run_mycel(args)
}

pub fn parse_json_stdout(output: &Output) -> Value {
    serde_json::from_slice(&output.stdout).expect("stdout should contain valid JSON")
}

pub fn assert_success(output: &Output) {
    assert!(
        output.status.success(),
        "expected success, stdout: {}, stderr: {}",
        stdout_text(output),
        stderr_text(output)
    );
}

pub fn assert_json_status(output: &Output, expected_status: &str) -> Value {
    let json = parse_json_stdout(output);
    assert_eq!(
        json["status"],
        expected_status,
        "expected JSON status '{expected_status}', stdout: {}",
        stdout_text(output)
    );
    json
}

pub fn assert_json_error_contains(output: &Output, expected_text: &str) -> Value {
    let json = assert_json_status(output, "failed");
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
        stdout_text(output)
    );
    json
}

pub fn assert_json_warning_contains(output: &Output, expected_text: &str) -> Value {
    let json = assert_json_status(output, "warning");
    let warnings = json["warnings"]
        .as_array()
        .expect("warnings should be an array");
    assert!(
        warnings.iter().any(|entry| {
            entry["message"]
                .as_str()
                .is_some_and(|message| message.contains(expected_text))
        }),
        "expected warning containing '{expected_text}', stdout: {}",
        stdout_text(output)
    );
    json
}

pub fn load_report(summary: &Value) -> Value {
    let report_path = summary["report_path"]
        .as_str()
        .expect("report_path should be a string");
    let content = fs::read_to_string(report_path).expect("report file should exist");
    serde_json::from_str(&content).expect("report file should contain valid JSON")
}

pub fn stderr_text(output: &Output) -> String {
    String::from_utf8_lossy(&output.stderr).into_owned()
}

pub fn stdout_text(output: &Output) -> String {
    String::from_utf8_lossy(&output.stdout).into_owned()
}

pub fn assert_exit_code(output: &Output, expected: i32) {
    assert_eq!(
        output.status.code(),
        Some(expected),
        "expected exit code {expected}, stdout: {}, stderr: {}",
        stdout_text(output),
        stderr_text(output)
    );
}

pub fn assert_stdout_contains(output: &Output, expected_text: &str) {
    let stdout = stdout_text(output);
    assert!(
        stdout.contains(expected_text),
        "expected stdout to contain '{expected_text}', stdout: {stdout}"
    );
}

pub fn assert_stderr_contains(output: &Output, expected_text: &str) {
    let stderr = stderr_text(output);
    assert!(
        stderr.contains(expected_text),
        "expected stderr to contain '{expected_text}', stderr: {stderr}"
    );
}

pub fn assert_empty_stderr(output: &Output) {
    let stderr = stderr_text(output);
    assert_eq!(stderr, "", "expected empty stderr, stderr: {stderr}");
}

pub fn assert_usage_sections(stdout: &str) {
    assert!(
        stdout.contains("mycel <command> [path]"),
        "expected usage header, stdout: {stdout}"
    );
    assert!(
        stdout.contains("Commands:"),
        "expected Commands section, stdout: {stdout}"
    );
    assert!(
        stdout.contains("Sim options:"),
        "expected Sim options section, stdout: {stdout}"
    );
    assert!(
        stdout.contains("Validate options:"),
        "expected Validate options section, stdout: {stdout}"
    );
}

pub fn assert_info_sections(stdout: &str) {
    assert!(
        stdout.contains("Mycel Rust workspace"),
        "expected workspace banner, stdout: {stdout}"
    );
    assert!(
        stdout.contains("Mycel simulator scaffold"),
        "expected simulator banner, stdout: {stdout}"
    );
    assert!(
        stdout.contains("fixtures:"),
        "expected fixtures path, stdout: {stdout}"
    );
    assert!(
        stdout.contains("peers:"),
        "expected peers path, stdout: {stdout}"
    );
    assert!(
        stdout.contains("topologies:"),
        "expected topologies path, stdout: {stdout}"
    );
    assert!(
        stdout.contains("tests:"),
        "expected tests path, stdout: {stdout}"
    );
    assert!(
        stdout.contains("reports:"),
        "expected reports path, stdout: {stdout}"
    );
}
