use std::collections::{BTreeMap, HashMap};
use std::fmt;
use std::fs;
use std::path::{Path, PathBuf};

use serde::{Deserialize, Serialize};
use serde_json::Value;
use sha2::{Digest, Sha256};

use crate::protocol::{parse_patch_object, parse_revision_object, parse_view_object};
use crate::verify::{
    canonical_json, hex_encode, verify_object_path, verify_object_value_with_object_index,
};

#[derive(Debug, Clone, Serialize)]
pub struct StoredObjectRecord {
    pub object_id: String,
    pub object_type: String,
    pub path: PathBuf,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct ViewGovernanceRecord {
    pub view_id: String,
    pub maintainer: String,
    pub profile_id: String,
    pub documents: BTreeMap<String, String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct StoreIndexManifest {
    pub version: String,
    pub stored_object_count: usize,
    pub object_ids_by_type: BTreeMap<String, Vec<String>>,
    pub doc_revisions: BTreeMap<String, Vec<String>>,
    pub revision_parents: BTreeMap<String, Vec<String>>,
    pub author_patches: BTreeMap<String, Vec<String>>,
    pub view_governance: Vec<ViewGovernanceRecord>,
    pub profile_heads: BTreeMap<String, BTreeMap<String, Vec<String>>>,
}

#[derive(Debug, Clone, Serialize)]
pub struct StoreIngestSummary {
    pub source: PathBuf,
    pub store_root: PathBuf,
    pub status: String,
    pub discovered_file_count: usize,
    pub identified_object_count: usize,
    pub verified_object_count: usize,
    pub written_object_count: usize,
    pub existing_object_count: usize,
    pub skipped_object_count: usize,
    pub stored_objects: Vec<StoredObjectRecord>,
    pub indexed_object_count: usize,
    pub index_manifest_path: Option<PathBuf>,
    pub notes: Vec<String>,
    pub errors: Vec<String>,
}

#[derive(Debug, Clone, Serialize)]
pub struct StoreRebuildSummary {
    pub target: PathBuf,
    pub status: String,
    pub discovered_file_count: usize,
    pub identified_object_count: usize,
    pub verified_object_count: usize,
    pub stored_object_count: usize,
    pub stored_objects: Vec<StoredObjectRecord>,
    pub doc_revisions: BTreeMap<String, Vec<String>>,
    pub revision_parents: BTreeMap<String, Vec<String>>,
    pub author_patches: BTreeMap<String, Vec<String>>,
    pub view_governance: Vec<ViewGovernanceRecord>,
    pub profile_heads: BTreeMap<String, BTreeMap<String, Vec<String>>>,
    pub index_manifest_path: Option<PathBuf>,
    pub notes: Vec<String>,
    pub errors: Vec<String>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct StoreRebuildError {
    message: String,
}

impl StoreRebuildError {
    fn new(message: impl Into<String>) -> Self {
        Self {
            message: message.into(),
        }
    }
}

impl fmt::Display for StoreRebuildError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(&self.message)
    }
}

impl std::error::Error for StoreRebuildError {}

impl StoreRebuildSummary {
    fn new(target: &Path) -> Self {
        Self {
            target: target.to_path_buf(),
            status: "ok".to_string(),
            discovered_file_count: 0,
            identified_object_count: 0,
            verified_object_count: 0,
            stored_object_count: 0,
            stored_objects: Vec::new(),
            doc_revisions: BTreeMap::new(),
            revision_parents: BTreeMap::new(),
            author_patches: BTreeMap::new(),
            view_governance: Vec::new(),
            profile_heads: BTreeMap::new(),
            index_manifest_path: None,
            notes: Vec::new(),
            errors: Vec::new(),
        }
    }

    pub fn is_ok(&self) -> bool {
        self.errors.is_empty()
    }

    fn push_error(&mut self, message: impl Into<String>) {
        self.status = "failed".to_string();
        self.errors.push(message.into());
    }

    fn push_note(&mut self, message: impl Into<String>) {
        self.notes.push(message.into());
        if self.status != "failed" {
            self.status = "warning".to_string();
        }
    }
}

impl StoreIngestSummary {
    fn new(source: &Path, store_root: &Path) -> Self {
        Self {
            source: source.to_path_buf(),
            store_root: store_root.to_path_buf(),
            status: "ok".to_string(),
            discovered_file_count: 0,
            identified_object_count: 0,
            verified_object_count: 0,
            written_object_count: 0,
            existing_object_count: 0,
            skipped_object_count: 0,
            stored_objects: Vec::new(),
            indexed_object_count: 0,
            index_manifest_path: None,
            notes: Vec::new(),
            errors: Vec::new(),
        }
    }

