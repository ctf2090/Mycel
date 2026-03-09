use std::collections::BTreeMap;
use std::env;
use std::fs;
use std::path::{Path, PathBuf};

use clap::{Args, CommandFactory, Parser, Subcommand, ValueEnum};
use mycel_core::head::inspect_heads_from_path;
use mycel_core::head::HeadInspectSummary;
use mycel_core::verify::{
    inspect_object_path, verify_object_path, ObjectInspectionSummary, ObjectVerificationSummary,
};
use mycel_core::workspace_banner;
use mycel_sim::manifest::SimulatorPaths;
use mycel_sim::model::{Report, ReportEvent, ReportFailure};
use mycel_sim::run::{run_test_case_with_options, RunOptions};
use mycel_sim::simulator_banner;
use mycel_sim::validate::validate_path;
use serde::Serialize;
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
    #[command(about = "Inspect or verify one Mycel object file")]
    Object(ObjectCliArgs),
    #[command(about = "Inspect, compare, and query simulator reports")]
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
    #[command(about = "Inspect one object file without verifying signatures")]
    Inspect(ObjectInspectCliArgs),
    #[command(about = "Verify one object file")]
    Verify(ObjectVerifyCliArgs),
    #[command(external_subcommand)]
    External(Vec<String>),
}

#[derive(Args)]
struct ObjectInspectCliArgs {
    #[arg(
        value_name = "PATH",
        help = "Object file to inspect",
        required = true,
        allow_hyphen_values = true
    )]
    target: String,
    #[arg(long, help = "Emit machine-readable object inspection output")]
    json: bool,
    #[arg(hide = true, allow_hyphen_values = true)]
    extra: Vec<String>,
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
    #[command(about = "Compare two simulator reports at the summary level")]
    Diff(ReportDiffCliArgs),
    #[command(about = "Inspect one simulator report")]
    Inspect(ReportInspectCliArgs),
    #[command(about = "List simulator reports under a directory or one file")]
    List(ReportListCliArgs),
    #[command(about = "Select the latest simulator report under a directory or one file")]
    Latest(ReportLatestCliArgs),
    #[command(about = "Summarize simulator reports under a directory or one file")]
    Stats(ReportStatsCliArgs),
    #[command(external_subcommand)]
    External(Vec<String>),
}

#[derive(Args)]
struct ReportDiffCliArgs {
    #[arg(
        value_name = "LEFT_PATH",
        help = "Left-hand simulator report to compare",
        required = true,
        allow_hyphen_values = true
    )]
    left: String,
    #[arg(
        value_name = "RIGHT_PATH",
        help = "Right-hand simulator report to compare",
        required = true,
        allow_hyphen_values = true
    )]
    right: String,
    #[arg(long, help = "Emit machine-readable report diff output")]
    json: bool,
    #[arg(
        long,
        help = "Compare event traces step-by-step instead of summary fields"
    )]
    events: bool,
    #[arg(hide = true, allow_hyphen_values = true)]
    extra: Vec<String>,
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
    #[arg(
        long,
        help = "Print only matching valid report paths",
        conflicts_with = "json"
    )]
    path_only: bool,
    #[arg(
        long,
        value_name = "RESULT",
        help = "List only reports with one result",
        value_enum
    )]
    result: Option<ReportResultFilter>,
    #[arg(
        long = "validation-status",
        value_name = "VALIDATION_STATUS",
        help = "List only reports with one validation status",
        value_enum
    )]
    validation_status: Option<ReportValidationStatusFilter>,
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
    #[arg(
        long,
        value_name = "RESULT",
        help = "Select only reports with one result",
        value_enum
    )]
    result: Option<ReportResultFilter>,
    #[arg(
        long = "validation-status",
        value_name = "VALIDATION_STATUS",
        help = "Select only reports with one validation status",
        value_enum
    )]
    validation_status: Option<ReportValidationStatusFilter>,
    #[arg(
        long,
        help = "Print only the selected report path",
        conflicts_with_all = ["json", "full"]
    )]
    path_only: bool,
    #[arg(
        long,
        help = "Emit the selected raw report (requires --json)",
        requires = "json"
    )]
    full: bool,
    #[arg(hide = true, allow_hyphen_values = true)]
    extra: Vec<String>,
}

#[derive(Args)]
struct ReportStatsCliArgs {
    #[arg(
        value_name = "PATH",
        help = "Report directory or one report file to summarize",
        allow_hyphen_values = true
    )]
    target: Option<String>,
    #[arg(long, help = "Emit machine-readable report statistics output")]
    json: bool,
    #[arg(
        long = "counts-only",
        help = "Emit only aggregate counts (requires --json)",
        requires = "json",
        conflicts_with_all = ["full_latest", "path_only_latest"]
    )]
    counts_only: bool,
    #[arg(
        long = "full-latest",
        help = "Emit the latest matching raw report (requires --json)",
        requires = "json",
        conflicts_with = "path_only_latest"
    )]
    full_latest: bool,
    #[arg(
        long = "path-only-latest",
        help = "Print only the latest matching valid report path",
        conflicts_with_all = ["json", "full_latest"]
    )]
    path_only_latest: bool,
    #[arg(
        long,
        value_name = "RESULT",
        help = "Summarize only reports with one result",
        value_enum
    )]
    result: Option<ReportResultFilter>,
    #[arg(
        long = "validation-status",
        value_name = "VALIDATION_STATUS",
        help = "Summarize only reports with one validation status",
        value_enum
    )]
    validation_status: Option<ReportValidationStatusFilter>,
    #[arg(hide = true, allow_hyphen_values = true)]
    extra: Vec<String>,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, ValueEnum)]
