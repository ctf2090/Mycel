use std::env;
use std::fs;
use std::path::{Path, PathBuf};

use clap::{Args, CommandFactory, Parser, Subcommand};
use mycel_core::head::inspect_heads_from_path;
use mycel_core::head::HeadInspectSummary;
use mycel_core::verify::{verify_object_path, ObjectVerificationSummary};
use mycel_core::workspace_banner;
use mycel_sim::manifest::SimulatorPaths;
use mycel_sim::model::{Report, ReportEvent, ReportFailure};
use mycel_sim::run::{run_test_case_with_options, RunOptions};
use mycel_sim::simulator_banner;
use mycel_sim::validate::validate_path;
use thiserror::Error;

#[derive(Debug, Error)]
enum CliError {
    #[error("{0}")]
    Usage(String),
    #[error("failed to serialize {context}: {source}")]
    Serialization {
        context: &'static str,
        #[source]
        source: serde_json::Error,
    },
    #[error("sim run failed: {0}")]
    SimRun(String),
}

impl CliError {
    fn usage(message: impl Into<String>) -> Self {
        Self::Usage(message.into())
    }

    fn serialization(context: &'static str, source: serde_json::Error) -> Self {
        Self::Serialization { context, source }
    }

    fn exit_code(&self) -> i32 {
        match self {
            Self::Usage(_) | Self::Serialization { .. } => 2,
            Self::SimRun(_) => 1,
        }
    }

    fn emit(&self) {
        match self {
            Self::Usage(message) => {
                emit_error_line(message);
                eprintln!();
                print_usage();
            }
            _ => emit_error_line(self),
        }
    }
}

fn emit_error_line(message: impl std::fmt::Display) {
    eprintln!("error: {message}");
}

fn emit_warning_line(message: impl std::fmt::Display) {
    eprintln!("warning: {message}");
}

#[derive(Parser)]
#[command(
    name = "mycel",
    about = "Mycel CLI for validation, inspection, and simulator workflows.",
    disable_version_flag = true,
    disable_help_subcommand = true
)]
struct Cli {
    #[command(subcommand)]
    command: Option<CliCommand>,
}

#[derive(Subcommand)]
enum CliCommand {
    #[command(about = "Inspect accepted-head selection from a local input bundle")]
    Head(HeadCliArgs),
    #[command(about = "Show workspace and simulator scaffold information")]
    Info,
    #[command(about = "Verify one Mycel object file")]
    Object(ObjectCliArgs),
    #[command(about = "Inspect one simulator report")]
    Report(ReportCliArgs),
    #[command(about = "Run a simulator test case")]
    Sim(SimCliArgs),
    #[command(about = "Validate the repo root, one file, or one supported directory")]
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
    #[command(about = "Inspect one document's accepted head")]
    Inspect(HeadInspectCliArgs),
    #[command(external_subcommand)]
    External(Vec<String>),
}

#[derive(Args)]
struct HeadInspectCliArgs {
    #[arg(
        value_name = "DOC_ID",
        help = "Document identifier to inspect",
        required = true,
        allow_hyphen_values = true
    )]
    doc_id: String,
    #[arg(
        long,
        value_name = "PATH_OR_FIXTURE",
        help = "Input bundle path or repo fixture name",
        required = true
    )]
    input: String,
    #[arg(long, help = "Emit machine-readable head inspection output")]
    json: bool,
    #[arg(hide = true, allow_hyphen_values = true)]
    extra: Vec<String>,
}

#[derive(Args)]
struct ObjectCliArgs {
    #[command(subcommand)]
    command: Option<ObjectSubcommand>,
}

#[derive(Subcommand)]
enum ObjectSubcommand {
    #[command(about = "Verify one object file")]
    Verify(ObjectVerifyCliArgs),
    #[command(external_subcommand)]
    External(Vec<String>),
}

#[derive(Args)]
struct ObjectVerifyCliArgs {
    #[arg(
        value_name = "PATH",
        help = "Object file to verify",
        required = true,
        allow_hyphen_values = true
    )]
    target: String,
    #[arg(long, help = "Emit machine-readable object verification output")]
    json: bool,
    #[arg(hide = true, allow_hyphen_values = true)]
    extra: Vec<String>,
}

