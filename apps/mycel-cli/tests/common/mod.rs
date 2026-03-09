#![allow(dead_code)]

use std::fs;
use std::path::{Path, PathBuf};
use std::process::{Command, Output};
use std::time::{SystemTime, UNIX_EPOCH};

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

pub struct TempDir {
    path: PathBuf,
}

impl TempDir {
    pub fn path(&self) -> &Path {
        &self.path
    }
}

impl Drop for TempDir {
    fn drop(&mut self) {
        let _ = fs::remove_dir_all(&self.path);
    }
}

pub fn create_temp_dir(prefix: &str) -> TempDir {
    let unique = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .expect("system time should be after unix epoch")
        .as_nanos();
    let path = std::env::temp_dir().join(format!(
        "mycel-cli-{prefix}-{}-{unique}",
        std::process::id()
    ));
    fs::create_dir_all(&path).expect("temporary directory should be created");

    TempDir { path }
}

pub fn run_mycel(args: &[&str]) -> Output {
    run_mycel_in_dir(&repo_root(), args)
}

pub fn run_mycel_in_dir(current_dir: &Path, args: &[&str]) -> Output {
    Command::new(mycel_bin())
        .current_dir(current_dir)
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

pub fn run_report(args: &[&str]) -> Output {
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

pub fn validate_generated_report(summary: &Value) -> Value {
    let report_path = summary["report_path"]
        .as_str()
        .expect("report_path should be a string");
    let output = run_validate(&["validate", report_path, "--json"]);

    assert_success(&output);
    assert_json_status(&output, "ok")
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

pub fn assert_stderr_starts_with(output: &Output, expected_prefix: &str) {
    let stderr = stderr_text(output);
    assert!(
        stderr.starts_with(expected_prefix),
        "expected stderr to start with '{expected_prefix}', stderr: {stderr}"
    );
}

pub fn assert_empty_stderr(output: &Output) {
    let stderr = stderr_text(output);
    assert_eq!(stderr, "", "expected empty stderr, stderr: {stderr}");
}

pub fn assert_top_level_help(stdout: &str) {
    assert!(
        stdout.contains("Mycel CLI for validation, inspection, and simulator workflows."),
        "expected CLI description, stdout: {stdout}"
    );
    assert!(
        stdout.contains("Usage: mycel [COMMAND]"),
        "expected usage header, stdout: {stdout}"
    );
    assert!(
        stdout.contains("Commands:"),
        "expected Commands section, stdout: {stdout}"
    );
    assert!(
        stdout.contains("Options:"),
        "expected Options section, stdout: {stdout}"
    );
    assert!(
        stdout.contains("head"),
        "expected head command in help, stdout: {stdout}"
    );
    assert!(
        stdout.contains("report"),
        "expected report command in help, stdout: {stdout}"
    );
    assert!(
        stdout.contains("validate"),
        "expected validate command in help, stdout: {stdout}"
    );
    assert!(
        stdout.contains("-h, --help"),
        "expected help flag in help output, stdout: {stdout}"
    );
}

pub fn assert_head_inspect_help(stdout: &str) {
    assert!(
        stdout.contains("Usage: mycel head inspect"),
        "expected head inspect usage, stdout: {stdout}"
    );
    assert!(
        stdout.contains("Inspect one document's accepted head"),
        "expected head inspect description, stdout: {stdout}"
    );
    assert!(
        stdout.contains("DOC_ID"),
        "expected doc id argument in help, stdout: {stdout}"
    );
    assert!(
        stdout.contains("--input"),
        "expected input flag in help, stdout: {stdout}"
    );
    assert!(
        stdout.contains("PATH_OR_FIXTURE"),
        "expected input value name in help, stdout: {stdout}"
    );
}

pub fn assert_report_inspect_help(stdout: &str) {
    assert!(
        stdout.contains("Usage: mycel report inspect"),
        "expected report inspect usage, stdout: {stdout}"
    );
    assert!(
        stdout.contains("Inspect one simulator report"),
        "expected report inspect description, stdout: {stdout}"
    );
    assert!(
        stdout.contains("--events"),
        "expected events flag in help, stdout: {stdout}"
    );
    assert!(
        stdout.contains("--step-range"),
        "expected step-range flag in help, stdout: {stdout}"
    );
    assert!(
        stdout.contains("START:END"),
        "expected step-range value name in help, stdout: {stdout}"
    );
    assert!(
        stdout.contains("NODE_ID"),
        "expected node value name in help, stdout: {stdout}"
    );
}

pub fn assert_report_list_help(stdout: &str) {
    assert!(
        stdout.contains("Usage: mycel report list"),
        "expected report list usage, stdout: {stdout}"
    );
    assert!(
        stdout.contains("List simulator reports under a directory or one file"),
        "expected report list description, stdout: {stdout}"
    );
    assert!(
        stdout.contains("[PATH]"),
        "expected optional report list path, stdout: {stdout}"
    );
    assert!(
        stdout.contains("--json"),
        "expected json flag in report list help, stdout: {stdout}"
    );
}

pub fn assert_report_latest_help(stdout: &str) {
    assert!(
        stdout.contains("Usage: mycel report latest"),
        "expected report latest usage, stdout: {stdout}"
    );
    assert!(
        stdout.contains("Select the latest simulator report under a directory or one file"),
        "expected report latest description, stdout: {stdout}"
    );
    assert!(
        stdout.contains("[PATH]"),
        "expected optional report latest path, stdout: {stdout}"
    );
    assert!(
        stdout.contains("--json"),
        "expected json flag in report latest help, stdout: {stdout}"
    );
    assert!(
        stdout.contains("--full"),
        "expected full flag in report latest help, stdout: {stdout}"
    );
}

pub fn assert_object_verify_help(stdout: &str) {
    assert!(
        stdout.contains("Usage: mycel object verify"),
        "expected object verify usage, stdout: {stdout}"
    );
    assert!(
        stdout.contains("Verify one object file"),
        "expected object verify description, stdout: {stdout}"
    );
    assert!(
        stdout.contains("<PATH>"),
        "expected object path argument in help, stdout: {stdout}"
    );
    assert!(
        stdout.contains("--json"),
        "expected json flag in help, stdout: {stdout}"
    );
}

pub fn assert_sim_run_help(stdout: &str) {
    assert!(
        stdout.contains("Usage: mycel sim run"),
        "expected sim run usage, stdout: {stdout}"
    );
    assert!(
        stdout.contains("Run one test case and write a report"),
        "expected sim run description, stdout: {stdout}"
    );
    assert!(
        stdout.contains("<PATH>"),
        "expected sim path argument in help, stdout: {stdout}"
    );
    assert!(
        stdout.contains("--seed <SEED>"),
        "expected seed flag in help, stdout: {stdout}"
    );
}

pub fn assert_validate_help(stdout: &str) {
    assert!(
        stdout.contains("Usage: mycel validate"),
        "expected validate usage, stdout: {stdout}"
    );
    assert!(
        stdout.contains("Validate the repo root, one file, or one supported directory"),
        "expected validate description, stdout: {stdout}"
    );
    assert!(
        stdout.contains("[PATH]"),
        "expected optional path argument in help, stdout: {stdout}"
    );
    assert!(
        stdout.contains("--strict"),
        "expected strict flag in help, stdout: {stdout}"
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
