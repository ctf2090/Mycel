use std::collections::BTreeMap;
use std::path::{Path, PathBuf};

use mycel_core::store::{inspect_governance_view, load_store_index_manifest};
use serde::Serialize;

use crate::{emit_error_line, CliError};

use super::ViewInspectCliArgs;

#[derive(Debug, Clone, Serialize)]
struct ViewInspectSummary {
    store_root: PathBuf,
    manifest_path: PathBuf,
    status: String,
    view_id: String,
    maintainer: Option<String>,
    profile_id: Option<String>,
    timestamp: Option<u64>,
    current_profile_view_id: Option<String>,
    current_profile_document_view_ids: BTreeMap<String, String>,
    documents: BTreeMap<String, String>,
    profile_heads: BTreeMap<String, Vec<String>>,
    maintainer_view_ids: Vec<String>,
    profile_view_ids: Vec<String>,
    document_view_ids: BTreeMap<String, Vec<String>>,
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
            current_profile_view_id: None,
            current_profile_document_view_ids: BTreeMap::new(),
            documents: BTreeMap::new(),
            profile_heads: BTreeMap::new(),
            maintainer_view_ids: Vec::new(),
            profile_view_ids: Vec::new(),
            document_view_ids: BTreeMap::new(),
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
    if let Some(current_profile_view_id) = &summary.current_profile_view_id {
        println!("current profile view id: {current_profile_view_id}");
    }
    println!("document count: {}", summary.documents.len());
    for (doc_id, revision_id) in &summary.documents {
        println!("document: {doc_id} -> {revision_id}");
    }
    println!(
        "current profile document view count: {}",
        summary.current_profile_document_view_ids.len()
    );
    for (doc_id, current_view_id) in &summary.current_profile_document_view_ids {
        println!("current profile document view: {doc_id} -> {current_view_id}");
    }
    println!("profile head doc count: {}", summary.profile_heads.len());
    for (doc_id, revision_ids) in &summary.profile_heads {
        println!("profile heads: {doc_id} -> {}", revision_ids.join(", "));
    }
    println!(
        "maintainer related view count: {}",
        summary.maintainer_view_ids.len()
    );
    if !summary.maintainer_view_ids.is_empty() {
        println!(
            "maintainer related views: {}",
            summary.maintainer_view_ids.join(", ")
        );
    }
    println!(
        "profile related view count: {}",
        summary.profile_view_ids.len()
    );
    if !summary.profile_view_ids.is_empty() {
        println!(
            "profile related views: {}",
            summary.profile_view_ids.join(", ")
        );
    }
    println!(
        "document related view doc count: {}",
        summary.document_view_ids.len()
    );
    for (doc_id, view_ids) in &summary.document_view_ids {
        println!(
            "document related views: {doc_id} -> {}",
            view_ids.join(", ")
        );
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

pub(super) fn handle(args: ViewInspectCliArgs) -> Result<i32, CliError> {
    let ViewInspectCliArgs {
        view_id,
        store_root,
        json,
        extra: _,
    } = args;
    let store_root = PathBuf::from(store_root);

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
    match inspect_governance_view(&manifest, &view_id) {
        Ok(inspection) => {
            summary.maintainer = Some(inspection.maintainer);
            summary.profile_id = Some(inspection.profile_id);
            summary.timestamp = Some(inspection.timestamp);
            summary.current_profile_view_id = inspection.current_profile_view_id;
            summary.current_profile_document_view_ids =
                inspection.current_profile_document_view_ids;
            summary.documents = inspection.documents;
            summary.profile_heads = inspection.profile_heads;
            summary.maintainer_view_ids = inspection.maintainer_view_ids;
            summary.profile_view_ids = inspection.profile_view_ids;
            summary.document_view_ids = inspection.document_view_ids;
        }
        Err(error) => {
            summary.push_error(error.to_string());
        }
    }
    summary.notes.push(
        "governance inspection is separate from reader-facing accepted-head workflows".to_string(),
    );
    summary.notes.push(
        "related maintainer/profile/document view IDs come from persisted governance indexes"
            .to_string(),
    );
    summary.notes.push(
        "current profile governance state comes from persisted governance summaries and latest-view indexes"
            .to_string(),
    );

    if json {
        print_view_inspect_json(&summary)
    } else {
        Ok(print_view_inspect_text(&summary))
    }
}