    pub fn is_ok(&self) -> bool {
        self.errors.is_empty()
    }

    fn push_error(&mut self, message: impl Into<String>) {
        self.status = "failed".to_string();
        self.errors.push(message.into());
    }

    fn push_note(&mut self, message: impl Into<String>) {
        self.notes.push(message.into());
        if self.status != "failed" {
            self.status = "warning".to_string();
        }
    }
}

pub fn rebuild_store_from_path(target: &Path) -> Result<StoreRebuildSummary, StoreRebuildError> {
    let target = normalize_path(target);
    let mut summary = StoreRebuildSummary::new(&target);
    let discovery = discover_store_paths(&target, "store target")?;
    summary.discovered_file_count = discovery.json_paths.len();

    let loaded = load_objects(&discovery.json_paths, SummaryAdapter::Rebuild(&mut summary))?;
    let object_index = build_object_index(&loaded, SummaryAdapter::Rebuild(&mut summary));
    summary.identified_object_count = object_index.len();

    for loaded_object in &loaded {
        let verification =
            verify_object_value_with_object_index(&loaded_object.value, Some(&object_index));
        if !verification.is_ok() {
            summary.push_error(format!(
                "{}: {}",
                loaded_object.path.display(),
                verification.errors.join("; ")
            ));
            continue;
        }

        let Some(record) = stored_record_from_loaded(loaded_object)? else {
            summary.push_note(format!(
                "skipping non-content-addressed object {} ({})",
                loaded_object.path.display(),
                loaded_object.object_type
            ));
            continue;
        };

        summary.verified_object_count += 1;
        index_loaded_object(loaded_object, &record, &mut summary)?;
        summary.stored_objects.push(record);
    }

    summary.stored_objects.sort_by(|left, right| {
        left.object_id
            .cmp(&right.object_id)
            .then_with(|| left.path.cmp(&right.path))
    });
    summary.stored_object_count = summary.stored_objects.len();
    summary.view_governance.sort_by(|left, right| {
        left.view_id
            .cmp(&right.view_id)
            .then_with(|| left.profile_id.cmp(&right.profile_id))
    });
    sort_string_map_values(&mut summary.doc_revisions);
    sort_string_map_values(&mut summary.revision_parents);
    sort_string_map_values(&mut summary.author_patches);
    sort_profile_heads(&mut summary.profile_heads);

    if summary.is_ok() {
        if let Some(store_root) = discovery.store_root {
            let manifest_path = persist_store_index_manifest(&store_root, &summary)?;
            summary.index_manifest_path = Some(manifest_path);
        }
    }

    Ok(summary)
}

pub fn ingest_store_from_path(
    source: &Path,
    store_root: &Path,
) -> Result<StoreIngestSummary, StoreRebuildError> {
    let source = normalize_path(source);
    let store_root = normalize_path(store_root);
    ensure_store_root(&store_root)?;

    let mut summary = StoreIngestSummary::new(&source, &store_root);
    let discovery = discover_store_paths(&source, "ingest source")?;
    summary.discovered_file_count = discovery.json_paths.len();

    let loaded = load_objects(&discovery.json_paths, SummaryAdapter::Ingest(&mut summary))?;
    let object_index = build_object_index(&loaded, SummaryAdapter::Ingest(&mut summary));
    summary.identified_object_count = object_index.len();

    for loaded_object in &loaded {
        let verification = if source.is_file() {
            verify_object_path(&loaded_object.path)
        } else {
            verify_object_value_with_object_index(&loaded_object.value, Some(&object_index))
        };
        if !verification.is_ok() {
            summary.push_error(format!(
                "{}: {}",
                loaded_object.path.display(),
                verification.errors.join("; ")
            ));
            continue;
        }

        let Some(record) = stored_record_from_loaded(loaded_object)? else {
            summary.skipped_object_count += 1;
            summary.push_note(format!(
                "skipping non-content-addressed object {} ({})",
                loaded_object.path.display(),
                loaded_object.object_type
            ));
            continue;
        };

        summary.verified_object_count += 1;
        let write_outcome = write_stored_object(&store_root, loaded_object, &record)?;
        match write_outcome {
            WriteOutcome::Written(path) => {
                summary.written_object_count += 1;
                summary
                    .stored_objects
                    .push(StoredObjectRecord { path, ..record });
            }
            WriteOutcome::AlreadyPresent(path) => {
                summary.existing_object_count += 1;
                summary
                    .stored_objects
                    .push(StoredObjectRecord { path, ..record });
            }
        }
    }

    summary.stored_objects.sort_by(|left, right| {
        left.object_id
            .cmp(&right.object_id)
            .then_with(|| left.path.cmp(&right.path))
    });

    match rebuild_store_from_path(&store_root) {
        Ok(rebuilt) => {
            summary.indexed_object_count = rebuilt.stored_object_count;
            summary.index_manifest_path = rebuilt.index_manifest_path;
        }
        Err(error) => summary.push_error(format!(
            "failed to rebuild store indexes after ingest: {error}"
        )),
    }

    Ok(summary)
}

#[derive(Debug, Clone)]
struct LoadedObject {
    path: PathBuf,
    value: Value,
    object_type: String,
    declared_id: Option<String>,
}

enum SummaryAdapter<'a> {
    Rebuild(&'a mut StoreRebuildSummary),
    Ingest(&'a mut StoreIngestSummary),
}

impl SummaryAdapter<'_> {
    fn push_error(&mut self, message: impl Into<String>) {
        match self {
            Self::Rebuild(summary) => summary.push_error(message),
            Self::Ingest(summary) => summary.push_error(message),
        }
    }

    fn push_note(&mut self, message: impl Into<String>) {
        match self {
            Self::Rebuild(summary) => summary.push_note(message),
            Self::Ingest(summary) => summary.push_note(message),
        }
    }
}

fn normalize_path(path: &Path) -> PathBuf {
    if path.is_absolute() {
        path.to_path_buf()
    } else {
        std::env::current_dir()
            .unwrap_or_else(|_| PathBuf::from("."))
            .join(path)
    }
}

fn ensure_store_root(store_root: &Path) -> Result<(), StoreRebuildError> {
    if store_root.exists() && !store_root.is_dir() {
        return Err(StoreRebuildError::new(format!(
            "store root must be a directory: {}",
            store_root.display()
        )));
    }

    fs::create_dir_all(store_root).map_err(|error| {
        StoreRebuildError::new(format!(
            "failed to create store root {}: {error}",
            store_root.display()
        ))
    })
}

fn objects_root(store_root: &Path) -> PathBuf {
    store_root.join("objects")
}

fn indexes_root(store_root: &Path) -> PathBuf {
    store_root.join("indexes")
}

fn store_index_manifest_path(store_root: &Path) -> PathBuf {
    indexes_root(store_root).join("manifest.json")
}

fn store_path_for_record(
    store_root: &Path,
    record: &StoredObjectRecord,
) -> Result<PathBuf, StoreRebuildError> {
    let (_, object_hash) = record.object_id.split_once(':').ok_or_else(|| {
        StoreRebuildError::new(format!(
            "stored object ID '{}' is missing type prefix separator",
            record.object_id
        ))
    })?;
    Ok(objects_root(store_root)
        .join(&record.object_type)
        .join(format!("{object_hash}.json")))
}

enum WriteOutcome {
    Written(PathBuf),
    AlreadyPresent(PathBuf),
}

fn write_stored_object(
    store_root: &Path,
    loaded_object: &LoadedObject,
    record: &StoredObjectRecord,
) -> Result<WriteOutcome, StoreRebuildError> {
    let store_path = store_path_for_record(store_root, record)?;
    if let Some(parent) = store_path.parent() {
        fs::create_dir_all(parent).map_err(|error| {
            StoreRebuildError::new(format!(
                "failed to create store object directory {}: {error}",
                parent.display()
            ))
        })?;
    }

    let rendered = serde_json::to_string_pretty(&loaded_object.value).map_err(|error| {
        StoreRebuildError::new(format!(
            "failed to serialize object '{}' for storage: {error}",
            record.object_id
        ))
    })?;

    if store_path.exists() {
        let existing = fs::read_to_string(&store_path).map_err(|error| {
            StoreRebuildError::new(format!(
                "failed to read existing stored object {}: {error}",
                store_path.display()
            ))
        })?;
        let existing_value: Value = serde_json::from_str(&existing).map_err(|error| {
            StoreRebuildError::new(format!(
                "failed to parse existing stored object {}: {error}",
                store_path.display()
            ))
        })?;
        if existing_value == loaded_object.value {
            return Ok(WriteOutcome::AlreadyPresent(store_path));
        }
        return Err(StoreRebuildError::new(format!(
            "store path conflict for object '{}' at {}",
            record.object_id,
            store_path.display()
        )));
    }

    fs::write(&store_path, rendered).map_err(|error| {
        StoreRebuildError::new(format!(
            "failed to write stored object {}: {error}",
            store_path.display()
        ))
    })?;

    Ok(WriteOutcome::Written(store_path))
}

fn collect_json_paths(
    target: &Path,
    target_label: &str,
) -> Result<Vec<PathBuf>, StoreRebuildError> {
    if !target.exists() {
        return Err(StoreRebuildError::new(format!(
            "{target_label} does not exist: {}",
            target.display()
        )));
    }

    if target.is_file() {
        return Ok(vec![target.to_path_buf()]);
    }

    let mut paths = Vec::new();
    collect_json_paths_recursive(target, &mut paths)?;
    paths.sort();
    Ok(paths)
}

struct StorePathDiscovery {
    json_paths: Vec<PathBuf>,
    store_root: Option<PathBuf>,
}

fn discover_store_paths(
    target: &Path,
    target_label: &str,
) -> Result<StorePathDiscovery, StoreRebuildError> {
    if !target.exists() {
        return Err(StoreRebuildError::new(format!(
            "{target_label} does not exist: {}",
            target.display()
        )));
    }

    if target.is_file() {
        return Ok(StorePathDiscovery {
            json_paths: vec![target.to_path_buf()],
            store_root: None,
        });
    }

    let looks_like_store_root = objects_root(target).exists() || indexes_root(target).exists();
    let json_paths = if looks_like_store_root {
        let objects_dir = objects_root(target);
        if objects_dir.exists() {
            collect_json_paths(&objects_dir, "store objects root")?
        } else {
            Vec::new()
        }
    } else {
        collect_json_paths(target, target_label)?
    };

    Ok(StorePathDiscovery {
        json_paths,
        store_root: looks_like_store_root.then(|| target.to_path_buf()),
    })
}

fn collect_json_paths_recursive(
    root: &Path,
    paths: &mut Vec<PathBuf>,
) -> Result<(), StoreRebuildError> {
    let entries = fs::read_dir(root).map_err(|error| {
        StoreRebuildError::new(format!(
            "failed to read store directory {}: {error}",
            root.display()
        ))
    })?;

    for entry in entries {
        let entry = entry.map_err(|error| {
            StoreRebuildError::new(format!(
                "failed to read store directory entry {}: {error}",
                root.display()
            ))
        })?;
        let path = entry.path();
        if path.is_dir() {
            collect_json_paths_recursive(&path, paths)?;
            continue;
        }
        if path.extension().and_then(|value| value.to_str()) == Some("json")
            && !path
                .file_name()
                .and_then(|value| value.to_str())
                .is_some_and(|name| name.ends_with(".schema.json"))
        {
            paths.push(path);
        }
    }

    Ok(())
}

fn load_objects(
    paths: &[PathBuf],
    mut summary: SummaryAdapter<'_>,
) -> Result<Vec<LoadedObject>, StoreRebuildError> {
    let mut loaded = Vec::new();

    for path in paths {
        let content = fs::read_to_string(path).map_err(|error| {
            StoreRebuildError::new(format!(
                "failed to read object file {}: {error}",
                path.display()
            ))
        })?;
        let value: Value = serde_json::from_str(&content).map_err(|error| {
            StoreRebuildError::new(format!(
                "failed to parse object JSON {}: {error}",
                path.display()
            ))
        })?;
        let (object_type, declared_id) = {
            let envelope = crate::protocol::parse_object_envelope(&value).map_err(|error| {
                StoreRebuildError::new(format!(
                    "failed to parse object envelope {}: {error}",
                    path.display()
                ))
            })?;
            let declared_id = envelope
                .declared_id()
                .map_err(|error| {
                    StoreRebuildError::new(format!(
                        "{}: failed to read declared object ID: {error:?}",
                        path.display()
                    ))
                })?
                .map(str::to_string);
            (envelope.object_type().to_string(), declared_id)
        };
        loaded.push(LoadedObject {
            path: path.clone(),
            value,
            object_type,
            declared_id,
        });
    }

    if loaded.is_empty() {
        summary.push_note("store target did not contain any JSON object files");
    }

    Ok(loaded)
}

fn build_object_index(
    loaded: &[LoadedObject],
    mut summary: SummaryAdapter<'_>,
) -> HashMap<String, Value> {
    let mut object_index = HashMap::new();

    for loaded_object in loaded {
        if let Some(object_id) = &loaded_object.declared_id {
            if let Some(existing) =
                object_index.insert(object_id.clone(), loaded_object.value.clone())
            {
                let _ = existing;
                summary.push_error(format!(
                    "duplicate declared object ID '{}' found while rebuilding store",
                    object_id
                ));
            }
        }
    }

    object_index
}

fn stored_record_from_loaded(
    loaded_object: &LoadedObject,
) -> Result<Option<StoredObjectRecord>, StoreRebuildError> {
    let Some(object_id) = &loaded_object.declared_id else {
        return Ok(None);
    };

    Ok(Some(StoredObjectRecord {
        object_id: object_id.clone(),
        object_type: loaded_object.object_type.clone(),
        path: loaded_object.path.clone(),
    }))
}

fn index_loaded_object(
    loaded_object: &LoadedObject,
    record: &StoredObjectRecord,
    summary: &mut StoreRebuildSummary,
) -> Result<(), StoreRebuildError> {
    match record.object_type.as_str() {
        "patch" => {
            let patch = parse_patch_object(&loaded_object.value).map_err(|error| {
                StoreRebuildError::new(format!(
                    "{}: failed to parse patch for indexing: {error}",
                    loaded_object.path.display()
                ))
            })?;
            summary
                .author_patches
                .entry(patch.author)
                .or_default()
                .push(patch.patch_id);
        }
        "revision" => {
            let revision = parse_revision_object(&loaded_object.value).map_err(|error| {
                StoreRebuildError::new(format!(
                    "{}: failed to parse revision for indexing: {error}",
                    loaded_object.path.display()
                ))
            })?;
            summary
                .doc_revisions
                .entry(revision.doc_id.clone())
                .or_default()
                .push(revision.revision_id.clone());
            summary
                .revision_parents
                .insert(revision.revision_id, revision.parents);
        }
        "view" => {
            let view = parse_view_object(&loaded_object.value).map_err(|error| {
                StoreRebuildError::new(format!(
                    "{}: failed to parse view for indexing: {error}",
                    loaded_object.path.display()
                ))
            })?;
            let profile_id = hash_value(&view.policy)?;
            let documents = view
                .documents
                .iter()
                .map(|(doc_id, revision_id)| (doc_id.clone(), revision_id.clone()))
                .collect::<BTreeMap<_, _>>();
            for (doc_id, revision_id) in &documents {
                summary
                    .profile_heads
                    .entry(profile_id.clone())
                    .or_default()
                    .entry(doc_id.clone())
                    .or_default()
                    .push(revision_id.clone());
            }
            summary.view_governance.push(ViewGovernanceRecord {
                view_id: view.view_id,
                maintainer: view.maintainer,
                profile_id,
                documents,
            });
        }
        _ => {}
    }

    Ok(())
}

fn hash_value(value: &Value) -> Result<String, StoreRebuildError> {
    let canonical = canonical_json(value).map_err(|error| {
        StoreRebuildError::new(format!("failed to canonicalize value: {error}"))
    })?;
    let mut hasher = Sha256::new();
    hasher.update(canonical.as_bytes());
    Ok(format!("hash:{}", hex_encode(&hasher.finalize())))
}

fn sort_string_map_values(index: &mut BTreeMap<String, Vec<String>>) {
    for values in index.values_mut() {
        values.sort();
        values.dedup();
    }
}

fn sort_profile_heads(index: &mut BTreeMap<String, BTreeMap<String, Vec<String>>>) {
    for documents in index.values_mut() {
        for revision_ids in documents.values_mut() {
            revision_ids.sort();
            revision_ids.dedup();
        }
    }
}

impl StoreIndexManifest {
    fn from_rebuild_summary(summary: &StoreRebuildSummary) -> Self {
        let mut object_ids_by_type = BTreeMap::<String, Vec<String>>::new();
        for record in &summary.stored_objects {
            object_ids_by_type
                .entry(record.object_type.clone())
                .or_default()
                .push(record.object_id.clone());
        }
        sort_string_map_values(&mut object_ids_by_type);

        Self {
            version: "mycel-store-index/0.1".to_string(),
            stored_object_count: summary.stored_object_count,
            object_ids_by_type,
            doc_revisions: summary.doc_revisions.clone(),
            revision_parents: summary.revision_parents.clone(),
            author_patches: summary.author_patches.clone(),
            view_governance: summary.view_governance.clone(),
            profile_heads: summary.profile_heads.clone(),
        }
    }
}

fn persist_store_index_manifest(
    store_root: &Path,
    summary: &StoreRebuildSummary,
) -> Result<PathBuf, StoreRebuildError> {
    let indexes_dir = indexes_root(store_root);
    fs::create_dir_all(&indexes_dir).map_err(|error| {
        StoreRebuildError::new(format!(
            "failed to create store index directory {}: {error}",
            indexes_dir.display()
        ))
    })?;

    let manifest = StoreIndexManifest::from_rebuild_summary(summary);
    let manifest_path = store_index_manifest_path(store_root);
    let rendered = serde_json::to_string_pretty(&manifest).map_err(|error| {
        StoreRebuildError::new(format!(
            "failed to serialize store index manifest {}: {error}",
            manifest_path.display()
        ))
    })?;
    fs::write(&manifest_path, rendered).map_err(|error| {
        StoreRebuildError::new(format!(
            "failed to write store index manifest {}: {error}",
            manifest_path.display()
        ))
    })?;

    Ok(manifest_path)
}

pub fn load_store_index_manifest(
    store_root: &Path,
) -> Result<StoreIndexManifest, StoreRebuildError> {
    let manifest_path = store_index_manifest_path(store_root);
    let content = fs::read_to_string(&manifest_path).map_err(|error| {
        StoreRebuildError::new(format!(
            "failed to read store index manifest {}: {error}",
            manifest_path.display()
        ))
    })?;
    serde_json::from_str(&content).map_err(|error| {
        StoreRebuildError::new(format!(
            "failed to parse store index manifest {}: {error}",
            manifest_path.display()
        ))
    })
}

#[cfg(test)]
mod tests {
    use std::fs;
    use std::path::PathBuf;

