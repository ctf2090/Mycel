use std::env;
use std::fs;
use std::path::PathBuf;

use mycel_core::head::inspect_heads_from_path;
use mycel_core::head::HeadInspectSummary;
use mycel_core::verify::{verify_object_path, ObjectVerificationSummary};
use mycel_core::workspace_banner;
use mycel_sim::manifest::SimulatorPaths;
use mycel_sim::model::{Report, ReportEvent, ReportFailure};
use mycel_sim::run::{run_test_case_with_options, RunOptions};
use mycel_sim::simulator_banner;
use mycel_sim::validate::validate_path;

fn print_usage() {
    println!("mycel <command> [path]");
    println!();
    println!("Commands:");
    println!("  head       Inspect accepted-head selection from a local input bundle");
    println!("  info       Show workspace and simulator scaffold information");
    println!("  object     Verify one Mycel object file");
    println!("  report     Inspect one simulator report");
    println!("  sim        Run a simulator test case");
    println!("  validate   Validate the repo root, one file, or one supported directory");
    println!("  help       Show this message");
    println!();
    println!("Head options:");
    println!("  inspect <doc_id> --input <path|fixture>  Inspect one document's accepted head");
    println!(
        "  --json                                   Emit machine-readable head inspection output"
    );
    println!();
    println!("Object options:");
    println!("  verify <path>  Verify one object file");
    println!("  --json         Emit machine-readable object verification output");
    println!();
    println!("Report options:");
    println!("  inspect <path>  Inspect one simulator report");
    println!("  --json          Emit machine-readable report inspection output");
    println!("  --events        Show only report events");
    println!("  --failures      Show only report failures");
    println!("  --full          Emit the full raw report (requires --json)");
    println!("  --phase <name>  Filter event inspection to one phase");
    println!();
    println!("Sim options:");
    println!("  run <path> Run one test-case and write a report to sim/reports/out/");
    println!("  --json     Emit machine-readable run output");
    println!("  --seed     Use a fixed seed, or 'random' / 'auto' to generate one");
    println!();
    println!("Validate options:");
    println!("  --json     Emit machine-readable validation output");
    println!("  --strict   Treat warnings as failures");
}

fn print_info() {
    let paths = SimulatorPaths::default();

    println!("{}", workspace_banner());
    println!("{}", simulator_banner());
    println!("fixtures: {}", paths.fixtures_root);
    println!("peers: {}", paths.peers_root);
    println!("topologies: {}", paths.topologies_root);
    println!("tests: {}", paths.tests_root);
    println!("reports: {}", paths.reports_root);
}

fn print_head_inspect_text(summary: &HeadInspectSummary) -> i32 {
    println!("input path: {}", summary.input_path.display());
    println!("doc id: {}", summary.doc_id);
    if let Some(profile_id) = &summary.profile_id {
        println!("profile id: {profile_id}");
    }
    if let Some(effective_selection_time) = summary.effective_selection_time {
        println!("effective selection time: {effective_selection_time}");
    }
    if let Some(selector_epoch) = summary.selector_epoch {
        println!("selector epoch: {selector_epoch}");
    }
    println!("verified revisions: {}", summary.verified_revision_count);
    println!("verified views: {}", summary.verified_view_count);
    println!("status: {}", summary.status);

    for head in &summary.eligible_heads {
        println!(
            "eligible head: {} timestamp={} score={} supporters={}",
            head.revision_id, head.revision_timestamp, head.selector_score, head.supporter_count
        );
    }

    if let Some(selected_head) = &summary.selected_head {
        println!("selected head: {selected_head}");
    }
    if let Some(tie_break_reason) = &summary.tie_break_reason {
        println!("tie break reason: {tie_break_reason}");
    }
    for trace in &summary.decision_trace {
        println!("trace: {}: {}", trace.step, trace.detail);
    }
    for note in &summary.notes {
        println!("note: {note}");
    }

    if summary.is_ok() {
        println!("head inspection: ok");
        0
    } else {
        println!("head inspection: failed");
        for error in &summary.errors {
            eprintln!("error: {error}");
        }
        1
    }
}

