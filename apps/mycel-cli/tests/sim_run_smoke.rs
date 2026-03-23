use std::fs;
use std::path::{Path, PathBuf};
use std::sync::{Mutex, MutexGuard, OnceLock};
use std::time::{SystemTime, UNIX_EPOCH};

use serde_json::Value;

mod common;

use common::{
    assert_empty_stderr, assert_exit_code, assert_stderr_contains, assert_stderr_starts_with,
    assert_stdout_contains, create_temp_dir, load_report, parse_json_stdout, repo_root,
    run_mycel_in_dir, run_sim, stderr_text, validate_generated_report,
};

fn sim_run_lock() -> MutexGuard<'static, ()> {
    static LOCK: OnceLock<Mutex<()>> = OnceLock::new();
    LOCK.get_or_init(|| Mutex::new(()))
        .lock()
        .unwrap_or_else(|poisoned| poisoned.into_inner())
}

fn assert_runtime_seed_mode(summary: &Value, report: &Value, expected_source: &str) {
    let deterministic_seed = summary["deterministic_seed"]
        .as_str()
        .expect("deterministic_seed should be a string");
    assert_eq!(summary["seed_source"], expected_source);
    assert!(
        deterministic_seed.starts_with(&format!("{expected_source}:")),
        "expected seed '{deterministic_seed}' to start with '{expected_source}:'"
    );

    let metadata = report["metadata"]
        .as_object()
        .expect("report metadata should be an object");
    assert_eq!(metadata["seed_source"], expected_source);
    assert_eq!(metadata["deterministic_seed"], deterministic_seed);
}

struct TempWorkspace {
    root: PathBuf,
}

impl Drop for TempWorkspace {
    fn drop(&mut self) {
        let _ = fs::remove_dir_all(&self.root);
    }
}

fn create_temp_workspace(prefix: &str) -> TempWorkspace {
    let unique = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .expect("system time should be after unix epoch")
        .as_nanos();
    let root = std::env::temp_dir().join(format!(
        "mycel-cli-{prefix}-{}-{unique}",
        std::process::id()
    ));

    copy_dir_recursive(&repo_root().join("fixtures"), &root.join("fixtures"));
    copy_dir_recursive(&repo_root().join("sim"), &root.join("sim"));
    fs::copy(repo_root().join("Cargo.toml"), root.join("Cargo.toml"))
        .expect("Cargo.toml should copy into temporary workspace");

    TempWorkspace { root }
}

fn copy_dir_recursive(source: &Path, destination: &Path) {
    fs::create_dir_all(destination).expect("destination directory should be created");

    for entry in fs::read_dir(source).expect("source directory should be readable") {
        let entry = entry.expect("directory entry should load");
        let entry_path = entry.path();
        let destination_path = destination.join(entry.file_name());
        let entry_type = entry.file_type().expect("file type should load");

        if entry_type.is_dir() {
            copy_dir_recursive(&entry_path, &destination_path);
        } else {
            fs::copy(&entry_path, &destination_path).unwrap_or_else(|err| {
                panic!(
                    "failed to copy {} to {}: {err}",
                    entry_path.display(),
                    destination_path.display()
                )
            });
        }
    }
}

fn write_json_fixture(workspace: &TempWorkspace, relative_path: &str, value: &Value) -> PathBuf {
    let path = workspace.root.join(relative_path);
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent).expect("parent directory should exist");
    }
    fs::write(
        &path,
        serde_json::to_string_pretty(value).expect("json should serialize"),
    )
    .unwrap_or_else(|err| panic!("failed to write {}: {err}", path.display()));
    path
}

fn load_json_value(path: &Path) -> Value {
    let content = fs::read_to_string(path)
        .unwrap_or_else(|err| panic!("failed to read {}: {err}", path.display()));
    serde_json::from_str(&content)
        .unwrap_or_else(|err| panic!("failed to parse {}: {err}", path.display()))
}

#[test]
fn three_peer_consistency_run_produces_pass_report() {
    let _guard = sim_run_lock();
    let output = run_sim(&[
        "sim",
        "run",
        "sim/tests/three-peer-consistency.example.json",
        "--json",
    ]);

    assert!(
        output.status.success(),
        "expected success, stderr: {}",
        String::from_utf8_lossy(&output.stderr)
    );

    let summary = parse_json_stdout(&output);
    assert_eq!(summary["result"], "pass");
    assert_eq!(summary["seed_source"], "derived");
    assert_eq!(summary["validation_status"], "ok");
    assert_eq!(summary["peer_count"], 3);
    assert!(
        summary["event_count"]
            .as_u64()
            .expect("event_count should be numeric")
            >= 1,
        "expected events in run summary"
    );

    let outcomes = summary["matched_expected_outcomes"]
        .as_array()
        .expect("matched_expected_outcomes should be an array");
    assert!(
        outcomes.iter().any(|entry| entry == "sync-success"),
        "expected sync-success outcome, stdout: {}",
        String::from_utf8_lossy(&output.stdout)
    );

    let report = load_report(&summary);
    assert_eq!(report["result"], "pass");
    assert_eq!(report["metadata"]["run_mode"], "peer-store-sync");
    let events = report["events"]
        .as_array()
        .expect("events should be an array");
    assert!(
        events
            .iter()
            .any(|entry| entry["action"] == "seed-advertise"),
        "expected seed-advertise event in report"
    );

    let validation = validate_generated_report(&summary);
    assert_eq!(validation["report_count"], 1);
}

