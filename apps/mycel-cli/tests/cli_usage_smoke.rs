mod common;

use common::{
    assert_empty_stderr, assert_exit_code, assert_stderr_contains, assert_usage_sections,
    run_mycel, stdout_text,
};

#[test]
fn help_command_prints_usage_and_succeeds() {
    let output = run_mycel(&["help"]);

    assert_exit_code(&output, 0);
    assert_empty_stderr(&output);
    assert_usage_sections(&stdout_text(&output));
}

#[test]
fn no_arguments_prints_usage_and_succeeds() {
    let output = run_mycel(&[]);

    assert_exit_code(&output, 0);
    assert_empty_stderr(&output);
    assert_usage_sections(&stdout_text(&output));
}

#[test]
fn unknown_command_prints_usage_and_fails_with_error() {
    let output = run_mycel(&["bogus"]);

    assert_exit_code(&output, 2);
    assert_usage_sections(&stdout_text(&output));
    assert_stderr_contains(&output, "unknown command: bogus");
}