#[derive(Args)]
struct ReportCliArgs {
    #[command(subcommand)]
    command: Option<ReportSubcommand>,
}

#[derive(Subcommand)]
enum ReportSubcommand {
    #[command(about = "Inspect one simulator report")]
    Inspect(ReportInspectCliArgs),
    #[command(about = "List simulator reports under a directory or one file")]
    List(ReportListCliArgs),
    #[command(about = "Select the latest simulator report under a directory or one file")]
    Latest(ReportLatestCliArgs),
    #[command(external_subcommand)]
    External(Vec<String>),
}

#[derive(Args)]
struct ReportInspectCliArgs {
    #[arg(
        value_name = "PATH",
        help = "Simulator report to inspect",
        allow_hyphen_values = true
    )]
    target: Option<String>,
    #[arg(long, help = "Emit machine-readable report inspection output")]
    json: bool,
    #[arg(long, help = "Show only report events", conflicts_with_all = ["failures", "full"])]
    events: bool,
    #[arg(
        long,
        help = "Show only report failures",
        conflicts_with_all = [
            "events",
            "full",
            "phase",
            "action",
            "outcome",
            "step",
            "step_range",
            "first",
            "last"
        ]
    )]
    failures: bool,
    #[arg(
        long,
        help = "Emit the full raw report (requires --json)",
        requires = "json",
        conflicts_with_all = [
            "events",
            "failures",
            "phase",
            "action",
            "outcome",
            "step",
            "step_range",
            "first",
            "last",
            "node"
        ]
    )]
    full: bool,
    #[arg(long, value_name = "NAME", help = "Filter event inspection to one phase", conflicts_with_all = ["failures", "full"])]
    phase: Option<String>,
    #[arg(long, value_name = "NAME", help = "Filter event inspection to one action", conflicts_with_all = ["failures", "full"])]
    action: Option<String>,
    #[arg(long, value_name = "NAME", help = "Filter event inspection to one outcome", conflicts_with_all = ["failures", "full"])]
    outcome: Option<String>,
    #[arg(long, value_name = "N", help = "Filter event inspection to one step number", value_parser = parse_report_step, conflicts_with_all = ["failures", "full", "step_range"])]
    step: Option<u64>,
    #[arg(long = "step-range", value_name = "START:END", help = "Filter event inspection to one inclusive step range", value_parser = parse_step_range, conflicts_with_all = ["failures", "full", "step"])]
    step_range: Option<(u64, u64)>,
    #[arg(long, value_name = "N", help = "Keep the first N matching events", value_parser = parse_report_first, conflicts_with_all = ["failures", "full"])]
    first: Option<usize>,
    #[arg(long, value_name = "N", help = "Keep the last N matching events", value_parser = parse_report_last, conflicts_with_all = ["failures", "full"])]
    last: Option<usize>,
    #[arg(
        long,
        value_name = "NODE_ID",
        help = "Filter event or failure inspection to one node",
        conflicts_with = "full"
    )]
    node: Option<String>,
    #[arg(hide = true, allow_hyphen_values = true)]
    extra: Vec<String>,
}

#[derive(Args)]
struct ReportListCliArgs {
    #[arg(
        value_name = "PATH",
        help = "Report directory or one report file to list",
        allow_hyphen_values = true
    )]
    target: Option<String>,
    #[arg(long, help = "Emit machine-readable report listing output")]
    json: bool,
    #[arg(hide = true, allow_hyphen_values = true)]
    extra: Vec<String>,
}

#[derive(Args)]
struct ReportLatestCliArgs {
    #[arg(
        value_name = "PATH",
        help = "Report directory or one report file to select from",
        allow_hyphen_values = true
    )]
    target: Option<String>,
    #[arg(long, help = "Emit machine-readable latest-report output")]
    json: bool,
    #[arg(hide = true, allow_hyphen_values = true)]
    extra: Vec<String>,
}

#[derive(Args)]
struct SimCliArgs {
    #[command(subcommand)]
    command: Option<SimSubcommand>,
}

#[derive(Subcommand)]
enum SimSubcommand {
    #[command(about = "Run one test case and write a report")]
    Run(SimRunCliArgs),
    #[command(external_subcommand)]
    External(Vec<String>),
}