fn print_head_inspect_json(summary: &HeadInspectSummary) -> i32 {
    match serde_json::to_string_pretty(summary) {
        Ok(json) => {
            println!("{json}");
            if summary.is_ok() {
                0
            } else {
                1
            }
        }
        Err(err) => {
            eprintln!("failed to serialize head inspection summary: {err}");
            2
        }
    }
}

fn head_inspect(doc_id: String, input_path: PathBuf, json: bool) -> i32 {
    let summary = inspect_heads_from_path(&input_path, &doc_id);
    if json {
        print_head_inspect_json(&summary)
    } else {
        print_head_inspect_text(&summary)
    }
}

fn print_object_verification_text(summary: &ObjectVerificationSummary) -> i32 {
    println!("object path: {}", summary.path.display());
    if let Some(object_type) = &summary.object_type {
        println!("object type: {object_type}");
    }
    if let Some(signature_rule) = &summary.signature_rule {
        println!("signature rule: {signature_rule}");
    }
    if let Some(signer_field) = &summary.signer_field {
        println!("signer field: {signer_field}");
    }
    if let Some(signer) = &summary.signer {
        println!("signer: {signer}");
    }
    if let Some(signature_verification) = &summary.signature_verification {
        println!("signature verification: {signature_verification}");
    }
    if let Some(declared_id) = &summary.declared_id {
        println!("declared id: {declared_id}");
    }
    if let Some(recomputed_id) = &summary.recomputed_id {
        println!("recomputed id: {recomputed_id}");
    }
    println!("status: {}", summary.status);

    for note in &summary.notes {
        println!("note: {note}");
    }

    if summary.is_ok() {
        println!("verification: ok");
        0
    } else {
        println!("verification: failed");
        for error in &summary.errors {
            eprintln!("error: {error}");
        }
        1
    }
}

fn print_object_verification_json(summary: &ObjectVerificationSummary) -> i32 {
    match serde_json::to_string_pretty(summary) {
        Ok(json) => {
            println!("{json}");
            if summary.is_ok() {
                0
            } else {
                1
            }
        }
        Err(err) => {
            eprintln!("failed to serialize object verification summary: {err}");
            2
        }
    }
}

fn object_verify(target: PathBuf, json: bool) -> i32 {
    let summary = verify_object_path(&target);
    if json {
        print_object_verification_json(&summary)
    } else {
        print_object_verification_text(&summary)
    }
}

fn print_validation_text(summary: &mycel_sim::validate::ValidationSummary) -> i32 {
    if let Some(root) = &summary.root {
        println!("repo root: {}", root.display());
    }
    if let Some(target) = &summary.target {
        println!("validated target: {}", target.display());
    }
    println!("status: {}", summary.status);
    println!("fixtures: {}", summary.fixture_count);
    println!("peers: {}", summary.peer_count);
    println!("topologies: {}", summary.topology_count);
    println!("tests: {}", summary.test_case_count);
    println!("reports: {}", summary.report_count);

    if !summary.warnings.is_empty() {
        for warning in &summary.warnings {
            eprintln!("warning: {}: {}", warning.path, warning.message);
        }
    }

    if !summary.is_ok() {
        println!("validation: failed");
        for error in &summary.errors {
            eprintln!("error: {}: {}", error.path, error.message);
        }
        1
    } else if summary.has_warnings() {
        println!("validation: warning");
        0
    } else {
        println!("validation: ok");
        0
    }
}

fn print_validation_json(summary: &mycel_sim::validate::ValidationSummary) -> i32 {
    match serde_json::to_string_pretty(summary) {
        Ok(json) => {
            println!("{json}");
            if summary.is_ok() {
                0
            } else {
                1
            }
        }
        Err(err) => {
            eprintln!("failed to serialize validation summary: {err}");
            2
        }
    }
}

fn validate(target: PathBuf, json: bool, strict: bool) -> i32 {
    let summary = validate_path(&target);
    let exit_code = if !summary.is_ok() {
        1
    } else if strict && summary.has_warnings() {
        1
    } else {
        0
    };

    let print_code = if json {
        print_validation_json(&summary)
    } else {
        print_validation_text(&summary)
    };

    if print_code != 0 {
        print_code
    } else {
        exit_code
    }
}

