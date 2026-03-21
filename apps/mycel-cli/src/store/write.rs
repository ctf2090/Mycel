use std::fs;
use std::path::{Path, PathBuf};
use std::time::{SystemTime, UNIX_EPOCH};

use mycel_core::author::{
    commit_revision_to_store, create_document_in_store, create_merge_revision_in_store,
    create_patch_in_store, parse_signing_key_seed, DocumentCreateParams, DocumentCreateSummary,
    MergeRevisionCreateParams, MergeRevisionCreateSummary, PatchCreateParams, PatchCreateSummary,
    RevisionCommitParams, RevisionCommitSummary,
};
use mycel_core::protocol::parse_json_value_strict;
use mycel_core::replay::DocumentState;
use mycel_core::store::{initialize_store_root, StoreInitSummary};
use serde::Serialize;
use serde_json::Value;

use crate::CliError;

use super::{
    StoreCommitRevisionCliArgs, StoreCreateDocumentCliArgs, StoreCreateMergeRevisionCliArgs,
    StoreCreatePatchCliArgs,
};

pub(super) fn store_init(store_root: PathBuf, json: bool) -> Result<i32, CliError> {
    match initialize_store_root(&store_root) {
        Ok(summary) => {
            if json {
                print_json(&summary, "store init summary")
            } else {
                Ok(print_store_init_text(&summary))
            }
        }
        Err(error) => Err(CliError::usage(error.to_string())),
    }
}

pub(super) fn store_create_document(args: StoreCreateDocumentCliArgs) -> Result<i32, CliError> {
    let signing_key = load_signing_key(&args.signing_key)?;
    let params = DocumentCreateParams {
        doc_id: args.doc_id,
        title: args.title,
        language: args.language,
        timestamp: resolve_timestamp(args.timestamp)?,
    };

    match create_document_in_store(Path::new(&args.store_root), &signing_key, &params) {
        Ok(summary) => {
            if args.json {
                print_json(&summary, "document create summary")
            } else {
                Ok(print_document_create_text(&summary))
            }
        }
        Err(error) => Err(CliError::usage(error.to_string())),
    }
}

pub(super) fn store_create_patch(args: StoreCreatePatchCliArgs) -> Result<i32, CliError> {
    let signing_key = load_signing_key(&args.signing_key)?;
    let ops = load_ops_value(&args.ops)?;
    if !ops.is_array() {
        return Err(CliError::usage(
            "patch ops file must contain a top-level JSON array",
        ));
    }

    let params = PatchCreateParams {
        doc_id: args.doc_id,
        base_revision: args.base_revision,
        timestamp: resolve_timestamp(args.timestamp)?,
        ops,
    };

    match create_patch_in_store(Path::new(&args.store_root), &signing_key, &params) {
        Ok(summary) => {
            if args.json {
                print_json(&summary, "patch create summary")
            } else {
                Ok(print_patch_create_text(&summary))
            }
        }
        Err(error) => Err(CliError::usage(error.to_string())),
    }
}

pub(super) fn store_commit_revision(args: StoreCommitRevisionCliArgs) -> Result<i32, CliError> {
    let signing_key = load_signing_key(&args.signing_key)?;
    let params = RevisionCommitParams {
        doc_id: args.doc_id,
        parents: args.parents,
        patches: args.patches,
        merge_strategy: args.merge_strategy,
        timestamp: resolve_timestamp(args.timestamp)?,
    };

    match commit_revision_to_store(Path::new(&args.store_root), &signing_key, &params) {
        Ok(summary) => {
            if args.json {
                print_json(&summary, "revision commit summary")
            } else {
                Ok(print_revision_commit_text(&summary))
            }
        }
        Err(error) => Err(CliError::usage(error.to_string())),
    }
}

pub(super) fn store_create_merge_revision(
    args: StoreCreateMergeRevisionCliArgs,
) -> Result<i32, CliError> {
    let signing_key = load_signing_key(&args.signing_key)?;
    let resolved_state = load_resolved_state(&args.resolved_state)?;
    let params = MergeRevisionCreateParams {
        doc_id: args.doc_id,
        parents: args.parents,
        resolved_state,
        merge_strategy: args.merge_strategy,
        timestamp: resolve_timestamp(args.timestamp)?,
    };

    match create_merge_revision_in_store(Path::new(&args.store_root), &signing_key, &params) {
        Ok(summary) => {
            if args.json {
                print_json(&summary, "merge revision create summary")
            } else {
                Ok(print_merge_revision_create_text(&summary))
            }
        }
        Err(error) => {
            if let Some(summary_value) = error.json_summary() {
                if args.json {
                    let print_code =
                        print_json(summary_value, "merge revision manual curation summary")?;
                    if print_code != 0 {
                        Ok(print_code)
                    } else {
                        Ok(1)
                    }
                } else {
                    Ok(print_manual_curation_text(summary_value))
                }
            } else {
                Err(CliError::usage(error.to_string()))
            }
        }
    }
}

fn load_signing_key(path: &str) -> Result<ed25519_dalek::SigningKey, CliError> {
    let content = fs::read_to_string(path).map_err(|error| {
        CliError::usage(format!("failed to read signing key file {path}: {error}"))
    })?;
    parse_signing_key_seed(&content).map_err(CliError::usage)
}

fn load_ops_value(path: &str) -> Result<Value, CliError> {
    let content = fs::read_to_string(path).map_err(|error| {
        CliError::usage(format!("failed to read patch ops file {path}: {error}"))
    })?;
    parse_json_value_strict(&content)
        .map_err(|error| CliError::usage(format!("failed to parse patch ops file {path}: {error}")))
}

