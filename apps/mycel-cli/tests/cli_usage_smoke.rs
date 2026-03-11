mod common;

use common::{
    assert_head_inspect_help, assert_head_render_help, assert_object_inspect_help,
    assert_object_verify_help, assert_report_diff_help, assert_report_inspect_help,
    assert_report_latest_help, assert_report_list_help, assert_report_stats_help,
    assert_sim_run_help, assert_stderr_text, assert_stdout_text, assert_top_level_help,
    assert_validate_help, assert_view_inspect_help, assert_view_publish_help, mycel_command,
};

#[test]
fn help_command_prints_usage_and_succeeds() {
    let assert = mycel_command(&["help"]).assert().success();
    let stdout = assert_stdout_text(&assert);

    assert_eq!(assert_stderr_text(&assert), "");
    assert_top_level_help(&stdout);
}

#[test]
fn no_arguments_prints_usage_and_succeeds() {
    let assert = mycel_command(&[]).assert().success();
    let stdout = assert_stdout_text(&assert);

    assert_eq!(assert_stderr_text(&assert), "");
    assert_top_level_help(&stdout);
}

#[test]
fn unknown_command_prints_usage_and_fails_with_error() {
    let assert = mycel_command(&["bogus"]).assert().code(2);
    let stdout = assert_stdout_text(&assert);
    let stderr = assert_stderr_text(&assert);

    assert_top_level_help(&stdout);
    assert!(
        stderr.starts_with("error: "),
        "expected clap error prefix, stderr: {stderr}"
    );
    assert!(
        stderr.contains("unknown command: bogus"),
        "expected unknown-command error, stderr: {stderr}"
    );
}

#[test]
fn head_inspect_help_prints_structured_clap_help() {
    let assert = mycel_command(&["head", "inspect", "--help"])
        .assert()
        .success();
    let stdout = assert_stdout_text(&assert);

    assert_eq!(assert_stderr_text(&assert), "");
    assert_head_inspect_help(&stdout);
}

#[test]
fn head_render_help_prints_structured_clap_help() {
    let assert = mycel_command(&["head", "render", "--help"])
        .assert()
        .success();
    let stdout = assert_stdout_text(&assert);

    assert_eq!(assert_stderr_text(&assert), "");
    assert_head_render_help(&stdout);
}

#[test]
fn view_inspect_help_prints_structured_clap_help() {
    let assert = mycel_command(&["view", "inspect", "--help"])
        .assert()
        .success();
    let stdout = assert_stdout_text(&assert);

    assert_eq!(assert_stderr_text(&assert), "");
    assert_view_inspect_help(&stdout);
}

#[test]
fn view_publish_help_prints_structured_clap_help() {
    let assert = mycel_command(&["view", "publish", "--help"])
        .assert()
        .success();
    let stdout = assert_stdout_text(&assert);

    assert_eq!(assert_stderr_text(&assert), "");
    assert_view_publish_help(&stdout);
}

#[test]
fn report_inspect_help_prints_structured_clap_help() {
    let assert = mycel_command(&["report", "inspect", "--help"])
        .assert()
        .success();
    let stdout = assert_stdout_text(&assert);

    assert_eq!(assert_stderr_text(&assert), "");
    assert_report_inspect_help(&stdout);
}

#[test]
fn report_diff_help_prints_structured_clap_help() {
    let assert = mycel_command(&["report", "diff", "--help"])
        .assert()
        .success();
    let stdout = assert_stdout_text(&assert);

    assert_eq!(assert_stderr_text(&assert), "");
    assert_report_diff_help(&stdout);
}

#[test]
fn report_list_help_prints_structured_clap_help() {
    let assert = mycel_command(&["report", "list", "--help"])
        .assert()
        .success();
    let stdout = assert_stdout_text(&assert);

    assert_eq!(assert_stderr_text(&assert), "");
    assert_report_list_help(&stdout);
}

#[test]
fn report_latest_help_prints_structured_clap_help() {
    let assert = mycel_command(&["report", "latest", "--help"])
        .assert()
        .success();
    let stdout = assert_stdout_text(&assert);

    assert_eq!(assert_stderr_text(&assert), "");
    assert_report_latest_help(&stdout);
}