#[derive(Clone, Copy, PartialEq, Eq)]
enum ReportInspectMode {
    Summary,
    Events,
    Failures,
    Full,
}

struct ReportInspectSummary {
    path: PathBuf,
    status: String,
    run_id: Option<String>,
    topology_id: Option<String>,
    fixture_id: Option<String>,
    test_id: Option<String>,
    execution_mode: Option<String>,
    started_at: Option<String>,
    finished_at: Option<String>,
    validation_status: Option<String>,
    deterministic_seed: Option<String>,
    seed_source: Option<String>,
    result: Option<String>,
    peer_count: usize,
    event_count: usize,
    failure_count: usize,
    verified_object_count: Option<u64>,
    rejected_object_count: Option<u64>,
    matched_expected_outcomes: Vec<String>,
    scheduled_peer_order: Vec<String>,
    fault_plan_count: usize,
    errors: Vec<String>,
}

impl ReportInspectSummary {
    fn new(path: PathBuf) -> Self {
        Self {
            path,
            status: "ok".to_string(),
            run_id: None,
            topology_id: None,
            fixture_id: None,
            test_id: None,
            execution_mode: None,
            started_at: None,
            finished_at: None,
            validation_status: None,
            deterministic_seed: None,
            seed_source: None,
            result: None,
            peer_count: 0,
            event_count: 0,
            failure_count: 0,
            verified_object_count: None,
            rejected_object_count: None,
            matched_expected_outcomes: Vec::new(),
            scheduled_peer_order: Vec::new(),
            fault_plan_count: 0,
            errors: Vec::new(),
        }
    }

    fn is_ok(&self) -> bool {
        self.errors.is_empty()
    }

    fn push_error(&mut self, message: impl Into<String>) {
        self.status = "failed".to_string();
        self.errors.push(message.into());
    }
}

struct InspectedReport {
    summary: ReportInspectSummary,
    report: Option<Report>,
    events: Vec<ReportEvent>,
    failures: Vec<ReportFailure>,
}

fn metadata_string(report: &Report, key: &str) -> Option<String> {
    report
        .metadata
        .as_ref()
        .and_then(|metadata| metadata.get(key))
        .and_then(|value| value.as_str())
        .map(ToOwned::to_owned)
}

fn metadata_string_vec(report: &Report, key: &str) -> Vec<String> {
    report
        .metadata
        .as_ref()
        .and_then(|metadata| metadata.get(key))
        .and_then(|value| value.as_array())
        .map(|values| {
            values
                .iter()
                .filter_map(|value| value.as_str().map(ToOwned::to_owned))
                .collect::<Vec<_>>()
        })
        .unwrap_or_default()
}

fn metadata_array_len(report: &Report, key: &str) -> usize {
    report
        .metadata
        .as_ref()
        .and_then(|metadata| metadata.get(key))
        .and_then(|value| value.as_array())
        .map(Vec::len)
        .unwrap_or(0)
}

