use std::fs;

mod common;

use common::{
    assert_exit_code, assert_success, create_temp_dir, parse_json_stdout, run_report, stdout_text,
};
use serde_json::json;

fn write_report_with_result(path: &std::path::Path, run_id: &str, finished_at: &str, result: &str) {
    let report = json!({
        "$schema": "../report.schema.json",
        "run_id": run_id,
        "topology_id": "three-peer-consistency",
        "fixture_id": "minimal-valid",
        "test_id": "three-peer-consistency",
        "execution_mode": "single-process",
        "started_at": finished_at,
        "finished_at": finished_at,
        "peers": [
            {
                "node_id": "node:peer-seed",
                "status": "ok",
                "verified_object_ids": ["obj:doc:sample-minimal:accepted-head"],
                "rejected_object_ids": [],
                "notes": []
            }
        ],
        "result": result,
        "events": [],
        "failures": [],
        "summary": {
            "verified_object_count": 1,
            "rejected_object_count": 0,
            "matched_expected_outcomes": ["sync-success"]
        },
        "metadata": {
            "validation_status": "ok",
            "seed_source": "derived"
        }
    });
    fs::write(
        path,
        serde_json::to_vec_pretty(&report).expect("report should serialize"),
    )
    .expect("report should be written");
}

#[test]
fn report_list_json_lists_default_root_and_skips_schema_file() {
    let output = run_report(&["report", "list", "--json"]);

    assert_success(&output);
    let json = parse_json_stdout(&output);
    assert_eq!(json["status"], "ok");
    assert!(
        json["report_count"].as_u64().unwrap_or(0) >= 5,
        "expected several reports, stdout: {}",
        stdout_text(&output)
    );

    let reports = json["reports"]
        .as_array()
        .expect("reports should be an array");
    assert!(
        reports
            .iter()
            .any(|entry| entry["path"] == "sim/reports/report.example.json"),
        "expected report.example.json in listing, stdout: {}",
        stdout_text(&output)
    );
    assert!(
        reports
            .iter()
            .any(|entry| entry["path"] == "sim/reports/out/three-peer-consistency.report.json"),
        "expected generated report in listing, stdout: {}",
        stdout_text(&output)
    );
    assert!(
        reports
            .iter()
            .all(|entry| entry["path"] != "sim/reports/report.schema.json"),
        "did not expect schema file in listing, stdout: {}",
        stdout_text(&output)
    );
}

#[test]
fn report_list_text_lists_default_root_summary() {
    let output = run_report(&["report", "list"]);

    assert_success(&output);
    let stdout = stdout_text(&output);
    assert!(stdout.contains("reports root: sim/reports"));
    assert!(stdout.contains("reports: "));
    assert!(stdout.contains("report: sim/reports/report.example.json status=ok"));
    assert!(!stdout.contains("report.schema.json"));
}

#[test]
fn report_list_json_accepts_single_file_target() {
    let output = run_report(&[
        "report",
        "list",
        "sim/reports/report.example.json",
        "--json",
    ]);

    assert_success(&output);
    let json = parse_json_stdout(&output);
    assert_eq!(json["root"], "sim/reports/report.example.json");
    assert_eq!(json["report_count"], 1);
    assert_eq!(json["valid_report_count"], 1);
    assert_eq!(json["invalid_report_count"], 0);
    let reports = json["reports"]
        .as_array()
        .expect("reports should be an array");
    assert_eq!(reports.len(), 1);
    assert_eq!(reports[0]["run_id"], "run:example-001");
    assert_eq!(reports[0]["result"], "pass");
}

#[test]
fn report_list_json_marks_parse_failures_as_invalid() {
    let temp_dir = create_temp_dir("report-list");
    let invalid_report_path = temp_dir.path().join("broken.report.json");
    let schema_path = temp_dir.path().join("report.schema.json");
    fs::write(&invalid_report_path, "{ broken json").expect("invalid report should be written");
    fs::write(&schema_path, "{}").expect("schema file should be written");

    let target = temp_dir.path().display().to_string();
    let output = run_report(&["report", "list", &target, "--json"]);

    assert_success(&output);
    let json = parse_json_stdout(&output);
    assert_eq!(json["status"], "warning");
    assert_eq!(json["report_count"], 1);
    assert_eq!(json["valid_report_count"], 0);
    assert_eq!(json["invalid_report_count"], 1);

    let reports = json["reports"]
        .as_array()
        .expect("reports should be an array");
    assert_eq!(reports.len(), 1);
    assert_eq!(reports[0]["status"], "failed");
    assert_eq!(
        reports[0]["path"],
        invalid_report_path.display().to_string()
    );
    assert!(
        reports[0]["parse_error"]
            .as_str()
            .is_some_and(|message| message.contains("failed to parse report JSON")),
        "expected parse error in listing, stdout: {}",
        stdout_text(&output)
    );
}