#[derive(Args)]
struct SimRunCliArgs {
    #[arg(
        value_name = "PATH",
        help = "Simulator test case to run",
        required = true,
        allow_hyphen_values = true
    )]
    target: String,
    #[arg(long, help = "Emit machine-readable run output")]
    json: bool,
    #[arg(
        long,
        value_name = "SEED",
        help = "Use a fixed seed, or 'random' / 'auto' to generate one"
    )]
    seed: Option<String>,
    #[arg(hide = true, allow_hyphen_values = true)]
    extra: Vec<String>,
}

#[derive(Args)]
struct ValidateCliArgs {
    #[arg(
        value_name = "PATH",
        help = "Repo root, file, or supported directory to validate",
        allow_hyphen_values = true
    )]
    target: Option<String>,
    #[arg(long, help = "Emit machine-readable validation output")]
    json: bool,
    #[arg(long, help = "Treat warnings as failures")]
    strict: bool,
    #[arg(hide = true, allow_hyphen_values = true)]
    extra: Vec<String>,
}

fn print_usage() {
    let mut command = Cli::command();
    command
        .print_long_help()
        .expect("top-level help should render");
    println!();
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
            emit_error_line(error);
        }
        1
    }
}

fn print_head_inspect_json(summary: &HeadInspectSummary) -> Result<i32, CliError> {
    match serde_json::to_string_pretty(summary) {
        Ok(json) => {
            println!("{json}");
            if summary.is_ok() {
                Ok(0)
            } else {
                Ok(1)
            }
        }
        Err(source) => Err(CliError::serialization("head inspection summary", source)),
    }
}

fn head_inspect(doc_id: String, input_path: PathBuf, json: bool) -> Result<i32, CliError> {
    let summary = inspect_heads_from_path(&input_path, &doc_id);
    if json {
        print_head_inspect_json(&summary)
    } else {
        Ok(print_head_inspect_text(&summary))
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
            emit_error_line(error);
        }
        1
    }
}

fn print_object_verification_json(summary: &ObjectVerificationSummary) -> Result<i32, CliError> {
    match serde_json::to_string_pretty(summary) {
        Ok(json) => {
            println!("{json}");
            if summary.is_ok() {
                Ok(0)
            } else {
                Ok(1)
            }
        }
        Err(source) => Err(CliError::serialization(
            "object verification summary",
            source,
        )),
    }
}