enum ReportResultFilter {
    Pass,
    Fail,
}

impl ReportResultFilter {
    fn as_str(self) -> &'static str {
        match self {
            Self::Pass => "pass",
            Self::Fail => "fail",
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, ValueEnum)]
enum ReportValidationStatusFilter {
    Ok,
    Warning,
    Failed,
}

impl ReportValidationStatusFilter {
    fn as_str(self) -> &'static str {
        match self {
            Self::Ok => "ok",
            Self::Warning => "warning",
            Self::Failed => "failed",
        }
    }
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

fn print_object_inspection_text(summary: &ObjectInspectionSummary) -> i32 {
    println!("object path: {}", summary.path.display());
    if let Some(object_type) = &summary.object_type {
        println!("object type: {object_type}");
    }
    if let Some(version) = &summary.version {
        println!("version: {version}");
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
    if let Some(declared_id_field) = &summary.declared_id_field {
        println!("declared id field: {declared_id_field}");
    }
    if let Some(declared_id) = &summary.declared_id {
        println!("declared id: {declared_id}");
    }
    println!(
        "has signature: {}",
        if summary.has_signature { "yes" } else { "no" }
    );
    if !summary.top_level_keys.is_empty() {
        println!("top-level keys: {}", summary.top_level_keys.join(", "));
    }
    println!("status: {}", summary.status);

    for note in &summary.notes {
        println!("note: {note}");
    }

    if summary.is_failed() {
        println!("inspection: failed");
        for error in &summary.errors {
            emit_error_line(error);
        }
        1
    } else {
        println!("inspection: {}", summary.status);
        0
    }
}

fn print_object_inspection_json(summary: &ObjectInspectionSummary) -> Result<i32, CliError> {
    match serde_json::to_string_pretty(summary) {
        Ok(json) => {
            println!("{json}");
            if summary.is_failed() {
                Ok(1)
            } else {
                Ok(0)
            }
        }
        Err(source) => Err(CliError::serialization("object inspection summary", source)),
    }
}

fn object_inspect(target: PathBuf, json: bool) -> Result<i32, CliError> {
    let summary = inspect_object_path(&target);
    if json {
        print_object_inspection_json(&summary)
    } else {
        Ok(print_object_inspection_text(&summary))
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
    result_filter: Option<ReportResultFilter>,
    validation_status_filter: Option<ReportValidationStatusFilter>,
    report_count: usize,
    valid_report_count: usize,
    invalid_report_count: usize,
    reports: Vec<ReportListEntry>,
    errors: Vec<String>,
}

struct ReportLatestSummary {
    root: PathBuf,
    status: String,
    result_filter: Option<ReportResultFilter>,
    validation_status_filter: Option<ReportValidationStatusFilter>,
    report_count: usize,
    valid_report_count: usize,
    invalid_report_count: usize,
    selected: Option<ReportListEntry>,
    errors: Vec<String>,
}

struct ReportStatsLatestReport {
    path: PathBuf,
    run_id: Option<String>,
    finished_at: Option<String>,
    result: Option<String>,
    validation_status: Option<String>,
}

struct ReportStatsSummary {
    root: PathBuf,
    status: String,
    result_filter: Option<ReportResultFilter>,
    validation_status_filter: Option<ReportValidationStatusFilter>,
    report_count: usize,
    valid_report_count: usize,
    invalid_report_count: usize,
    result_counts: BTreeMap<String, usize>,
    validation_status_counts: BTreeMap<String, usize>,
    latest_finished_at: Option<String>,
    latest_valid_report: Option<ReportStatsLatestReport>,
    errors: Vec<String>,
}

#[derive(Debug, Clone, Serialize)]
struct ReportDiffSideSummary {
    path: PathBuf,
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

#[derive(Debug, Clone, Serialize)]
struct ReportDiffEntry {
    field: String,
    left: serde_json::Value,
    right: serde_json::Value,
}

#[derive(Debug, Clone, Serialize)]
struct ReportDiffSummary {
    status: String,
    comparison: String,
    difference_count: usize,
    left: ReportDiffSideSummary,
    right: ReportDiffSideSummary,
    differences: Vec<ReportDiffEntry>,
    errors: Vec<String>,
}

#[derive(Debug, Clone, Serialize)]
struct ReportEventDiffEntry {
    step: u64,
    change: String,
    left: Option<ReportEvent>,
    right: Option<ReportEvent>,
}

#[derive(Debug, Clone, Serialize)]
struct ReportEventDiffSummary {
    status: String,
    comparison: String,
    event_difference_count: usize,
    left: ReportDiffSideSummary,
    right: ReportDiffSideSummary,
    event_differences: Vec<ReportEventDiffEntry>,
    errors: Vec<String>,
}

#[derive(Clone, Copy)]
struct ReportQuerySummaryView<'a> {
    root: &'a Path,
    status: &'a str,
    result_filter: Option<ReportResultFilter>,
    validation_status_filter: Option<ReportValidationStatusFilter>,
    report_count: usize,
    valid_report_count: usize,
    invalid_report_count: usize,
    errors: &'a [String],
}

#[derive(Clone, Copy, Default, PartialEq, Eq)]
struct ReportQuery {
    result_filter: Option<ReportResultFilter>,
    validation_status_filter: Option<ReportValidationStatusFilter>,
}

impl ReportQuery {
    fn new(
        result_filter: Option<ReportResultFilter>,
        validation_status_filter: Option<ReportValidationStatusFilter>,
    ) -> Self {
        Self {
            result_filter,
            validation_status_filter,
        }
    }

    fn matches_valid_report(self, report: &ReportListEntry) -> bool {
        self.result_filter
            .is_none_or(|expected| report.result.as_deref() == Some(expected.as_str()))
            && self.validation_status_filter.is_none_or(|expected| {
                report.validation_status.as_deref() == Some(expected.as_str())
            })
    }

    fn describe_missing(self) -> String {
        let mut filters = Vec::new();
        if let Some(result_filter) = self.result_filter {
            filters.push(format!("result={}", result_filter.as_str()));
        }
        if let Some(validation_status_filter) = self.validation_status_filter {
            filters.push(format!(
                "validation_status={}",
                validation_status_filter.as_str()
            ));
        }

        if filters.is_empty() {
            "no valid reports found under target".to_string()
        } else {
            format!(
                "no valid reports found under target with {}",
                filters.join(", ")
            )
        }
    }
}

impl ReportLatestSummary {
    fn is_ok(&self) -> bool {
        self.errors.is_empty()
    }
}

impl ReportStatsSummary {
    fn is_ok(&self) -> bool {
        self.errors.is_empty()
    }
}

impl<'a> From<&'a ReportListSummary> for ReportQuerySummaryView<'a> {
    fn from(summary: &'a ReportListSummary) -> Self {
        Self {
            root: &summary.root,
            status: &summary.status,
            result_filter: summary.result_filter,
            validation_status_filter: summary.validation_status_filter,
            report_count: summary.report_count,
            valid_report_count: summary.valid_report_count,
            invalid_report_count: summary.invalid_report_count,
            errors: &summary.errors,
        }
    }
}

impl<'a> From<&'a ReportLatestSummary> for ReportQuerySummaryView<'a> {
    fn from(summary: &'a ReportLatestSummary) -> Self {
        Self {
            root: &summary.root,
            status: &summary.status,
            result_filter: summary.result_filter,
            validation_status_filter: summary.validation_status_filter,
            report_count: summary.report_count,
            valid_report_count: summary.valid_report_count,
            invalid_report_count: summary.invalid_report_count,
            errors: &summary.errors,
        }
    }
}

