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
    Index(StoreIndexCliArgs),
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

#[derive(Args)]
struct StoreIndexCliArgs {
    #[arg(
        value_name = "STORE_ROOT",
        help = "Store root directory to read indexes from",
        required = true,
        allow_hyphen_values = true
    )]
    store_root: String,
    #[arg(long, help = "Only return revision indexes for one document ID")]
    doc_id: Option<String>,
    #[arg(long, help = "Only return patch indexes for one author ID")]
    author: Option<String>,
    #[arg(long, help = "Only return indexes related to one revision ID")]
    revision_id: Option<String>,
    #[arg(long, help = "Only return governance records for one view ID")]
    view_id: Option<String>,
    #[arg(long, help = "Only return head indexes for one profile ID")]
    profile_id: Option<String>,
    #[arg(long, help = "Only return object IDs for one stored object type")]
    object_type: Option<String>,
    #[arg(long, help = "Emit machine-readable store-index output")]
    json: bool,
    #[arg(long, help = "Print only the persisted manifest path")]
    path_only: bool,
    #[arg(hide = true, allow_hyphen_values = true)]
    extra: Vec<String>,
}

#[derive(Debug, Clone, Serialize)]
struct StoreIndexQuerySummary {
    store_root: PathBuf,
    manifest_path: PathBuf,
    status: String,
    stored_object_count: usize,
    object_ids_by_type: std::collections::BTreeMap<String, Vec<String>>,
    doc_revisions: std::collections::BTreeMap<String, Vec<String>>,
    revision_parents: std::collections::BTreeMap<String, Vec<String>>,
    author_patches: std::collections::BTreeMap<String, Vec<String>>,
    view_governance: Vec<ViewGovernanceRecord>,
    profile_heads:
        std::collections::BTreeMap<String, std::collections::BTreeMap<String, Vec<String>>>,
    filters: StoreIndexQueryFilters,
}

