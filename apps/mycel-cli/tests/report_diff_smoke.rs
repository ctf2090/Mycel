mod common;

use common::{
    assert_exit_code, assert_stderr_contains, assert_stdout_contains, assert_success,
    parse_json_stdout, run_report, stdout_text,
};

#[test]
fn report_diff_json_reports_match_for_same_report() {
    let output = run_report(&[
        "report",
        "diff",
        "sim/reports/report.example.json",
        "sim/reports/report.example.json",
        "--json",
    ]);

    assert_success(&output);
    let json = parse_json_stdout(&output);
    assert_eq!(json["status"], "ok");
    assert_eq!(json["comparison"], "match");
    assert_eq!(json["difference_count"], 0);
    assert_eq!(
        json["differences"].as_array().map(Vec::len),
        Some(0),
        "expected no differences, stdout: {}",
        stdout_text(&output)
    );
}

#[test]
fn report_diff_json_reports_summary_level_differences() {
    let output = run_report(&[
        "report",
        "diff",
        "sim/reports/report.example.json",
        "sim/reports/invalid/missing-seed-source.example.json",
        "--json",
    ]);

    assert_success(&output);
    let json = parse_json_stdout(&output);
    assert_eq!(json["status"], "ok");
    assert_eq!(json["comparison"], "different");
    assert!(
        json["difference_count"]
            .as_u64()
            .is_some_and(|count| count >= 3),
        "expected multiple summary differences, stdout: {}",
        stdout_text(&output)
    );
    let differences = json["differences"]
        .as_array()
        .expect("differences should be an array");
    assert!(
        differences.iter().any(|entry| entry["field"] == "run_id"),
        "expected run_id diff, stdout: {}",
        stdout_text(&output)
    );
    assert!(
        differences
            .iter()
            .any(|entry| entry["field"] == "peer_count"),
        "expected peer_count diff, stdout: {}",
        stdout_text(&output)
    );
    assert!(
        differences
            .iter()
            .any(|entry| entry["field"] == "seed_source"),
        "expected seed_source diff, stdout: {}",
        stdout_text(&output)
    );
}

#[test]
fn report_diff_json_can_fail_on_summary_differences() {
    let output = run_report(&[
        "report",
        "diff",
        "sim/reports/report.example.json",
        "sim/reports/invalid/missing-seed-source.example.json",
        "--json",
        "--fail-on-diff",
    ]);

    assert_exit_code(&output, 1);
    let json = parse_json_stdout(&output);
    assert_eq!(json["status"], "ok");
    assert_eq!(json["comparison"], "different");
}

#[test]
fn report_diff_events_json_reports_match_for_same_report() {
    let output = run_report(&[
        "report",
        "diff",
        "sim/reports/report.example.json",
        "sim/reports/report.example.json",
        "--events",
        "--json",
    ]);

    assert_success(&output);
    let json = parse_json_stdout(&output);
    assert_eq!(json["status"], "ok");
    assert_eq!(json["comparison"], "match");
    assert_eq!(json["event_difference_count"], 0);
    assert_eq!(
        json["event_differences"].as_array().map(Vec::len),
        Some(0),
        "expected no event differences, stdout: {}",
        stdout_text(&output)
    );
}

#[test]
fn report_diff_events_json_reports_step_level_differences() {
    let output = run_report(&[
        "report",
        "diff",
        "sim/reports/report.example.json",
        "sim/reports/invalid/missing-seed-source.example.json",
        "--events",
        "--json",
    ]);

    assert_success(&output);
    let json = parse_json_stdout(&output);
    assert_eq!(json["status"], "ok");
    assert_eq!(json["comparison"], "different");
    assert!(
        json["event_difference_count"]
            .as_u64()
            .is_some_and(|count| count >= 1),
        "expected event differences, stdout: {}",
        stdout_text(&output)
    );
    let differences = json["event_differences"]
        .as_array()
        .expect("event_differences should be an array");
    assert!(
        differences
            .iter()
            .any(|entry| entry["step"] == 1 && entry["change"] == "changed"),
        "expected changed step 1 event, stdout: {}",
        stdout_text(&output)
    );
}