#[test]
fn hash_mismatch_run_produces_fault_plan_and_fail_result() {
    let _guard = sim_run_lock();
    let output = run_sim(&[
        "sim",
        "run",
        "sim/tests/hash-mismatch.example.json",
        "--json",
        "--seed",
        "custom-seed",
    ]);

    assert!(
        output.status.success(),
        "expected success, stderr: {}",
        String::from_utf8_lossy(&output.stderr)
    );

    let summary = parse_json_stdout(&output);
    assert_eq!(summary["result"], "fail");
    assert_eq!(summary["deterministic_seed"], "custom-seed");
    assert_eq!(summary["seed_source"], "override");
    assert_eq!(summary["rejected_object_count"], 1);

    let fault_plan = summary["fault_plan"]
        .as_array()
        .expect("fault_plan should be an array");
    assert_eq!(fault_plan.len(), 1);
    assert_eq!(fault_plan[0]["fault"], "hash-mismatch");

    let report = load_report(&summary);
    let events = report["events"]
        .as_array()
        .expect("events should be an array");
    let failures = report["failures"]
        .as_array()
        .expect("failures should be an array");
    assert!(
        events.iter().any(|entry| entry["action"] == "inject-fault"),
        "expected inject-fault event in report"
    );
    assert!(
        events
            .iter()
            .any(|entry| entry["action"] == "reject-object-set"),
        "expected reject-object-set event in report"
    );
    assert!(
        failures.iter().any(|entry| {
            entry["description"]
                .as_str()
                .is_some_and(|description| description.contains("Reader rejected planned fault"))
        }),
        "expected reader rejection failure in report"
    );

    let validation = validate_generated_report(&summary);
    assert_eq!(validation["report_count"], 1);
}

#[test]
fn hash_mismatch_run_supports_random_seed_mode() {
    let _guard = sim_run_lock();
    let output = run_sim(&[
        "sim",
        "run",
        "sim/tests/hash-mismatch.example.json",
        "--json",
        "--seed",
        "random",
    ]);

    assert!(
        output.status.success(),
        "expected success, stderr: {}",
        String::from_utf8_lossy(&output.stderr)
    );

    let summary = parse_json_stdout(&output);
    assert_eq!(summary["result"], "fail");
    assert_eq!(summary["validation_status"], "ok");

    let report = load_report(&summary);
    assert_runtime_seed_mode(&summary, &report, "random");

    let validation = validate_generated_report(&summary);
    assert_eq!(validation["status"], "ok");
}

#[test]
fn hash_mismatch_run_supports_auto_seed_mode() {
    let _guard = sim_run_lock();
    let output = run_sim(&[
        "sim",
        "run",
        "sim/tests/hash-mismatch.example.json",
        "--json",
        "--seed",
        "auto",
    ]);

    assert!(
        output.status.success(),
        "expected success, stderr: {}",
        String::from_utf8_lossy(&output.stderr)
    );

    let summary = parse_json_stdout(&output);
    assert_eq!(summary["result"], "fail");
    assert_eq!(summary["validation_status"], "ok");

    let report = load_report(&summary);
    assert_runtime_seed_mode(&summary, &report, "auto");

    let validation = validate_generated_report(&summary);
    assert_eq!(validation["status"], "ok");
}

#[test]
fn partial_want_recovery_run_records_recovery_flow() {
    let _guard = sim_run_lock();
    let output = run_sim(&[
        "sim",
        "run",
        "sim/tests/partial-want-recovery.example.json",
        "--json",
    ]);

    assert!(
        output.status.success(),
        "expected success, stderr: {}",
        String::from_utf8_lossy(&output.stderr)
    );

    let summary = parse_json_stdout(&output);
    assert_eq!(summary["result"], "pass");
    assert_eq!(summary["seed_source"], "derived");
    assert_eq!(summary["fault_plan"], Value::Array(Vec::new()));

    let outcomes = summary["matched_expected_outcomes"]
        .as_array()
        .expect("matched_expected_outcomes should be an array");
    assert!(
        outcomes.iter().any(|entry| entry == "recovery-success"),
        "expected recovery-success outcome, stdout: {}",
        String::from_utf8_lossy(&output.stdout)
    );

    let report = load_report(&summary);
    assert_eq!(report["metadata"]["run_mode"], "peer-store-sync");
    let events = report["events"]
        .as_array()
        .expect("events should be an array");
    assert!(
        events.iter().any(|entry| {
            entry["action"] == "request-missing-objects"
                && entry["detail"]
                    .as_str()
                    .is_some_and(|detail| detail.contains("HEADS/WANT"))
        }),
        "expected request-missing-objects event in report"
    );

    let validation = validate_generated_report(&summary);
    assert_eq!(validation["report_count"], 1);
}