fn object_verify(target: PathBuf, json: bool) -> Result<i32, CliError> {
    let summary = verify_object_path(&target);
    if json {
        print_object_verification_json(&summary)
    } else {
        Ok(print_object_verification_text(&summary))
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
            emit_warning_line(format_args!("{}: {}", warning.path, warning.message));
        }
    }

    if !summary.is_ok() {
        println!("validation: failed");
        for error in &summary.errors {
            emit_error_line(format_args!("{}: {}", error.path, error.message));
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

fn print_validation_json(
    summary: &mycel_sim::validate::ValidationSummary,
) -> Result<i32, CliError> {
    match serde_json::to_string_pretty(summary) {
        Ok(json) => {
            println!("{json}");
            if summary.is_ok() {
                Ok(0)
            } else {
                Ok(1)
            }
        }
        Err(source) => Err(CliError::serialization("validation summary", source)),
    }
}

fn validate(target: PathBuf, json: bool, strict: bool) -> Result<i32, CliError> {
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
        Ok(print_validation_text(&summary))
    };

    let print_code = print_code?;

    if print_code != 0 {
        Ok(print_code)
    } else {
        Ok(exit_code)
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

#[derive(Clone)]
struct ReportListEntry {
    path: PathBuf,
    status: String,
    run_id: Option<String>,
    topology_id: Option<String>,
    fixture_id: Option<String>,
    test_id: Option<String>,
    started_at: Option<String>,
    finished_at: Option<String>,
    validation_status: Option<String>,
    result: Option<String>,
    peer_count: usize,
    event_count: usize,
    failure_count: usize,
    parse_error: Option<String>,
}

struct ReportListSummary {
    root: PathBuf,
    status: String,
    report_count: usize,
    valid_report_count: usize,
    invalid_report_count: usize,
    reports: Vec<ReportListEntry>,
    errors: Vec<String>,
}

struct ReportLatestSummary {
    root: PathBuf,
    status: String,
    report_count: usize,
    valid_report_count: usize,
    invalid_report_count: usize,
    selected: Option<ReportListEntry>,
    errors: Vec<String>,
}

impl ReportLatestSummary {
    fn is_ok(&self) -> bool {
        self.errors.is_empty()
    }
}

impl ReportListSummary {
    fn new(root: PathBuf) -> Self {
        Self {
            root,
            status: "ok".to_string(),
            report_count: 0,
            valid_report_count: 0,
            invalid_report_count: 0,
            reports: Vec::new(),
            errors: Vec::new(),
        }
    }

    fn is_ok(&self) -> bool {
        self.errors.is_empty()
    }

    fn refresh_status(&mut self) {
        self.status = if !self.errors.is_empty() {
            "failed".to_string()
        } else if self.invalid_report_count > 0 {
            "warning".to_string()
        } else {
            "ok".to_string()
        };
    }

    fn push_error(&mut self, message: impl Into<String>) {
        self.errors.push(message.into());
        self.refresh_status();
    }

    fn push_report(&mut self, entry: ReportListEntry) {
        self.report_count += 1;
        if entry.status == "ok" {
            self.valid_report_count += 1;
        } else {
            self.invalid_report_count += 1;
        }
        self.reports.push(entry);
        self.refresh_status();
    }
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

fn is_report_schema_file(path: &Path) -> bool {
    path.file_name().and_then(|name| name.to_str()) == Some("report.schema.json")
}

fn is_json_file(path: &Path) -> bool {
    path.extension().and_then(|extension| extension.to_str()) == Some("json")
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

    if is_report_schema_file(&target) {
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

fn collect_report_targets(target: &Path) -> Result<Vec<PathBuf>, String> {
    if !target.exists() {
        return Err(format!(
            "report list target does not exist: {}",
            target.display()
        ));
    }

    if target.is_file() {
        if is_report_schema_file(target) {
            return Err("report schema files are not list targets".to_string());
        }
        if !is_json_file(target) {
            return Err(format!(
                "report list target is not a JSON file: {}",
                target.display()
            ));
        }
        return Ok(vec![target.to_path_buf()]);
    }

    if !target.is_dir() {
        return Err(format!(
            "report list target is not a file or directory: {}",
            target.display()
        ));
    }

    let mut targets = Vec::new();
    collect_report_targets_from_dir(target, &mut targets)?;
    targets.sort();
    Ok(targets)
}

fn collect_report_targets_from_dir(
    target: &Path,
    targets: &mut Vec<PathBuf>,
) -> Result<(), String> {
    let mut entries = fs::read_dir(target)
        .map_err(|err| {
            format!(
                "failed to read report directory {}: {err}",
                target.display()
            )
        })?
        .collect::<Result<Vec<_>, _>>()
        .map_err(|err| {
            format!(
                "failed to read report directory {}: {err}",
                target.display()
            )
        })?;
    entries.sort_by_key(|entry| entry.path());

    for entry in entries {
        let path = entry.path();
        if path.is_dir() {
            collect_report_targets_from_dir(&path, targets)?;
            continue;
        }
        if is_json_file(&path) && !is_report_schema_file(&path) {
            targets.push(path);
        }
    }

    Ok(())
}

fn build_report_list_entry(path: PathBuf) -> ReportListEntry {
    let inspected = inspect_report(path.clone());
    let parse_error = (!inspected.summary.is_ok()).then(|| inspected.summary.errors.join("; "));

    ReportListEntry {
        path,
        status: if parse_error.is_some() {
            "failed".to_string()
        } else {
            "ok".to_string()
        },
        run_id: inspected.summary.run_id,
        topology_id: inspected.summary.topology_id,
        fixture_id: inspected.summary.fixture_id,
        test_id: inspected.summary.test_id,
        started_at: inspected.summary.started_at,
        finished_at: inspected.summary.finished_at,
        validation_status: inspected.summary.validation_status,
        result: inspected.summary.result,
        peer_count: inspected.summary.peer_count,
        event_count: inspected.summary.event_count,
        failure_count: inspected.summary.failure_count,
        parse_error,
    }
}

fn list_reports(target: PathBuf) -> ReportListSummary {
    let mut summary = ReportListSummary::new(target.clone());

    let targets = match collect_report_targets(&target) {
        Ok(targets) => targets,
        Err(message) => {
            summary.push_error(message);
            return summary;
        }
    };

    for report_target in targets {
        summary.push_report(build_report_list_entry(report_target));
    }

    summary
}

fn latest_report_sort_key(report: &ReportListEntry) -> (Option<&str>, Option<&str>, String) {
    (
        report.finished_at.as_deref(),
        report.started_at.as_deref(),
        report.path.to_string_lossy().into_owned(),
    )
}

fn latest_report(summary: ReportListSummary) -> ReportLatestSummary {
    if !summary.is_ok() {
        return ReportLatestSummary {
            root: summary.root,
            status: "failed".to_string(),
            report_count: summary.report_count,
            valid_report_count: summary.valid_report_count,
            invalid_report_count: summary.invalid_report_count,
            selected: None,
            errors: summary.errors,
        };
    }

    if summary.report_count == 0 {
        return ReportLatestSummary {
            root: summary.root,
            status: "failed".to_string(),
            report_count: 0,
            valid_report_count: 0,
            invalid_report_count: 0,
            selected: None,
            errors: vec!["no reports found under target".to_string()],
        };
    }

    let selected = summary
        .reports
        .iter()
        .filter(|report| report.status == "ok")
        .max_by_key(|report| latest_report_sort_key(report))
        .cloned();

    match selected {
        Some(selected) => ReportLatestSummary {
            root: summary.root,
            status: if summary.invalid_report_count > 0 {
                "warning".to_string()
            } else {
                "ok".to_string()
            },
            report_count: summary.report_count,
            valid_report_count: summary.valid_report_count,
            invalid_report_count: summary.invalid_report_count,
            selected: Some(selected),
            errors: Vec::new(),
        },
        None => ReportLatestSummary {
            root: summary.root,
            status: "failed".to_string(),
            report_count: summary.report_count,
            valid_report_count: summary.valid_report_count,
            invalid_report_count: summary.invalid_report_count,
            selected: None,
            errors: vec!["no valid reports found under target".to_string()],
        },
    }
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
            emit_error_line(error);
        }
        1
    }
}

fn print_report_list_text(summary: &ReportListSummary) -> i32 {
    println!("reports root: {}", summary.root.display());
    println!("status: {}", summary.status);
    println!("reports: {}", summary.report_count);
    println!("valid reports: {}", summary.valid_report_count);
    println!("invalid reports: {}", summary.invalid_report_count);

    for report in &summary.reports {
        print!("report: {} status={}", report.path.display(), report.status);
        if let Some(run_id) = &report.run_id {
            print!(" run_id={run_id}");
        }
        if let Some(finished_at) = &report.finished_at {
            print!(" finished_at={finished_at}");
        }
        if let Some(result) = &report.result {
            print!(" result={result}");
        }
        if let Some(validation_status) = &report.validation_status {
            print!(" validation_status={validation_status}");
        }
        if let Some(parse_error) = &report.parse_error {
            print!(" parse_error={parse_error}");
        }
        println!();
    }

    if summary.is_ok() {
        println!("report listing: {}", summary.status);
        0
    } else {
        println!("report listing: failed");
        for error in &summary.errors {
            emit_error_line(error);
        }
        1
    }
}

fn print_report_latest_text(summary: &ReportLatestSummary) -> i32 {
    println!("reports root: {}", summary.root.display());
    println!("status: {}", summary.status);
    println!("reports: {}", summary.report_count);
    println!("valid reports: {}", summary.valid_report_count);
    println!("invalid reports: {}", summary.invalid_report_count);

    if let Some(selected) = &summary.selected {
        println!("selected report: {}", selected.path.display());
        if let Some(run_id) = &selected.run_id {
            println!("run id: {run_id}");
        }
        if let Some(fixture_id) = &selected.fixture_id {
            println!("fixture id: {fixture_id}");
        }
        if let Some(test_id) = &selected.test_id {
            println!("test id: {test_id}");
        }
        if let Some(started_at) = &selected.started_at {
            println!("started at: {started_at}");
        }
        if let Some(finished_at) = &selected.finished_at {
            println!("finished at: {finished_at}");
        }
        if let Some(validation_status) = &selected.validation_status {
            println!("validation status: {validation_status}");
        }
        if let Some(result) = &selected.result {
            println!("result: {result}");
        }
        println!("peers: {}", selected.peer_count);
        println!("events: {}", selected.event_count);
        println!("failures: {}", selected.failure_count);
    }

    if summary.is_ok() {
        println!("report latest: {}", summary.status);
        0
    } else {
        println!("report latest: failed");
        for error in &summary.errors {
            emit_error_line(error);
        }
        1
    }
}

fn print_report_latest_json(summary: &ReportLatestSummary) -> Result<i32, CliError> {
    let selected = summary.selected.as_ref().map(|report| {
        serde_json::json!({
            "path": report.path,
            "status": report.status,
            "run_id": report.run_id,
            "topology_id": report.topology_id,
            "fixture_id": report.fixture_id,
            "test_id": report.test_id,
            "started_at": report.started_at,
            "finished_at": report.finished_at,
            "validation_status": report.validation_status,
            "result": report.result,
            "peer_count": report.peer_count,
            "event_count": report.event_count,
            "failure_count": report.failure_count,
            "parse_error": report.parse_error,
        })
    });
    let json = serde_json::json!({
        "root": summary.root,
        "status": summary.status,
        "report_count": summary.report_count,
        "valid_report_count": summary.valid_report_count,
        "invalid_report_count": summary.invalid_report_count,
        "selected": selected,
        "errors": summary.errors,
    });

    match serde_json::to_string_pretty(&json) {
        Ok(json) => {
            println!("{json}");
            if summary.is_ok() {
                Ok(0)
            } else {
                Ok(1)
            }
        }
        Err(source) => Err(CliError::serialization("latest report summary", source)),
    }
}

fn print_report_list_json(summary: &ReportListSummary) -> Result<i32, CliError> {
    let reports = summary
        .reports
        .iter()
        .map(|report| {
            serde_json::json!({
                "path": report.path,
                "status": report.status,
                "run_id": report.run_id,
                "topology_id": report.topology_id,
                "fixture_id": report.fixture_id,
                "test_id": report.test_id,
                "started_at": report.started_at,
                "finished_at": report.finished_at,
                "validation_status": report.validation_status,
                "result": report.result,
                "peer_count": report.peer_count,
                "event_count": report.event_count,
                "failure_count": report.failure_count,
                "parse_error": report.parse_error,
            })
        })
        .collect::<Vec<_>>();
    let json = serde_json::json!({
        "root": summary.root,
        "status": summary.status,
        "report_count": summary.report_count,
        "valid_report_count": summary.valid_report_count,
        "invalid_report_count": summary.invalid_report_count,
        "reports": reports,
        "errors": summary.errors,
    });

    match serde_json::to_string_pretty(&json) {
        Ok(json) => {
            println!("{json}");
            if summary.is_ok() {
                Ok(0)
            } else {
                Ok(1)
            }
        }
        Err(source) => Err(CliError::serialization("report listing summary", source)),
    }
}

fn print_report_summary_json(summary: &ReportInspectSummary) -> Result<i32, CliError> {
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
                Ok(0)
            } else {
                Ok(1)
            }
        }
        Err(source) => Err(CliError::serialization("report inspection summary", source)),
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
            emit_error_line(error);
        }
        1
    }
}