#[test]
fn report_list_json_filters_to_pass_result() {
    let temp_dir = create_temp_dir("report-list-result-pass");
    let pass_report = temp_dir.path().join("pass.report.json");
    let fail_report = temp_dir.path().join("fail.report.json");
    write_report_with_result(
        &pass_report,
        "run:pass",
        "2026-03-09T11:00:05+08:00",
        "pass",
    );
    write_report_with_result(
        &fail_report,
        "run:fail",
        "2026-03-09T12:00:05+08:00",
        "fail",
    );

    let target = temp_dir.path().display().to_string();
    let output = run_report(&["report", "list", &target, "--result", "pass", "--json"]);

    assert_success(&output);
    let json = parse_json_stdout(&output);
    assert_eq!(json["result_filter"], "pass");
    assert_eq!(json["report_count"], 1);
    assert_eq!(json["valid_report_count"], 1);
    assert_eq!(json["invalid_report_count"], 0);
    let reports = json["reports"]
        .as_array()
        .expect("reports should be an array");
    assert_eq!(reports.len(), 1);
    assert_eq!(reports[0]["run_id"], "run:pass");
    assert_eq!(reports[0]["result"], "pass");
}

#[test]
fn report_list_text_filters_to_fail_result_and_keeps_invalid_entries() {
    let temp_dir = create_temp_dir("report-list-result-fail");
    let pass_report = temp_dir.path().join("pass.report.json");
    let fail_report = temp_dir.path().join("fail.report.json");
    let invalid_report = temp_dir.path().join("broken.report.json");
    write_report_with_result(
        &pass_report,
        "run:pass",
        "2026-03-09T11:00:05+08:00",
        "pass",
    );
    write_report_with_result(
        &fail_report,
        "run:fail",
        "2026-03-09T12:00:05+08:00",
        "fail",
    );
    fs::write(&invalid_report, "{ broken json").expect("invalid report should be written");

    let target = temp_dir.path().display().to_string();
    let output = run_report(&["report", "list", &target, "--result", "fail"]);

    assert_success(&output);
    let stdout = stdout_text(&output);
    assert!(stdout.contains("result filter: fail"));
    assert!(stdout.contains("run_id=run:fail"));
    assert!(!stdout.contains("run_id=run:pass"));
    assert!(stdout.contains("parse_error=failed to parse report JSON"));
}

#[test]
fn report_list_json_returns_empty_valid_results_when_no_report_matches_filter() {
    let temp_dir = create_temp_dir("report-list-result-miss");
    let pass_report = temp_dir.path().join("pass.report.json");
    write_report_with_result(
        &pass_report,
        "run:pass",
        "2026-03-09T11:00:05+08:00",
        "pass",
    );

    let target = temp_dir.path().display().to_string();
    let output = run_report(&["report", "list", &target, "--result", "fail", "--json"]);

    assert_success(&output);
    let json = parse_json_stdout(&output);
    assert_eq!(json["status"], "ok");
    assert_eq!(json["result_filter"], "fail");
    assert_eq!(json["report_count"], 0);
    assert_eq!(json["valid_report_count"], 0);
    assert_eq!(json["invalid_report_count"], 0);
}

#[test]
fn report_list_json_reports_missing_target_as_failed() {
    let output = run_report(&["report", "list", "sim/reports/missing-directory", "--json"]);

    assert_exit_code(&output, 1);
    let json = parse_json_stdout(&output);
    assert_eq!(json["status"], "failed");
    assert_eq!(json["report_count"], 0);
    assert_eq!(
        json["errors"][0],
        "report list target does not exist: sim/reports/missing-directory"
    );
}