#[test]
fn mixed_reader_recovery_run_converges_across_partial_and_empty_readers() {
    let _guard = sim_run_lock();
    let output = run_sim(&[
        "sim",
        "run",
        "sim/tests/mixed-reader-recovery.example.json",
        "--json",
    ]);

    assert!(
        output.status.success(),
        "expected success, stderr: {}",
        String::from_utf8_lossy(&output.stderr)
    );

    let summary = parse_json_stdout(&output);
    assert_eq!(summary["result"], "pass");
    let outcomes = summary["matched_expected_outcomes"]
        .as_array()
        .expect("matched_expected_outcomes should be an array");
    assert!(
        outcomes.iter().any(|entry| entry == "recovery-success"),
        "expected recovery-success outcome, stdout: {}",
        String::from_utf8_lossy(&output.stdout)
    );
    assert!(
        outcomes.iter().any(|entry| entry == "matching-object-sets"),
        "expected matching-object-sets outcome, stdout: {}",
        String::from_utf8_lossy(&output.stdout)
    );

    let report = load_report(&summary);
    assert_eq!(report["metadata"]["run_mode"], "peer-store-sync");

    let peers = report["peers"]
        .as_array()
        .expect("peers should be an array");
    let reader_a = peers
        .iter()
        .find(|peer| peer["node_id"] == "node:peer-reader-a")
        .expect("expected partial reader in report");
    let reader_b = peers
        .iter()
        .find(|peer| peer["node_id"] == "node:peer-reader-b")
        .expect("expected empty reader in report");
    assert_eq!(
        reader_a["verified_object_ids"], reader_b["verified_object_ids"],
        "expected both readers to converge on the same object set, report: {report}"
    );

    let events = report["events"]
        .as_array()
        .expect("events should be an array");
    assert!(
        events.iter().any(|entry| {
            entry["node_id"] == "node:peer-reader-a"
                && entry["action"] == "request-missing-objects"
                && entry["detail"]
                    .as_str()
                    .is_some_and(|detail| detail.contains("HEADS/WANT"))
        }),
        "expected partial reader recovery event in report"
    );
    assert!(
        events.iter().any(|entry| {
            entry["node_id"] == "node:peer-reader-b"
                && entry["action"] == "reader-accept"
                && entry["detail"]
                    .as_str()
                    .is_some_and(|detail| detail.contains("MANIFEST/WANT"))
        }),
        "expected empty reader sync event in report"
    );

    let validation = validate_generated_report(&summary);
    assert_eq!(validation["report_count"], 1);
}

#[test]
fn sim_run_with_seed_view_sync_publishes_governance_views_to_readers() {
    let _guard = sim_run_lock();
    let workspace = create_temp_workspace("sim-view-sync");

    let mut fixture = load_json_value(
        &workspace
            .root
            .join("fixtures/object-sets/minimal-valid/fixture.json"),
    );
    fixture["fixture_id"] = Value::String("view-sync-minimal".to_owned());
    fixture["metadata"] = serde_json::json!({
        "publish_seed_view": true
    });
    let fixture_path = write_json_fixture(
        &workspace,
        "fixtures/object-sets/view-sync-minimal/fixture.json",
        &fixture,
    );

    let mut topology = load_json_value(
        &workspace
            .root
            .join("sim/topologies/three-peer-consistency.example.json"),
    );
    topology["topology_id"] = Value::String("view-sync-three-peer".to_owned());
    topology["fixture_set"] = Value::String("fixtures/object-sets/view-sync-minimal".to_owned());
    let topology_path = write_json_fixture(
        &workspace,
        "sim/topologies/view-sync-three-peer.example.json",
        &topology,
    );

    let mut test_case = load_json_value(
        &workspace
            .root
            .join("sim/tests/three-peer-consistency.example.json"),
    );
    test_case["test_id"] = Value::String("view-sync-three-peer".to_owned());
    test_case["topology"] = Value::String(
        topology_path
            .strip_prefix(&workspace.root)
            .expect("topology path should live under the temporary workspace")
            .to_string_lossy()
            .into_owned(),
    );
    test_case["fixture_set"] = Value::String(
        fixture_path
            .parent()
            .expect("fixture path should have a parent")
            .strip_prefix(&workspace.root)
            .expect("fixture path should live under the temporary workspace")
            .to_string_lossy()
            .into_owned(),
    );
    let target_path = write_json_fixture(
        &workspace,
        "sim/tests/view-sync-three-peer.example.json",
        &test_case,
    );
    let target_owned = target_path.to_string_lossy().into_owned();

    let output = run_mycel_in_dir(&workspace.root, &["sim", "run", &target_owned, "--json"]);

    assert!(
        output.status.success(),
        "expected success, stderr: {}",
        String::from_utf8_lossy(&output.stderr)
    );

    let summary = parse_json_stdout(&output);
    assert_eq!(summary["result"], "pass");
    let report = load_report(&summary);
    let peers = report["peers"]
        .as_array()
        .expect("peers should be an array");
    let reader = peers
        .iter()
        .find(|peer| peer["node_id"] == "node:peer-reader-a")
        .expect("expected reader peer in report");
    let verified_object_ids = reader["verified_object_ids"]
        .as_array()
        .expect("verified_object_ids should be an array");
    assert!(
        verified_object_ids.iter().any(|value| value
            .as_str()
            .is_some_and(|object_id| object_id.starts_with("view:"))),
        "expected synchronized view object in reader set, report: {report}"
    );

    let events = report["events"]
        .as_array()
        .expect("events should be an array");
    assert!(
        events.iter().any(|entry| {
            entry["action"] == "reader-accept"
                && entry["detail"]
                    .as_str()
                    .is_some_and(|detail| detail.contains("transferred 2 OBJECT messages"))
        }),
        "expected reader-accept event to report view-aware peer-store transfer"
    );
}

#[test]
fn sim_run_rejects_view_announce_without_advertised_capability() {
    let _guard = sim_run_lock();
    let output = run_sim(&[
        "sim",
        "run",
        "sim/tests/view-sync-without-capability.example.json",
        "--json",
    ]);

    assert!(
        output.status.success(),
        "expected success, stderr: {}",
        String::from_utf8_lossy(&output.stderr)
    );

    let summary = parse_json_stdout(&output);
    assert_eq!(summary["result"], "fail");
    assert_eq!(summary["rejected_object_count"], 0);
    let outcomes = summary["matched_expected_outcomes"]
        .as_array()
        .expect("matched_expected_outcomes should be an array");
    assert!(
        outcomes
            .iter()
            .any(|entry| entry == "view-announce-capability-rejected"),
        "expected capability-rejected outcome, stdout: {}",
        String::from_utf8_lossy(&output.stdout)
    );

    let report = load_report(&summary);
    let failures = report["failures"]
        .as_array()
        .expect("failures should be an array");
    assert!(
        failures.iter().any(|entry| entry["description"]
            .as_str()
            .is_some_and(|description| description
                .contains("wire VIEW_ANNOUNCE requires advertised capability 'view-sync'"))),
        "expected capability error in report failures, report: {report}"
    );
    let reader = report["peers"]
        .as_array()
        .expect("peers should be an array")
        .iter()
        .find(|peer| peer["node_id"] == "node:peer-reader-a")
        .expect("expected reader peer in report");
    assert!(
        reader["notes"]
            .as_array()
            .is_some_and(|notes| notes.iter().any(|note| {
                note.as_str()
                    .is_some_and(|text| text.contains("suppressed advertised capabilities"))
            })),
        "expected reader notes to mention suppressed capabilities, report: {report}"
    );
}

