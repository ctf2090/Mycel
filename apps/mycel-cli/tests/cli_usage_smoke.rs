mod common;

use common::{
    assert_empty_stderr, assert_exit_code, assert_head_inspect_help, assert_object_inspect_help,
    assert_object_verify_help, assert_report_diff_help, assert_report_inspect_help,
    assert_report_latest_help, assert_report_list_help, assert_report_stats_help,
    assert_sim_run_help, assert_stderr_contains, assert_stderr_starts_with, assert_top_level_help,
    assert_validate_help, run_mycel, stdout_text,
};

#[test]
fn help_command_prints_usage_and_succeeds() {
    let output = run_mycel(&["help"]);

    assert_exit_code(&output, 0);
    assert_empty_stderr(&output);
    assert_top_level_help(&stdout_text(&output));
}

#[test]
fn no_arguments_prints_usage_and_succeeds() {
    let output = run_mycel(&[]);

    assert_exit_code(&output, 0);
    assert_empty_stderr(&output);
    assert_top_level_help(&stdout_text(&output));
}

#[test]
fn unknown_command_prints_usage_and_fails_with_error() {
    let output = run_mycel(&["bogus"]);

    assert_exit_code(&output, 2);
    assert_top_level_help(&stdout_text(&output));
    assert_stderr_starts_with(&output, "error: ");
    assert_stderr_contains(&output, "unknown command: bogus");
}

#[test]
fn head_inspect_help_prints_structured_clap_help() {
    let output = run_mycel(&["head", "inspect", "--help"]);

    assert_exit_code(&output, 0);
    assert_empty_stderr(&output);
    assert_head_inspect_help(&stdout_text(&output));
}

#[test]
fn report_inspect_help_prints_structured_clap_help() {
    let output = run_mycel(&["report", "inspect", "--help"]);

    assert_exit_code(&output, 0);
    assert_empty_stderr(&output);
    assert_report_inspect_help(&stdout_text(&output));
}

#[test]
fn report_diff_help_prints_structured_clap_help() {
    let output = run_mycel(&["report", "diff", "--help"]);

    assert_exit_code(&output, 0);
    assert_empty_stderr(&output);
    assert_report_diff_help(&stdout_text(&output));
}

#[test]
fn report_list_help_prints_structured_clap_help() {
    let output = run_mycel(&["report", "list", "--help"]);

    assert_exit_code(&output, 0);
    assert_empty_stderr(&output);
    assert_report_list_help(&stdout_text(&output));
}

#[test]
fn report_latest_help_prints_structured_clap_help() {
    let output = run_mycel(&["report", "latest", "--help"]);

    assert_exit_code(&output, 0);
    assert_empty_stderr(&output);
    assert_report_latest_help(&stdout_text(&output));
}

#[test]
fn report_stats_help_prints_structured_clap_help() {
    let output = run_mycel(&["report", "stats", "--help"]);

    assert_exit_code(&output, 0);
    assert_empty_stderr(&output);
    assert_report_stats_help(&stdout_text(&output));
}

#[test]
fn object_verify_help_prints_structured_clap_help() {
    let output = run_mycel(&["object", "verify", "--help"]);

    assert_exit_code(&output, 0);
    assert_empty_stderr(&output);
    assert_object_verify_help(&stdout_text(&output));
}

#[test]
fn object_inspect_help_prints_structured_clap_help() {
    let output = run_mycel(&["object", "inspect", "--help"]);

    assert_exit_code(&output, 0);
    assert_empty_stderr(&output);
    assert_object_inspect_help(&stdout_text(&output));
}

#[test]
fn sim_run_help_prints_structured_clap_help() {
    let output = run_mycel(&["sim", "run", "--help"]);

    assert_exit_code(&output, 0);
    assert_empty_stderr(&output);
    assert_sim_run_help(&stdout_text(&output));
}

#[test]
fn store_rebuild_help_prints_structured_clap_help() {
    let output = run_mycel(&["store", "rebuild", "--help"]);

    assert_exit_code(&output, 0);
    assert_empty_stderr(&output);
    let stdout = stdout_text(&output);
    assert!(stdout.contains("Rebuild local object-store indexes from stored objects"));
    assert!(stdout.contains("Usage: mycel store rebuild [OPTIONS] <PATH>"));
    assert!(stdout.contains("--json"));
}

#[test]
fn store_ingest_help_prints_structured_clap_help() {
    let output = run_mycel(&["store", "ingest", "--help"]);

    assert_exit_code(&output, 0);
    assert_empty_stderr(&output);
    let stdout = stdout_text(&output);
    assert!(stdout.contains("Verify and ingest objects into a local object store"));
    assert!(stdout.contains("Usage: mycel store ingest [OPTIONS] --into <STORE_ROOT> <SOURCE>"));
    assert!(stdout.contains("--into <STORE_ROOT>"));
    assert!(stdout.contains("--json"));
}

#[test]
fn store_index_help_prints_structured_clap_help() {
    let output = run_mycel(&["store", "index", "--help"]);

    assert_exit_code(&output, 0);
    assert_empty_stderr(&output);
    let stdout = stdout_text(&output);
    assert!(stdout.contains("Query persisted local object-store indexes"));
    assert!(stdout.contains("Usage: mycel store index [OPTIONS] <STORE_ROOT>"));
    assert!(stdout.contains("--doc-id <DOC_ID>"));
    assert!(stdout.contains("--author <AUTHOR>"));
    assert!(stdout.contains("--revision-id <REVISION_ID>"));
    assert!(stdout.contains("--view-id <VIEW_ID>"));
    assert!(stdout.contains("--profile-id <PROFILE_ID>"));
    assert!(stdout.contains("--object-type <OBJECT_TYPE>"));
    assert!(stdout.contains("--path-only"));
    assert!(stdout.contains("--doc-only"));
    assert!(stdout.contains("--governance-only"));
    assert!(stdout.contains("--parents-only"));
    assert!(stdout.contains("--json"));
}

#[test]
fn validate_help_prints_structured_clap_help() {
    let output = run_mycel(&["validate", "--help"]);

    assert_exit_code(&output, 0);
    assert_empty_stderr(&output);
    assert_validate_help(&stdout_text(&output));
}