fn inspect_report(target: PathBuf) -> InspectedReport {
    let mut inspected = InspectedReport {
        summary: ReportInspectSummary::new(target.clone()),
        report: None,
        events: Vec::new(),
        failures: Vec::new(),
    };

    if !target.exists() {
        inspected
            .summary
            .push_error(format!("report path does not exist: {}", target.display()));
        return inspected;
    }

    if target.is_dir() {
        inspected.summary.push_error(format!(
            "report inspect target is a directory: {}",
            target.display()
        ));
        return inspected;
    }

    if target.file_name().and_then(|name| name.to_str()) == Some("report.schema.json") {
        inspected
            .summary
            .push_error("report schema files are not inspect targets");
        return inspected;
    }

    let content = match fs::read_to_string(&target) {
        Ok(content) => content,
        Err(err) => {
            inspected
                .summary
                .push_error(format!("failed to read report file: {err}"));
            return inspected;
        }
    };

    let report: Report = match serde_json::from_str(&content) {
        Ok(report) => report,
        Err(err) => {
            inspected
                .summary
                .push_error(format!("failed to parse report JSON: {err}"));
            return inspected;
        }
    };

    inspected.summary.run_id = Some(report.run_id.clone());
    inspected.summary.topology_id = Some(report.topology_id.clone());
    inspected.summary.fixture_id = Some(report.fixture_id.clone());
    inspected.summary.test_id = report.test_id.clone();
    inspected.summary.execution_mode = report.execution_mode.clone();
    inspected.summary.started_at = report.started_at.clone();
    inspected.summary.finished_at = report.finished_at.clone();
    inspected.summary.validation_status = metadata_string(&report, "validation_status");
    inspected.summary.deterministic_seed = metadata_string(&report, "deterministic_seed");
    inspected.summary.seed_source = metadata_string(&report, "seed_source");
    inspected.summary.result = Some(report.result.clone());
    inspected.summary.peer_count = report.peers.len();
    inspected.summary.event_count = report.events.len();
    inspected.summary.failure_count = report.failures.len();
    inspected.summary.verified_object_count = report
        .summary
        .as_ref()
        .and_then(|report_summary| report_summary.verified_object_count);
    inspected.summary.rejected_object_count = report
        .summary
        .as_ref()
        .and_then(|report_summary| report_summary.rejected_object_count);
    inspected.summary.matched_expected_outcomes = report
        .summary
        .as_ref()
        .map(|report_summary| report_summary.matched_expected_outcomes.clone())
        .unwrap_or_default();
    inspected.summary.scheduled_peer_order = metadata_string_vec(&report, "scheduled_peer_order");
    inspected.summary.fault_plan_count = metadata_array_len(&report, "fault_plan");
    inspected.events = report.events.clone();
    inspected.failures = report.failures.clone();
    inspected.report = Some(report);

    inspected
}

fn print_report_text(summary: &ReportInspectSummary) -> i32 {
    println!("report path: {}", summary.path.display());
    if let Some(run_id) = &summary.run_id {
        println!("run id: {run_id}");
    }
    if let Some(topology_id) = &summary.topology_id {
        println!("topology id: {topology_id}");
    }
    if let Some(fixture_id) = &summary.fixture_id {
        println!("fixture id: {fixture_id}");
    }
    if let Some(test_id) = &summary.test_id {
        println!("test id: {test_id}");
    }
    if let Some(execution_mode) = &summary.execution_mode {
        println!("execution mode: {execution_mode}");
    }
    if let Some(started_at) = &summary.started_at {
        println!("started at: {started_at}");
    }
    if let Some(finished_at) = &summary.finished_at {
        println!("finished at: {finished_at}");
    }
    if let Some(validation_status) = &summary.validation_status {
        println!("validation status: {validation_status}");
    }
    if let Some(deterministic_seed) = &summary.deterministic_seed {
        println!("deterministic seed: {deterministic_seed}");
    }
    if let Some(seed_source) = &summary.seed_source {
        println!("seed source: {seed_source}");
    }
    if let Some(result) = &summary.result {
        println!("result: {result}");
    }
    println!("status: {}", summary.status);
    println!("peers: {}", summary.peer_count);
    println!("events: {}", summary.event_count);
    println!("failures: {}", summary.failure_count);
    if let Some(verified_object_count) = summary.verified_object_count {
        println!("verified objects: {verified_object_count}");
    }
    if let Some(rejected_object_count) = summary.rejected_object_count {
        println!("rejected objects: {rejected_object_count}");
    }
    println!("fault plan entries: {}", summary.fault_plan_count);
    if !summary.scheduled_peer_order.is_empty() {
        println!(
            "scheduled peer order: {}",
            summary.scheduled_peer_order.join(" -> ")
        );
    }
    if !summary.matched_expected_outcomes.is_empty() {
        println!(
            "matched expected outcomes: {}",
            summary.matched_expected_outcomes.join(", ")
        );
    }

    if summary.is_ok() {
        println!("report inspection: ok");
        0
    } else {
        println!("report inspection: failed");
        for error in &summary.errors {
            eprintln!("error: {error}");
        }
        1
    }
}

