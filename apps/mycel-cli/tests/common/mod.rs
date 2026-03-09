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
