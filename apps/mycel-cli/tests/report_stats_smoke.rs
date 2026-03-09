use std::fs;

mod common;

use common::{
    assert_exit_code, assert_success, create_temp_dir, parse_json_stdout, run_report, stdout_text,
};
use serde_json::json;

fn write_report_with_result_and_validation_status(
    path: &std::path::Path,
    run_id: &str,
    finished_at: &str,
    result: &str,
    validation_status: &str,
) {
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
            "validation_status": validation_status,
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
fn report_stats_json_summarizes_directory() {
    let temp_dir = create_temp_dir("report-stats");
    let older_pass = temp_dir.path().join("older-pass.report.json");
    let newer_fail = temp_dir.path().join("newer-fail.report.json");
    let invalid = temp_dir.path().join("broken.report.json");
    write_report_with_result_and_validation_status(
        &older_pass,
        "run:older-pass",
        "2026-03-09T11:00:05+08:00",
        "pass",
        "ok",
    );
    write_report_with_result_and_validation_status(
        &newer_fail,
        "run:newer-fail",
        "2026-03-09T12:00:05+08:00",
        "fail",
        "warning",
    );
    fs::write(&invalid, "{ broken json").expect("invalid report should be written");

    let target = temp_dir.path().display().to_string();
    let output = run_report(&["report", "stats", &target, "--json"]);

    assert_success(&output);
    let json = parse_json_stdout(&output);
    assert_eq!(json["status"], "warning");
    assert_eq!(json["report_count"], 3);
    assert_eq!(json["valid_report_count"], 2);
    assert_eq!(json["invalid_report_count"], 1);
    assert_eq!(json["result_counts"]["pass"], 1);
    assert_eq!(json["result_counts"]["fail"], 1);
    assert_eq!(json["validation_status_counts"]["ok"], 1);
    assert_eq!(json["validation_status_counts"]["warning"], 1);
    assert_eq!(json["latest_finished_at"], "2026-03-09T12:00:05+08:00");
    assert_eq!(
        json["latest_valid_report"]["path"],
        newer_fail.display().to_string()
    );
    assert_eq!(json["latest_valid_report"]["run_id"], "run:newer-fail");
}

#[test]
fn report_stats_json_filters_to_result_and_validation_status_intersection() {
    let temp_dir = create_temp_dir("report-stats-filtered");
    let matching_report = temp_dir.path().join("matching.report.json");
    let wrong_result = temp_dir.path().join("wrong-result.report.json");
    let wrong_validation = temp_dir.path().join("wrong-validation.report.json");
    let invalid = temp_dir.path().join("broken.report.json");
    write_report_with_result_and_validation_status(
        &matching_report,
        "run:matching",
        "2026-03-09T12:00:05+08:00",
        "pass",
        "warning",
    );
    write_report_with_result_and_validation_status(
        &wrong_result,
        "run:wrong-result",
        "2026-03-09T13:00:05+08:00",
        "fail",
        "warning",
    );
    write_report_with_result_and_validation_status(
        &wrong_validation,
        "run:wrong-validation",
        "2026-03-09T14:00:05+08:00",
        "pass",
        "ok",
    );
    fs::write(&invalid, "{ broken json").expect("invalid report should be written");

    let target = temp_dir.path().display().to_string();
    let output = run_report(&[
        "report",
        "stats",
        &target,
        "--result",
        "pass",
        "--validation-status",
        "warning",
        "--json",
    ]);

    assert_success(&output);
    let json = parse_json_stdout(&output);
    assert_eq!(json["status"], "warning");
    assert_eq!(json["result_filter"], "pass");
    assert_eq!(json["validation_status_filter"], "warning");
    assert_eq!(json["report_count"], 2);
    assert_eq!(json["valid_report_count"], 1);
    assert_eq!(json["invalid_report_count"], 1);
    assert_eq!(json["result_counts"]["pass"], 1);
    assert!(json["result_counts"].get("fail").is_none());
    assert_eq!(json["validation_status_counts"]["warning"], 1);
    assert_eq!(
        json["latest_valid_report"]["path"],
        matching_report.display().to_string()
    );
}

#[test]
fn report_stats_text_reports_human_summary() {
    let temp_dir = create_temp_dir("report-stats-text");
    let pass_report = temp_dir.path().join("pass.report.json");
    let fail_report = temp_dir.path().join("fail.report.json");
    write_report_with_result_and_validation_status(
        &pass_report,
        "run:pass",
        "2026-03-09T11:00:05+08:00",
        "pass",
        "ok",
    );
    write_report_with_result_and_validation_status(
        &fail_report,
        "run:fail",
        "2026-03-09T12:00:05+08:00",
        "fail",
        "warning",
    );

    let target = temp_dir.path().display().to_string();
    let output = run_report(&["report", "stats", &target]);

    assert_success(&output);
    let stdout = stdout_text(&output);
    assert!(stdout.contains("reports root: "));
    assert!(stdout.contains("result counts:"));
    assert!(stdout.contains("pass: 1"));
    assert!(stdout.contains("fail: 1"));
    assert!(stdout.contains("validation status counts:"));
    assert!(stdout.contains("ok: 1"));
    assert!(stdout.contains("warning: 1"));
    assert!(stdout.contains("latest valid report: "));
}

#[test]
fn report_stats_text_reports_active_filters() {
    let temp_dir = create_temp_dir("report-stats-text-filtered");
    let report = temp_dir.path().join("matching.report.json");
    write_report_with_result_and_validation_status(
        &report,
        "run:matching",
        "2026-03-09T12:00:05+08:00",
        "pass",
        "warning",
    );

    let target = temp_dir.path().display().to_string();
    let output = run_report(&[
        "report",
        "stats",
        &target,
        "--result",
        "pass",
        "--validation-status",
        "warning",
    ]);

    assert_success(&output);
    let stdout = stdout_text(&output);
    assert!(stdout.contains("result filter: pass"));
    assert!(stdout.contains("validation status filter: warning"));
}

#[test]
fn report_stats_json_reports_missing_target_as_failed() {
    let output = run_report(&["report", "stats", "sim/reports/missing-directory", "--json"]);

    assert_exit_code(&output, 1);
    let json = parse_json_stdout(&output);
    assert_eq!(json["status"], "failed");
    assert_eq!(json["report_count"], 0);
    assert_eq!(
        json["errors"][0],
        "report list target does not exist: sim/reports/missing-directory"
    );
}