fn print_report_events_json(
    summary: &ReportInspectSummary,
    events: &[ReportEvent],
) -> Result<i32, CliError> {
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
                Ok(0)
            } else {
                Ok(1)
            }
        }
        Err(source) => Err(CliError::serialization(
            "report event inspection summary",
            source,
        )),
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

fn parse_report_step(value: &str) -> Result<u64, String> {
    parse_u64_flag(value, "--step")
}

fn parse_report_first(value: &str) -> Result<usize, String> {
    parse_usize_flag(value, "--first")
}

fn parse_report_last(value: &str) -> Result<usize, String> {
    parse_usize_flag(value, "--last")
}

fn unexpected_extra(extra: &[String], context: &str) -> Option<String> {
    extra
        .first()
        .map(|arg| format!("unexpected {context} argument: {arg}"))
}

fn handle_head_command(command: HeadCliArgs) -> Result<i32, CliError> {
    match command.command {
        Some(HeadSubcommand::Inspect(args)) => {
            if let Some(message) = unexpected_extra(&args.extra, "head inspect") {
                return Err(CliError::usage(message));
            }

            head_inspect(args.doc_id, PathBuf::from(args.input), args.json)
        }
        Some(HeadSubcommand::External(args)) => {
            let other = args.first().map(String::as_str).unwrap_or("<unknown>");
            Err(CliError::usage(format!("unknown head subcommand: {other}")))
        }
        None => Err(CliError::usage("missing head subcommand")),
    }
}

