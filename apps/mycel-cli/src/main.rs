use std::env;
use std::fs;
use std::path::PathBuf;

use clap::{Args, Parser, Subcommand};
use mycel_core::head::inspect_heads_from_path;
use mycel_core::head::HeadInspectSummary;
use mycel_core::verify::{verify_object_path, ObjectVerificationSummary};
use mycel_core::workspace_banner;
use mycel_sim::manifest::SimulatorPaths;
use mycel_sim::model::{Report, ReportEvent, ReportFailure};
use mycel_sim::run::{run_test_case_with_options, RunOptions};
use mycel_sim::simulator_banner;
use mycel_sim::validate::validate_path;

#[derive(Parser)]
#[command(
    name = "mycel",
    disable_version_flag = true,
    disable_help_subcommand = true
)]
struct Cli {
    #[command(subcommand)]
    command: Option<CliCommand>,
}

#[derive(Subcommand)]
enum CliCommand {
    Head(HeadCliArgs),
    Info,
    Object(ObjectCliArgs),
    Report(ReportCliArgs),
    Sim(SimCliArgs),
    Validate(ValidateCliArgs),
    #[command(external_subcommand)]
    External(Vec<String>),
}

#[derive(Args)]
struct HeadCliArgs {
    #[command(subcommand)]
    command: Option<HeadSubcommand>,
}

#[derive(Subcommand)]
enum HeadSubcommand {
    Inspect(HeadInspectCliArgs),
    #[command(external_subcommand)]
    External(Vec<String>),
}

#[derive(Args)]
struct HeadInspectCliArgs {
    #[arg(allow_hyphen_values = true)]
    doc_id: Option<String>,
    #[arg(long, num_args = 0..=1)]
    input: Option<Option<String>>,
    #[arg(long)]
    json: bool,
    #[arg(allow_hyphen_values = true)]
    extra: Vec<String>,
}

#[derive(Args)]
struct ObjectCliArgs {
    #[command(subcommand)]
    command: Option<ObjectSubcommand>,
}

#[derive(Subcommand)]
enum ObjectSubcommand {
    Verify(ObjectVerifyCliArgs),
    #[command(external_subcommand)]
    External(Vec<String>),
}

#[derive(Args)]
struct ObjectVerifyCliArgs {
    #[arg(allow_hyphen_values = true)]
    target: Option<String>,
    #[arg(long)]
    json: bool,
    #[arg(allow_hyphen_values = true)]
    extra: Vec<String>,
}

#[derive(Args)]
struct ReportCliArgs {
    #[command(subcommand)]
    command: Option<ReportSubcommand>,
}

#[derive(Subcommand)]
enum ReportSubcommand {
    Inspect(ReportInspectCliArgs),
    #[command(external_subcommand)]
    External(Vec<String>),
}

#[derive(Args)]
struct ReportInspectCliArgs {
    #[arg(allow_hyphen_values = true)]
    target: Option<String>,
    #[arg(long)]
    json: bool,
    #[arg(long)]
    events: bool,
    #[arg(long)]
    failures: bool,
    #[arg(long)]
    full: bool,
    #[arg(long, num_args = 0..=1)]
    phase: Option<Option<String>>,
    #[arg(long, num_args = 0..=1)]
    action: Option<Option<String>>,
    #[arg(long, num_args = 0..=1)]
    outcome: Option<Option<String>>,
    #[arg(long, num_args = 0..=1)]
    step: Option<Option<String>>,
    #[arg(long = "step-range", num_args = 0..=1)]
    step_range: Option<Option<String>>,
    #[arg(long, num_args = 0..=1)]
    first: Option<Option<String>>,
    #[arg(long, num_args = 0..=1)]
    last: Option<Option<String>>,
    #[arg(long, num_args = 0..=1)]
    node: Option<Option<String>>,
    #[arg(allow_hyphen_values = true)]
    extra: Vec<String>,
}

