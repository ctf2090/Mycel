use std::path::PathBuf;

use clap::{Args, Subcommand};
use mycel_core::store::{
    ingest_store_from_path, load_store_index_manifest, rebuild_store_from_path, StoreIndexManifest,
    StoreIngestSummary, StoreRebuildSummary, ViewGovernanceRecord,
};
use serde::Serialize;

use crate::{emit_error_line, CliError};

#[derive(Args)]
pub(crate) struct StoreCliArgs {
    #[command(subcommand)]
    command: Option<StoreSubcommand>,
}

#[derive(Subcommand)]
enum StoreSubcommand {
    #[command(about = "Verify and ingest objects into a local object store")]
    Ingest(StoreIngestCliArgs),
    #[command(about = "Query persisted local object-store indexes")]
    Index(index::StoreIndexCliArgs),
    #[command(about = "Rebuild local object-store indexes from stored objects")]
    Rebuild(StoreRebuildCliArgs),
    #[command(external_subcommand)]
    External(Vec<String>),
}

#[derive(Args)]
struct StoreRebuildCliArgs {
    #[arg(
        value_name = "PATH",
        help = "Object-store directory or one object file to rebuild from",
        required = true,
        allow_hyphen_values = true
    )]
    target: String,
    #[arg(long, help = "Emit machine-readable store-rebuild output")]
    json: bool,
    #[arg(hide = true, allow_hyphen_values = true)]
    extra: Vec<String>,
}

#[derive(Args)]
struct StoreIngestCliArgs {
    #[arg(
        value_name = "SOURCE",
        help = "Object file or directory to ingest from",
        required = true,
        allow_hyphen_values = true
    )]
    source: String,
    #[arg(
        long = "into",
        value_name = "STORE_ROOT",
        help = "Store root directory to write into",
        required = true
    )]
    into: String,
    #[arg(long, help = "Emit machine-readable store-ingest output")]
    json: bool,
    #[arg(hide = true, allow_hyphen_values = true)]
    extra: Vec<String>,
}

#[path = "store/index.rs"]
mod index;

fn unexpected_extra(extra: &[String], context: &str) -> Option<String> {
    extra
        .first()
        .map(|arg| format!("unexpected {context} argument: {arg}"))
}

pub(crate) fn handle_store_command(command: StoreCliArgs) -> Result<i32, CliError> {
    match command.command {
        Some(StoreSubcommand::Ingest(args)) => {
            if let Some(message) = unexpected_extra(&args.extra, "store ingest") {
                return Err(CliError::usage(message));
            }

            index::store_ingest(
                PathBuf::from(args.source),
                PathBuf::from(args.into),
                args.json,
            )
        }
        Some(StoreSubcommand::Index(args)) => {
            if let Some(message) = unexpected_extra(&args.extra, "store index") {
                return Err(CliError::usage(message));
            }

            index::store_index(args)
        }
        Some(StoreSubcommand::Rebuild(args)) => {
            if let Some(message) = unexpected_extra(&args.extra, "store rebuild") {
                return Err(CliError::usage(message));
            }

            index::store_rebuild(PathBuf::from(args.target), args.json)
        }
        Some(StoreSubcommand::External(args)) => {
            let other = args.first().map(String::as_str).unwrap_or("<unknown>");
            Err(CliError::usage(format!(
                "unknown store subcommand: {other}"
            )))
        }
        None => Err(CliError::usage("missing store subcommand")),
    }
}