fn load_resolved_state(path: &str) -> Result<DocumentState, CliError> {
    let content = fs::read_to_string(path).map_err(|error| {
        CliError::usage(format!(
            "failed to read resolved state file {path}: {error}"
        ))
    })?;
    let value = parse_json_value_strict(&content).map_err(|error| {
        CliError::usage(format!(
            "failed to parse resolved state file {path}: {error}"
        ))
    })?;
    serde_json::from_value(value).map_err(|error| {
        CliError::usage(format!(
            "failed to decode resolved state file {path} as DocumentState: {error}"
        ))
    })
}

fn resolve_timestamp(timestamp: Option<u64>) -> Result<u64, CliError> {
    match timestamp {
        Some(timestamp) => Ok(timestamp),
        None => SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .map(|duration| duration.as_secs())
            .map_err(|error| {
                CliError::usage(format!("failed to resolve current timestamp: {error}"))
            }),
    }
}

fn print_store_init_text(summary: &StoreInitSummary) -> i32 {
    println!("store init: {}", summary.status);
    println!("store root: {}", summary.store_root.display());
    println!("index manifest: {}", summary.index_manifest_path.display());
    println!("local policy: {}", summary.local_policy_path.display());
    0
}

fn print_document_create_text(summary: &DocumentCreateSummary) -> i32 {
    println!("document create: {}", summary.status);
    println!("store root: {}", summary.store_root.display());
    println!("document: {}", summary.document_object_id);
    println!("genesis revision: {}", summary.genesis_revision_id);
    println!("written objects: {}", summary.written_object_count);
    println!("existing objects: {}", summary.existing_object_count);
    if let Some(path) = &summary.index_manifest_path {
        println!("index manifest: {}", path.display());
    }
    0
}

fn print_patch_create_text(summary: &PatchCreateSummary) -> i32 {
    println!("patch create: {}", summary.status);
    println!("store root: {}", summary.store_root.display());
    println!("doc_id: {}", summary.doc_id);
    println!("patch: {}", summary.patch_id);
    println!("base revision: {}", summary.base_revision);
    println!("written objects: {}", summary.written_object_count);
    println!("existing objects: {}", summary.existing_object_count);
    if let Some(path) = &summary.index_manifest_path {
        println!("index manifest: {}", path.display());
    }
    0
}

fn print_revision_commit_text(summary: &RevisionCommitSummary) -> i32 {
    println!("revision commit: {}", summary.status);
    println!("store root: {}", summary.store_root.display());
    println!("doc_id: {}", summary.doc_id);
    println!("revision: {}", summary.revision_id);
    println!("state_hash: {}", summary.recomputed_state_hash);
    println!("written objects: {}", summary.written_object_count);
    println!("existing objects: {}", summary.existing_object_count);
    if let Some(path) = &summary.index_manifest_path {
        println!("index manifest: {}", path.display());
    }
    0
}

fn print_merge_revision_create_text(summary: &MergeRevisionCreateSummary) -> i32 {
    println!("merge revision create: {}", summary.status);
    println!("store root: {}", summary.store_root.display());
    println!("doc_id: {}", summary.doc_id);
    println!("merge outcome: {}", summary.merge_outcome.as_str());
    println!("patch: {}", summary.patch_id);
    println!("patch ops: {}", summary.patch_op_count);
    println!("revision: {}", summary.revision_id);
    println!("state_hash: {}", summary.recomputed_state_hash);
    println!("written objects: {}", summary.written_object_count);
    println!("existing objects: {}", summary.existing_object_count);
    if !summary.merge_reasons.is_empty() {
        println!("merge reasons: {}", summary.merge_reasons.join("; "));
    }
    if let Some(path) = &summary.index_manifest_path {
        println!("index manifest: {}", path.display());
    }
    0
}

fn print_manual_curation_text(summary: &Value) -> i32 {
    let status = summary["status"].as_str().unwrap_or("failed");
    println!("merge revision create: {status}");
    if let Some(doc_id) = summary["doc_id"].as_str() {
        println!("doc_id: {doc_id}");
    }
    if let Some(merge_outcome) = summary["merge_outcome"].as_str() {
        println!("merge outcome: {merge_outcome}");
    }
    if let Some(parent_revision_ids) = summary["parent_revision_ids"].as_array() {
        let parent_revision_ids = parent_revision_ids
            .iter()
            .filter_map(|entry| entry.as_str())
            .collect::<Vec<_>>();
        if !parent_revision_ids.is_empty() {
            println!("parent revisions: {}", parent_revision_ids.join(", "));
        }
    }
    if let Some(merge_reasons) = summary["merge_reasons"].as_array() {
        let merge_reasons = merge_reasons
            .iter()
            .filter_map(|entry| entry.as_str())
            .collect::<Vec<_>>();
        if !merge_reasons.is_empty() {
            println!("merge reasons: {}", merge_reasons.join("; "));
        }
    }
    if let Some(errors) = summary["errors"].as_array() {
        for error in errors.iter().filter_map(|entry| entry.as_str()) {
            println!("error: {error}");
        }
    }
    1
}

fn print_json<T: Serialize>(value: &T, context: &'static str) -> Result<i32, CliError> {
    match serde_json::to_string_pretty(value) {
        Ok(json) => {
            println!("{json}");
            Ok(0)
        }
        Err(source) => Err(CliError::serialization(context, source)),
    }
}
