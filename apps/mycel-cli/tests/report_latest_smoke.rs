use std::fs;

mod common;

use common::{
    assert_exit_code, assert_stderr_contains, assert_success, create_temp_dir, parse_json_stdout,
    run_report, stdout_text,
};
use serde_json::json;

fn write_report(path: &std::path::Path, run_id: &str, started_at: &str, finished_at: &str) {
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
        "result": "pass",
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
fn report_latest_full_json_requires_json() {
    let output = run_report(&["report", "latest", "sim/reports", "--full"]);

    assert_exit_code(&output, 2);
    assert_stderr_contains(&output, "--json");
}