    use base64::Engine;
    use ed25519_dalek::{Signer, SigningKey};
    use serde_json::{json, Value};
    use sha2::{Digest, Sha256};

    use super::{ingest_store_from_path, load_store_index_manifest, rebuild_store_from_path};

    fn signing_key() -> SigningKey {
        SigningKey::from_bytes(&[7u8; 32])
    }

    fn signer_id(signing_key: &SigningKey) -> String {
        format!(
            "pk:ed25519:{}",
            base64::engine::general_purpose::STANDARD
                .encode(signing_key.verifying_key().as_bytes())
        )
    }

    fn canonical_json(value: &Value) -> String {
        match value {
            Value::Null => panic!("test values should not use null"),
            Value::Bool(boolean) => boolean.to_string(),
            Value::Number(number) => number.to_string(),
            Value::String(string) => serde_json::to_string(string).expect("string should encode"),
            Value::Array(values) => format!(
                "[{}]",
                values
                    .iter()
                    .map(canonical_json)
                    .collect::<Vec<_>>()
                    .join(",")
            ),
            Value::Object(entries) => {
                let mut keys: Vec<&String> = entries.keys().collect();
                keys.sort_unstable();
                let parts = keys
                    .into_iter()
                    .map(|key| {
                        format!(
                            "{}:{}",
                            serde_json::to_string(key).expect("key should encode"),
                            canonical_json(&entries[key])
                        )
                    })
                    .collect::<Vec<_>>();
                format!("{{{}}}", parts.join(","))
            }
        }
    }

