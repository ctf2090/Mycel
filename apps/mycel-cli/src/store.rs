use std::path::PathBuf;

use clap::{Args, Subcommand};
use mycel_core::store::{
    ingest_store_from_path, rebuild_store_from_path, StoreIngestSummary, StoreRebuildSummary,
};

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

fn print_store_rebuild_text(summary: &StoreRebuildSummary) -> i32 {
    println!("store target: {}", summary.target.display());
    println!("status: {}", summary.status);
    println!("discovered files: {}", summary.discovered_file_count);
    println!("identified objects: {}", summary.identified_object_count);
    println!("verified objects: {}", summary.verified_object_count);
    println!("stored objects: {}", summary.stored_object_count);
    println!("document revision indexes: {}", summary.doc_revisions.len());
    println!(
        "revision parent indexes: {}",
        summary.revision_parents.len()
    );
    println!("author patch indexes: {}", summary.author_patches.len());
    println!("view governance records: {}", summary.view_governance.len());
    println!("profile head indexes: {}", summary.profile_heads.len());

    for note in &summary.notes {
        println!("note: {note}");
    }

    if summary.is_ok() {
        println!("store rebuild: {}", summary.status);
        0
    } else {
        println!("store rebuild: failed");
        for error in &summary.errors {
            emit_error_line(error);
        }
        1
    }
}

fn print_store_rebuild_json(summary: &StoreRebuildSummary) -> Result<i32, CliError> {
    match serde_json::to_string_pretty(summary) {
        Ok(json) => {
            println!("{json}");
            if summary.is_ok() {
                Ok(0)
            } else {
                Ok(1)
            }
        }
        Err(source) => Err(CliError::serialization("store rebuild summary", source)),
    }
}

fn print_store_ingest_text(summary: &StoreIngestSummary) -> i32 {
    println!("source: {}", summary.source.display());
    println!("store root: {}", summary.store_root.display());
    println!("status: {}", summary.status);
    println!("discovered files: {}", summary.discovered_file_count);
    println!("identified objects: {}", summary.identified_object_count);
    println!("verified objects: {}", summary.verified_object_count);
    println!("written objects: {}", summary.written_object_count);
    println!("existing objects: {}", summary.existing_object_count);
    println!("skipped objects: {}", summary.skipped_object_count);

    for note in &summary.notes {
        println!("note: {note}");
    }

    if summary.is_ok() {
        println!("store ingest: {}", summary.status);
        0
    } else {
        println!("store ingest: failed");
        for error in &summary.errors {
            emit_error_line(error);
        }
        1
    }
}

fn print_store_ingest_json(summary: &StoreIngestSummary) -> Result<i32, CliError> {
    match serde_json::to_string_pretty(summary) {
        Ok(json) => {
            println!("{json}");
            if summary.is_ok() {
                Ok(0)
            } else {
                Ok(1)
            }
        }
        Err(source) => Err(CliError::serialization("store ingest summary", source)),
    }
}

fn store_rebuild(target: PathBuf, json: bool) -> Result<i32, CliError> {
    match rebuild_store_from_path(&target) {
        Ok(summary) => {
            if json {
                print_store_rebuild_json(&summary)
            } else {
                Ok(print_store_rebuild_text(&summary))
            }
        }
        Err(error) => Err(CliError::usage(error.to_string())),
    }
}

fn store_ingest(source: PathBuf, store_root: PathBuf, json: bool) -> Result<i32, CliError> {
    match ingest_store_from_path(&source, &store_root) {
        Ok(summary) => {
            if json {
                print_store_ingest_json(&summary)
            } else {
                Ok(print_store_ingest_text(&summary))
            }
        }
        Err(error) => Err(CliError::usage(error.to_string())),
    }
}

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

            store_ingest(
                PathBuf::from(args.source),
                PathBuf::from(args.into),
                args.json,
            )
        }
        Some(StoreSubcommand::Rebuild(args)) => {
            if let Some(message) = unexpected_extra(&args.extra, "store rebuild") {
                return Err(CliError::usage(message));
            }

            store_rebuild(PathBuf::from(args.target), args.json)
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