#[test]
fn sim_run_rejects_snapshot_offer_without_advertised_capability() {
    let _guard = sim_run_lock();
    let output = run_sim(&[
        "sim",
        "run",
        "sim/tests/snapshot-sync-without-capability.example.json",
        "--json",
    ]);

    assert!(
        output.status.success(),
        "expected success, stderr: {}",
        String::from_utf8_lossy(&output.stderr)
    );

    let summary = parse_json_stdout(&output);
    assert_eq!(summary["result"], "fail");
    assert_eq!(summary["rejected_object_count"], 0);
    let outcomes = summary["matched_expected_outcomes"]
        .as_array()
        .expect("matched_expected_outcomes should be an array");
    assert!(
        outcomes
            .iter()
            .any(|entry| entry == "snapshot-offer-capability-rejected"),
        "expected capability-rejected outcome, stdout: {}",
        String::from_utf8_lossy(&output.stdout)
    );

    let report = load_report(&summary);
    let failures = report["failures"]
        .as_array()
        .expect("failures should be an array");
    assert!(
        failures.iter().any(|entry| entry["description"]
            .as_str()
            .is_some_and(|description| description
                .contains("wire SNAPSHOT_OFFER requires advertised capability 'snapshot-sync'"))),
        "expected capability error in report failures, report: {report}"
    );
    let reader = report["peers"]
        .as_array()
        .expect("peers should be an array")
        .iter()
        .find(|peer| peer["node_id"] == "node:peer-reader-a")
        .expect("expected reader peer in report");
    assert!(
        reader["notes"]
            .as_array()
            .is_some_and(|notes| notes.iter().any(|note| {
                note.as_str()
                    .is_some_and(|text| text.contains("suppressed advertised capabilities"))
            })),
        "expected reader notes to mention suppressed capabilities, report: {report}"
    );
}

#[test]
fn sim_run_rejects_messages_after_bye() {
    let _guard = sim_run_lock();
    let output = run_sim(&[
        "sim",
        "run",
        "sim/tests/session-messages-after-bye.example.json",
        "--json",
    ]);

    assert!(
        output.status.success(),
        "expected success, stderr: {}",
        String::from_utf8_lossy(&output.stderr)
    );

    let summary = parse_json_stdout(&output);
    assert_eq!(summary["result"], "fail");
    let outcomes = summary["matched_expected_outcomes"]
        .as_array()
        .expect("matched_expected_outcomes should be an array");
    assert!(
        outcomes
            .iter()
            .any(|entry| entry == "messages-after-bye-rejected"),
        "expected messages-after-bye outcome, stdout: {}",
        String::from_utf8_lossy(&output.stdout)
    );

    let report = load_report(&summary);
    let failures = report["failures"]
        .as_array()
        .expect("failures should be an array");
    assert!(
        failures.iter().any(|entry| entry["description"]
            .as_str()
            .is_some_and(|description| description
                .contains("wire session for 'node:peer-seed' is already closed"))),
        "expected closed-session error in report failures, report: {report}"
    );
    let reader = report["peers"]
        .as_array()
        .expect("peers should be an array")
        .iter()
        .find(|peer| peer["node_id"] == "node:peer-reader-a")
        .expect("expected reader peer in report");
    assert!(
        reader["verified_object_ids"]
            .as_array()
            .is_some_and(|verified| verified.is_empty()),
        "expected no verified objects after closed-session rejection, report: {report}"
    );
    assert!(
        reader["notes"]
            .as_array()
            .is_some_and(|notes| notes.iter().any(|note| {
                note.as_str()
                    .is_some_and(|text| text.contains("session fault"))
            })),
        "expected reader notes to mention the injected session fault, report: {report}"
    );
}

#[test]
fn sim_run_rejects_duplicate_hello() {
    let _guard = sim_run_lock();
    let output = run_sim(&[
        "sim",
        "run",
        "sim/tests/session-duplicate-hello.example.json",
        "--json",
    ]);

    assert!(
        output.status.success(),
        "expected success, stderr: {}",
        String::from_utf8_lossy(&output.stderr)
    );

    let summary = parse_json_stdout(&output);
    assert_eq!(summary["result"], "fail");
    let outcomes = summary["matched_expected_outcomes"]
        .as_array()
        .expect("matched_expected_outcomes should be an array");
    assert!(
        outcomes
            .iter()
            .any(|entry| entry == "duplicate-hello-rejected"),
        "expected duplicate-hello outcome, stdout: {}",
        String::from_utf8_lossy(&output.stdout)
    );

    let report = load_report(&summary);
    let failures = report["failures"]
        .as_array()
        .expect("failures should be an array");
    assert!(
        failures.iter().any(|entry| entry["description"]
            .as_str()
            .is_some_and(|description| description
                .contains("wire session already received HELLO from 'node:peer-seed'"))),
        "expected duplicate-HELLO error in report failures, report: {report}"
    );
    let reader = report["peers"]
        .as_array()
        .expect("peers should be an array")
        .iter()
        .find(|peer| peer["node_id"] == "node:peer-reader-a")
        .expect("expected reader peer in report");
    assert!(
        reader["verified_object_ids"]
            .as_array()
            .is_some_and(|verified| verified.is_empty()),
        "expected no verified objects after duplicate-HELLO rejection, report: {report}"
    );
    assert!(
        reader["notes"]
            .as_array()
            .is_some_and(|notes| notes.iter().any(|note| {
                note.as_str()
                    .is_some_and(|text| text.contains("session fault"))
            })),
        "expected reader notes to mention the injected session fault, report: {report}"
    );
}