fn print_report_summary_json(summary: &ReportInspectSummary) -> i32 {
    let json = serde_json::json!({
        "path": summary.path,
        "status": summary.status,
        "run_id": summary.run_id,
        "topology_id": summary.topology_id,
        "fixture_id": summary.fixture_id,
        "test_id": summary.test_id,
        "execution_mode": summary.execution_mode,
        "started_at": summary.started_at,
        "finished_at": summary.finished_at,
        "validation_status": summary.validation_status,
        "deterministic_seed": summary.deterministic_seed,
        "seed_source": summary.seed_source,
        "result": summary.result,
        "peer_count": summary.peer_count,
        "event_count": summary.event_count,
        "failure_count": summary.failure_count,
        "verified_object_count": summary.verified_object_count,
        "rejected_object_count": summary.rejected_object_count,
        "matched_expected_outcomes": summary.matched_expected_outcomes,
        "scheduled_peer_order": summary.scheduled_peer_order,
        "fault_plan_count": summary.fault_plan_count,
        "errors": summary.errors,
    });

    match serde_json::to_string_pretty(&json) {
        Ok(json) => {
            println!("{json}");
            if summary.is_ok() {
                0
            } else {
                1
            }
        }
        Err(err) => {
            eprintln!("failed to serialize report inspection summary: {err}");
            2
        }
    }
}

fn print_report_events_text(summary: &ReportInspectSummary, events: &[ReportEvent]) -> i32 {
    println!("report path: {}", summary.path.display());
    if let Some(run_id) = &summary.run_id {
        println!("run id: {run_id}");
    }
    println!("status: {}", summary.status);
    println!("events: {}", events.len());
    for event in events {
        println!(
            "event #{} phase={} action={} outcome={}",
            event.step, event.phase, event.action, event.outcome
        );
        if let Some(node_id) = &event.node_id {
            println!("  node: {node_id}");
        }
        if !event.object_ids.is_empty() {
            println!("  object ids: {}", event.object_ids.join(", "));
        }
        if let Some(detail) = &event.detail {
            println!("  detail: {detail}");
        }
    }

    if summary.is_ok() {
        println!("report inspection: ok");
        0
    } else {
        println!("report inspection: failed");
        for error in &summary.errors {
            eprintln!("error: {error}");
        }
        1
    }
}

fn print_report_events_json(summary: &ReportInspectSummary, events: &[ReportEvent]) -> i32 {
    let json = serde_json::json!({
        "path": summary.path,
        "status": summary.status,
        "run_id": summary.run_id,
        "result": summary.result,
        "event_count": events.len(),
        "events": events,
        "errors": summary.errors,
    });

    match serde_json::to_string_pretty(&json) {
        Ok(json) => {
            println!("{json}");
            if summary.is_ok() {
                0
            } else {
                1
            }
        }
        Err(err) => {
            eprintln!("failed to serialize report event inspection summary: {err}");
            2
        }
    }
}

fn filter_events_by_phase(events: &[ReportEvent], phase_filter: Option<&str>) -> Vec<ReportEvent> {
    match phase_filter {
        Some(phase) => events
            .iter()
            .filter(|event| event.phase == phase)
            .cloned()
            .collect(),
        None => events.to_vec(),
    }
}

fn print_report_failures_text(summary: &ReportInspectSummary, failures: &[ReportFailure]) -> i32 {
    println!("report path: {}", summary.path.display());
    if let Some(run_id) = &summary.run_id {
        println!("run id: {run_id}");
    }
    println!("status: {}", summary.status);
    println!("failures: {}", failures.len());
    for failure in failures {
        println!("failure {}: {}", failure.failure_id, failure.description);
        if let Some(node_id) = &failure.node_id {
            println!("  node: {node_id}");
        }
        if let Some(severity) = &failure.severity {
            println!("  severity: {severity}");
        }
    }

    if summary.is_ok() {
        println!("report inspection: ok");
        0
    } else {
        println!("report inspection: failed");
        for error in &summary.errors {
            eprintln!("error: {error}");
        }
        1
    }
}

fn print_report_failures_json(summary: &ReportInspectSummary, failures: &[ReportFailure]) -> i32 {
    let json = serde_json::json!({
        "path": summary.path,
        "status": summary.status,
        "run_id": summary.run_id,
        "result": summary.result,
        "failure_count": failures.len(),
        "failures": failures,
        "errors": summary.errors,
    });

    match serde_json::to_string_pretty(&json) {
        Ok(json) => {
            println!("{json}");
            if summary.is_ok() {
                0
            } else {
                1
            }
        }
        Err(err) => {
            eprintln!("failed to serialize report failure inspection summary: {err}");
            2
        }
    }
}