    fn recompute_id(value: &Value, id_field: &str, prefix: &str) -> String {
        let mut object = value
            .as_object()
            .cloned()
            .expect("test object should be JSON object");
        object.remove(id_field);
        object.remove("signature");
        let canonical = canonical_json(&Value::Object(object));
        let mut hasher = Sha256::new();
        hasher.update(canonical.as_bytes());
        format!("{prefix}:{:x}", hasher.finalize())
    }

    fn sign_value(signing_key: &SigningKey, value: &Value) -> String {
        let mut object = value
            .as_object()
            .cloned()
            .expect("test object should be JSON object");
        object.remove("signature");
        let canonical = canonical_json(&Value::Object(object));
        let signature = signing_key.sign(canonical.as_bytes());
        format!(
            "sig:ed25519:{}",
            base64::engine::general_purpose::STANDARD.encode(signature.to_bytes())
        )
    }

    fn signed_object(
        mut value: Value,
        signer_field: &str,
        id_field: &str,
        id_prefix: &str,
    ) -> Value {
        let signing_key = signing_key();
        value[signer_field] = Value::String(signer_id(&signing_key));
        let id = recompute_id(&value, id_field, id_prefix);
        value[id_field] = Value::String(id);
        let signature = sign_value(&signing_key, &value);
        value["signature"] = Value::String(signature);
        value
    }

