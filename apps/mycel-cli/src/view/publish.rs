use std::collections::BTreeMap;
use std::path::{Path, PathBuf};

use mycel_core::protocol::parse_view_object;
use mycel_core::store::{load_store_index_manifest, write_object_value_to_store};
use mycel_core::verify::verify_object_path;
use serde::Serialize;

use crate::{emit_error_line, CliError};

use super::shared::{find_view_record, load_source_value, related_document_view_ids};
use super::ViewPublishCliArgs;

#[derive(Debug, Clone, Serialize)]
struct ViewPublishSummary {
    source_path: PathBuf,
    store_root: PathBuf,
    status: String,
    view_id: Option<String>,
    maintainer: Option<String>,
    profile_id: Option<String>,
    documents: BTreeMap<String, String>,
    maintainer_view_ids: Vec<String>,
    profile_view_ids: Vec<String>,
    document_view_ids: BTreeMap<String, Vec<String>>,
    created: bool,
    stored_path: Option<PathBuf>,
    index_manifest_path: Option<PathBuf>,
    notes: Vec<String>,
    errors: Vec<String>,
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
            maintainer_view_ids: Vec::new(),
            profile_view_ids: Vec::new(),
            document_view_ids: BTreeMap::new(),
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

pub(super) fn handle(args: ViewPublishCliArgs) -> Result<i32, CliError> {
    let ViewPublishCliArgs {
        source,
        into,
        json,
        extra: _,
    } = args;
    let source_path = PathBuf::from(source);
    let store_root = PathBuf::from(into);

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
    summary.profile_id = Some(record.profile_id.clone());
    summary.documents = record.documents.clone();
    summary.maintainer_view_ids = manifest
        .maintainer_views
        .get(&record.maintainer)
        .cloned()
        .unwrap_or_default();
    summary.profile_view_ids = manifest
        .profile_views
        .get(&record.profile_id)
        .cloned()
        .unwrap_or_default();
    summary.document_view_ids = related_document_view_ids(&manifest, &record.documents);
    summary.created = write.created;
    summary.stored_path = Some(write.record.path);
    summary.index_manifest_path = write.index_manifest_path;
    summary.notes.push(
        "published governance state is separate from revision publication and accepted-head inspection".to_string(),
    );
    summary.notes.push(
        "related maintainer/profile/document view IDs come from persisted governance indexes"
            .to_string(),
    );

    if json {
        print_view_publish_json(&summary)
    } else {
        Ok(print_view_publish_text(&summary))
    }
}