#[test]
fn sim_run_rejects_manifest_before_hello() {
    let _guard = sim_run_lock();
    let output = run_sim(&[
        "sim",
        "run",
        "sim/tests/session-manifest-before-hello.example.json",
        "--json",
    ]);

    assert!(
        output.status.success(),
        "expected success, stderr: {}",
        String::from_utf8_lossy(&output.stderr)
    );

    let summary = parse_json_stdout(&output);
    assert_eq!(summary["result"], "fail");
    let outcomes = summary["matched_expected_outcomes"]
        .as_array()
        .expect("matched_expected_outcomes should be an array");
    assert!(
        outcomes
            .iter()
            .any(|entry| entry == "manifest-before-hello-rejected"),
        "expected manifest-before-hello outcome, stdout: {}",
        String::from_utf8_lossy(&output.stdout)
    );

    let report = load_report(&summary);
    let failures = report["failures"]
        .as_array()
        .expect("failures should be an array");
    assert!(
        failures.iter().any(|entry| entry["description"]
            .as_str()
            .is_some_and(|description| description
                .contains("wire MANIFEST requires prior HELLO from 'node:peer-seed'"))),
        "expected prior-HELLO MANIFEST error in report failures, report: {report}"
    );
    let reader = report["peers"]
        .as_array()
        .expect("peers should be an array")
        .iter()
        .find(|peer| peer["node_id"] == "node:peer-reader-a")
        .expect("expected reader peer in report");
    assert!(
        reader["verified_object_ids"]
            .as_array()
            .is_some_and(|verified| verified.is_empty()),
        "expected no verified objects after prior-HELLO rejection, report: {report}"
    );
    assert!(
        reader["notes"]
            .as_array()
            .is_some_and(|notes| notes.iter().any(|note| {
                note.as_str()
                    .is_some_and(|text| text.contains("session fault"))
            })),
        "expected reader notes to mention the injected session fault, report: {report}"
    );
}

#[test]
fn sim_run_rejects_heads_before_hello() {
    let _guard = sim_run_lock();
    let output = run_sim(&[
        "sim",
        "run",
        "sim/tests/session-heads-before-hello.example.json",
        "--json",
    ]);

    assert!(
        output.status.success(),
        "expected success, stderr: {}",
        String::from_utf8_lossy(&output.stderr)
    );

    let summary = parse_json_stdout(&output);
    assert_eq!(summary["result"], "fail");
    let outcomes = summary["matched_expected_outcomes"]
        .as_array()
        .expect("matched_expected_outcomes should be an array");
    assert!(
        outcomes
            .iter()
            .any(|entry| entry == "heads-before-hello-rejected"),
        "expected heads-before-hello outcome, stdout: {}",
        String::from_utf8_lossy(&output.stdout)
    );

    let report = load_report(&summary);
    let failures = report["failures"]
        .as_array()
        .expect("failures should be an array");
    assert!(
        failures.iter().any(
            |entry| entry["description"].as_str().is_some_and(|description| {
                description.contains("wire HEADS requires prior HELLO from 'node:peer-seed'")
            })
        ),
        "expected prior-HELLO HEADS error in report failures, report: {report}"
    );
    let reader = report["peers"]
        .as_array()
        .expect("peers should be an array")
        .iter()
        .find(|peer| peer["node_id"] == "node:peer-reader-a")
        .expect("expected reader peer in report");
    assert!(
        reader["verified_object_ids"]
            .as_array()
            .is_some_and(|verified| verified.is_empty()),
        "expected no verified objects after prior-HELLO rejection, report: {report}"
    );
    assert!(
        reader["notes"]
            .as_array()
            .is_some_and(|notes| notes.iter().any(|note| {
                note.as_str()
                    .is_some_and(|text| text.contains("session fault"))
            })),
        "expected reader notes to mention the injected session fault, report: {report}"
    );
}

#[test]
fn sim_run_rejects_want_before_hello() {
    let _guard = sim_run_lock();
    let output = run_sim(&[
        "sim",
        "run",
        "sim/tests/session-want-before-hello.example.json",
        "--json",
    ]);

    assert!(
        output.status.success(),
        "expected success, stderr: {}",
        String::from_utf8_lossy(&output.stderr)
    );

    let summary = parse_json_stdout(&output);
    assert_eq!(summary["result"], "fail");
    let outcomes = summary["matched_expected_outcomes"]
        .as_array()
        .expect("matched_expected_outcomes should be an array");
    assert!(
        outcomes
            .iter()
            .any(|entry| entry == "want-before-hello-rejected"),
        "expected want-before-hello outcome, stdout: {}",
        String::from_utf8_lossy(&output.stdout)
    );

    let report = load_report(&summary);
    let failures = report["failures"]
        .as_array()
        .expect("failures should be an array");
    assert!(
        failures.iter().any(|entry| entry["description"]
            .as_str()
            .is_some_and(|description| description
                .contains("wire WANT requires prior HELLO from 'node:peer-seed'"))),
        "expected prior-HELLO error in report failures, report: {report}"
    );
    let reader = report["peers"]
        .as_array()
        .expect("peers should be an array")
        .iter()
        .find(|peer| peer["node_id"] == "node:peer-reader-a")
        .expect("expected reader peer in report");
    assert!(
        reader["verified_object_ids"]
            .as_array()
            .is_some_and(|verified| verified.is_empty()),
        "expected no verified objects after prior-HELLO rejection, report: {report}"
    );
    assert!(
        reader["notes"]
            .as_array()
            .is_some_and(|notes| notes.iter().any(|note| {
                note.as_str()
                    .is_some_and(|text| text.contains("session fault"))
            })),
        "expected reader notes to mention the injected session fault, report: {report}"
    );
}