fn handle_object_command(command: ObjectCliArgs) -> Result<i32, CliError> {
    match command.command {
        Some(ObjectSubcommand::Verify(args)) => {
            if let Some(message) = unexpected_extra(&args.extra, "object verify") {
                return Err(CliError::usage(message));
            }

            object_verify(PathBuf::from(args.target), args.json)
        }
        Some(ObjectSubcommand::External(args)) => {
            let other = args.first().map(String::as_str).unwrap_or("<unknown>");
            Err(CliError::usage(format!(
                "unknown object subcommand: {other}"
            )))
        }
        None => Err(CliError::usage("missing object subcommand")),
    }
}

fn handle_report_command(command: ReportCliArgs) -> Result<i32, CliError> {
    match command.command {
        Some(ReportSubcommand::Inspect(args)) => {
            if let Some(message) = unexpected_extra(&args.extra, "report inspect") {
                return Err(CliError::usage(message));
            }

            let mut mode = ReportInspectMode::Summary;
            if args.events {
                mode = ReportInspectMode::Events;
            }
            if args.failures {
                mode = ReportInspectMode::Failures;
            }
            if args.full {
                mode = ReportInspectMode::Full;
            }

            let phase = args.phase;
            let action = args.action;
            let outcome = args.outcome;
            let step = args.step;
            let step_range = args.step_range;
            let first = args.first;
            let last = args.last;
            let node = args.node;

            let Some(target) = args.target else {
                return Err(CliError::usage("missing report inspect target"));
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

            report_inspect(PathBuf::from(target), args.json, mode, &filters)
        }
        Some(ReportSubcommand::List(args)) => {
            if let Some(message) = unexpected_extra(&args.extra, "report list") {
                return Err(CliError::usage(message));
            }

            let target = args.target.unwrap_or_else(|| "sim/reports".to_owned());
            report_list(PathBuf::from(target), args.json)
        }
        Some(ReportSubcommand::Latest(args)) => {
            if let Some(message) = unexpected_extra(&args.extra, "report latest") {
                return Err(CliError::usage(message));
            }

            let target = args.target.unwrap_or_else(|| "sim/reports".to_owned());
            report_latest(PathBuf::from(target), args.json)
        }
        Some(ReportSubcommand::External(args)) => {
            let other = args.first().map(String::as_str).unwrap_or("<unknown>");
            Err(CliError::usage(format!(
                "unknown report subcommand: {other}"
            )))
        }
        None => Err(CliError::usage("missing report subcommand")),
    }
}

fn handle_validate_command(args: ValidateCliArgs) -> Result<i32, CliError> {
    if let Some(message) = unexpected_extra(&args.extra, "validate") {
        return Err(CliError::usage(message));
    }

    let target = args.target.unwrap_or_else(|| ".".to_owned());
    validate(PathBuf::from(target), args.json, args.strict)
}

fn handle_sim_command(command: SimCliArgs) -> Result<i32, CliError> {
    match command.command {
        Some(SimSubcommand::Run(args)) => {
            if let Some(message) = unexpected_extra(&args.extra, "sim run") {
                return Err(CliError::usage(message));
            }

            sim_run(PathBuf::from(args.target), args.json, args.seed)
        }
        Some(SimSubcommand::External(args)) => {
            let other = args.first().map(String::as_str).unwrap_or("<unknown>");
            Err(CliError::usage(format!("unknown sim subcommand: {other}")))
        }
        None => Err(CliError::usage("missing sim subcommand")),
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
            emit_error_line(error);
        }
        1
    }
}

