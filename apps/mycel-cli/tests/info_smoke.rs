mod common;

use common::{assert_empty_stderr, assert_exit_code, assert_info_sections, run_mycel, stdout_text};

#[test]
fn info_command_prints_workspace_and_path_sections() {
    let output = run_mycel(&["info"]);

    assert_exit_code(&output, 0);
    assert_empty_stderr(&output);
    assert_info_sections(&stdout_text(&output));
}