#[derive(Args)]
struct SimCliArgs {
    #[command(subcommand)]
    command: Option<SimSubcommand>,
}

#[derive(Subcommand)]
enum SimSubcommand {
    Run(SimRunCliArgs),
    #[command(external_subcommand)]
    External(Vec<String>),
}

#[derive(Args)]
struct SimRunCliArgs {
    #[arg(allow_hyphen_values = true)]
    target: Option<String>,
    #[arg(long)]
    json: bool,
    #[arg(long, num_args = 0..=1)]
    seed: Option<Option<String>>,
    #[arg(allow_hyphen_values = true)]
    extra: Vec<String>,
}

#[derive(Args)]
struct ValidateCliArgs {
    #[arg(allow_hyphen_values = true)]
    target: Option<String>,
    #[arg(long)]
    json: bool,
    #[arg(long)]
    strict: bool,
    #[arg(allow_hyphen_values = true)]
    extra: Vec<String>,
}

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
    println!("  --action <name> Filter event inspection to one action");
    println!("  --outcome <name> Filter event inspection to one outcome");
    println!("  --step <n>      Filter event inspection to one step number");
    println!("  --step-range <a>:<b>  Filter event inspection to one inclusive step range");
    println!("  --first <n>     Filter event inspection to the first N matching events");
    println!("  --last <n>      Filter event inspection to the last N matching events");
    println!("  --node <id>     Filter event or failure inspection to one node");
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