fn print_report_failures_json(
    summary: &ReportInspectSummary,
    failures: &[ReportFailure],
) -> Result<i32, CliError> {
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
                Ok(0)
            } else {
                Ok(1)
            }
        }
        Err(source) => Err(CliError::serialization(
            "report failure inspection summary",
            source,
        )),
    }
}

fn print_report_full_json(
    summary: &ReportInspectSummary,
    report: &Report,
) -> Result<i32, CliError> {
    if !summary.is_ok() {
        return print_report_summary_json(summary);
    }

    match serde_json::to_string_pretty(report) {
        Ok(json) => {
            println!("{json}");
            Ok(0)
        }
        Err(source) => Err(CliError::serialization("full report JSON", source)),
    }
}

fn report_inspect(
    target: PathBuf,
    json: bool,
    mode: ReportInspectMode,
    filters: &ReportInspectFilters,
) -> Result<i32, CliError> {
    let inspected = inspect_report(target);
    match mode {
        ReportInspectMode::Summary => {
            if json {
                print_report_summary_json(&inspected.summary)
            } else {
                Ok(print_report_text(&inspected.summary))
            }
        }
        ReportInspectMode::Events => {
            let filtered_events = filter_events(&inspected.events, filters);
            if json {
                print_report_events_json(&inspected.summary, &filtered_events)
            } else {
                Ok(print_report_events_text(
                    &inspected.summary,
                    &filtered_events,
                ))
            }
        }
        ReportInspectMode::Failures => {
            let filtered_failures = filter_failures(&inspected.failures, filters);
            if json {
                print_report_failures_json(&inspected.summary, &filtered_failures)
            } else {
                Ok(print_report_failures_text(
                    &inspected.summary,
                    &filtered_failures,
                ))
            }
        }
        ReportInspectMode::Full => {
            if json {
                match inspected.report.as_ref() {
                    Some(report) => print_report_full_json(&inspected.summary, report),
                    None => print_report_summary_json(&inspected.summary),
                }
            } else {
                Err(CliError::usage("report inspect --full requires --json"))
            }
        }
    }
}

