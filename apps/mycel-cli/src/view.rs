use clap::{Args, Subcommand, ValueEnum};

use crate::CliError;

mod current;
mod inspect;
mod list;
mod publish;
mod shared;

#[derive(Args)]
pub(crate) struct ViewCliArgs {
    #[command(subcommand)]
    command: Option<ViewSubcommand>,
}

#[derive(Subcommand)]
enum ViewSubcommand {
    #[command(about = "Inspect the current persisted governance state for one profile")]
    Current(ViewCurrentCliArgs),
    #[command(about = "Inspect one persisted governance View object")]
    Inspect(ViewInspectCliArgs),
    #[command(about = "List persisted governance View records with optional filters")]
    List(ViewListCliArgs),
    #[command(about = "Verify and publish one governance View object into the store")]
    Publish(ViewPublishCliArgs),
    #[command(external_subcommand)]
    External(Vec<String>),
}

#[derive(Args)]
struct ViewCurrentCliArgs {
    #[arg(
        long,
        value_name = "STORE_ROOT",
        help = "Store root directory to read governance indexes from",
        required = true
    )]
    store_root: String,
    #[arg(long, help = "Governance profile ID to inspect", required = true)]
    profile_id: String,
    #[arg(
        long,
        help = "Only inspect the current governance state for one document ID"
    )]
    doc_id: Option<String>,
    #[arg(long, help = "Emit machine-readable current-governance output")]
    json: bool,
    #[arg(hide = true, allow_hyphen_values = true)]
    extra: Vec<String>,
}

#[derive(Args)]
struct ViewInspectCliArgs {
    #[arg(
        value_name = "VIEW_ID",
        help = "View identifier to inspect from the store",
        required = true,
        allow_hyphen_values = true
    )]
    view_id: String,
    #[arg(
        long,
        value_name = "STORE_ROOT",
        help = "Store root directory to read governance indexes from",
        required = true
    )]
    store_root: String,
    #[arg(long, help = "Emit machine-readable view inspection output")]
    json: bool,
    #[arg(hide = true, allow_hyphen_values = true)]
    extra: Vec<String>,
}

#[derive(Args)]
struct ViewPublishCliArgs {
    #[arg(
        value_name = "PATH",
        help = "View object file to publish",
        required = true,
        allow_hyphen_values = true
    )]
    source: String,
    #[arg(
        long = "into",
        value_name = "STORE_ROOT",
        help = "Store root directory to publish into",
        required = true
    )]
    into: String,
    #[arg(long, help = "Emit machine-readable view publish output")]
    json: bool,
    #[arg(hide = true, allow_hyphen_values = true)]
    extra: Vec<String>,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, serde::Serialize, ValueEnum)]
#[serde(rename_all = "kebab-case")]
enum ViewListSort {
    ViewId,
    TimestampAsc,
    TimestampDesc,
    ProfileId,
    Maintainer,
}

impl ViewListSort {
    fn as_str(self) -> &'static str {
        match self {
            Self::ViewId => "view-id",
            Self::TimestampAsc => "timestamp-asc",
            Self::TimestampDesc => "timestamp-desc",
            Self::ProfileId => "profile-id",
            Self::Maintainer => "maintainer",
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, serde::Serialize, ValueEnum)]
#[serde(rename_all = "kebab-case")]
enum ViewListGroupBy {
    ProfileId,
    Maintainer,
    DocId,
}

impl ViewListGroupBy {
    fn as_str(self) -> &'static str {
        match self {
            Self::ProfileId => "profile-id",
            Self::Maintainer => "maintainer",
            Self::DocId => "doc-id",
        }
    }
}

#[derive(Args)]
struct ViewListCliArgs {
    #[arg(
        long,
        value_name = "STORE_ROOT",
        help = "Store root directory to read governance indexes from",
        required = true
    )]
    store_root: String,
    #[arg(long, help = "Only return one persisted view ID")]
    view_id: Option<String>,
    #[arg(long, help = "Only return one governance profile ID")]
    profile_id: Option<String>,
    #[arg(long, help = "Only return one governance maintainer key")]
    maintainer: Option<String>,
    #[arg(long, help = "Only return views that mention one document ID")]
    doc_id: Option<String>,
    #[arg(long, help = "Only return views that mention one revision ID")]
    revision_id: Option<String>,
    #[arg(
        long,
        value_name = "TIMESTAMP",
        help = "Only return views at or after one timestamp"
    )]
    timestamp_min: Option<u64>,
    #[arg(
        long,
        value_name = "TIMESTAMP",
        help = "Only return views at or before one timestamp"
    )]
    timestamp_max: Option<u64>,
    #[arg(
        long,
        value_enum,
        default_value_t = ViewListSort::TimestampDesc,
        help = "Sort listed governance records"
    )]
    sort: ViewListSort,
    #[arg(
        long,
        value_name = "COUNT",
        help = "Return at most this many records after projection"
    )]
    limit: Option<usize>,
    #[arg(
        long,
        help = "Keep only the latest governance record for each profile ID"
    )]
    latest_per_profile: bool,
    #[arg(long, help = "Omit per-record output and emit only summary metadata")]
    summary_only: bool,
    #[arg(
        long,
        value_name = "GROUP_BY",
        value_enum,
        help = "Emit grouped summaries by one governance field"
    )]
    group_by: Vec<ViewListGroupBy>,
    #[arg(long, help = "Emit machine-readable view listing output")]
    json: bool,
    #[arg(hide = true, allow_hyphen_values = true)]
    extra: Vec<String>,
}

#[derive(Debug, Clone, serde::Serialize)]
struct ViewListFilters {
    view_id: Option<String>,
    profile_id: Option<String>,
    maintainer: Option<String>,
    doc_id: Option<String>,
    revision_id: Option<String>,
    timestamp_min: Option<u64>,
    timestamp_max: Option<u64>,
}

fn unexpected_extra(extra: &[String], context: &str) -> Option<String> {
    extra
        .first()
        .map(|arg| format!("unexpected {context} argument: {arg}"))
}

pub(crate) fn handle_view_command(command: ViewCliArgs) -> Result<i32, CliError> {
    match command.command {
        Some(ViewSubcommand::Current(args)) => {
            if let Some(message) = unexpected_extra(&args.extra, "view current") {
                return Err(CliError::usage(message));
            }
            current::handle(args)
        }
        Some(ViewSubcommand::Inspect(args)) => {
            if let Some(message) = unexpected_extra(&args.extra, "view inspect") {
                return Err(CliError::usage(message));
            }
            inspect::handle(args)
        }
        Some(ViewSubcommand::List(args)) => {
            if let Some(message) = unexpected_extra(&args.extra, "view list") {
                return Err(CliError::usage(message));
            }
            list::handle(args)
        }
        Some(ViewSubcommand::Publish(args)) => {
            if let Some(message) = unexpected_extra(&args.extra, "view publish") {
                return Err(CliError::usage(message));
            }
            publish::handle(args)
        }
        Some(ViewSubcommand::External(args)) => {
            let other = args.first().map(String::as_str).unwrap_or("<unknown>");
            Err(CliError::usage(format!("unknown view subcommand: {other}")))
        }
        None => Err(CliError::usage("missing view subcommand")),
    }
}
