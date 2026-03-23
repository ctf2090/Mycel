use std::collections::BTreeMap;
use std::path::{Path, PathBuf};

use mycel_core::store::load_store_index_manifest;
use serde::Serialize;

use crate::{emit_error_line, CliError};

use super::shared::find_view_record;
use super::ViewCurrentCliArgs;

#[derive(Debug, Clone, Serialize)]
pub(super) struct ViewCurrentDocumentSummary {
    doc_id: String,
    current_view_id: String,
    current_revision_id: String,
    maintainer: String,
    timestamp: u64,
}

#[derive(Debug, Clone, Serialize)]
struct ViewCurrentSummary {
    store_root: PathBuf,
    manifest_path: PathBuf,
    status: String,
    profile_id: String,
    doc_id: Option<String>,
    current_view_id: Option<String>,
    profile_current_view_id: Option<String>,
    maintainer: Option<String>,
    timestamp: Option<u64>,
    current_document_revision_id: Option<String>,
    documents: BTreeMap<String, String>,
    current_profile_document_view_ids: BTreeMap<String, String>,
    current_documents: Vec<ViewCurrentDocumentSummary>,
    notes: Vec<String>,
    errors: Vec<String>,
}

impl ViewCurrentSummary {
    fn new(store_root: &Path, profile_id: &str, doc_id: Option<String>) -> Self {
        Self {
            store_root: store_root.to_path_buf(),
            manifest_path: store_root.join("indexes").join("manifest.json"),
            status: "ok".to_string(),
            profile_id: profile_id.to_string(),
            doc_id,
            current_view_id: None,
            profile_current_view_id: None,
            maintainer: None,
            timestamp: None,
            current_document_revision_id: None,
            documents: BTreeMap::new(),
            current_profile_document_view_ids: BTreeMap::new(),
            current_documents: Vec::new(),
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

fn print_view_current_text(summary: &ViewCurrentSummary) -> i32 {
    println!("store root: {}", summary.store_root.display());
    println!("manifest path: {}", summary.manifest_path.display());
    println!("profile id: {}", summary.profile_id);
    if let Some(doc_id) = &summary.doc_id {
        println!("doc id: {doc_id}");
    }
    if let Some(current_view_id) = &summary.current_view_id {
        println!("current view id: {current_view_id}");
    }
    if let Some(profile_current_view_id) = &summary.profile_current_view_id {
        println!("profile current view id: {profile_current_view_id}");
    }
    if let Some(maintainer) = &summary.maintainer {
        println!("maintainer: {maintainer}");
    }
    if let Some(timestamp) = summary.timestamp {
        println!("timestamp: {timestamp}");
    }
    if let Some(current_document_revision_id) = &summary.current_document_revision_id {
        println!("current document revision id: {current_document_revision_id}");
    }
    println!(
        "current profile document view count: {}",
        summary.current_profile_document_view_ids.len()
    );
    for (doc_id, current_view_id) in &summary.current_profile_document_view_ids {
        println!("current profile document view: {doc_id} -> {current_view_id}");
    }
    println!(
        "current document summary count: {}",
        summary.current_documents.len()
    );
    for current in &summary.current_documents {
        println!(
            "current document: {} view={} revision={} maintainer={} timestamp={}",
            current.doc_id,
            current.current_view_id,
            current.current_revision_id,
            current.maintainer,
            current.timestamp
        );
    }
    println!("document count: {}", summary.documents.len());
    for (doc_id, revision_id) in &summary.documents {
        println!("document: {doc_id} -> {revision_id}");
    }
    for note in &summary.notes {
        println!("note: {note}");
    }

    if summary.is_ok() {
        println!("view current: ok");
        0
    } else {
        println!("view current: failed");
        for error in &summary.errors {
            emit_error_line(error);
        }
        1
    }
}

fn print_view_current_json(summary: &ViewCurrentSummary) -> Result<i32, CliError> {
    match serde_json::to_string_pretty(summary) {
        Ok(json) => {
            println!("{json}");
            if summary.is_ok() {
                Ok(0)
            } else {
                Ok(1)
            }
        }
        Err(source) => Err(CliError::serialization("view current summary", source)),
    }
}

pub(super) fn handle(args: ViewCurrentCliArgs) -> Result<i32, CliError> {
    let ViewCurrentCliArgs {
        store_root,
        profile_id,
        doc_id,
        json,
        extra: _,
    } = args;
    let store_root = PathBuf::from(store_root);

    let mut summary = ViewCurrentSummary::new(&store_root, &profile_id, doc_id.clone());
    let manifest = match load_store_index_manifest(&store_root) {
        Ok(manifest) => manifest,
        Err(error) => {
            summary.push_error(format!("failed to read store index manifest: {error}"));
            return if json {
                print_view_current_json(&summary)
            } else {
                Ok(print_view_current_text(&summary))
            };
        }
    };

    let Some(profile_current_view_id) = manifest.latest_profile_views.get(&profile_id).cloned()
    else {
        summary.push_error(format!(
            "profile '{}' was not found in persisted current governance state",
            profile_id
        ));
        return if json {
            print_view_current_json(&summary)
        } else {
            Ok(print_view_current_text(&summary))
        };
    };
    summary.profile_current_view_id = Some(profile_current_view_id.clone());
    summary.current_profile_document_view_ids = manifest
        .latest_document_profile_views
        .get(&profile_id)
        .cloned()
        .unwrap_or_default();

    let selected_view_id = if let Some(doc_id) = &doc_id {
        let Some(current_view_id) = summary
            .current_profile_document_view_ids
            .get(doc_id)
            .cloned()
        else {
            summary.push_error(format!(
                "document '{}' was not found in persisted current governance state for profile '{}'",
                doc_id, profile_id
            ));
            return if json {
                print_view_current_json(&summary)
            } else {
                Ok(print_view_current_text(&summary))
            };
        };
        current_view_id
    } else {
        profile_current_view_id
    };

    let Some(record) = find_view_record(&manifest, &selected_view_id) else {
        summary.push_error(format!(
            "current governance view '{}' is missing from persisted governance indexes",
            selected_view_id
        ));
        return if json {
            print_view_current_json(&summary)
        } else {
            Ok(print_view_current_text(&summary))
        };
    };

    summary.current_view_id = Some(record.view_id.clone());
    summary.maintainer = Some(record.maintainer.clone());
    summary.timestamp = Some(record.timestamp);
    summary.documents = record.documents.clone();
    if let Some(doc_id) = &doc_id {
        summary.current_document_revision_id = record.documents.get(doc_id).cloned();
    }
    let mut current_documents = Vec::new();
    for (current_doc_id, current_view_id) in &summary.current_profile_document_view_ids {
        let Some(current_record) = find_view_record(&manifest, current_view_id) else {
            summary.push_error(format!(
                "current governance view '{}' for document '{}' is missing from persisted governance indexes",
                current_view_id, current_doc_id
            ));
            return if json {
                print_view_current_json(&summary)
            } else {
                Ok(print_view_current_text(&summary))
            };
        };
        let Some(current_revision_id) = current_record.documents.get(current_doc_id).cloned()
        else {
            summary.push_error(format!(
                "current governance view '{}' does not carry document '{}'",
                current_view_id, current_doc_id
            ));
            return if json {
                print_view_current_json(&summary)
            } else {
                Ok(print_view_current_text(&summary))
            };
        };
        current_documents.push(ViewCurrentDocumentSummary {
            doc_id: current_doc_id.clone(),
            current_view_id: current_view_id.clone(),
            current_revision_id,
            maintainer: current_record.maintainer,
            timestamp: current_record.timestamp,
        });
    }
    current_documents.sort_by(|left, right| left.doc_id.cmp(&right.doc_id));
    summary.current_documents = current_documents;
    summary.notes.push(
        "current governance state is read from persisted latest-view indexes instead of replaying all stored views"
            .to_string(),
    );
    if doc_id.is_some() {
        summary.notes.push(
            "doc-scoped current governance may differ from the latest profile-wide view when a newer view does not mention that document"
                .to_string(),
        );
    }

    if json {
        print_view_current_json(&summary)
    } else {
        Ok(print_view_current_text(&summary))
    }
}