#[test]
fn sim_run_rejects_want_before_manifest() {
    let _guard = sim_run_lock();
    let output = run_sim(&[
        "sim",
        "run",
        "sim/tests/session-want-before-manifest.example.json",
        "--json",
    ]);

    assert!(
        output.status.success(),
        "expected success, stderr: {}",
        String::from_utf8_lossy(&output.stderr)
    );

    let summary = parse_json_stdout(&output);
    assert_eq!(summary["result"], "fail");
    let outcomes = summary["matched_expected_outcomes"]
        .as_array()
        .expect("matched_expected_outcomes should be an array");
    assert!(
        outcomes
            .iter()
            .any(|entry| entry == "want-before-manifest-rejected"),
        "expected want-before-manifest outcome, stdout: {}",
        String::from_utf8_lossy(&output.stdout)
    );

    let report = load_report(&summary);
    let failures = report["failures"]
        .as_array()
        .expect("failures should be an array");
    assert!(
        failures.iter().any(|entry| entry["description"]
            .as_str()
            .is_some_and(|description| description
                .contains("wire WANT requires prior MANIFEST or HEADS from 'node:peer-seed'"))),
        "expected missing-head-context error in report failures, report: {report}"
    );
    let reader = report["peers"]
        .as_array()
        .expect("peers should be an array")
        .iter()
        .find(|peer| peer["node_id"] == "node:peer-reader-a")
        .expect("expected reader peer in report");
    assert!(
        reader["verified_object_ids"]
            .as_array()
            .is_some_and(|verified| verified.is_empty()),
        "expected no verified objects after missing-head-context rejection, report: {report}"
    );
    assert!(
        reader["notes"]
            .as_array()
            .is_some_and(|notes| notes.iter().any(|note| {
                note.as_str()
                    .is_some_and(|text| text.contains("session fault"))
            })),
        "expected reader notes to mention the injected session fault, report: {report}"
    );
}

#[test]
fn sim_run_rejects_object_before_manifest() {
    let _guard = sim_run_lock();
    let output = run_sim(&[
        "sim",
        "run",
        "sim/tests/session-object-before-manifest.example.json",
        "--json",
    ]);

    assert!(
        output.status.success(),
        "expected success, stderr: {}",
        String::from_utf8_lossy(&output.stderr)
    );

    let summary = parse_json_stdout(&output);
    assert_eq!(summary["result"], "fail");
    let outcomes = summary["matched_expected_outcomes"]
        .as_array()
        .expect("matched_expected_outcomes should be an array");
    assert!(
        outcomes
            .iter()
            .any(|entry| entry == "object-before-manifest-rejected"),
        "expected object-before-manifest outcome, stdout: {}",
        String::from_utf8_lossy(&output.stdout)
    );

    let report = load_report(&summary);
    let failures = report["failures"]
        .as_array()
        .expect("failures should be an array");
    assert!(
        failures.iter().any(
            |entry| entry["description"].as_str().is_some_and(
                |description| description.contains("was not requested from 'node:peer-seed'")
            )
        ),
        "expected unrequested-OBJECT error in report failures, report: {report}"
    );
    let reader = report["peers"]
        .as_array()
        .expect("peers should be an array")
        .iter()
        .find(|peer| peer["node_id"] == "node:peer-reader-a")
        .expect("expected reader peer in report");
    assert!(
        reader["verified_object_ids"]
            .as_array()
            .is_some_and(|verified| verified.is_empty()),
        "expected no verified objects after unrequested-OBJECT rejection, report: {report}"
    );
    assert!(
        reader["notes"]
            .as_array()
            .is_some_and(|notes| notes.iter().any(|note| {
                note.as_str()
                    .is_some_and(|text| text.contains("session fault"))
            })),
        "expected reader notes to mention the injected session fault, report: {report}"
    );
}

