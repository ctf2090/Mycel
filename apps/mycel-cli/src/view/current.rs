use std::collections::BTreeMap;
use std::path::{Path, PathBuf};

use mycel_core::store::{inspect_current_governance, load_store_index_manifest};
use serde::Serialize;

use crate::{emit_error_line, CliError};

use super::shared::load_view_editor_role_summary;
use super::ViewCurrentCliArgs;

#[derive(Debug, Clone, Serialize)]
pub(super) struct ViewCurrentDocumentSummary {
    doc_id: String,
    current_view_id: String,
    current_revision_id: String,
    maintainer: String,
    timestamp: u64,
    accepted_editor_keys: Vec<String>,
    maintainer_is_admitted_editor: bool,
    admitted_editor_only_keys: Vec<String>,
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
    accepted_editor_keys: Vec<String>,
    maintainer_is_admitted_editor: bool,
    admitted_editor_only_keys: Vec<String>,
    documents: BTreeMap<String, String>,
    current_profile_document_view_ids: BTreeMap<String, String>,
    current_documents: Vec<ViewCurrentDocumentSummary>,
    profile_heads: BTreeMap<String, Vec<String>>,
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
            accepted_editor_keys: Vec::new(),
            maintainer_is_admitted_editor: false,
            admitted_editor_only_keys: Vec::new(),
            documents: BTreeMap::new(),
            current_profile_document_view_ids: BTreeMap::new(),
            current_documents: Vec::new(),
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
        "accepted editor key count: {}",
        summary.accepted_editor_keys.len()
    );
    if !summary.accepted_editor_keys.is_empty() {
        println!(
            "accepted editor keys: {}",
            summary.accepted_editor_keys.join(", ")
        );
    }
    println!(
        "maintainer is admitted editor: {}",
        summary.maintainer_is_admitted_editor
    );
    println!(
        "admitted editor-only key count: {}",
        summary.admitted_editor_only_keys.len()
    );
    if !summary.admitted_editor_only_keys.is_empty() {
        println!(
            "admitted editor-only keys: {}",
            summary.admitted_editor_only_keys.join(", ")
        );
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
            "current document: {} view={} revision={} maintainer={} timestamp={} admitted_editors={} maintainer_is_admitted_editor={} editor_only_keys={}",
            current.doc_id,
            current.current_view_id,
            current.current_revision_id,
            current.maintainer,
            current.timestamp,
            current.accepted_editor_keys.join(", "),
            current.maintainer_is_admitted_editor,
            current.admitted_editor_only_keys.join(", "),
        );
    }
    println!("profile head doc count: {}", summary.profile_heads.len());
    for (doc_id, revision_ids) in &summary.profile_heads {
        println!("profile heads: {doc_id} -> {}", revision_ids.join(", "));
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

    match inspect_current_governance(&manifest, &profile_id, doc_id.as_deref()) {
        Ok(current) => {
            match load_view_editor_role_summary(
                &store_root,
                &current.current_view_id,
                &current.maintainer,
            ) {
                Ok(editor_roles) => {
                    summary.accepted_editor_keys = editor_roles.accepted_editor_keys;
                    summary.maintainer_is_admitted_editor =
                        editor_roles.maintainer_is_admitted_editor;
                    summary.admitted_editor_only_keys = editor_roles.admitted_editor_only_keys;
                }
                Err(error) => {
                    summary.push_error(error);
                }
            }
            summary.current_view_id = Some(current.current_view_id);
            summary.profile_current_view_id = Some(current.profile_current_view_id);
            summary.maintainer = Some(current.maintainer);
            summary.timestamp = Some(current.timestamp);
            summary.current_document_revision_id = current.current_document_revision_id;
            summary.documents = current.documents;
            summary.current_profile_document_view_ids = current.current_profile_document_view_ids;
            let mut current_documents = Vec::with_capacity(current.current_documents.len());
            for current_document in current.current_documents {
                match load_view_editor_role_summary(
                    &store_root,
                    &current_document.current_view_id,
                    &current_document.maintainer,
                ) {
                    Ok(editor_roles) => current_documents.push(ViewCurrentDocumentSummary {
                        doc_id: current_document.doc_id,
                        current_view_id: current_document.current_view_id,
                        current_revision_id: current_document.current_revision_id,
                        maintainer: current_document.maintainer,
                        timestamp: current_document.timestamp,
                        accepted_editor_keys: editor_roles.accepted_editor_keys,
                        maintainer_is_admitted_editor: editor_roles.maintainer_is_admitted_editor,
                        admitted_editor_only_keys: editor_roles.admitted_editor_only_keys,
                    }),
                    Err(error) => {
                        summary.push_error(error);
                    }
                }
            }
            summary.current_documents = current_documents;
            summary.profile_heads = current.profile_heads;
        }
        Err(error) => {
            summary.push_error(error.to_string());
        }
    }
    summary.notes.push(
        "current governance state is read from persisted governance summaries instead of replaying all stored views"
            .to_string(),
    );
    summary.notes.push(
        "profile head IDs come from persisted governance head indexes for the selected profile"
            .to_string(),
    );
    if doc_id.is_some() {
        summary.notes.push(
            "doc-scoped current governance may differ from the latest profile-wide view when a newer view does not mention that document"
                .to_string(),
        );
    }
    summary.notes.push(
        "accepted editor keys come from the persisted current view policy so editor-maintainer and view-maintainer assignments stay visible together"
            .to_string(),
    );

    if json {
        print_view_current_json(&summary)
    } else {
        Ok(print_view_current_text(&summary))
    }
}