fn report_list(target: PathBuf, json: bool) -> Result<i32, CliError> {
    let summary = list_reports(target);
    if json {
        print_report_list_json(&summary)
    } else {
        Ok(print_report_list_text(&summary))
    }
}

fn report_latest(target: PathBuf, json: bool) -> Result<i32, CliError> {
    let summary = latest_report(list_reports(target));
    if json {
        print_report_latest_json(&summary)
    } else {
        Ok(print_report_latest_text(&summary))
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
        emit_warning_line(warning);
    }

    0
}

fn print_run_json(summary: &mycel_sim::run::SimulationRunSummary) -> Result<i32, CliError> {
    match serde_json::to_string_pretty(summary) {
        Ok(json) => {
            println!("{json}");
            Ok(0)
        }
        Err(source) => Err(CliError::serialization("run summary", source)),
    }
}

fn sim_run(target: PathBuf, json: bool, seed_override: Option<String>) -> Result<i32, CliError> {
    let options = RunOptions { seed_override };
    match run_test_case_with_options(&target, &options) {
        Ok(summary) => {
            if json {
                print_run_json(&summary)
            } else {
                Ok(print_run_text(&summary))
            }
        }
        Err(message) => Err(CliError::SimRun(message)),
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
            if matches!(
                err.kind(),
                clap::error::ErrorKind::DisplayHelp | clap::error::ErrorKind::DisplayVersion
            ) {
                err.print().expect("clap help should print");
            } else {
                eprintln!("{err}");
            }
            std::process::exit(err.exit_code());
        }
    };

    let result = match cli.command {
        Some(CliCommand::Head(command)) => handle_head_command(command),
        Some(CliCommand::Info) => {
            print_info();
            Ok(0)
        }
        Some(CliCommand::Object(command)) => handle_object_command(command),
        Some(CliCommand::Report(command)) => handle_report_command(command),
        Some(CliCommand::Sim(command)) => handle_sim_command(command),
        Some(CliCommand::Validate(command)) => handle_validate_command(command),
        Some(CliCommand::External(args)) => {
            let other = args.first().map(String::as_str).unwrap_or("<unknown>");
            Err(CliError::usage(format!("unknown command: {other}")))
        }
        None => {
            print_usage();
            Ok(0)
        }
    };

    match result {
        Ok(exit_code) => std::process::exit(exit_code),
        Err(error) => {
            error.emit();
            std::process::exit(error.exit_code());
        }
    }
}