impl<'a> From<&'a ReportStatsSummary> for ReportQuerySummaryView<'a> {
    fn from(summary: &'a ReportStatsSummary) -> Self {
        Self {
            root: &summary.root,
            status: &summary.status,
            result_filter: summary.result_filter,
            validation_status_filter: summary.validation_status_filter,
            report_count: summary.report_count,
            valid_report_count: summary.valid_report_count,
            invalid_report_count: summary.invalid_report_count,
            errors: &summary.errors,
        }
    }
}

impl ReportListSummary {
    fn new(root: PathBuf) -> Self {
        Self {
            root,
            status: "ok".to_string(),
            result_filter: None,
            validation_status_filter: None,
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

fn report_diff_side(summary: &ReportInspectSummary) -> ReportDiffSideSummary {
    ReportDiffSideSummary {
        path: summary.path.clone(),
        run_id: summary.run_id.clone(),
        topology_id: summary.topology_id.clone(),
        fixture_id: summary.fixture_id.clone(),
        test_id: summary.test_id.clone(),
        execution_mode: summary.execution_mode.clone(),
        started_at: summary.started_at.clone(),
        finished_at: summary.finished_at.clone(),
        validation_status: summary.validation_status.clone(),
        deterministic_seed: summary.deterministic_seed.clone(),
        seed_source: summary.seed_source.clone(),
        result: summary.result.clone(),
        peer_count: summary.peer_count,
        event_count: summary.event_count,
        failure_count: summary.failure_count,
        verified_object_count: summary.verified_object_count,
        rejected_object_count: summary.rejected_object_count,
        matched_expected_outcomes: summary.matched_expected_outcomes.clone(),
        scheduled_peer_order: summary.scheduled_peer_order.clone(),
        fault_plan_count: summary.fault_plan_count,
        errors: summary.errors.clone(),
    }
}

fn push_report_diff_if_changed(
    differences: &mut Vec<ReportDiffEntry>,
    field: &str,
    left: serde_json::Value,
    right: serde_json::Value,
) {
    if left != right {
        differences.push(ReportDiffEntry {
            field: field.to_string(),
            left,
            right,
        });
    }
}

fn collect_report_diff_errors(
    left: &ReportDiffSideSummary,
    right: &ReportDiffSideSummary,
) -> Vec<String> {
    let mut errors = Vec::new();
    errors.extend(
        left.errors
            .iter()
            .map(|message| format!("left report: {message}")),
    );
    errors.extend(
        right
            .errors
            .iter()
            .map(|message| format!("right report: {message}")),
    );
    errors
}

fn diff_reports(left_path: PathBuf, right_path: PathBuf) -> ReportDiffSummary {
    let left_inspected = inspect_report(left_path);
    let right_inspected = inspect_report(right_path);

    let left = report_diff_side(&left_inspected.summary);
    let right = report_diff_side(&right_inspected.summary);

    let errors = collect_report_diff_errors(&left, &right);

    if !errors.is_empty() {
        return ReportDiffSummary {
            status: "failed".to_string(),
            comparison: "failed".to_string(),
            difference_count: 0,
            left,
            right,
            differences: Vec::new(),
            errors,
        };
    }

    let mut differences = Vec::new();
    push_report_diff_if_changed(
        &mut differences,
        "run_id",
        serde_json::json!(left.run_id),
        serde_json::json!(right.run_id),
    );
    push_report_diff_if_changed(
        &mut differences,
        "topology_id",
        serde_json::json!(left.topology_id),
        serde_json::json!(right.topology_id),
    );
    push_report_diff_if_changed(
        &mut differences,
        "fixture_id",
        serde_json::json!(left.fixture_id),
        serde_json::json!(right.fixture_id),
    );
    push_report_diff_if_changed(
        &mut differences,
        "test_id",
        serde_json::json!(left.test_id),
        serde_json::json!(right.test_id),
    );
    push_report_diff_if_changed(
        &mut differences,
        "execution_mode",
        serde_json::json!(left.execution_mode),
        serde_json::json!(right.execution_mode),
    );
    push_report_diff_if_changed(
        &mut differences,
        "started_at",
        serde_json::json!(left.started_at),
        serde_json::json!(right.started_at),
    );
    push_report_diff_if_changed(
        &mut differences,
        "finished_at",
        serde_json::json!(left.finished_at),
        serde_json::json!(right.finished_at),
    );
    push_report_diff_if_changed(
        &mut differences,
        "validation_status",
        serde_json::json!(left.validation_status),
        serde_json::json!(right.validation_status),
    );
    push_report_diff_if_changed(
        &mut differences,
        "deterministic_seed",
        serde_json::json!(left.deterministic_seed),
        serde_json::json!(right.deterministic_seed),
    );
    push_report_diff_if_changed(
        &mut differences,
        "seed_source",
        serde_json::json!(left.seed_source),
        serde_json::json!(right.seed_source),
    );
    push_report_diff_if_changed(
        &mut differences,
        "result",
        serde_json::json!(left.result),
        serde_json::json!(right.result),
    );
    push_report_diff_if_changed(
        &mut differences,
        "peer_count",
        serde_json::json!(left.peer_count),
        serde_json::json!(right.peer_count),
    );
    push_report_diff_if_changed(
        &mut differences,
        "event_count",
        serde_json::json!(left.event_count),
        serde_json::json!(right.event_count),
    );
    push_report_diff_if_changed(
        &mut differences,
        "failure_count",
        serde_json::json!(left.failure_count),
        serde_json::json!(right.failure_count),
    );
    push_report_diff_if_changed(
        &mut differences,
        "verified_object_count",
        serde_json::json!(left.verified_object_count),
        serde_json::json!(right.verified_object_count),
    );
    push_report_diff_if_changed(
        &mut differences,
        "rejected_object_count",
        serde_json::json!(left.rejected_object_count),
        serde_json::json!(right.rejected_object_count),
    );
    push_report_diff_if_changed(
        &mut differences,
        "matched_expected_outcomes",
        serde_json::json!(left.matched_expected_outcomes),
        serde_json::json!(right.matched_expected_outcomes),
    );
    push_report_diff_if_changed(
        &mut differences,
        "scheduled_peer_order",
        serde_json::json!(left.scheduled_peer_order),
        serde_json::json!(right.scheduled_peer_order),
    );
    push_report_diff_if_changed(
        &mut differences,
        "fault_plan_count",
        serde_json::json!(left.fault_plan_count),
        serde_json::json!(right.fault_plan_count),
    );

    let comparison = if differences.is_empty() {
        "match"
    } else {
        "different"
    };

    ReportDiffSummary {
        status: "ok".to_string(),
        comparison: comparison.to_string(),
        difference_count: differences.len(),
        left,
        right,
        differences,
        errors: Vec::new(),
    }
}

fn diff_report_events(left_path: PathBuf, right_path: PathBuf) -> ReportEventDiffSummary {
    let left_inspected = inspect_report(left_path);
    let right_inspected = inspect_report(right_path);

    let left = report_diff_side(&left_inspected.summary);
    let right = report_diff_side(&right_inspected.summary);

    let errors = collect_report_diff_errors(&left, &right);
    if !errors.is_empty() {
        return ReportEventDiffSummary {
            status: "failed".to_string(),
            comparison: "failed".to_string(),
            event_difference_count: 0,
            left,
            right,
            event_differences: Vec::new(),
            errors,
        };
    }

    let mut left_by_step = BTreeMap::new();
    for event in left_inspected.events {
        left_by_step.insert(event.step, event);
    }

    let mut right_by_step = BTreeMap::new();
    for event in right_inspected.events {
        right_by_step.insert(event.step, event);
    }

    let all_steps = left_by_step
        .keys()
        .chain(right_by_step.keys())
        .copied()
        .collect::<std::collections::BTreeSet<_>>();

    let mut event_differences = Vec::new();
    for step in all_steps {
        let left_event = left_by_step.get(&step).cloned();
        let right_event = right_by_step.get(&step).cloned();
        match (&left_event, &right_event) {
            (Some(left_event), Some(right_event))
                if !report_events_equal(left_event, right_event) =>
            {
                event_differences.push(ReportEventDiffEntry {
                    step,
                    change: "changed".to_string(),
                    left: Some(left_event.clone()),
                    right: Some(right_event.clone()),
                });
            }
            (Some(_), None) => {
                event_differences.push(ReportEventDiffEntry {
                    step,
                    change: "left_only".to_string(),
                    left: left_event,
                    right: None,
                });
            }
            (None, Some(_)) => {
                event_differences.push(ReportEventDiffEntry {
                    step,
                    change: "right_only".to_string(),
                    left: None,
                    right: right_event,
                });
            }
            _ => {}
        }
    }

    let comparison = if event_differences.is_empty() {
        "match"
    } else {
        "different"
    };

    ReportEventDiffSummary {
        status: "ok".to_string(),
        comparison: comparison.to_string(),
        event_difference_count: event_differences.len(),
        left,
        right,
        event_differences,
        errors: Vec::new(),
    }
}

fn format_report_diff_value(value: &serde_json::Value) -> String {
    match value {
        serde_json::Value::String(text) => format!("{text:?}"),
        _ => value.to_string(),
    }
}

fn report_events_equal(left: &ReportEvent, right: &ReportEvent) -> bool {
    serde_json::to_value(left).ok() == serde_json::to_value(right).ok()
}

fn format_report_event_text(event: &ReportEvent) -> String {
    let node = event.node_id.as_deref().unwrap_or("-");
    let detail = event.detail.as_deref().unwrap_or("-");
    format!(
        "phase={} action={} outcome={} node={} objects={} detail={detail:?}",
        event.phase,
        event.action,
        event.outcome,
        node,
        event.object_ids.len(),
    )
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

fn apply_report_query(mut summary: ReportListSummary, query: ReportQuery) -> ReportListSummary {
    summary.result_filter = query.result_filter;
    summary.validation_status_filter = query.validation_status_filter;

    if query == ReportQuery::default() {
        return summary;
    }

    summary
        .reports
        .retain(|report| report.status != "ok" || query.matches_valid_report(report));
    summary.report_count = summary.reports.len();
    summary.valid_report_count = summary
        .reports
        .iter()
        .filter(|report| report.status == "ok")
        .count();
    summary.invalid_report_count = summary
        .reports
        .iter()
        .filter(|report| report.status != "ok")
        .count();
    summary.refresh_status();
    summary
}

fn query_reports(target: PathBuf, query: ReportQuery) -> ReportListSummary {
    apply_report_query(list_reports(target), query)
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
            result_filter: summary.result_filter,
            validation_status_filter: summary.validation_status_filter,
            report_count: summary.report_count,
            valid_report_count: summary.valid_report_count,
            invalid_report_count: summary.invalid_report_count,
            selected: None,
            errors: summary.errors,
        };
    }

    if summary.report_count == 0 {
        let query = ReportQuery::new(summary.result_filter, summary.validation_status_filter);
        return ReportLatestSummary {
            root: summary.root,
            status: "failed".to_string(),
            result_filter: summary.result_filter,
            validation_status_filter: summary.validation_status_filter,
            report_count: 0,
            valid_report_count: 0,
            invalid_report_count: 0,
            selected: None,
            errors: vec![if query == ReportQuery::default() {
                "no reports found under target".to_string()
            } else {
                query.describe_missing()
            }],
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
            result_filter: summary.result_filter,
            validation_status_filter: summary.validation_status_filter,
            report_count: summary.report_count,
            valid_report_count: summary.valid_report_count,
            invalid_report_count: summary.invalid_report_count,
            selected: Some(selected),
            errors: Vec::new(),
        },
        None => ReportLatestSummary {
            root: summary.root,
            status: "failed".to_string(),
            result_filter: summary.result_filter,
            validation_status_filter: summary.validation_status_filter,
            report_count: summary.report_count,
            valid_report_count: summary.valid_report_count,
            invalid_report_count: summary.invalid_report_count,
            selected: None,
            errors: vec![
                ReportQuery::new(summary.result_filter, summary.validation_status_filter)
                    .describe_missing(),
            ],
        },
    }
}

fn latest_report_path(summary: &ReportLatestSummary) -> Option<PathBuf> {
    summary
        .selected
        .as_ref()
        .map(|selected| selected.path.clone())
}

fn summarize_reports(summary: ReportListSummary) -> ReportStatsSummary {
    let mut result_counts = BTreeMap::new();
    let mut validation_status_counts = BTreeMap::new();
    let latest_valid_report = summary
        .reports
        .iter()
        .filter(|report| report.status == "ok")
        .max_by_key(|report| latest_report_sort_key(report))
        .cloned();

    for report in summary
        .reports
        .iter()
        .filter(|report| report.status == "ok")
    {
        let result_key = report.result.as_deref().unwrap_or("unknown").to_string();
        *result_counts.entry(result_key).or_insert(0) += 1;

        let validation_status_key = report
            .validation_status
            .as_deref()
            .unwrap_or("unknown")
            .to_string();
        *validation_status_counts
            .entry(validation_status_key)
            .or_insert(0) += 1;
    }

    let latest_finished_at = latest_valid_report
        .as_ref()
        .and_then(|report| report.finished_at.clone());
    let latest_valid_report = latest_valid_report.map(|report| ReportStatsLatestReport {
        path: report.path,
        run_id: report.run_id,
        finished_at: report.finished_at,
        result: report.result,
        validation_status: report.validation_status,
    });

    ReportStatsSummary {
        root: summary.root,
        status: summary.status,
        result_filter: summary.result_filter,
        validation_status_filter: summary.validation_status_filter,
        report_count: summary.report_count,
        valid_report_count: summary.valid_report_count,
        invalid_report_count: summary.invalid_report_count,
        result_counts,
        validation_status_counts,
        latest_finished_at,
        latest_valid_report,
        errors: summary.errors,
    }
}

fn print_report_query_text_header(summary: ReportQuerySummaryView<'_>) {
    println!("reports root: {}", summary.root.display());
    println!("status: {}", summary.status);
    if let Some(result_filter) = summary.result_filter {
        println!("result filter: {}", result_filter.as_str());
    }
    if let Some(validation_status_filter) = summary.validation_status_filter {
        println!(
            "validation status filter: {}",
            validation_status_filter.as_str()
        );
    }
    println!("reports: {}", summary.report_count);
    println!("valid reports: {}", summary.valid_report_count);
    println!("invalid reports: {}", summary.invalid_report_count);
}

fn report_query_summary_json(
    summary: ReportQuerySummaryView<'_>,
) -> serde_json::Map<String, serde_json::Value> {
    let mut json = serde_json::Map::new();
    json.insert("root".to_string(), serde_json::json!(summary.root));
    json.insert("status".to_string(), serde_json::json!(summary.status));
    json.insert(
        "result_filter".to_string(),
        serde_json::json!(summary.result_filter.map(ReportResultFilter::as_str)),
    );
    json.insert(
        "validation_status_filter".to_string(),
        serde_json::json!(summary
            .validation_status_filter
            .map(ReportValidationStatusFilter::as_str)),
    );
    json.insert(
        "report_count".to_string(),
        serde_json::json!(summary.report_count),
    );
    json.insert(
        "valid_report_count".to_string(),
        serde_json::json!(summary.valid_report_count),
    );
    json.insert(
        "invalid_report_count".to_string(),
        serde_json::json!(summary.invalid_report_count),
    );
    json.insert("errors".to_string(), serde_json::json!(summary.errors));
    json
}

fn report_list_entry_json(report: &ReportListEntry) -> serde_json::Value {
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
}

fn finish_report_query_text(command: &str, summary: ReportQuerySummaryView<'_>) -> i32 {
    if summary.errors.is_empty() {
        println!("{command}: {}", summary.status);
        0
    } else {
        println!("{command}: failed");
        for error in summary.errors {
            emit_error_line(error);
        }
        1
    }
}

fn finish_report_query_paths(summary: ReportQuerySummaryView<'_>) -> i32 {
    if summary.errors.is_empty() {
        0
    } else {
        for error in summary.errors {
            emit_error_line(error);
        }
        1
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
    let query_summary = ReportQuerySummaryView::from(summary);
    print_report_query_text_header(query_summary);

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

    finish_report_query_text("report listing", query_summary)
}

fn print_report_latest_text(summary: &ReportLatestSummary) -> i32 {
    let query_summary = ReportQuerySummaryView::from(summary);
    print_report_query_text_header(query_summary);

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

    finish_report_query_text("report latest", query_summary)
}

fn print_report_latest_json(summary: &ReportLatestSummary) -> Result<i32, CliError> {
    let mut json = report_query_summary_json(ReportQuerySummaryView::from(summary));
    json.insert(
        "selected".to_string(),
        serde_json::json!(summary.selected.as_ref().map(report_list_entry_json)),
    );

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
        .map(report_list_entry_json)
        .collect::<Vec<_>>();
    let mut json = report_query_summary_json(ReportQuerySummaryView::from(summary));
    json.insert("reports".to_string(), serde_json::json!(reports));

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

fn print_report_list_paths(summary: &ReportListSummary) -> i32 {
    for report in &summary.reports {
        if report.status == "ok" {
            println!("{}", report.path.display());
        }
    }

    finish_report_query_paths(ReportQuerySummaryView::from(summary))
}

fn print_report_latest_path(summary: &ReportLatestSummary) -> i32 {
    match latest_report_path(summary) {
        Some(path) => {
            println!("{}", path.display());
            0
        }
        None => finish_report_query_paths(ReportQuerySummaryView::from(summary)),
    }
}

fn print_report_stats_text(summary: &ReportStatsSummary) -> i32 {
    let query_summary = ReportQuerySummaryView::from(summary);
    print_report_query_text_header(query_summary);
    println!("result counts:");
    if summary.result_counts.is_empty() {
        println!("  (none)");
    } else {
        for (result, count) in &summary.result_counts {
            println!("  {result}: {count}");
        }
    }
    println!("validation status counts:");
    if summary.validation_status_counts.is_empty() {
        println!("  (none)");
    } else {
        for (status, count) in &summary.validation_status_counts {
            println!("  {status}: {count}");
        }
    }
    if let Some(latest_finished_at) = &summary.latest_finished_at {
        println!("latest finished at: {latest_finished_at}");
    }
    if let Some(latest_valid_report) = &summary.latest_valid_report {
        println!(
            "latest valid report: {}",
            latest_valid_report.path.display()
        );
        if let Some(run_id) = &latest_valid_report.run_id {
            println!("latest run id: {run_id}");
        }
        if let Some(result) = &latest_valid_report.result {
            println!("latest result: {result}");
        }
        if let Some(validation_status) = &latest_valid_report.validation_status {
            println!("latest validation status: {validation_status}");
        }
    }

    finish_report_query_text("report stats", query_summary)
}

fn print_report_stats_json(summary: &ReportStatsSummary) -> Result<i32, CliError> {
    let mut json = report_query_summary_json(ReportQuerySummaryView::from(summary));
    json.insert(
        "result_counts".to_string(),
        serde_json::json!(summary.result_counts),
    );
    json.insert(
        "validation_status_counts".to_string(),
        serde_json::json!(summary.validation_status_counts),
    );
    json.insert(
        "latest_finished_at".to_string(),
        serde_json::json!(summary.latest_finished_at),
    );
    json.insert(
        "latest_valid_report".to_string(),
        serde_json::json!(summary.latest_valid_report.as_ref().map(|report| {
            serde_json::json!({
                "path": report.path,
                "run_id": report.run_id,
                "finished_at": report.finished_at,
                "result": report.result,
                "validation_status": report.validation_status,
            })
        })),
    );

    render_report_stats_json(json, summary.is_ok())
}

fn print_report_stats_counts_json(summary: &ReportStatsSummary) -> Result<i32, CliError> {
    let mut json = report_query_summary_json(ReportQuerySummaryView::from(summary));
    json.insert(
        "result_counts".to_string(),
        serde_json::json!(summary.result_counts),
    );
    json.insert(
        "validation_status_counts".to_string(),
        serde_json::json!(summary.validation_status_counts),
    );
    render_report_stats_json(json, summary.is_ok())
}

fn render_report_stats_json(
    json: serde_json::Map<String, serde_json::Value>,
    ok: bool,
) -> Result<i32, CliError> {
    match serde_json::to_string_pretty(&json) {
        Ok(json) => {
            println!("{json}");
            if ok {
                Ok(0)
            } else {
                Ok(1)
            }
        }
        Err(source) => Err(CliError::serialization("report statistics summary", source)),
    }
}

fn print_report_stats_latest_path(summary: &ReportStatsSummary) -> i32 {
    match summary.latest_valid_report.as_ref() {
        Some(report) => {
            println!("{}", report.path.display());
            0
        }
        None => {
            if !summary.errors.is_empty() {
                return finish_report_query_paths(ReportQuerySummaryView::from(summary));
            }

            emit_error_line(
                ReportQuery::new(summary.result_filter, summary.validation_status_filter)
                    .describe_missing(),
            );
            1
        }
    }
}

fn print_report_stats_full_latest(summary: &ReportStatsSummary) -> Result<i32, CliError> {
    match summary.latest_valid_report.as_ref() {
        Some(report) => {
            let inspected = inspect_report(report.path.clone());
            match inspected.report.as_ref() {
                Some(report) => print_report_full_json(&inspected.summary, report),
                None => print_report_stats_json(summary),
            }
        }
        None => {
            if !summary.errors.is_empty() {
                return print_report_stats_json(summary);
            }

            let error = ReportQuery::new(summary.result_filter, summary.validation_status_filter)
                .describe_missing();
            let mut json = report_query_summary_json(ReportQuerySummaryView::from(summary));
            json.insert("status".to_string(), serde_json::json!("failed"));
            json.insert(
                "result_counts".to_string(),
                serde_json::json!(summary.result_counts),
            );
            json.insert(
                "validation_status_counts".to_string(),
                serde_json::json!(summary.validation_status_counts),
            );
            json.insert(
                "latest_finished_at".to_string(),
                serde_json::json!(summary.latest_finished_at),
            );
            json.insert(
                "latest_valid_report".to_string(),
                serde_json::json!(serde_json::Value::Null),
            );
            json.insert("errors".to_string(), serde_json::json!([error]));
            render_report_stats_json(json, false)
        }
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
        Some(ObjectSubcommand::Inspect(args)) => {
            if let Some(message) = unexpected_extra(&args.extra, "object inspect") {
                return Err(CliError::usage(message));
            }

            object_inspect(PathBuf::from(args.target), args.json)
        }
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
        Some(ReportSubcommand::Diff(args)) => {
            if let Some(message) = unexpected_extra(&args.extra, "report diff") {
                return Err(CliError::usage(message));
            }

            report_diff(
                PathBuf::from(args.left),
                PathBuf::from(args.right),
                args.json,
                args.events,
            )
        }
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
            report_list(
                PathBuf::from(target),
                args.json,
                args.path_only,
                args.result,
                args.validation_status,
            )
        }
        Some(ReportSubcommand::Latest(args)) => {
            if let Some(message) = unexpected_extra(&args.extra, "report latest") {
                return Err(CliError::usage(message));
            }

            let target = args.target.unwrap_or_else(|| "sim/reports".to_owned());
            report_latest(
                PathBuf::from(target),
                args.json,
                args.full,
                args.path_only,
                args.result,
                args.validation_status,
            )
        }
        Some(ReportSubcommand::Stats(args)) => {
            if let Some(message) = unexpected_extra(&args.extra, "report stats") {
                return Err(CliError::usage(message));
            }

            let target = args.target.unwrap_or_else(|| "sim/reports".to_owned());
            report_stats(
                PathBuf::from(target),
                args.json,
                args.counts_only,
                args.full_latest,
                args.path_only_latest,
                args.result,
                args.validation_status,
            )
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

fn print_report_diff_text(summary: &ReportDiffSummary) -> i32 {
    println!("left report: {}", summary.left.path.display());
    println!("right report: {}", summary.right.path.display());
    println!("comparison: {}", summary.comparison);
    println!("difference count: {}", summary.difference_count);

    for difference in &summary.differences {
        println!(
            "difference {}: left={} right={}",
            difference.field,
            format_report_diff_value(&difference.left),
            format_report_diff_value(&difference.right)
        );
    }

    if summary.status == "failed" {
        println!("report diff: failed");
        for error in &summary.errors {
            emit_error_line(error);
        }
        1
    } else {
        println!("report diff: {}", summary.comparison);
        0
    }
}

fn print_report_event_diff_text(summary: &ReportEventDiffSummary) -> i32 {
    println!("left report: {}", summary.left.path.display());
    println!("right report: {}", summary.right.path.display());
    println!("comparison: {}", summary.comparison);
    println!("event difference count: {}", summary.event_difference_count);

    for difference in &summary.event_differences {
        println!("event step {}: {}", difference.step, difference.change);
        if let Some(left) = &difference.left {
            println!("  left: {}", format_report_event_text(left));
        }
        if let Some(right) = &difference.right {
            println!("  right: {}", format_report_event_text(right));
        }
    }

    if summary.status == "failed" {
        println!("report diff: failed");
        for error in &summary.errors {
            emit_error_line(error);
        }
        1
    } else {
        println!("report diff: {}", summary.comparison);
        0
    }
}

fn print_report_diff_json(summary: &ReportDiffSummary) -> Result<i32, CliError> {
    match serde_json::to_string_pretty(summary) {
        Ok(json) => {
            println!("{json}");
            if summary.status == "failed" {
                Ok(1)
            } else {
                Ok(0)
            }
        }
        Err(source) => Err(CliError::serialization("report diff summary", source)),
    }
}

fn print_report_event_diff_json(summary: &ReportEventDiffSummary) -> Result<i32, CliError> {
    match serde_json::to_string_pretty(summary) {
        Ok(json) => {
            println!("{json}");
            if summary.status == "failed" {
                Ok(1)
            } else {
                Ok(0)
            }
        }
        Err(source) => Err(CliError::serialization("report event diff summary", source)),
    }
}

fn report_diff(left: PathBuf, right: PathBuf, json: bool, events: bool) -> Result<i32, CliError> {
    if events {
        let summary = diff_report_events(left, right);
        if json {
            print_report_event_diff_json(&summary)
        } else {
            Ok(print_report_event_diff_text(&summary))
        }
    } else {
        let summary = diff_reports(left, right);
        if json {
            print_report_diff_json(&summary)
        } else {
            Ok(print_report_diff_text(&summary))
        }
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

fn report_list(
    target: PathBuf,
    json: bool,
    path_only: bool,
    result_filter: Option<ReportResultFilter>,
    validation_status_filter: Option<ReportValidationStatusFilter>,
) -> Result<i32, CliError> {
    let summary = query_reports(
        target,
        ReportQuery::new(result_filter, validation_status_filter),
    );
    if path_only {
        Ok(print_report_list_paths(&summary))
    } else if json {
        print_report_list_json(&summary)
    } else {
        Ok(print_report_list_text(&summary))
    }
}

fn report_latest(
    target: PathBuf,
    json: bool,
    full: bool,
    path_only: bool,
    result_filter: Option<ReportResultFilter>,
    validation_status_filter: Option<ReportValidationStatusFilter>,
) -> Result<i32, CliError> {
    let summary = latest_report(query_reports(
        target,
        ReportQuery::new(result_filter, validation_status_filter),
    ));
    if path_only {
        return Ok(print_report_latest_path(&summary));
    }

    if full {
        if !json {
            return Err(CliError::usage("report latest --full requires --json"));
        }

        return match latest_report_path(&summary) {
            Some(path) => {
                let inspected = inspect_report(path);
                match inspected.report.as_ref() {
                    Some(report) => print_report_full_json(&inspected.summary, report),
                    None => print_report_latest_json(&summary),
                }
            }
            None => print_report_latest_json(&summary),
        };
    }

    if json {
        print_report_latest_json(&summary)
    } else {
        Ok(print_report_latest_text(&summary))
    }
}

fn report_stats(
    target: PathBuf,
    json: bool,
    counts_only: bool,
    full_latest: bool,
    path_only_latest: bool,
    result_filter: Option<ReportResultFilter>,
    validation_status_filter: Option<ReportValidationStatusFilter>,
) -> Result<i32, CliError> {
    let summary = summarize_reports(query_reports(
        target,
        ReportQuery::new(result_filter, validation_status_filter),
    ));
    if path_only_latest {
        Ok(print_report_stats_latest_path(&summary))
    } else if counts_only {
        print_report_stats_counts_json(&summary)
    } else if full_latest {
        print_report_stats_full_latest(&summary)
    } else if json {
        print_report_stats_json(&summary)
    } else {
        Ok(print_report_stats_text(&summary))
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