    fn write_temp_dir(prefix: &str) -> PathBuf {
        let unique = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .expect("system time should be after unix epoch")
            .as_nanos();
        let path = std::env::temp_dir().join(format!("mycel-store-{prefix}-{unique}"));
        fs::create_dir_all(&path).expect("temp dir should be created");
        path
    }

    #[test]
    fn rebuild_store_indexes_verified_objects() {
        let dir = write_temp_dir("rebuild");
        let patch = signed_object(
            json!({
                "type": "patch",
                "version": "mycel/0.1",
                "doc_id": "doc:test",
                "base_revision": "rev:genesis-null",
                "timestamp": 1u64,
                "ops": [
                    {
                        "op": "insert_block",
                        "new_block": {
                            "block_id": "blk:001",
                            "block_type": "paragraph",
                            "content": "Hello",
                            "attrs": {},
                            "children": []
                        }
                    }
                ]
            }),
            "author",
            "patch_id",
            "patch",
        );
        let patch_id = patch["patch_id"]
            .as_str()
            .expect("patch id should exist")
            .to_string();
        fs::write(
            dir.join("patch.json"),
            serde_json::to_string_pretty(&patch).expect("patch should serialize"),
        )
        .expect("patch should write");

        let state = json!({
            "doc_id": "doc:test",
            "blocks": [
                {
                    "block_id": "blk:001",
                    "block_type": "paragraph",
                    "content": "Hello",
                    "attrs": {},
                    "children": []
                }
            ]
        });
        let mut hasher = Sha256::new();
        hasher.update(canonical_json(&state).as_bytes());
        let state_hash = format!("hash:{:x}", hasher.finalize());

        let revision = signed_object(
            json!({
                "type": "revision",
                "version": "mycel/0.1",
                "doc_id": "doc:test",
                "parents": [],
                "patches": [patch_id.clone()],
                "state_hash": state_hash,
                "timestamp": 2u64
            }),
            "author",
            "revision_id",
            "rev",
        );
        let revision_id = revision["revision_id"]
            .as_str()
            .expect("revision id should exist")
            .to_string();
        fs::write(
            dir.join("revision.json"),
            serde_json::to_string_pretty(&revision).expect("revision should serialize"),
        )
        .expect("revision should write");

        let policy = json!({
            "accept_keys": [signer_id(&signing_key())],
            "merge_rule": "manual-reviewed",
            "preferred_branches": ["main"]
        });
        let view = signed_object(
            json!({
                "type": "view",
                "version": "mycel/0.1",
                "documents": {
                    "doc:test": revision_id
                },
                "policy": policy,
                "timestamp": 3u64
            }),
            "maintainer",
            "view_id",
            "view",
        );
        fs::write(
            dir.join("view.json"),
            serde_json::to_string_pretty(&view).expect("view should serialize"),
        )
        .expect("view should write");

        let summary = rebuild_store_from_path(&dir).expect("store rebuild should succeed");

        assert!(summary.is_ok(), "expected ok summary, got {summary:?}");
        assert_eq!(summary.verified_object_count, 3);
        assert_eq!(summary.stored_object_count, 3);
        assert_eq!(
            summary.doc_revisions.get("doc:test"),
            Some(&vec![revision["revision_id"].as_str().unwrap().to_string()])
        );
        assert_eq!(
            summary
                .revision_parents
                .get(revision["revision_id"].as_str().unwrap()),
            Some(&Vec::<String>::new())
        );
        assert_eq!(
            summary
                .author_patches
                .get(signer_id(&signing_key()).as_str())
                .expect("author patches should be indexed"),
            &vec![patch_id]
        );
        assert_eq!(summary.view_governance.len(), 1);
        assert_eq!(summary.profile_heads.len(), 1);

        let _ = fs::remove_dir_all(dir);
    }