fn print_report_full_json(summary: &ReportInspectSummary, report: &Report) -> i32 {
    if !summary.is_ok() {
        return print_report_summary_json(summary);
    }

    match serde_json::to_string_pretty(report) {
        Ok(json) => {
            println!("{json}");
            0
        }
        Err(err) => {
            eprintln!("failed to serialize full report JSON: {err}");
            2
        }
    }
}

fn report_inspect(
    target: PathBuf,
    json: bool,
    mode: ReportInspectMode,
    phase_filter: Option<&str>,
) -> i32 {
    let inspected = inspect_report(target);
    match mode {
        ReportInspectMode::Summary => {
            if json {
                print_report_summary_json(&inspected.summary)
            } else {
                print_report_text(&inspected.summary)
            }
        }
        ReportInspectMode::Events => {
            let filtered_events = filter_events_by_phase(&inspected.events, phase_filter);
            if json {
                print_report_events_json(&inspected.summary, &filtered_events)
            } else {
                print_report_events_text(&inspected.summary, &filtered_events)
            }
        }
        ReportInspectMode::Failures => {
            if json {
                print_report_failures_json(&inspected.summary, &inspected.failures)
            } else {
                print_report_failures_text(&inspected.summary, &inspected.failures)
            }
        }
        ReportInspectMode::Full => {
            if json {
                match inspected.report.as_ref() {
                    Some(report) => print_report_full_json(&inspected.summary, report),
                    None => print_report_summary_json(&inspected.summary),
                }
            } else {
                eprintln!("report inspect --full requires --json");
                2
            }
        }
    }
}

fn print_run_text(summary: &mycel_sim::run::SimulationRunSummary) -> i32 {
    println!("repo root: {}", summary.root.display());
    println!("run target: {}", summary.target.display());
    println!("started at: {}", summary.started_at);
    println!("finished at: {}", summary.finished_at);
    println!("run duration ms: {}", summary.run_duration_ms);
    println!("deterministic seed: {}", summary.deterministic_seed);
    println!("seed source: {}", summary.seed_source);
    println!("events per second: {:.3}", summary.events_per_second);
    println!("ms per event: {:.3}", summary.ms_per_event);
    println!(
        "scheduled peer order: {}",
        summary.scheduled_peer_order.join(" -> ")
    );
    if summary.fault_plan.is_empty() {
        println!("fault plan: none");
    } else {
        println!(
            "fault plan: {}",
            summary
                .fault_plan
                .iter()
                .map(|entry| format!(
                    "#{}:{}:{}->{}",
                    entry.order,
                    entry.fault,
                    entry.source_node_id,
                    entry
                        .target_node_id
                        .as_deref()
                        .unwrap_or("unspecified-target")
                ))
                .collect::<Vec<_>>()
                .join(", ")
        );
    }
    println!("validation status: {}", summary.validation_status);
    println!("report path: {}", summary.report_path.display());
    println!("result: {}", summary.result);
    println!("peers: {}", summary.peer_count);
    println!("events: {}", summary.event_count);
    println!("verified objects: {}", summary.verified_object_count);
    println!("rejected objects: {}", summary.rejected_object_count);

    if !summary.matched_expected_outcomes.is_empty() {
        println!(
            "matched expected outcomes: {}",
            summary.matched_expected_outcomes.join(", ")
        );
    }

    for warning in &summary.validation_warnings {
        eprintln!("warning: {warning}");
    }

    0
}

fn print_run_json(summary: &mycel_sim::run::SimulationRunSummary) -> i32 {
    match serde_json::to_string_pretty(summary) {
        Ok(json) => {
            println!("{json}");
            0
        }
        Err(err) => {
            eprintln!("failed to serialize run summary: {err}");
            2
        }
    }
}

fn sim_run(target: PathBuf, json: bool, seed_override: Option<String>) -> i32 {
    let options = RunOptions { seed_override };
    match run_test_case_with_options(&target, &options) {
        Ok(summary) => {
            if json {
                print_run_json(&summary)
            } else {
                print_run_text(&summary)
            }
        }
        Err(message) => {
            eprintln!("sim run failed: {message}");
            1
        }
    }
}