#[derive(Debug, Clone, Serialize)]
struct StoreIndexQueryFilters {
    doc_id: Option<String>,
    author: Option<String>,
    revision_id: Option<String>,
    view_id: Option<String>,
    profile_id: Option<String>,
    object_type: Option<String>,
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
    if let Some(path) = &summary.index_manifest_path {
        println!("index manifest: {}", path.display());
    }

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
    println!("indexed objects: {}", summary.indexed_object_count);
    if let Some(path) = &summary.index_manifest_path {
        println!("index manifest: {}", path.display());
    }

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

fn filter_single_map<T: Clone>(
    map: &std::collections::BTreeMap<String, T>,
    key: &Option<String>,
) -> std::collections::BTreeMap<String, T> {
    match key {
        Some(key) => map
            .get(key)
            .cloned()
            .map(|value| [(key.clone(), value)].into_iter().collect())
            .unwrap_or_default(),
        None => map.clone(),
    }
}

fn selected_revision_ids(
    doc_revisions: &std::collections::BTreeMap<String, Vec<String>>,
    revision_id: &Option<String>,
) -> std::collections::BTreeSet<String> {
    let mut revision_ids = doc_revisions
        .values()
        .flat_map(|values| values.iter().cloned())
        .collect::<std::collections::BTreeSet<_>>();

    if let Some(revision_id) = revision_id {
        revision_ids.retain(|current| current == revision_id);
    }

    revision_ids
}

fn filtered_revision_parents(
    manifest: &StoreIndexManifest,
    revision_ids: &std::collections::BTreeSet<String>,
) -> std::collections::BTreeMap<String, Vec<String>> {
    if revision_ids.is_empty() {
        return std::collections::BTreeMap::new();
    }

    manifest
        .revision_parents
        .iter()
        .filter(|(revision_id, _)| revision_ids.contains(*revision_id))
        .map(|(revision_id, parents)| (revision_id.clone(), parents.clone()))
        .collect()
}

fn filtered_profile_heads(
    view_governance: &[ViewGovernanceRecord],
    profile_id: &Option<String>,
    doc_id: &Option<String>,
    revision_id: &Option<String>,
) -> std::collections::BTreeMap<String, std::collections::BTreeMap<String, Vec<String>>> {
    let mut filtered = std::collections::BTreeMap::new();
    for record in view_governance {
        if profile_id
            .as_ref()
            .is_some_and(|requested| requested != &record.profile_id)
        {
            continue;
        }

        for (current_doc_id, current_revision_id) in &record.documents {
            if doc_id
                .as_ref()
                .is_some_and(|requested| requested != current_doc_id)
            {
                continue;
            }
            if revision_id
                .as_ref()
                .is_some_and(|requested| requested != current_revision_id)
            {
                continue;
            }

            filtered
                .entry(record.profile_id.clone())
                .or_insert_with(std::collections::BTreeMap::new)
                .entry(current_doc_id.clone())
                .or_insert_with(Vec::new)
                .push(current_revision_id.clone());
        }
    }

    for documents in filtered.values_mut() {
        for revision_ids in documents.values_mut() {
            revision_ids.sort();
            revision_ids.dedup();
        }
    }

    filtered
}

fn filtered_view_governance(
    manifest: &StoreIndexManifest,
    view_id: &Option<String>,
    profile_id: &Option<String>,
    doc_id: &Option<String>,
    revision_id: &Option<String>,
) -> Vec<ViewGovernanceRecord> {
    manifest
        .view_governance
        .iter()
        .filter(|record| {
            view_id
                .as_ref()
                .is_none_or(|requested| requested == &record.view_id)
        })
        .filter(|record| {
            profile_id
                .as_ref()
                .is_none_or(|requested| requested == &record.profile_id)
        })
        .filter_map(|record| {
            let mut filtered = record.clone();
            if let Some(doc_id) = doc_id {
                filtered
                    .documents
                    .retain(|current_doc_id, _| current_doc_id == doc_id);
                if filtered.documents.is_empty() {
                    return None;
                }
            }
            if let Some(revision_id) = revision_id {
                filtered
                    .documents
                    .retain(|_, current_revision_id| current_revision_id == revision_id);
                if filtered.documents.is_empty() {
                    return None;
                }
            }
            Some(filtered)
        })
        .collect()
}

fn filtered_doc_revisions(
    manifest: &StoreIndexManifest,
    doc_id: &Option<String>,
    revision_id: &Option<String>,
) -> std::collections::BTreeMap<String, Vec<String>> {
    let mut filtered = filter_single_map(&manifest.doc_revisions, doc_id);
    if let Some(revision_id) = revision_id {
        for revision_ids in filtered.values_mut() {
            revision_ids.retain(|current_revision_id| current_revision_id == revision_id);
        }
        filtered.retain(|_, revision_ids| !revision_ids.is_empty());
    }
    filtered
}

fn build_store_index_query_summary(
    store_root: PathBuf,
    manifest: StoreIndexManifest,
    filters: StoreIndexQueryFilters,
) -> StoreIndexQuerySummary {
    let doc_revisions = filtered_doc_revisions(&manifest, &filters.doc_id, &filters.revision_id);
    let author_patches = filter_single_map(&manifest.author_patches, &filters.author);
    let object_ids_by_type = filter_single_map(&manifest.object_ids_by_type, &filters.object_type);
    let view_governance = filtered_view_governance(
        &manifest,
        &filters.view_id,
        &filters.profile_id,
        &filters.doc_id,
        &filters.revision_id,
    );
    let profile_heads = filtered_profile_heads(
        &view_governance,
        &filters.profile_id,
        &filters.doc_id,
        &filters.revision_id,
    );
    let revision_ids = selected_revision_ids(&doc_revisions, &filters.revision_id);
    let revision_parents = filtered_revision_parents(&manifest, &revision_ids);

    StoreIndexQuerySummary {
        manifest_path: store_root.join("indexes").join("manifest.json"),
        store_root,
        status: "ok".to_string(),
        stored_object_count: manifest.stored_object_count,
        object_ids_by_type,
        doc_revisions,
        revision_parents,
        author_patches,
        view_governance,
        profile_heads,
        filters,
    }
}

fn print_store_index_text(summary: &StoreIndexQuerySummary) -> i32 {
    println!("store root: {}", summary.store_root.display());
    println!("manifest path: {}", summary.manifest_path.display());
    println!("status: {}", summary.status);
    println!("stored objects: {}", summary.stored_object_count);
    println!("object type indexes: {}", summary.object_ids_by_type.len());
    println!("document revision indexes: {}", summary.doc_revisions.len());
    println!(
        "revision parent indexes: {}",
        summary.revision_parents.len()
    );
    println!("author patch indexes: {}", summary.author_patches.len());
    println!("view governance records: {}", summary.view_governance.len());
    println!("profile head indexes: {}", summary.profile_heads.len());
    if let Some(doc_id) = &summary.filters.doc_id {
        println!("filter doc_id: {doc_id}");
    }
    if let Some(author) = &summary.filters.author {
        println!("filter author: {author}");
    }
    if let Some(revision_id) = &summary.filters.revision_id {
        println!("filter revision_id: {revision_id}");
    }
    if let Some(view_id) = &summary.filters.view_id {
        println!("filter view_id: {view_id}");
    }
    if let Some(profile_id) = &summary.filters.profile_id {
        println!("filter profile_id: {profile_id}");
    }
    if let Some(object_type) = &summary.filters.object_type {
        println!("filter object_type: {object_type}");
    }
    println!("store index: ok");
    0
}

fn print_store_index_json(summary: &StoreIndexQuerySummary) -> Result<i32, CliError> {
    match serde_json::to_string_pretty(summary) {
        Ok(json) => {
            println!("{json}");
            Ok(0)
        }
        Err(source) => Err(CliError::serialization("store index summary", source)),
    }
}

fn print_store_index_path_only(store_root: &std::path::Path) -> i32 {
    println!(
        "{}",
        store_root.join("indexes").join("manifest.json").display()
    );
    0
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

fn store_index(args: StoreIndexCliArgs) -> Result<i32, CliError> {
    let store_root = PathBuf::from(args.store_root);
    let manifest = load_store_index_manifest(&store_root)
        .map_err(|error| CliError::usage(error.to_string()))?;
    if args.path_only {
        if args.json {
            return Err(CliError::usage(
                "store index --path-only cannot be used with --json",
            ));
        }
        return Ok(print_store_index_path_only(&store_root));
    }
    let summary = build_store_index_query_summary(
        store_root,
        manifest,
        StoreIndexQueryFilters {
            doc_id: args.doc_id,
            author: args.author,
            revision_id: args.revision_id,
            view_id: args.view_id,
            profile_id: args.profile_id,
            object_type: args.object_type,
        },
    );

    if args.json {
        print_store_index_json(&summary)
    } else {
        Ok(print_store_index_text(&summary))
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
        Some(StoreSubcommand::Index(args)) => {
            if let Some(message) = unexpected_extra(&args.extra, "store index") {
                return Err(CliError::usage(message));
            }

            store_index(args)
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