#[test]
fn sim_run_with_seed_snapshot_sync_publishes_snapshots_to_readers() {
    let _guard = sim_run_lock();
    let workspace = create_temp_workspace("sim-snapshot-sync");

    let mut fixture = load_json_value(
        &workspace
            .root
            .join("fixtures/object-sets/minimal-valid/fixture.json"),
    );
    fixture["fixture_id"] = Value::String("snapshot-sync-minimal".to_owned());
    fixture["metadata"] = serde_json::json!({
        "publish_seed_snapshot": true
    });
    let fixture_path = write_json_fixture(
        &workspace,
        "fixtures/object-sets/snapshot-sync-minimal/fixture.json",
        &fixture,
    );

    let mut topology = load_json_value(
        &workspace
            .root
            .join("sim/topologies/three-peer-consistency.example.json"),
    );
    topology["topology_id"] = Value::String("snapshot-sync-three-peer".to_owned());
    topology["fixture_set"] =
        Value::String("fixtures/object-sets/snapshot-sync-minimal".to_owned());
    let topology_path = write_json_fixture(
        &workspace,
        "sim/topologies/snapshot-sync-three-peer.example.json",
        &topology,
    );

    let mut test_case = load_json_value(
        &workspace
            .root
            .join("sim/tests/three-peer-consistency.example.json"),
    );
    test_case["test_id"] = Value::String("snapshot-sync-three-peer".to_owned());
    test_case["topology"] = Value::String(
        topology_path
            .strip_prefix(&workspace.root)
            .expect("topology path should live under the temporary workspace")
            .to_string_lossy()
            .into_owned(),
    );
    test_case["fixture_set"] = Value::String(
        fixture_path
            .parent()
            .expect("fixture path should have a parent")
            .strip_prefix(&workspace.root)
            .expect("fixture path should live under the temporary workspace")
            .to_string_lossy()
            .into_owned(),
    );
    let target_path = write_json_fixture(
        &workspace,
        "sim/tests/snapshot-sync-three-peer.example.json",
        &test_case,
    );
    let target_owned = target_path.to_string_lossy().into_owned();

    let output = run_mycel_in_dir(&workspace.root, &["sim", "run", &target_owned, "--json"]);

    assert!(
        output.status.success(),
        "expected success, stderr: {}",
        String::from_utf8_lossy(&output.stderr)
    );

    let summary = parse_json_stdout(&output);
    assert_eq!(summary["result"], "pass");
    let report = load_report(&summary);
    let peers = report["peers"]
        .as_array()
        .expect("peers should be an array");
    let reader = peers
        .iter()
        .find(|peer| peer["node_id"] == "node:peer-reader-a")
        .expect("expected reader peer in report");
    let verified_object_ids = reader["verified_object_ids"]
        .as_array()
        .expect("verified_object_ids should be an array");
    assert!(
        verified_object_ids.iter().any(|value| value
            .as_str()
            .is_some_and(|object_id| object_id.starts_with("snap:"))),
        "expected synchronized snapshot object in reader set, report: {report}"
    );

    let events = report["events"]
        .as_array()
        .expect("events should be an array");
    assert!(
        events.iter().any(|entry| {
            entry["action"] == "reader-accept"
                && entry["detail"]
                    .as_str()
                    .is_some_and(|detail| detail.contains("transferred 2 OBJECT messages"))
        }),
        "expected reader-accept event to report snapshot-aware peer-store transfer"
    );
}

#[test]
fn signature_mismatch_run_produces_fault_plan_and_fail_result() {
    let _guard = sim_run_lock();
    let output = run_sim(&[
        "sim",
        "run",
        "sim/tests/signature-mismatch.example.json",
        "--json",
    ]);

    assert!(
        output.status.success(),
        "expected success, stderr: {}",
        String::from_utf8_lossy(&output.stderr)
    );

    let summary = parse_json_stdout(&output);
    assert_eq!(summary["result"], "fail");
    assert_eq!(summary["seed_source"], "derived");
    assert_eq!(summary["rejected_object_count"], 1);

    let fault_plan = summary["fault_plan"]
        .as_array()
        .expect("fault_plan should be an array");
    assert_eq!(fault_plan.len(), 1);
    assert_eq!(fault_plan[0]["fault"], "signature-mismatch");

    let report = load_report(&summary);
    let events = report["events"]
        .as_array()
        .expect("events should be an array");
    let failures = report["failures"]
        .as_array()
        .expect("failures should be an array");
    assert!(
        events.iter().any(|entry| entry["action"] == "inject-fault"),
        "expected inject-fault event in report"
    );
    assert!(
        events
            .iter()
            .any(|entry| entry["action"] == "reject-object-set"),
        "expected reject-object-set event in report"
    );
    assert!(
        failures.iter().any(|entry| {
            entry["description"]
                .as_str()
                .is_some_and(|description| description.contains("Reader rejected planned fault"))
        }),
        "expected reader rejection failure in report"
    );

    let validation = validate_generated_report(&summary);
    assert_eq!(validation["report_count"], 1);
}

#[test]
fn three_peer_consistency_run_text_reports_human_summary() {
    let _guard = sim_run_lock();
    let output = run_sim(&[
        "sim",
        "run",
        "sim/tests/three-peer-consistency.example.json",
    ]);

    assert_exit_code(&output, 0);
    assert_empty_stderr(&output);
    assert_stdout_contains(&output, "repo root:");
    assert_stdout_contains(&output, "run target:");
    assert_stdout_contains(&output, "seed source: derived");
    assert_stdout_contains(&output, "fault plan: none");
    assert_stdout_contains(&output, "validation status: ok");
    assert_stdout_contains(&output, "result: pass");
    assert_stdout_contains(&output, "matched expected outcomes:");
}

#[test]
fn hash_mismatch_run_text_reports_fault_summary() {
    let _guard = sim_run_lock();
    let output = run_sim(&[
        "sim",
        "run",
        "sim/tests/hash-mismatch.example.json",
        "--seed",
        "custom-seed",
    ]);

    assert_exit_code(&output, 0);
    assert_empty_stderr(&output);
    assert_stdout_contains(&output, "deterministic seed: custom-seed");
    assert_stdout_contains(&output, "seed source: override");
    assert_stdout_contains(&output, "fault plan: #1:hash-mismatch:");
    assert_stdout_contains(&output, "validation status: ok");
    assert_stdout_contains(&output, "result: fail");
    assert_stdout_contains(&output, "rejected objects: 1");
}

