use std::collections::BTreeMap;
use std::path::{Path, PathBuf};

use clap::{Args, Subcommand, ValueEnum};
use mycel_sim::model::{Report, ReportEvent, ReportFailure};
use serde::Serialize;

use crate::CliError;

mod diff;
mod query;
mod render;

use self::{diff::*, query::*, render::*};

fn emit_error_line(message: impl std::fmt::Display) {
    eprintln!("error: {message}");
}

#[derive(Args)]
pub(crate) struct ReportCliArgs {
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
    #[arg(
        long = "field",
        value_enum,
        value_name = "FIELD",
        help = "Compare only one diff field; repeat to select multiple fields",
        conflicts_with = "ignore_fields"
    )]
    fields: Vec<ReportDiffIgnoreField>,
    #[arg(
        long = "ignore-field",
        value_enum,
        value_name = "FIELD",
        help = "Ignore one diff field; repeat to ignore multiple fields",
        conflicts_with = "fields"
    )]
    ignore_fields: Vec<ReportDiffIgnoreField>,
    #[arg(
        long,
        help = "Exit with failure when the compared reports are different"
    )]
    fail_on_diff: bool,
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

#[derive(Clone, Copy, Debug, PartialEq, Eq, ValueEnum)]
enum ReportDiffIgnoreField {
    RunId,
    TopologyId,
    FixtureId,
    TestId,
    ExecutionMode,
    StartedAt,
    FinishedAt,
    ValidationStatus,
    DeterministicSeed,
    SeedSource,
    Result,
    PeerCount,
    EventCount,
    FailureCount,
    VerifiedObjectCount,
    RejectedObjectCount,
    MatchedExpectedOutcomes,
    ScheduledPeerOrder,
    FaultPlanCount,
    EventPhase,
    EventAction,
    EventOutcome,
    EventNodeId,
    EventObjectIds,
    EventDetail,
}

impl ReportDiffIgnoreField {
    fn as_str(self) -> &'static str {
        match self {
            Self::RunId => "run-id",
            Self::TopologyId => "topology-id",
            Self::FixtureId => "fixture-id",
            Self::TestId => "test-id",
            Self::ExecutionMode => "execution-mode",
            Self::StartedAt => "started-at",
            Self::FinishedAt => "finished-at",
            Self::ValidationStatus => "validation-status",
            Self::DeterministicSeed => "deterministic-seed",
            Self::SeedSource => "seed-source",
            Self::Result => "result",
            Self::PeerCount => "peer-count",
            Self::EventCount => "event-count",
            Self::FailureCount => "failure-count",
            Self::VerifiedObjectCount => "verified-object-count",
            Self::RejectedObjectCount => "rejected-object-count",
            Self::MatchedExpectedOutcomes => "matched-expected-outcomes",
            Self::ScheduledPeerOrder => "scheduled-peer-order",
            Self::FaultPlanCount => "fault-plan-count",
            Self::EventPhase => "event-phase",
            Self::EventAction => "event-action",
            Self::EventOutcome => "event-outcome",
            Self::EventNodeId => "event-node-id",
            Self::EventObjectIds => "event-object-ids",
            Self::EventDetail => "event-detail",
        }
    }
}

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
    selected_fields: Vec<String>,
    ignored_fields: Vec<String>,
    left: ReportDiffSideSummary,
    right: ReportDiffSideSummary,
    differences: Vec<ReportDiffEntry>,
    errors: Vec<String>,
}

#[derive(Debug, Clone, Serialize)]
struct ReportEventTraceIdentity {
    phase: Option<String>,
    action: Option<String>,
    node_id: Option<String>,
    object_ids: Vec<String>,
    occurrence: usize,
}

#[derive(Debug, Clone, Serialize)]
struct ReportEventDiffEntry {
    step: u64,
    left_step: Option<u64>,
    right_step: Option<u64>,
    trace_identity: ReportEventTraceIdentity,
    change: String,
    left: Option<ReportEvent>,
    right: Option<ReportEvent>,
}

#[derive(Debug, Clone, Serialize)]
struct ReportEventDiffSummary {
    status: String,
    comparison: String,
    event_difference_count: usize,
    selected_fields: Vec<String>,
    ignored_fields: Vec<String>,
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

pub(crate) fn handle_report_command(command: ReportCliArgs) -> Result<i32, CliError> {
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
                &args.fields,
                &args.ignore_fields,
                args.fail_on_diff,
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

fn report_diff(
    left: PathBuf,
    right: PathBuf,
    json: bool,
    events: bool,
    fields: &[ReportDiffIgnoreField],
    ignore_fields: &[ReportDiffIgnoreField],
    fail_on_diff: bool,
) -> Result<i32, CliError> {
    if events {
        let summary = diff_report_events(left, right, fields, ignore_fields);
        if json {
            print_report_event_diff_json(&summary, fail_on_diff)
        } else {
            Ok(print_report_event_diff_text(&summary, fail_on_diff))
        }
    } else {
        let summary = diff_reports(left, right, fields, ignore_fields);
        if json {
            print_report_diff_json(&summary, fail_on_diff)
        } else {
            Ok(print_report_diff_text(&summary, fail_on_diff))
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