fn main() {
    let mut args = env::args().skip(1);

    match args.next().as_deref() {
        Some("head") => match args.next().as_deref() {
            Some("inspect") => {
                let mut doc_id = None;
                let mut input_path = None;
                let mut json = false;
                let mut expect_input_path = false;

                for arg in args {
                    if expect_input_path {
                        input_path = Some(PathBuf::from(arg));
                        expect_input_path = false;
                    } else if arg == "--json" {
                        json = true;
                    } else if arg == "--input" {
                        expect_input_path = true;
                    } else if doc_id.is_none() {
                        doc_id = Some(arg);
                    } else {
                        eprintln!("unexpected head inspect argument: {arg}");
                        eprintln!();
                        print_usage();
                        std::process::exit(2);
                    }
                }

                if expect_input_path {
                    eprintln!("missing value for --input");
                    eprintln!();
                    print_usage();
                    std::process::exit(2);
                }

                let Some(doc_id) = doc_id else {
                    eprintln!("missing head inspect doc_id");
                    eprintln!();
                    print_usage();
                    std::process::exit(2);
                };
                let Some(input_path) = input_path else {
                    eprintln!("missing --input for head inspect");
                    eprintln!();
                    print_usage();
                    std::process::exit(2);
                };

                std::process::exit(head_inspect(doc_id, input_path, json));
            }
            Some(other) => {
                eprintln!("unknown head subcommand: {other}");
                eprintln!();
                print_usage();
                std::process::exit(2);
            }
            None => {
                eprintln!("missing head subcommand");
                eprintln!();
                print_usage();
                std::process::exit(2);
            }
        },
        Some("info") => print_info(),
        Some("object") => match args.next().as_deref() {
            Some("verify") => {
                let mut target = None;
                let mut json = false;

                for arg in args {
                    if arg == "--json" {
                        json = true;
                    } else if target.is_none() {
                        target = Some(PathBuf::from(arg));
                    } else {
                        eprintln!("unexpected object verify argument: {arg}");
                        eprintln!();
                        print_usage();
                        std::process::exit(2);
                    }
                }

                let Some(target) = target else {
                    eprintln!("missing object verify target");
                    eprintln!();
                    print_usage();
                    std::process::exit(2);
                };

                std::process::exit(object_verify(target, json));
            }
            Some(other) => {
                eprintln!("unknown object subcommand: {other}");
                eprintln!();
                print_usage();
                std::process::exit(2);
            }
            None => {
                eprintln!("missing object subcommand");
                eprintln!();
                print_usage();
                std::process::exit(2);
            }
        },
        Some("report") => match args.next().as_deref() {
            Some("inspect") => {
                let mut target = None;
                let mut json = false;
                let mut mode = ReportInspectMode::Summary;
                let mut phase_filter = None;
                let mut expect_phase_value = false;

                for arg in args {
                    if expect_phase_value {
                        phase_filter = Some(arg);
                        expect_phase_value = false;
                        if mode == ReportInspectMode::Summary {
                            mode = ReportInspectMode::Events;
                        }
                    } else if arg == "--json" {
                        json = true;
                    } else if arg == "--full" {
                        if phase_filter.is_some() {
                            eprintln!("report inspect --phase cannot be combined with --full");
                            eprintln!();
                            print_usage();
                            std::process::exit(2);
                        }
                        if mode != ReportInspectMode::Summary {
                            eprintln!(
                                "report inspect accepts only one of --events, --failures, or --full"
                            );
                            eprintln!();
                            print_usage();
                            std::process::exit(2);
                        }
                        mode = ReportInspectMode::Full;
                    } else if arg == "--events" {
                        if mode != ReportInspectMode::Summary {
                            eprintln!(
                                "report inspect accepts only one of --events, --failures, or --full"
                            );
                            eprintln!();
                            print_usage();
                            std::process::exit(2);
                        }
                        mode = ReportInspectMode::Events;
                    } else if arg == "--failures" {
                        if phase_filter.is_some() {
                            eprintln!("report inspect --phase cannot be combined with --failures");
                            eprintln!();
                            print_usage();
                            std::process::exit(2);
                        }
                        if mode != ReportInspectMode::Summary {
                            eprintln!(
                                "report inspect accepts only one of --events, --failures, or --full"
                            );
                            eprintln!();
                            print_usage();
                            std::process::exit(2);
                        }
                        mode = ReportInspectMode::Failures;
                    } else if arg == "--phase" {
                        if expect_phase_value {
                            eprintln!("missing value for --phase");
                            eprintln!();
                            print_usage();
                            std::process::exit(2);
                        }
                        if mode == ReportInspectMode::Full {
                            eprintln!("report inspect --phase cannot be combined with --full");
                            eprintln!();
                            print_usage();
                            std::process::exit(2);
                        }
                        if mode == ReportInspectMode::Failures {
                            eprintln!("report inspect --phase cannot be combined with --failures");
                            eprintln!();
                            print_usage();
                            std::process::exit(2);
                        }
                        expect_phase_value = true;
                    } else if target.is_none() {
                        target = Some(PathBuf::from(arg));
                    } else {
                        eprintln!("unexpected report inspect argument: {arg}");
                        eprintln!();
                        print_usage();
                        std::process::exit(2);
                    }
                }

                if expect_phase_value {
                    eprintln!("missing value for --phase");
                    eprintln!();
                    print_usage();
                    std::process::exit(2);
                }

                let Some(target) = target else {
                    eprintln!("missing report inspect target");
                    eprintln!();
                    print_usage();
                    std::process::exit(2);
                };

                if mode == ReportInspectMode::Full && !json {
                    eprintln!("report inspect --full requires --json");
                    eprintln!();
                    print_usage();
                    std::process::exit(2);
                }

                std::process::exit(report_inspect(target, json, mode, phase_filter.as_deref()));
            }
            Some(other) => {
                eprintln!("unknown report subcommand: {other}");
                eprintln!();
                print_usage();
                std::process::exit(2);
            }
            None => {
                eprintln!("missing report subcommand");
                eprintln!();
                print_usage();
                std::process::exit(2);
            }
        },
        Some("validate") => {
            let mut target = PathBuf::from(".");
            let mut json = false;
            let mut strict = false;

            for arg in args {
                if arg == "--json" {
                    json = true;
                } else if arg == "--strict" {
                    strict = true;
                } else if target == PathBuf::from(".") {
                    target = PathBuf::from(arg);
                } else {
                    eprintln!("unexpected validate argument: {arg}");
                    eprintln!();
                    print_usage();
                    std::process::exit(2);
                }
            }

            std::process::exit(validate(target, json, strict));
        }
        Some("sim") => match args.next().as_deref() {
            Some("run") => {
                let mut target = None;
                let mut json = false;
                let mut seed_override = None;
                let mut expect_seed_value = false;

                for arg in args {
                    if expect_seed_value {
                        seed_override = Some(arg);
                        expect_seed_value = false;
                    } else if arg == "--json" {
                        json = true;
                    } else if arg == "--seed" {
                        expect_seed_value = true;
                    } else if target.is_none() {
                        target = Some(PathBuf::from(arg));
                    } else {
                        eprintln!("unexpected sim run argument: {arg}");
                        eprintln!();
                        print_usage();
                        std::process::exit(2);
                    }
                }

                if expect_seed_value {
                    eprintln!("missing value for --seed");
                    eprintln!();
                    print_usage();
                    std::process::exit(2);
                }

                let Some(target) = target else {
                    eprintln!("missing sim run target");
                    eprintln!();
                    print_usage();
                    std::process::exit(2);
                };

                std::process::exit(sim_run(target, json, seed_override));
            }
            Some(other) => {
                eprintln!("unknown sim subcommand: {other}");
                eprintln!();
                print_usage();
                std::process::exit(2);
            }
            None => {
                eprintln!("missing sim subcommand");
                eprintln!();
                print_usage();
                std::process::exit(2);
            }
        },
        Some("help") | None => print_usage(),
        Some(other) => {
            eprintln!("unknown command: {other}");
            eprintln!();
            print_usage();
            std::process::exit(2);
        }
    }
}
