use std::env;
use std::path::PathBuf;

use clap::{Args, CommandFactory, Parser, Subcommand};
use mycel_core::workspace_banner;
use mycel_sim::manifest::SimulatorPaths;
use mycel_sim::run::{run_test_case_with_options, RunOptions};
use mycel_sim::simulator_banner;
use mycel_sim::validate::validate_path;
use thiserror::Error;

mod head;
mod object;
mod report;
mod store;
mod view;
use head::HeadCliArgs;
use object::ObjectCliArgs;
use report::ReportCliArgs;
use store::StoreCliArgs;
use view::ViewCliArgs;

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

pub(crate) fn emit_error_line(message: impl std::fmt::Display) {
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
    #[command(about = "Rebuild local object-store indexes from stored objects")]
    Store(StoreCliArgs),
    #[command(about = "Validate the repo root, one file, or one supported directory")]
    Validate(ValidateCliArgs),
    #[command(about = "Publish and inspect governance View objects")]
    View(ViewCliArgs),
    #[command(external_subcommand)]
    External(Vec<String>),
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
        Some(CliCommand::Head(command)) => head::handle_head_command(command),
        Some(CliCommand::Info) => {
            print_info();
            Ok(0)
        }
        Some(CliCommand::Object(command)) => object::handle_object_command(command),
        Some(CliCommand::Report(command)) => report::handle_report_command(command),
        Some(CliCommand::Sim(command)) => handle_sim_command(command),
        Some(CliCommand::Store(command)) => store::handle_store_command(command),
        Some(CliCommand::Validate(command)) => handle_validate_command(command),
        Some(CliCommand::View(command)) => view::handle_view_command(command),
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
