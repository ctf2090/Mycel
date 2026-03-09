mod common;

use common::{
    assert_empty_stderr, assert_exit_code, assert_head_inspect_help, assert_report_inspect_help,
    assert_stderr_contains, assert_top_level_help, run_mycel, stdout_text,
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