#[test]
fn sim_run_outside_repo_reports_root_detection_failure() {
    let _guard = sim_run_lock();
    let temp_dir = create_temp_dir("sim-run-outside-repo");
    let output = run_mycel_in_dir(
        temp_dir.path(),
        &["sim", "run", "sim/tests/hash-mismatch.example.json"],
    );

    assert_exit_code(&output, 1);
    assert_stderr_starts_with(&output, "error: ");
    assert_stderr_contains(&output, "sim run failed:");
    assert_stderr_contains(&output, "could not find repository root");
}

#[test]
fn sim_run_rejects_unsupported_test_case_execution_mode() {
    let _guard = sim_run_lock();
    let workspace = create_temp_workspace("unsupported-test-mode");
    let mut topology = load_json_value(
        &workspace
            .root
            .join("sim/topologies/hash-mismatch.example.json"),
    );
    topology["topology_id"] = Value::String("unsupported-test-execution-mode".to_owned());
    topology["execution_mode"] = Value::String("distributed".to_owned());

    let topology_path = write_json_fixture(
        &workspace,
        "sim/topologies/unsupported-test-execution-mode.example.json",
        &topology,
    );

    let mut test_case =
        load_json_value(&workspace.root.join("sim/tests/hash-mismatch.example.json"));
    test_case["test_id"] = Value::String("unsupported-test-execution-mode".to_owned());
    test_case["execution_mode"] = Value::String("distributed".to_owned());
    test_case["topology"] = Value::String(
        topology_path
            .strip_prefix(&workspace.root)
            .expect("topology path should live under the temporary workspace")
            .to_string_lossy()
            .into_owned(),
    );

    let target_path = write_json_fixture(
        &workspace,
        "sim/tests/unsupported-test-execution-mode.example.json",
        &test_case,
    );
    let target_owned = target_path.to_string_lossy().into_owned();
    let output = run_mycel_in_dir(&workspace.root, &["sim", "run", &target_owned]);

    assert_exit_code(&output, 1);
    assert_stderr_contains(
        &output,
        "unsupported execution_mode 'distributed'; supported: single-process, multi-process",
    );
}

#[test]
fn sim_run_rejects_unsupported_topology_execution_mode() {
    let _guard = sim_run_lock();
    let workspace = create_temp_workspace("unsupported-topology-mode");
    let mut topology = load_json_value(
        &workspace
            .root
            .join("sim/topologies/hash-mismatch.example.json"),
    );
    topology["topology_id"] = Value::String("unsupported-topology-execution-mode".to_owned());
    topology["execution_mode"] = Value::String("distributed".to_owned());

    let topology_path = write_json_fixture(
        &workspace,
        "sim/topologies/unsupported-topology-execution-mode.example.json",
        &topology,
    );

    let mut test_case =
        load_json_value(&workspace.root.join("sim/tests/hash-mismatch.example.json"));
    test_case["test_id"] = Value::String("unsupported-topology-execution-mode".to_owned());
    test_case["topology"] = Value::String(
        topology_path
            .strip_prefix(&workspace.root)
            .expect("topology path should live under the temporary workspace")
            .to_string_lossy()
            .into_owned(),
    );

    let target_path = write_json_fixture(
        &workspace,
        "sim/tests/unsupported-topology-execution-mode.example.json",
        &test_case,
    );
    let target_owned = target_path.to_string_lossy().into_owned();
    let output = run_mycel_in_dir(&workspace.root, &["sim", "run", &target_owned]);

    assert_exit_code(&output, 1);
    assert_stderr_contains(
        &output,
        "does not match topology execution_mode 'distributed'",
    );
}

#[test]
fn sim_run_rejects_schema_file_targets() {
    let _guard = sim_run_lock();
    let output = run_sim(&["sim", "run", "sim/tests/test-case.schema.json"]);

    assert_exit_code(&output, 1);
    assert_stderr_contains(&output, "schema files are not");
}

#[test]
fn sim_run_requires_seed_value_after_flag() {
    let _guard = sim_run_lock();
    let output = run_sim(&[
        "sim",
        "run",
        "sim/tests/hash-mismatch.example.json",
        "--seed",
    ]);

    assert_exit_code(&output, 2);
    assert_stderr_contains(&output, "a value is required for '--seed <SEED>'");
}

#[test]
fn sim_run_rejects_unexpected_extra_arguments() {
    let _guard = sim_run_lock();
    let output = run_sim(&[
        "sim",
        "run",
        "sim/tests/hash-mismatch.example.json",
        "--json",
        "unexpected",
    ]);

    assert_exit_code(&output, 2);
    assert_stderr_contains(&output, "unexpected sim run argument: unexpected");
}

#[test]
fn sim_requires_subcommand() {
    let _guard = sim_run_lock();
    let output = run_sim(&["sim"]);

    assert_exit_code(&output, 2);
    assert_stderr_contains(&output, "missing sim subcommand");
}

#[test]
fn sim_rejects_unknown_subcommand() {
    let _guard = sim_run_lock();
    let output = run_sim(&["sim", "bogus"]);

    assert_exit_code(&output, 2);
    assert_stderr_contains(&output, "unknown sim subcommand: bogus");
}

#[test]
fn sim_run_requires_target_path() {
    let _guard = sim_run_lock();
    let output = run_sim(&["sim", "run"]);

    assert_exit_code(&output, 2);
    assert_stderr_contains(&output, "required arguments were not provided");
    assert_stderr_contains(&output, "<PATH>");
}

#[test]
fn sim_run_rejects_directory_targets() {
    let _guard = sim_run_lock();
    let output = run_sim(&["sim", "run", "sim/tests"]);

    assert_exit_code(&output, 1);
    let stderr = stderr_text(&output);
    assert!(
        stderr.contains("failed to read") && stderr.contains("Is a directory"),
        "expected directory target read failure, stderr: {stderr}"
    );
}