#[test]
fn report_diff_events_json_can_fail_on_event_differences() {
    let output = run_report(&[
        "report",
        "diff",
        "sim/reports/report.example.json",
        "sim/reports/invalid/missing-seed-source.example.json",
        "--events",
        "--json",
        "--fail-on-diff",
    ]);

    assert_exit_code(&output, 1);
    let json = parse_json_stdout(&output);
    assert_eq!(json["status"], "ok");
    assert_eq!(json["comparison"], "different");
    assert!(
        json["event_difference_count"]
            .as_u64()
            .is_some_and(|count| count >= 1),
        "expected event differences, stdout: {}",
        stdout_text(&output)
    );
}

#[test]
fn report_diff_text_reports_human_summary() {
    let output = run_report(&[
        "report",
        "diff",
        "sim/reports/report.example.json",
        "sim/reports/invalid/missing-seed-source.example.json",
    ]);

    assert_success(&output);
    assert_stdout_contains(&output, "left report: sim/reports/report.example.json");
    assert_stdout_contains(
        &output,
        "right report: sim/reports/invalid/missing-seed-source.example.json",
    );
    assert_stdout_contains(&output, "comparison: different");
    assert_stdout_contains(&output, "difference seed_source:");
    assert_stdout_contains(&output, "report diff: different");
}

#[test]
fn report_diff_text_can_fail_on_summary_differences() {
    let output = run_report(&[
        "report",
        "diff",
        "sim/reports/report.example.json",
        "sim/reports/invalid/missing-seed-source.example.json",
        "--fail-on-diff",
    ]);

    assert_exit_code(&output, 1);
    assert_stdout_contains(&output, "comparison: different");
    assert_stdout_contains(&output, "report diff: different");
}

#[test]
fn report_diff_events_text_reports_human_event_summary() {
    let output = run_report(&[
        "report",
        "diff",
        "sim/reports/report.example.json",
        "sim/reports/invalid/missing-seed-source.example.json",
        "--events",
    ]);

    assert_success(&output);
    assert_stdout_contains(&output, "comparison: different");
    assert_stdout_contains(&output, "event difference count:");
    assert_stdout_contains(&output, "event step 1: changed");
    assert_stdout_contains(&output, "report diff: different");
}

#[test]
fn report_diff_events_text_can_fail_on_event_differences() {
    let output = run_report(&[
        "report",
        "diff",
        "sim/reports/report.example.json",
        "sim/reports/invalid/missing-seed-source.example.json",
        "--events",
        "--fail-on-diff",
    ]);

    assert_exit_code(&output, 1);
    assert_stdout_contains(&output, "comparison: different");
    assert_stdout_contains(&output, "report diff: different");
}

#[test]
fn report_diff_fails_when_one_side_is_not_a_report_target() {
    let output = run_report(&[
        "report",
        "diff",
        "sim/reports/report.schema.json",
        "sim/reports/report.example.json",
        "--json",
    ]);

    assert_exit_code(&output, 1);
    let json = parse_json_stdout(&output);
    assert_eq!(json["status"], "failed");
    assert_eq!(json["comparison"], "failed");
    assert!(
        json["errors"].as_array().is_some_and(|errors| errors
            .iter()
            .any(|entry| entry.as_str().is_some_and(|message| message
                .contains("left report: report schema files are not inspect targets")))),
        "expected left-side error, stdout: {}",
        stdout_text(&output)
    );
}

#[test]
fn report_diff_events_fail_when_one_side_is_not_a_report_target() {
    let output = run_report(&[
        "report",
        "diff",
        "sim/reports/report.schema.json",
        "sim/reports/report.example.json",
        "--events",
        "--json",
    ]);

    assert_exit_code(&output, 1);
    let json = parse_json_stdout(&output);
    assert_eq!(json["status"], "failed");
    assert_eq!(json["comparison"], "failed");
    assert!(
        json["errors"].as_array().is_some_and(|errors| errors
            .iter()
            .any(|entry| entry.as_str().is_some_and(|message| message
                .contains("left report: report schema files are not inspect targets")))),
        "expected left-side event diff error, stdout: {}",
        stdout_text(&output)
    );
}

#[test]
fn report_diff_missing_right_target_fails_cleanly() {
    let output = run_report(&["report", "diff", "sim/reports/report.example.json"]);

    assert_exit_code(&output, 2);
    assert_stderr_contains(&output, "required arguments were not provided");
    assert_stderr_contains(&output, "<RIGHT_PATH>");
}
