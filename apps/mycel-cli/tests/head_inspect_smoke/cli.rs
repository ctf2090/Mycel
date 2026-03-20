use super::*;

#[test]
fn head_inspect_requires_input_path() {
    let output = run_mycel(&["head", "inspect", "doc:sample"]);

    assert_exit_code(&output, 2);
    assert_stderr_contains(&output, "required arguments were not provided");
    assert_stderr_contains(&output, "--input <PATH_OR_FIXTURE>");
}

#[test]
fn head_inspect_reports_unknown_repo_native_fixture() {
    let output = run_mycel(&["head", "inspect", "doc:sample", "--input", "does-not-exist"]);

    assert_exit_code(&output, 1);
    assert_stdout_contains(&output, "Head inspection: failed");
    assert_stdout_contains(&output, "- input: does-not-exist");
    assert_stderr_contains(
        &output,
        "could not resolve head-inspect input 'does-not-exist'",
    );
}

#[test]
fn head_rejects_unknown_subcommand() {
    let output = run_mycel(&["head", "bogus"]);

    assert_exit_code(&output, 2);
    assert_stderr_contains(&output, "unknown head subcommand: bogus");
    assert_top_level_help(&stdout_text(&output));
}
