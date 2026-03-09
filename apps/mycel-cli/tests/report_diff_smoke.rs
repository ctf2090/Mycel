mod common;

use std::fs;

use common::{
    assert_exit_code, assert_stderr_contains, assert_stdout_contains, assert_success,
    create_temp_dir, parse_json_stdout, repo_root, run_report, stdout_text,
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
fn report_diff_json_can_ignore_summary_field_differences() {
    let output = run_report(&[
        "report",
        "diff",
        "sim/reports/report.example.json",
        "sim/reports/invalid/missing-seed-source.example.json",
        "--json",
        "--ignore-field",
        "run-id",
        "--ignore-field",
        "peer-count",
        "--ignore-field",
        "seed-source",
    ]);

    assert_success(&output);
    let json = parse_json_stdout(&output);
    assert_eq!(json["status"], "ok");
    assert_eq!(json["comparison"], "different");
    assert!(
        json["ignored_fields"]
            .as_array()
            .is_some_and(|fields| fields.len() == 3),
        "expected ignored field list, stdout: {}",
        stdout_text(&output)
    );
    let differences = json["differences"]
        .as_array()
        .expect("differences should be an array");
    assert!(
        differences.iter().all(|entry| entry["field"] != "run_id"),
        "expected run_id to be ignored, stdout: {}",
        stdout_text(&output)
    );
    assert!(
        differences
            .iter()
            .all(|entry| entry["field"] != "peer_count"),
        "expected peer_count to be ignored, stdout: {}",
        stdout_text(&output)
    );
    assert!(
        differences
            .iter()
            .all(|entry| entry["field"] != "seed_source"),
        "expected seed_source to be ignored, stdout: {}",
        stdout_text(&output)
    );
}

#[test]
fn report_diff_json_can_select_summary_fields() {
    let output = run_report(&[
        "report",
        "diff",
        "sim/reports/report.example.json",
        "sim/reports/invalid/missing-seed-source.example.json",
        "--json",
        "--field",
        "run-id",
        "--field",
        "peer-count",
    ]);

    assert_success(&output);
    let json = parse_json_stdout(&output);
    assert_eq!(json["status"], "ok");
    assert_eq!(json["comparison"], "different");
    assert_eq!(
        json["selected_fields"],
        serde_json::json!(["run-id", "peer-count"])
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
            .all(|entry| entry["field"] != "seed_source"),
        "expected seed_source to be excluded, stdout: {}",
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
fn report_diff_events_json_can_select_event_fields() {
    let output = run_report(&[
        "report",
        "diff",
        "sim/reports/report.example.json",
        "sim/reports/invalid/missing-seed-source.example.json",
        "--events",
        "--json",
        "--field",
        "event-detail",
    ]);

    assert_success(&output);
    let json = parse_json_stdout(&output);
    assert_eq!(json["status"], "ok");
    assert_eq!(json["comparison"], "different");
    assert_eq!(json["selected_fields"], serde_json::json!(["event-detail"]));
    assert!(
        json["event_difference_count"]
            .as_u64()
            .is_some_and(|count| count >= 1),
        "expected event-detail diff, stdout: {}",
        stdout_text(&output)
    );
}

#[test]
fn report_diff_events_json_can_ignore_event_field_differences() {
    let output = run_report(&[
        "report",
        "diff",
        "sim/reports/report.example.json",
        "sim/reports/invalid/missing-seed-source.example.json",
        "--events",
        "--json",
        "--ignore-field",
        "event-detail",
    ]);

    assert_success(&output);
    let json = parse_json_stdout(&output);
    assert_eq!(json["status"], "ok");
    assert_eq!(json["comparison"], "match");
    assert_eq!(json["event_difference_count"], 0);
    assert_eq!(json["ignored_fields"], serde_json::json!(["event-detail"]));
}

#[test]
fn report_diff_rejects_field_with_ignore_field() {
    let output = run_report(&[
        "report",
        "diff",
        "sim/reports/report.example.json",
        "sim/reports/invalid/missing-seed-source.example.json",
        "--field",
        "run-id",
        "--ignore-field",
        "seed-source",
    ]);

    assert_exit_code(&output, 2);
    assert_stderr_contains(&output, "cannot be used with");
    assert_stderr_contains(&output, "--field <FIELD>");
    assert_stderr_contains(&output, "--ignore-field <FIELD>");
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
fn report_diff_text_reports_selected_fields() {
    let output = run_report(&[
        "report",
        "diff",
        "sim/reports/report.example.json",
        "sim/reports/invalid/missing-seed-source.example.json",
        "--field",
        "run-id",
    ]);

    assert_success(&output);
    assert_stdout_contains(&output, "selected fields: run-id");
}

#[test]
fn report_diff_rejects_unknown_ignore_field_value() {
    let output = run_report(&[
        "report",
        "diff",
        "sim/reports/report.example.json",
        "sim/reports/invalid/missing-seed-source.example.json",
        "--ignore-field",
        "bogus-field",
    ]);

    assert_exit_code(&output, 2);
    assert_stderr_contains(&output, "invalid value 'bogus-field'");
    assert_stderr_contains(&output, "--ignore-field <FIELD>");
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
        differences.iter().any(|entry| {
            entry["step"] == 1
                && entry["change"] == "changed"
                && entry["trace_identity"]["phase"] == "load"
                && entry["trace_identity"]["action"] == "load-fixture"
        }),
        "expected changed step 1 event, stdout: {}",
        stdout_text(&output)
    );
}

#[test]
fn report_diff_events_json_matches_shifted_steps_by_trace_identity() {
    let temp = create_temp_dir("report-diff-trace-identity");
    let left_path = temp.path().join("left.json");
    let right_path = temp.path().join("right.json");
    let source_path = repo_root().join("sim/reports/report.example.json");
    fs::copy(&source_path, &left_path).expect("left report fixture should copy");

    let mut right_report: serde_json::Value = serde_json::from_str(
        &fs::read_to_string(&source_path).expect("right source report should read"),
    )
    .expect("right source report should parse");

    let events = right_report["events"]
        .as_array_mut()
        .expect("events should be an array");
    for (index, event) in events.iter_mut().enumerate() {
        event["step"] = serde_json::json!((index as u64) + 11);
    }

    fs::write(
        &right_path,
        serde_json::to_string_pretty(&right_report).expect("shifted report should serialize"),
    )
    .expect("shifted report should write");

    let left = left_path.to_string_lossy().into_owned();
    let right = right_path.to_string_lossy().into_owned();
    let output = run_report(&["report", "diff", &left, &right, "--events", "--json"]);

    assert_success(&output);
    let json = parse_json_stdout(&output);
    assert_eq!(json["status"], "ok");
    assert_eq!(json["comparison"], "match");
    assert_eq!(json["event_difference_count"], 0);
    assert_eq!(
        json["event_differences"].as_array().map(Vec::len),
        Some(0),
        "expected trace identity to absorb step-only drift, stdout: {}",
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
fn report_diff_text_reports_ignored_fields() {
    let output = run_report(&[
        "report",
        "diff",
        "sim/reports/report.example.json",
        "sim/reports/invalid/missing-seed-source.example.json",
        "--ignore-field",
        "run-id",
    ]);

    assert_success(&output);
    assert_stdout_contains(&output, "ignored fields: run-id");
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
    assert_stdout_contains(
        &output,
        "event step 1: changed (phase=load action=load-fixture",
    );
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