#[test]
fn report_stats_help_prints_structured_clap_help() {
    let assert = mycel_command(&["report", "stats", "--help"])
        .assert()
        .success();
    let stdout = assert_stdout_text(&assert);

    assert_eq!(assert_stderr_text(&assert), "");
    assert_report_stats_help(&stdout);
}

#[test]
fn object_verify_help_prints_structured_clap_help() {
    let assert = mycel_command(&["object", "verify", "--help"])
        .assert()
        .success();
    let stdout = assert_stdout_text(&assert);

    assert_eq!(assert_stderr_text(&assert), "");
    assert_object_verify_help(&stdout);
}

#[test]
fn object_inspect_help_prints_structured_clap_help() {
    let assert = mycel_command(&["object", "inspect", "--help"])
        .assert()
        .success();
    let stdout = assert_stdout_text(&assert);

    assert_eq!(assert_stderr_text(&assert), "");
    assert_object_inspect_help(&stdout);
}

#[test]
fn sim_run_help_prints_structured_clap_help() {
    let assert = mycel_command(&["sim", "run", "--help"]).assert().success();
    let stdout = assert_stdout_text(&assert);

    assert_eq!(assert_stderr_text(&assert), "");
    assert_sim_run_help(&stdout);
}

#[test]
fn store_rebuild_help_prints_structured_clap_help() {
    let assert = mycel_command(&["store", "rebuild", "--help"])
        .assert()
        .success();
    let stdout = assert_stdout_text(&assert);

    assert_eq!(assert_stderr_text(&assert), "");
    assert!(stdout.contains("Rebuild local object-store indexes from stored objects"));
    assert!(stdout.contains("Usage: mycel store rebuild [OPTIONS] <PATH>"));
    assert!(stdout.contains("--json"));
}

#[test]
fn store_ingest_help_prints_structured_clap_help() {
    let assert = mycel_command(&["store", "ingest", "--help"])
        .assert()
        .success();
    let stdout = assert_stdout_text(&assert);

    assert_eq!(assert_stderr_text(&assert), "");
    assert!(stdout.contains("Verify and ingest objects into a local object store"));
    assert!(stdout.contains("Usage: mycel store ingest [OPTIONS] --into <STORE_ROOT> <SOURCE>"));
    assert!(stdout.contains("--into <STORE_ROOT>"));
    assert!(stdout.contains("--json"));
}

#[test]
fn store_index_help_prints_structured_clap_help() {
    let assert = mycel_command(&["store", "index", "--help"])
        .assert()
        .success();
    let stdout = assert_stdout_text(&assert);

    assert_eq!(assert_stderr_text(&assert), "");
    assert!(stdout.contains("Query persisted local object-store indexes"));
    assert!(stdout.contains("Usage: mycel store index [OPTIONS] <STORE_ROOT>"));
    assert!(stdout.contains("--doc-id <DOC_ID>"));
    assert!(stdout.contains("--author <AUTHOR>"));
    assert!(stdout.contains("--revision-id <REVISION_ID>"));
    assert!(stdout.contains("--view-id <VIEW_ID>"));
    assert!(stdout.contains("--profile-id <PROFILE_ID>"));
    assert!(stdout.contains("--object-type <OBJECT_TYPE>"));
    assert!(stdout.contains("--path-only"));
    assert!(stdout.contains("--filters-only"));
    assert!(stdout.contains("--counts-only"));
    assert!(stdout.contains("--manifest-only"));
    assert!(stdout.contains("--empty-ok"));
    assert!(stdout.contains("--doc-only"));
    assert!(stdout.contains("--head-only"));
    assert!(stdout.contains("--governance-only"));
    assert!(stdout.contains("--patches-only"));
    assert!(stdout.contains("--parents-only"));
    assert!(stdout.contains("--json"));
}

#[test]
fn validate_help_prints_structured_clap_help() {
    let assert = mycel_command(&["validate", "--help"]).assert().success();
    let stdout = assert_stdout_text(&assert);

    assert_eq!(assert_stderr_text(&assert), "");
    assert_validate_help(&stdout);
}
