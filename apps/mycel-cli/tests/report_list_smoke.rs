use std::fs;

mod common;

use common::{
    assert_exit_code, assert_success, create_temp_dir, parse_json_stdout, run_report, stdout_text,
};

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