    #[test]
    fn ingest_store_writes_verified_objects_to_content_addressed_layout() {
        let source_dir = write_temp_dir("ingest-source");
        let store_dir = write_temp_dir("ingest-store");
        let patch = signed_object(
            json!({
                "type": "patch",
                "version": "mycel/0.1",
                "doc_id": "doc:ingest",
                "base_revision": "rev:genesis-null",
                "timestamp": 1u64,
                "ops": [
                    {
                        "op": "insert_block",
                        "new_block": {
                            "block_id": "blk:ingest-001",
                            "block_type": "paragraph",
                            "content": "Hello ingest",
                            "attrs": {},
                            "children": []
                        }
                    }
                ]
            }),
            "author",
            "patch_id",
            "patch",
        );
        let patch_id = patch["patch_id"]
            .as_str()
            .expect("patch id should exist")
            .to_string();
        fs::write(
            source_dir.join("patch.json"),
            serde_json::to_string_pretty(&patch).expect("patch should serialize"),
        )
        .expect("patch should write");

        let state = json!({
            "doc_id": "doc:ingest",
            "blocks": [
                {
                    "block_id": "blk:ingest-001",
                    "block_type": "paragraph",
                    "content": "Hello ingest",
                    "attrs": {},
                    "children": []
                }
            ]
        });
        let mut hasher = Sha256::new();
        hasher.update(canonical_json(&state).as_bytes());
        let state_hash = format!("hash:{:x}", hasher.finalize());

        let revision = signed_object(
            json!({
                "type": "revision",
                "version": "mycel/0.1",
                "doc_id": "doc:ingest",
                "parents": [],
                "patches": [patch_id],
                "state_hash": state_hash,
                "timestamp": 2u64
            }),
            "author",
            "revision_id",
            "rev",
        );
        let revision_id = revision["revision_id"]
            .as_str()
            .expect("revision id should exist")
            .to_string();
        fs::write(
            source_dir.join("revision.json"),
            serde_json::to_string_pretty(&revision).expect("revision should serialize"),
        )
        .expect("revision should write");

        let summary =
            ingest_store_from_path(&source_dir, &store_dir).expect("store ingest should succeed");

        assert!(summary.is_ok(), "expected ok summary, got {summary:?}");
        assert_eq!(summary.verified_object_count, 2);
        assert_eq!(summary.written_object_count, 2);
        assert_eq!(summary.existing_object_count, 0);
        assert_eq!(summary.skipped_object_count, 0);
        assert_eq!(summary.indexed_object_count, 2);
        assert_eq!(
            summary
                .index_manifest_path
                .as_ref()
                .map(|path| path.file_name().and_then(|value| value.to_str())),
            Some(Some("manifest.json"))
        );

        let patch_hash = patch["patch_id"]
            .as_str()
            .and_then(|value| value.split_once(':'))
            .map(|(_, hash)| hash)
            .expect("patch id should include hash");
        let revision_hash = revision_id
            .split_once(':')
            .map(|(_, hash)| hash)
            .expect("revision id should include hash");

        assert!(store_dir
            .join("objects")
            .join("patch")
            .join(format!("{patch_hash}.json"))
            .exists());
        assert!(store_dir
            .join("objects")
            .join("revision")
            .join(format!("{revision_hash}.json"))
            .exists());
        let manifest = load_store_index_manifest(&store_dir)
            .expect("ingested store should expose a persisted index manifest");
        assert_eq!(manifest.stored_object_count, 2);
        assert_eq!(
            manifest.doc_revisions.get("doc:ingest"),
            Some(&vec![revision_id.clone()])
        );

        let rebuild =
            rebuild_store_from_path(&store_dir).expect("rebuild from ingested store should work");
        assert!(
            rebuild.is_ok(),
            "expected ok rebuild summary, got {rebuild:?}"
        );
        assert_eq!(rebuild.stored_object_count, 2);
        assert_eq!(
            rebuild
                .index_manifest_path
                .as_ref()
                .map(|path| path.file_name().and_then(|value| value.to_str())),
            Some(Some("manifest.json"))
        );

        let _ = fs::remove_dir_all(source_dir);
        let _ = fs::remove_dir_all(store_dir);
    }

