use std::fs;

mod common;

use common::{
    assert_exit_code, assert_stderr_contains, assert_success, create_temp_dir, parse_json_stdout,
    run_report, stdout_text,
};
use serde_json::json;

fn write_report_with_result(
    path: &std::path::Path,
    run_id: &str,
    started_at: &str,
    finished_at: &str,
    result: &str,
) {
    write_report_with_result_and_validation_status(
        path,
        run_id,
        started_at,
        finished_at,
        result,
        "ok",
    );
}

fn write_report_with_result_and_validation_status(
    path: &std::path::Path,
    run_id: &str,
    started_at: &str,
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
        "started_at": started_at,
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
        "events": [
            {
                "step": 1,
                "phase": "sync",
                "action": "seed-advertise",
                "outcome": "ok",
                "node_id": "node:peer-seed",
                "object_ids": ["obj:doc:sample-minimal:accepted-head"],
                "detail": "Seed advertised the accepted head."
            }
        ],
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

fn write_report(path: &std::path::Path, run_id: &str, started_at: &str, finished_at: &str) {
    write_report_with_result(path, run_id, started_at, finished_at, "pass");
}

#[test]
fn report_latest_json_selects_latest_finished_at_from_directory() {
    let temp_dir = create_temp_dir("report-latest");
    let older = temp_dir.path().join("older.report.json");
    let newer = temp_dir.path().join("newer.report.json");
    write_report(
        &older,
        "run:older",
        "2026-03-09T10:00:00+08:00",
        "2026-03-09T10:00:05+08:00",
    );
    write_report(
        &newer,
        "run:newer",
        "2026-03-09T11:00:00+08:00",
        "2026-03-09T11:00:05+08:00",
    );

    let target = temp_dir.path().display().to_string();
    let output = run_report(&["report", "latest", &target, "--json"]);

    assert_success(&output);
    let json = parse_json_stdout(&output);
    assert_eq!(json["status"], "ok");
    assert_eq!(json["report_count"], 2);
    assert_eq!(json["selected"]["path"], newer.display().to_string());
    assert_eq!(json["selected"]["run_id"], "run:newer");
    assert_eq!(json["selected"]["finished_at"], "2026-03-09T11:00:05+08:00");
}

#[test]
fn report_latest_json_filters_to_pass_result() {
    let temp_dir = create_temp_dir("report-latest-result-pass");
    let pass_report = temp_dir.path().join("pass.report.json");
    let fail_report = temp_dir.path().join("fail.report.json");
    write_report(
        &pass_report,
        "run:pass",
        "2026-03-09T11:00:00+08:00",
        "2026-03-09T11:00:05+08:00",
    );
    write_report(
        &fail_report,
        "run:fail",
        "2026-03-09T12:00:00+08:00",
        "2026-03-09T12:00:05+08:00",
    );
    write_report_with_result(
        &fail_report,
        "run:fail",
        "2026-03-09T12:00:00+08:00",
        "2026-03-09T12:00:05+08:00",
        "fail",
    );

    let target = temp_dir.path().display().to_string();
    let output = run_report(&["report", "latest", &target, "--result", "pass", "--json"]);

    assert_success(&output);
    let json = parse_json_stdout(&output);
    assert_eq!(json["result_filter"], "pass");
    assert_eq!(json["selected"]["run_id"], "run:pass");
    assert_eq!(json["selected"]["result"], "pass");
}

#[test]
fn report_latest_json_filters_to_warning_validation_status() {
    let temp_dir = create_temp_dir("report-latest-validation-warning");
    let ok_report = temp_dir.path().join("ok.report.json");
    let warning_report = temp_dir.path().join("warning.report.json");
    write_report_with_result_and_validation_status(
        &ok_report,
        "run:ok",
        "2026-03-09T11:00:00+08:00",
        "2026-03-09T11:00:05+08:00",
        "pass",
        "ok",
    );
    write_report_with_result_and_validation_status(
        &warning_report,
        "run:warning",
        "2026-03-09T12:00:00+08:00",
        "2026-03-09T12:00:05+08:00",
        "pass",
        "warning",
    );

    let target = temp_dir.path().display().to_string();
    let output = run_report(&[
        "report",
        "latest",
        &target,
        "--validation-status",
        "warning",
        "--json",
    ]);

    assert_success(&output);
    let json = parse_json_stdout(&output);
    assert_eq!(json["validation_status_filter"], "warning");
    assert_eq!(json["selected"]["run_id"], "run:warning");
    assert_eq!(json["selected"]["validation_status"], "warning");
}

#[test]
fn report_latest_json_filters_to_result_and_validation_status_intersection() {
    let temp_dir = create_temp_dir("report-latest-result-validation-intersection");
    let older_matching = temp_dir.path().join("older-matching.report.json");
    let newer_matching = temp_dir.path().join("newer-matching.report.json");
    let wrong_result = temp_dir.path().join("wrong-result.report.json");
    let wrong_validation = temp_dir.path().join("wrong-validation.report.json");
    write_report_with_result_and_validation_status(
        &older_matching,
        "run:older-matching",
        "2026-03-09T11:00:00+08:00",
        "2026-03-09T11:00:05+08:00",
        "pass",
        "warning",
    );
    write_report_with_result_and_validation_status(
        &newer_matching,
        "run:newer-matching",
        "2026-03-09T12:00:00+08:00",
        "2026-03-09T12:00:05+08:00",
        "pass",
        "warning",
    );
    write_report_with_result_and_validation_status(
        &wrong_result,
        "run:wrong-result",
        "2026-03-09T13:00:00+08:00",
        "2026-03-09T13:00:05+08:00",
        "fail",
        "warning",
    );
    write_report_with_result_and_validation_status(
        &wrong_validation,
        "run:wrong-validation",
        "2026-03-09T14:00:00+08:00",
        "2026-03-09T14:00:05+08:00",
        "pass",
        "ok",
    );

    let target = temp_dir.path().display().to_string();
    let output = run_report(&[
        "report",
        "latest",
        &target,
        "--result",
        "pass",
        "--validation-status",
        "warning",
        "--json",
    ]);

    assert_success(&output);
    let json = parse_json_stdout(&output);
    assert_eq!(json["result_filter"], "pass");
    assert_eq!(json["validation_status_filter"], "warning");
    assert_eq!(json["selected"]["run_id"], "run:newer-matching");
    assert_eq!(json["selected"]["result"], "pass");
    assert_eq!(json["selected"]["validation_status"], "warning");
}

#[test]
fn report_latest_full_json_filters_to_result_and_validation_status_intersection() {
    let temp_dir = create_temp_dir("report-latest-full-result-validation-intersection");
    let matching_report = temp_dir.path().join("matching.report.json");
    let wrong_result = temp_dir.path().join("wrong-result.report.json");
    let wrong_validation = temp_dir.path().join("wrong-validation.report.json");
    write_report_with_result_and_validation_status(
        &matching_report,
        "run:matching",
        "2026-03-09T12:00:00+08:00",
        "2026-03-09T12:00:05+08:00",
        "pass",
        "warning",
    );
    write_report_with_result_and_validation_status(
        &wrong_result,
        "run:wrong-result",
        "2026-03-09T13:00:00+08:00",
        "2026-03-09T13:00:05+08:00",
        "fail",
        "warning",
    );
    write_report_with_result_and_validation_status(
        &wrong_validation,
        "run:wrong-validation",
        "2026-03-09T14:00:00+08:00",
        "2026-03-09T14:00:05+08:00",
        "pass",
        "ok",
    );

    let target = temp_dir.path().display().to_string();
    let output = run_report(&[
        "report",
        "latest",
        &target,
        "--result",
        "pass",
        "--validation-status",
        "warning",
        "--full",
        "--json",
    ]);

    assert_success(&output);
    let json = parse_json_stdout(&output);
    assert_eq!(json["run_id"], "run:matching");
    assert_eq!(json["result"], "pass");
    assert_eq!(json["metadata"]["validation_status"], "warning");
    assert!(json.get("selected").is_none(), "expected raw report JSON");
}

#[test]
fn report_latest_path_only_filters_to_fail_result() {
    let temp_dir = create_temp_dir("report-latest-result-fail");
    let pass_report = temp_dir.path().join("pass.report.json");
    let fail_report = temp_dir.path().join("fail.report.json");
    write_report(
        &pass_report,
        "run:pass",
        "2026-03-09T11:00:00+08:00",
        "2026-03-09T11:00:05+08:00",
    );
    write_report(
        &fail_report,
        "run:fail",
        "2026-03-09T12:00:00+08:00",
        "2026-03-09T12:00:05+08:00",
    );
    write_report_with_result(
        &fail_report,
        "run:fail",
        "2026-03-09T12:00:00+08:00",
        "2026-03-09T12:00:05+08:00",
        "fail",
    );

    let target = temp_dir.path().display().to_string();
    let output = run_report(&[
        "report",
        "latest",
        &target,
        "--result",
        "fail",
        "--path-only",
    ]);

    assert_success(&output);
    assert_eq!(
        stdout_text(&output).trim(),
        fail_report.display().to_string()
    );
}

#[test]
fn report_latest_path_only_filters_to_failed_validation_status() {
    let temp_dir = create_temp_dir("report-latest-validation-failed");
    let ok_report = temp_dir.path().join("ok.report.json");
    let failed_report = temp_dir.path().join("failed.report.json");
    write_report_with_result_and_validation_status(
        &ok_report,
        "run:ok",
        "2026-03-09T11:00:00+08:00",
        "2026-03-09T11:00:05+08:00",
        "pass",
        "ok",
    );
    write_report_with_result_and_validation_status(
        &failed_report,
        "run:failed",
        "2026-03-09T12:00:00+08:00",
        "2026-03-09T12:00:05+08:00",
        "pass",
        "failed",
    );

    let target = temp_dir.path().display().to_string();
    let output = run_report(&[
        "report",
        "latest",
        &target,
        "--validation-status",
        "failed",
        "--path-only",
    ]);

    assert_success(&output);
    assert_eq!(
        stdout_text(&output).trim(),
        failed_report.display().to_string()
    );
}

#[test]
fn report_latest_path_only_filters_to_result_and_validation_status_intersection() {
    let temp_dir = create_temp_dir("report-latest-path-only-result-validation-intersection");
    let older_matching = temp_dir.path().join("older-matching.report.json");
    let newer_matching = temp_dir.path().join("newer-matching.report.json");
    let wrong_result = temp_dir.path().join("wrong-result.report.json");
    let wrong_validation = temp_dir.path().join("wrong-validation.report.json");
    write_report_with_result_and_validation_status(
        &older_matching,
        "run:older-matching",
        "2026-03-09T11:00:00+08:00",
        "2026-03-09T11:00:05+08:00",
        "pass",
        "warning",
    );
    write_report_with_result_and_validation_status(
        &newer_matching,
        "run:newer-matching",
        "2026-03-09T12:00:00+08:00",
        "2026-03-09T12:00:05+08:00",
        "pass",
        "warning",
    );
    write_report_with_result_and_validation_status(
        &wrong_result,
        "run:wrong-result",
        "2026-03-09T13:00:00+08:00",
        "2026-03-09T13:00:05+08:00",
        "fail",
        "warning",
    );
    write_report_with_result_and_validation_status(
        &wrong_validation,
        "run:wrong-validation",
        "2026-03-09T14:00:00+08:00",
        "2026-03-09T14:00:05+08:00",
        "pass",
        "ok",
    );

    let target = temp_dir.path().display().to_string();
    let output = run_report(&[
        "report",
        "latest",
        &target,
        "--result",
        "pass",
        "--validation-status",
        "warning",
        "--path-only",
    ]);

    assert_success(&output);
    assert_eq!(
        stdout_text(&output).trim(),
        newer_matching.display().to_string()
    );
}

#[test]
fn report_latest_full_json_emits_raw_selected_report() {
    let temp_dir = create_temp_dir("report-latest-full");
    let older = temp_dir.path().join("older.report.json");
    let newer = temp_dir.path().join("newer.report.json");
    write_report(
        &older,
        "run:older",
        "2026-03-09T10:00:00+08:00",
        "2026-03-09T10:00:05+08:00",
    );
    write_report(
        &newer,
        "run:newer",
        "2026-03-09T11:00:00+08:00",
        "2026-03-09T11:00:05+08:00",
    );

    let target = temp_dir.path().display().to_string();
    let output = run_report(&["report", "latest", &target, "--full", "--json"]);

    assert_success(&output);
    let json = parse_json_stdout(&output);
    assert_eq!(json["run_id"], "run:newer");
    assert_eq!(json["fixture_id"], "minimal-valid");
    assert_eq!(json["result"], "pass");
    assert_eq!(json["finished_at"], "2026-03-09T11:00:05+08:00");
}

#[test]
fn report_latest_text_reports_selected_summary() {
    let output = run_report(&["report", "latest", "sim/reports"]);

    assert_success(&output);
    let stdout = stdout_text(&output);
    assert!(stdout.contains("reports root: sim/reports"));
    assert!(stdout.contains("selected report: "));
    assert!(stdout.contains("report latest: ok"));
}

#[test]
fn report_latest_path_only_prints_selected_path() {
    let temp_dir = create_temp_dir("report-latest-path-only");
    let older = temp_dir.path().join("older.report.json");
    let newer = temp_dir.path().join("newer.report.json");
    write_report(
        &older,
        "run:older",
        "2026-03-09T10:00:00+08:00",
        "2026-03-09T10:00:05+08:00",
    );
    write_report(
        &newer,
        "run:newer",
        "2026-03-09T11:00:00+08:00",
        "2026-03-09T11:00:05+08:00",
    );

    let target = temp_dir.path().display().to_string();
    let output = run_report(&["report", "latest", &target, "--path-only"]);

    assert_success(&output);
    assert_eq!(stdout_text(&output).trim(), newer.display().to_string());
}

#[test]
fn report_latest_json_warns_when_invalid_reports_exist_but_valid_latest_is_selected() {
    let temp_dir = create_temp_dir("report-latest-warning");
    let valid = temp_dir.path().join("valid.report.json");
    let invalid = temp_dir.path().join("broken.report.json");
    write_report(
        &valid,
        "run:valid",
        "2026-03-09T11:00:00+08:00",
        "2026-03-09T11:00:05+08:00",
    );
    fs::write(&invalid, "{ broken json").expect("invalid report should be written");

    let target = temp_dir.path().display().to_string();
    let output = run_report(&["report", "latest", &target, "--json"]);

    assert_success(&output);
    let json = parse_json_stdout(&output);
    assert_eq!(json["status"], "warning");
    assert_eq!(json["report_count"], 2);
    assert_eq!(json["valid_report_count"], 1);
    assert_eq!(json["invalid_report_count"], 1);
    assert_eq!(json["selected"]["run_id"], "run:valid");
}

#[test]
fn report_latest_full_json_ignores_invalid_reports_when_valid_latest_exists() {
    let temp_dir = create_temp_dir("report-latest-full-warning");
    let valid = temp_dir.path().join("valid.report.json");
    let invalid = temp_dir.path().join("broken.report.json");
    write_report(
        &valid,
        "run:valid",
        "2026-03-09T11:00:00+08:00",
        "2026-03-09T11:00:05+08:00",
    );
    fs::write(&invalid, "{ broken json").expect("invalid report should be written");

    let target = temp_dir.path().display().to_string();
    let output = run_report(&["report", "latest", &target, "--full", "--json"]);

    assert_success(&output);
    let json = parse_json_stdout(&output);
    assert_eq!(json["run_id"], "run:valid");
    assert_eq!(json["result"], "pass");
}

#[test]
fn report_latest_path_only_ignores_invalid_reports_when_valid_latest_exists() {
    let temp_dir = create_temp_dir("report-latest-path-only-warning");
    let valid = temp_dir.path().join("valid.report.json");
    let invalid = temp_dir.path().join("broken.report.json");
    write_report(
        &valid,
        "run:valid",
        "2026-03-09T11:00:00+08:00",
        "2026-03-09T11:00:05+08:00",
    );
    fs::write(&invalid, "{ broken json").expect("invalid report should be written");

    let target = temp_dir.path().display().to_string();
    let output = run_report(&["report", "latest", &target, "--path-only"]);

    assert_success(&output);
    assert_eq!(stdout_text(&output).trim(), valid.display().to_string());
}

#[test]
fn report_latest_json_fails_when_no_valid_reports_exist() {
    let temp_dir = create_temp_dir("report-latest-invalid");
    let invalid = temp_dir.path().join("broken.report.json");
    fs::write(&invalid, "{ broken json").expect("invalid report should be written");

    let target = temp_dir.path().display().to_string();
    let output = run_report(&["report", "latest", &target, "--json"]);

    assert_exit_code(&output, 1);
    let json = parse_json_stdout(&output);
    assert_eq!(json["status"], "failed");
    assert_eq!(json["selected"], serde_json::Value::Null);
    assert_eq!(json["errors"][0], "no valid reports found under target");
}

#[test]
fn report_latest_json_fails_when_no_report_matches_result_filter() {
    let temp_dir = create_temp_dir("report-latest-result-miss");
    let pass_report = temp_dir.path().join("pass.report.json");
    write_report(
        &pass_report,
        "run:pass",
        "2026-03-09T11:00:00+08:00",
        "2026-03-09T11:00:05+08:00",
    );

    let target = temp_dir.path().display().to_string();
    let output = run_report(&["report", "latest", &target, "--result", "fail", "--json"]);

    assert_exit_code(&output, 1);
    let json = parse_json_stdout(&output);
    assert_eq!(json["status"], "failed");
    assert_eq!(json["result_filter"], "fail");
    assert_eq!(
        json["errors"][0],
        "no valid reports found under target with result=fail"
    );
}

#[test]
fn report_latest_json_fails_when_no_report_matches_validation_status_filter() {
    let temp_dir = create_temp_dir("report-latest-validation-miss");
    let ok_report = temp_dir.path().join("ok.report.json");
    write_report_with_result_and_validation_status(
        &ok_report,
        "run:ok",
        "2026-03-09T11:00:00+08:00",
        "2026-03-09T11:00:05+08:00",
        "pass",
        "ok",
    );

    let target = temp_dir.path().display().to_string();
    let output = run_report(&[
        "report",
        "latest",
        &target,
        "--validation-status",
        "warning",
        "--json",
    ]);

    assert_exit_code(&output, 1);
    let json = parse_json_stdout(&output);
    assert_eq!(json["status"], "failed");
    assert_eq!(json["validation_status_filter"], "warning");
    assert_eq!(
        json["errors"][0],
        "no valid reports found under target with validation_status=warning"
    );
}

#[test]
fn report_latest_json_reports_missing_target_as_failed() {
    let output = run_report(&[
        "report",
        "latest",
        "sim/reports/missing-directory",
        "--json",
    ]);

    assert_exit_code(&output, 1);
    let json = parse_json_stdout(&output);
    assert_eq!(json["status"], "failed");
    assert_eq!(json["selected"], serde_json::Value::Null);
    assert_eq!(
        json["errors"][0],
        "report list target does not exist: sim/reports/missing-directory"
    );
}

#[test]
fn report_latest_path_only_fails_when_no_valid_reports_exist() {
    let temp_dir = create_temp_dir("report-latest-path-only-invalid");
    let invalid = temp_dir.path().join("broken.report.json");
    fs::write(&invalid, "{ broken json").expect("invalid report should be written");

    let target = temp_dir.path().display().to_string();
    let output = run_report(&["report", "latest", &target, "--path-only"]);

    assert_exit_code(&output, 1);
    assert_stderr_contains(&output, "no valid reports found under target");
}

#[test]
fn report_latest_full_json_requires_json() {
    let output = run_report(&["report", "latest", "sim/reports", "--full"]);

    assert_exit_code(&output, 2);
    assert_stderr_contains(&output, "--json");
}

#[test]
fn report_latest_rejects_path_only_with_json() {
    let output = run_report(&["report", "latest", "sim/reports", "--path-only", "--json"]);

    assert_exit_code(&output, 2);
    assert_stderr_contains(&output, "cannot be used with");
    assert_stderr_contains(&output, "--path-only");
    assert_stderr_contains(&output, "--json");
}