#[derive(Default)]
struct ReportInspectFilters {
    phase: Option<String>,
    action: Option<String>,
    outcome: Option<String>,
    step: Option<u64>,
    step_range: Option<(u64, u64)>,
    first: Option<usize>,
    last: Option<usize>,
    node: Option<String>,
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

fn filter_events(events: &[ReportEvent], filters: &ReportInspectFilters) -> Vec<ReportEvent> {
    let filtered: Vec<_> = events
        .iter()
        .filter(|event| {
            filters
                .phase
                .as_deref()
                .is_none_or(|phase| event.phase == phase)
        })
        .filter(|event| {
            filters
                .action
                .as_deref()
                .is_none_or(|action| event.action == action)
        })
        .filter(|event| {
            filters
                .outcome
                .as_deref()
                .is_none_or(|outcome| event.outcome == outcome)
        })
        .filter(|event| filters.step.is_none_or(|step| event.step == step))
        .filter(|event| {
            filters
                .step_range
                .is_none_or(|(start, end)| start <= event.step && event.step <= end)
        })
        .filter(|event| {
            filters.node.as_deref().is_none_or(|node| {
                event
                    .node_id
                    .as_deref()
                    .is_some_and(|event_node_id| event_node_id == node)
            })
        })
        .cloned()
        .collect();

    let filtered = match filters.first {
        Some(first) => filtered.into_iter().take(first).collect(),
        None => filtered,
    };

    match filters.last {
        Some(last) => {
            let skip = filtered.len().saturating_sub(last);
            filtered.into_iter().skip(skip).collect()
        }
        None => filtered,
    }
}

fn parse_step_range(value: &str) -> Result<(u64, u64), String> {
    let Some((start, end)) = value.split_once(':') else {
        return Err(format!("invalid value for --step-range: {value}"));
    };
    let start = start
        .parse::<u64>()
        .map_err(|_| format!("invalid value for --step-range: {value}"))?;
    let end = end
        .parse::<u64>()
        .map_err(|_| format!("invalid value for --step-range: {value}"))?;
    if start > end {
        return Err(format!(
            "invalid value for --step-range: {value} (start must be <= end)"
        ));
    }
    Ok((start, end))
}

fn parse_usize_flag(value: &str, flag: &str) -> Result<usize, String> {
    value
        .parse::<usize>()
        .map_err(|_| format!("invalid value for {flag}: {value}"))
}

fn parse_u64_flag(value: &str, flag: &str) -> Result<u64, String> {
    value
        .parse::<u64>()
        .map_err(|_| format!("invalid value for {flag}: {value}"))
}

fn exit_usage_error(message: impl AsRef<str>) -> ! {
    eprintln!("{}", message.as_ref());
    eprintln!();
    print_usage();
    std::process::exit(2);
}

fn require_optional_flag_value(
    value: Option<Option<String>>,
    flag: &str,
) -> Result<Option<String>, String> {
    match value {
        Some(Some(value)) => Ok(Some(value)),
        Some(None) => Err(format!("missing value for {flag}")),
        None => Ok(None),
    }
}

fn unexpected_extra(extra: &[String], context: &str) -> Option<String> {
    extra
        .first()
        .map(|arg| format!("unexpected {context} argument: {arg}"))
}

fn handle_head_command(command: HeadCliArgs) -> ! {
    match command.command {
        Some(HeadSubcommand::Inspect(args)) => {
            if let Some(message) = unexpected_extra(&args.extra, "head inspect") {
                exit_usage_error(message);
            }
            let Some(doc_id) = args.doc_id else {
                exit_usage_error("missing head inspect doc_id");
            };
            let input = match require_optional_flag_value(args.input, "--input") {
                Ok(Some(input)) => PathBuf::from(input),
                Ok(None) => exit_usage_error("missing --input for head inspect"),
                Err(message) => exit_usage_error(message),
            };

            std::process::exit(head_inspect(doc_id, input, args.json));
        }
        Some(HeadSubcommand::External(args)) => {
            let other = args.first().map(String::as_str).unwrap_or("<unknown>");
            exit_usage_error(format!("unknown head subcommand: {other}"));
        }
        None => exit_usage_error("missing head subcommand"),
    }
}

fn handle_object_command(command: ObjectCliArgs) -> ! {
    match command.command {
        Some(ObjectSubcommand::Verify(args)) => {
            if let Some(message) = unexpected_extra(&args.extra, "object verify") {
                exit_usage_error(message);
            }
            let Some(target) = args.target else {
                exit_usage_error("missing object verify target");
            };

            std::process::exit(object_verify(PathBuf::from(target), args.json));
        }
        Some(ObjectSubcommand::External(args)) => {
            let other = args.first().map(String::as_str).unwrap_or("<unknown>");
            exit_usage_error(format!("unknown object subcommand: {other}"));
        }
        None => exit_usage_error("missing object subcommand"),
    }
}

fn handle_report_command(command: ReportCliArgs) -> ! {
    match command.command {
        Some(ReportSubcommand::Inspect(args)) => {
            if let Some(message) = unexpected_extra(&args.extra, "report inspect") {
                exit_usage_error(message);
            }

            let mut mode = ReportInspectMode::Summary;
            if args.events {
                mode = ReportInspectMode::Events;
            }
            if args.failures {
                if mode != ReportInspectMode::Summary {
                    exit_usage_error(
                        "report inspect accepts only one of --events, --failures, or --full",
                    );
                }
                mode = ReportInspectMode::Failures;
            }
            if args.full {
                if mode != ReportInspectMode::Summary {
                    exit_usage_error(
                        "report inspect accepts only one of --events, --failures, or --full",
                    );
                }
                mode = ReportInspectMode::Full;
            }

            let phase = require_optional_flag_value(args.phase, "--phase")
                .unwrap_or_else(|message| exit_usage_error(message));
            let action = require_optional_flag_value(args.action, "--action")
                .unwrap_or_else(|message| exit_usage_error(message));
            let outcome = require_optional_flag_value(args.outcome, "--outcome")
                .unwrap_or_else(|message| exit_usage_error(message));
            let step = require_optional_flag_value(args.step, "--step")
                .unwrap_or_else(|message| exit_usage_error(message))
                .map(|value| {
                    parse_u64_flag(&value, "--step")
                        .unwrap_or_else(|message| exit_usage_error(message))
                });
            let step_range = require_optional_flag_value(args.step_range, "--step-range")
                .unwrap_or_else(|message| exit_usage_error(message))
                .map(|value| {
                    parse_step_range(&value).unwrap_or_else(|message| exit_usage_error(message))
                });
            let first = require_optional_flag_value(args.first, "--first")
                .unwrap_or_else(|message| exit_usage_error(message))
                .map(|value| {
                    parse_usize_flag(&value, "--first")
                        .unwrap_or_else(|message| exit_usage_error(message))
                });
            let last = require_optional_flag_value(args.last, "--last")
                .unwrap_or_else(|message| exit_usage_error(message))
                .map(|value| {
                    parse_usize_flag(&value, "--last")
                        .unwrap_or_else(|message| exit_usage_error(message))
                });
            let node = require_optional_flag_value(args.node, "--node")
                .unwrap_or_else(|message| exit_usage_error(message));

            if phase.is_some() && matches!(mode, ReportInspectMode::Failures) {
                exit_usage_error("report inspect --phase cannot be combined with --failures");
            }
            if phase.is_some() && matches!(mode, ReportInspectMode::Full) {
                exit_usage_error("report inspect --phase cannot be combined with --full");
            }
            if action.is_some() && matches!(mode, ReportInspectMode::Failures) {
                exit_usage_error("report inspect --action cannot be combined with --failures");
            }
            if action.is_some() && matches!(mode, ReportInspectMode::Full) {
                exit_usage_error("report inspect --action cannot be combined with --full");
            }
            if outcome.is_some() && matches!(mode, ReportInspectMode::Failures) {
                exit_usage_error("report inspect --outcome cannot be combined with --failures");
            }
            if outcome.is_some() && matches!(mode, ReportInspectMode::Full) {
                exit_usage_error("report inspect --outcome cannot be combined with --full");
            }
            if step.is_some() && matches!(mode, ReportInspectMode::Failures) {
                exit_usage_error("report inspect --step cannot be combined with --failures");
            }
            if step.is_some() && matches!(mode, ReportInspectMode::Full) {
                exit_usage_error("report inspect --step cannot be combined with --full");
            }
            if step_range.is_some() && matches!(mode, ReportInspectMode::Failures) {
                exit_usage_error("report inspect --step-range cannot be combined with --failures");
            }
            if step_range.is_some() && matches!(mode, ReportInspectMode::Full) {
                exit_usage_error("report inspect --step-range cannot be combined with --full");
            }
            if first.is_some() && matches!(mode, ReportInspectMode::Failures) {
                exit_usage_error("report inspect --first cannot be combined with --failures");
            }
            if first.is_some() && matches!(mode, ReportInspectMode::Full) {
                exit_usage_error("report inspect --first cannot be combined with --full");
            }
            if last.is_some() && matches!(mode, ReportInspectMode::Failures) {
                exit_usage_error("report inspect --last cannot be combined with --failures");
            }
            if last.is_some() && matches!(mode, ReportInspectMode::Full) {
                exit_usage_error("report inspect --last cannot be combined with --full");
            }
            if node.is_some() && matches!(mode, ReportInspectMode::Full) {
                exit_usage_error("report inspect --node cannot be combined with --full");
            }
            if step.is_some() && step_range.is_some() {
                exit_usage_error("report inspect accepts only one of --step or --step-range");
            }

            let Some(target) = args.target else {
                exit_usage_error("missing report inspect target");
            };

            let filters = ReportInspectFilters {
                phase,
                action,
                outcome,
                step,
                step_range,
                first,
                last,
                node,
            };
            let mode = if matches!(mode, ReportInspectMode::Summary)
                && (filters.phase.is_some()
                    || filters.action.is_some()
                    || filters.outcome.is_some()
                    || filters.step.is_some()
                    || filters.step_range.is_some()
                    || filters.first.is_some()
                    || filters.last.is_some()
                    || filters.node.is_some())
            {
                ReportInspectMode::Events
            } else {
                mode
            };

            if matches!(mode, ReportInspectMode::Full) && !args.json {
                exit_usage_error("report inspect --full requires --json");
            }

            std::process::exit(report_inspect(
                PathBuf::from(target),
                args.json,
                mode,
                &filters,
            ));
        }
        Some(ReportSubcommand::External(args)) => {
            let other = args.first().map(String::as_str).unwrap_or("<unknown>");
            exit_usage_error(format!("unknown report subcommand: {other}"));
        }
        None => exit_usage_error("missing report subcommand"),
    }
}

fn handle_validate_command(args: ValidateCliArgs) -> ! {
    if let Some(message) = unexpected_extra(&args.extra, "validate") {
        exit_usage_error(message);
    }

    let target = args.target.unwrap_or_else(|| ".".to_owned());
    std::process::exit(validate(PathBuf::from(target), args.json, args.strict));
}

fn handle_sim_command(command: SimCliArgs) -> ! {
    match command.command {
        Some(SimSubcommand::Run(args)) => {
            if let Some(message) = unexpected_extra(&args.extra, "sim run") {
                exit_usage_error(message);
            }
            let seed_override = match require_optional_flag_value(args.seed, "--seed") {
                Ok(value) => value,
                Err(message) => exit_usage_error(message),
            };
            let Some(target) = args.target else {
                exit_usage_error("missing sim run target");
            };

            std::process::exit(sim_run(PathBuf::from(target), args.json, seed_override));
        }
        Some(SimSubcommand::External(args)) => {
            let other = args.first().map(String::as_str).unwrap_or("<unknown>");
            exit_usage_error(format!("unknown sim subcommand: {other}"));
        }
        None => exit_usage_error("missing sim subcommand"),
    }
}

fn filter_failures(
    failures: &[ReportFailure],
    filters: &ReportInspectFilters,
) -> Vec<ReportFailure> {
    failures
        .iter()
        .filter(|failure| {
            filters.node.as_deref().is_none_or(|node| {
                failure
                    .node_id
                    .as_deref()
                    .is_some_and(|failure_node_id| failure_node_id == node)
            })
        })
        .cloned()
        .collect()
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
    filters: &ReportInspectFilters,
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
            let filtered_events = filter_events(&inspected.events, filters);
            if json {
                print_report_events_json(&inspected.summary, &filtered_events)
            } else {
                print_report_events_text(&inspected.summary, &filtered_events)
            }
        }
        ReportInspectMode::Failures => {
            let filtered_failures = filter_failures(&inspected.failures, filters);
            if json {
                print_report_failures_json(&inspected.summary, &filtered_failures)
            } else {
                print_report_failures_text(&inspected.summary, &filtered_failures)
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
    let args: Vec<String> = env::args().collect();
    match args.get(1).map(String::as_str) {
        None | Some("help") | Some("-h") | Some("--help") => {
            print_usage();
            return;
        }
        _ => {}
    }

    let cli = match Cli::try_parse() {
        Ok(cli) => cli,
        Err(err) => {
            eprintln!("{err}");
            std::process::exit(err.exit_code());
        }
    };

    match cli.command {
        Some(CliCommand::Head(command)) => handle_head_command(command),
        Some(CliCommand::Info) => print_info(),
        Some(CliCommand::Object(command)) => handle_object_command(command),
        Some(CliCommand::Report(command)) => handle_report_command(command),
        Some(CliCommand::Sim(command)) => handle_sim_command(command),
        Some(CliCommand::Validate(command)) => handle_validate_command(command),
        Some(CliCommand::External(args)) => {
            let other = args.first().map(String::as_str).unwrap_or("<unknown>");
            exit_usage_error(format!("unknown command: {other}"));
        }
        None => print_usage(),
    }
}
