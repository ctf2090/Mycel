use std::env;
use std::path::PathBuf;

use clap::{Args, CommandFactory, Parser, Subcommand};
use mycel_core::head::inspect_heads_from_path;
use mycel_core::head::HeadInspectSummary;
use mycel_core::verify::{
    inspect_object_path, verify_object_path, ObjectInspectionSummary, ObjectVerificationSummary,
};
use mycel_core::workspace_banner;
use mycel_sim::manifest::SimulatorPaths;
use mycel_sim::run::{run_test_case_with_options, RunOptions};
use mycel_sim::simulator_banner;
use mycel_sim::validate::validate_path;
use thiserror::Error;

mod report;
use report::ReportCliArgs;

#[derive(Debug, Error)]
pub(crate) enum CliError {
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
    pub(crate) fn usage(message: impl Into<String>) -> Self {
        Self::Usage(message.into())
    }

    pub(crate) fn serialization(context: &'static str, source: serde_json::Error) -> Self {
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
        Some(CliCommand::Report(command)) => report::handle_report_command(command),
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