    #[test]
    fn ingest_store_marks_existing_objects_on_repeat_ingest() {
        let source_dir = write_temp_dir("repeat-source");
        let store_dir = write_temp_dir("repeat-store");
        let patch = signed_object(
            json!({
                "type": "patch",
                "version": "mycel/0.1",
                "doc_id": "doc:repeat",
                "base_revision": "rev:genesis-null",
                "timestamp": 1u64,
                "ops": []
            }),
            "author",
            "patch_id",
            "patch",
        );
        fs::write(
            source_dir.join("patch.json"),
            serde_json::to_string_pretty(&patch).expect("patch should serialize"),
        )
        .expect("patch should write");

        let first =
            ingest_store_from_path(&source_dir, &store_dir).expect("first ingest should succeed");
        assert!(first.is_ok(), "expected ok first ingest, got {first:?}");
        assert_eq!(first.written_object_count, 1);
        assert_eq!(first.existing_object_count, 0);

        let second =
            ingest_store_from_path(&source_dir, &store_dir).expect("second ingest should succeed");
        assert!(second.is_ok(), "expected ok second ingest, got {second:?}");
        assert_eq!(second.written_object_count, 0);
        assert_eq!(second.existing_object_count, 1);
        assert_eq!(second.verified_object_count, 1);
        assert_eq!(second.indexed_object_count, 1);
        let manifest = load_store_index_manifest(&store_dir)
            .expect("repeat ingest should keep the persisted index manifest updated");
        assert_eq!(manifest.stored_object_count, 1);

        let _ = fs::remove_dir_all(source_dir);
        let _ = fs::remove_dir_all(store_dir);
    }
}
