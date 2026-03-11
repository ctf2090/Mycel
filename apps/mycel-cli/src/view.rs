use std::collections::BTreeMap;
use std::fs;
use std::path::{Path, PathBuf};

use clap::{Args, Subcommand};
use mycel_core::protocol::{parse_json_strict, parse_view_object};
use mycel_core::store::{
    load_store_index_manifest, load_stored_object_value, write_object_value_to_store,
    ViewGovernanceRecord,
};
use mycel_core::verify::verify_object_path;
use serde::Serialize;
use serde_json::Value;

use crate::{emit_error_line, CliError};

#[derive(Args)]
pub(crate) struct ViewCliArgs {
    #[command(subcommand)]
    command: Option<ViewSubcommand>,
}

#[derive(Subcommand)]
enum ViewSubcommand {
    #[command(about = "Inspect one persisted governance View object")]
    Inspect(ViewInspectCliArgs),
    #[command(about = "Verify and publish one governance View object into the store")]
    Publish(ViewPublishCliArgs),
    #[command(external_subcommand)]
    External(Vec<String>),
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

#[derive(Debug, Clone, Serialize)]
struct ViewInspectSummary {
    store_root: PathBuf,
    manifest_path: PathBuf,
    status: String,
    view_id: String,
    maintainer: Option<String>,
    profile_id: Option<String>,
    timestamp: Option<u64>,
    documents: BTreeMap<String, String>,
    profile_heads: BTreeMap<String, Vec<String>>,
    notes: Vec<String>,
    errors: Vec<String>,
}

#[derive(Debug, Clone, Serialize)]
struct ViewPublishSummary {
    source_path: PathBuf,
    store_root: PathBuf,
    status: String,
    view_id: Option<String>,
    maintainer: Option<String>,
    profile_id: Option<String>,
    documents: BTreeMap<String, String>,
    created: bool,
    stored_path: Option<PathBuf>,
    index_manifest_path: Option<PathBuf>,
    notes: Vec<String>,
    errors: Vec<String>,
}

impl ViewInspectSummary {
    fn new(store_root: &Path, view_id: &str) -> Self {
        Self {
            store_root: store_root.to_path_buf(),
            manifest_path: store_root.join("indexes").join("manifest.json"),
            status: "ok".to_string(),
            view_id: view_id.to_string(),
            maintainer: None,
            profile_id: None,
            timestamp: None,
            documents: BTreeMap::new(),
            profile_heads: BTreeMap::new(),
            notes: Vec::new(),
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

impl ViewPublishSummary {
    fn new(source_path: &Path, store_root: &Path) -> Self {
        Self {
            source_path: source_path.to_path_buf(),
            store_root: store_root.to_path_buf(),
            status: "ok".to_string(),
            view_id: None,
            maintainer: None,
            profile_id: None,
            documents: BTreeMap::new(),
            created: false,
            stored_path: None,
            index_manifest_path: None,
            notes: Vec::new(),
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

fn find_view_record(
    manifest: &mycel_core::store::StoreIndexManifest,
    view_id: &str,
) -> Option<ViewGovernanceRecord> {
    manifest
        .view_governance
        .iter()
        .find(|record| record.view_id == view_id)
        .cloned()
}

fn load_source_value(source_path: &Path) -> Result<Value, String> {
    let content = fs::read_to_string(source_path).map_err(|error| {
        format!(
            "failed to read view source '{}': {error}",
            source_path.display()
        )
    })?;
    parse_json_strict(&content).map_err(|error| {
        format!(
            "failed to parse view source JSON '{}': {error}",
            source_path.display()
        )
    })
}

fn print_view_inspect_text(summary: &ViewInspectSummary) -> i32 {
    println!("store root: {}", summary.store_root.display());
    println!("manifest path: {}", summary.manifest_path.display());
    println!("view id: {}", summary.view_id);
    if let Some(maintainer) = &summary.maintainer {
        println!("maintainer: {maintainer}");
    }
    if let Some(profile_id) = &summary.profile_id {
        println!("profile id: {profile_id}");
    }
    if let Some(timestamp) = summary.timestamp {
        println!("timestamp: {timestamp}");
    }
    println!("document count: {}", summary.documents.len());
    for (doc_id, revision_id) in &summary.documents {
        println!("document: {doc_id} -> {revision_id}");
    }
    println!("profile head doc count: {}", summary.profile_heads.len());
    for (doc_id, revision_ids) in &summary.profile_heads {
        println!("profile heads: {doc_id} -> {}", revision_ids.join(", "));
    }
    for note in &summary.notes {
        println!("note: {note}");
    }

    if summary.is_ok() {
        println!("view inspection: ok");
        0
    } else {
        println!("view inspection: failed");
        for error in &summary.errors {
            emit_error_line(error);
        }
        1
    }
}

fn print_view_inspect_json(summary: &ViewInspectSummary) -> Result<i32, CliError> {
    match serde_json::to_string_pretty(summary) {
        Ok(json) => {
            println!("{json}");
            if summary.is_ok() {
                Ok(0)
            } else {
                Ok(1)
            }
        }
        Err(source) => Err(CliError::serialization("view inspection summary", source)),
    }
}

fn print_view_publish_text(summary: &ViewPublishSummary) -> i32 {
    println!("source path: {}", summary.source_path.display());
    println!("store root: {}", summary.store_root.display());
    if let Some(view_id) = &summary.view_id {
        println!("view id: {view_id}");
    }
    if let Some(maintainer) = &summary.maintainer {
        println!("maintainer: {maintainer}");
    }
    if let Some(profile_id) = &summary.profile_id {
        println!("profile id: {profile_id}");
    }
    println!("document count: {}", summary.documents.len());
    for (doc_id, revision_id) in &summary.documents {
        println!("document: {doc_id} -> {revision_id}");
    }
    println!("created: {}", if summary.created { "yes" } else { "no" });
    if let Some(stored_path) = &summary.stored_path {
        println!("stored path: {}", stored_path.display());
    }
    if let Some(index_manifest_path) = &summary.index_manifest_path {
        println!("index manifest: {}", index_manifest_path.display());
    }
    for note in &summary.notes {
        println!("note: {note}");
    }

    if summary.is_ok() {
        println!("view publish: ok");
        0
    } else {
        println!("view publish: failed");
        for error in &summary.errors {
            emit_error_line(error);
        }
        1
    }
}

fn print_view_publish_json(summary: &ViewPublishSummary) -> Result<i32, CliError> {
    match serde_json::to_string_pretty(summary) {
        Ok(json) => {
            println!("{json}");
            if summary.is_ok() {
                Ok(0)
            } else {
                Ok(1)
            }
        }
        Err(source) => Err(CliError::serialization("view publish summary", source)),
    }
}

fn view_inspect(store_root: PathBuf, view_id: String, json: bool) -> Result<i32, CliError> {
    let mut summary = ViewInspectSummary::new(&store_root, &view_id);
    let manifest = match load_store_index_manifest(&store_root) {
        Ok(manifest) => manifest,
        Err(error) => {
            summary.push_error(format!("failed to read store index manifest: {error}"));
            return if json {
                print_view_inspect_json(&summary)
            } else {
                Ok(print_view_inspect_text(&summary))
            };
        }
    };
    let Some(record) = find_view_record(&manifest, &view_id) else {
        summary.push_error(format!(
            "view '{}' was not found in persisted governance indexes",
            view_id
        ));
        return if json {
            print_view_inspect_json(&summary)
        } else {
            Ok(print_view_inspect_text(&summary))
        };
    };

    let value = match load_stored_object_value(&store_root, &view_id) {
        Ok(value) => value,
        Err(error) => {
            summary.push_error(format!("failed to load stored view '{}': {error}", view_id));
            return if json {
                print_view_inspect_json(&summary)
            } else {
                Ok(print_view_inspect_text(&summary))
            };
        }
    };
    let view = match parse_view_object(&value) {
        Ok(view) => view,
        Err(error) => {
            summary.push_error(format!(
                "failed to parse stored view '{}': {error}",
                view_id
            ));
            return if json {
                print_view_inspect_json(&summary)
            } else {
                Ok(print_view_inspect_text(&summary))
            };
        }
    };

    summary.maintainer = Some(record.maintainer.clone());
    summary.profile_id = Some(record.profile_id.clone());
    summary.timestamp = Some(view.timestamp);
    summary.documents = record.documents.clone();
    summary.profile_heads = manifest
        .profile_heads
        .get(&record.profile_id)
        .cloned()
        .unwrap_or_default();
    summary.notes.push(
        "governance inspection is separate from reader-facing accepted-head workflows".to_string(),
    );

    if json {
        print_view_inspect_json(&summary)
    } else {
        Ok(print_view_inspect_text(&summary))
    }
}

fn view_publish(source_path: PathBuf, store_root: PathBuf, json: bool) -> Result<i32, CliError> {
    let mut summary = ViewPublishSummary::new(&source_path, &store_root);
    let verification = verify_object_path(&source_path);
    if !verification.is_ok() {
        summary.push_error(format!(
            "view publish source failed verification: {}",
            verification.errors.join("; ")
        ));
        return if json {
            print_view_publish_json(&summary)
        } else {
            Ok(print_view_publish_text(&summary))
        };
    }

    let value = match load_source_value(&source_path) {
        Ok(value) => value,
        Err(error) => {
            summary.push_error(error);
            return if json {
                print_view_publish_json(&summary)
            } else {
                Ok(print_view_publish_text(&summary))
            };
        }
    };
    let view = match parse_view_object(&value) {
        Ok(view) => view,
        Err(error) => {
            summary.push_error(format!(
                "view publish source is not a valid view object: {error}"
            ));
            return if json {
                print_view_publish_json(&summary)
            } else {
                Ok(print_view_publish_text(&summary))
            };
        }
    };

    let write = match write_object_value_to_store(&store_root, &value) {
        Ok(write) => write,
        Err(error) => {
            summary.push_error(format!("failed to publish view into store: {error}"));
            return if json {
                print_view_publish_json(&summary)
            } else {
                Ok(print_view_publish_text(&summary))
            };
        }
    };
    let manifest = match load_store_index_manifest(&store_root) {
        Ok(manifest) => manifest,
        Err(error) => {
            summary.push_error(format!("failed to reload store index manifest: {error}"));
            return if json {
                print_view_publish_json(&summary)
            } else {
                Ok(print_view_publish_text(&summary))
            };
        }
    };
    let Some(record) = find_view_record(&manifest, &view.view_id) else {
        summary.push_error(format!(
            "published view '{}' is missing from persisted governance indexes",
            view.view_id
        ));
        return if json {
            print_view_publish_json(&summary)
        } else {
            Ok(print_view_publish_text(&summary))
        };
    };

    summary.view_id = Some(view.view_id);
    summary.maintainer = Some(view.maintainer);
    summary.profile_id = Some(record.profile_id);
    summary.documents = record.documents;
    summary.created = write.created;
    summary.stored_path = Some(write.record.path);
    summary.index_manifest_path = write.index_manifest_path;
    summary.notes.push(
        "published governance state is separate from revision publication and accepted-head inspection".to_string(),
    );

    if json {
        print_view_publish_json(&summary)
    } else {
        Ok(print_view_publish_text(&summary))
    }
}

fn unexpected_extra(extra: &[String], context: &str) -> Option<String> {
    extra
        .first()
        .map(|arg| format!("unexpected {context} argument: {arg}"))
}

pub(crate) fn handle_view_command(command: ViewCliArgs) -> Result<i32, CliError> {
    match command.command {
        Some(ViewSubcommand::Inspect(args)) => {
            if let Some(message) = unexpected_extra(&args.extra, "view inspect") {
                return Err(CliError::usage(message));
            }

            view_inspect(PathBuf::from(args.store_root), args.view_id, args.json)
        }
        Some(ViewSubcommand::Publish(args)) => {
            if let Some(message) = unexpected_extra(&args.extra, "view publish") {
                return Err(CliError::usage(message));
            }

            view_publish(
                PathBuf::from(args.source),
                PathBuf::from(args.into),
                args.json,
            )
        }
        Some(ViewSubcommand::External(args)) => {
            let other = args.first().map(String::as_str).unwrap_or("<unknown>");
            Err(CliError::usage(format!("unknown view subcommand: {other}")))
        }
        None => Err(CliError::usage("missing view subcommand")),
    }
}
