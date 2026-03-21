use std::collections::{BTreeMap, BTreeSet, HashMap};
use std::fmt;
use std::fs;
use std::path::{Path, PathBuf};

use serde::{Deserialize, Serialize};
use serde_json::Value;

use crate::protocol::{
    parse_json_strict, parse_json_value_strict, parse_patch_object, parse_revision_object,
    parse_view_object, prefixed_canonical_hash,
};
use crate::verify::{verify_object_path, verify_object_value_with_object_index};

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

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct LocalStorePolicy {
    pub version: String,
    #[serde(default)]
    pub transport: BTreeMap<String, Value>,
    #[serde(default)]
    pub safety: BTreeMap<String, Value>,
}

impl Default for LocalStorePolicy {
    fn default() -> Self {
        Self {
            version: "mycel-local-policy/0.1".to_string(),
            transport: BTreeMap::new(),
            safety: BTreeMap::new(),
        }
    }
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
    #[serde(default)]
    pub maintainer_views: BTreeMap<String, Vec<String>>,
    #[serde(default)]
    pub profile_views: BTreeMap<String, Vec<String>>,
    #[serde(default)]
    pub document_views: BTreeMap<String, Vec<String>>,
    pub profile_heads: BTreeMap<String, BTreeMap<String, Vec<String>>>,
    #[serde(default)]
    pub doc_heads: BTreeMap<String, Vec<String>>,
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
    pub maintainer_views: BTreeMap<String, Vec<String>>,
    pub profile_views: BTreeMap<String, Vec<String>>,
    pub document_views: BTreeMap<String, Vec<String>>,
    pub profile_heads: BTreeMap<String, BTreeMap<String, Vec<String>>>,
    pub doc_heads: BTreeMap<String, Vec<String>>,
    pub index_manifest_path: Option<PathBuf>,
    pub notes: Vec<String>,
    pub errors: Vec<String>,
}

#[derive(Debug, Clone, Serialize)]
pub struct StoreInitSummary {
    pub store_root: PathBuf,
    pub status: String,
    pub index_manifest_path: PathBuf,
    pub local_policy_path: PathBuf,
    pub notes: Vec<String>,
    pub errors: Vec<String>,
}

#[derive(Debug, Clone)]
pub struct StoreWriteSummary {
    pub store_root: PathBuf,
    pub record: StoredObjectRecord,
    pub created: bool,
    pub index_manifest_path: Option<PathBuf>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct StoreRebuildError {
    message: String,
    json_summary: Option<Value>,
}

impl StoreRebuildError {
    pub(crate) fn new(message: impl Into<String>) -> Self {
        Self {
            message: message.into(),
            json_summary: None,
        }
    }

    pub(crate) fn with_json_summary(message: impl Into<String>, json_summary: Value) -> Self {
        Self {
            message: message.into(),
            json_summary: Some(json_summary),
        }
    }

    pub fn json_summary(&self) -> Option<&Value> {
        self.json_summary.as_ref()
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
            maintainer_views: BTreeMap::new(),
            profile_views: BTreeMap::new(),
            document_views: BTreeMap::new(),
            profile_heads: BTreeMap::new(),
            doc_heads: BTreeMap::new(),
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

impl StoreInitSummary {
    pub fn is_ok(&self) -> bool {
        self.errors.is_empty()
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
    sort_string_map_values(&mut summary.maintainer_views);
    sort_string_map_values(&mut summary.profile_views);
    sort_string_map_values(&mut summary.document_views);
    sort_profile_heads(&mut summary.profile_heads);

    let all_parent_ids: BTreeSet<String> = summary
        .revision_parents
        .values()
        .flatten()
        .cloned()
        .collect();
    for (doc_id, revision_ids) in &summary.doc_revisions {
        let mut heads: Vec<String> = revision_ids
            .iter()
            .filter(|revision_id| !all_parent_ids.contains(*revision_id))
            .cloned()
            .collect();
        heads.sort();
        heads.dedup();
        if !heads.is_empty() {
            summary.doc_heads.insert(doc_id.clone(), heads);
        }
    }

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
    ensure_local_store_policy_file(&store_root)?;

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

pub fn initialize_store_root(store_root: &Path) -> Result<StoreInitSummary, StoreRebuildError> {
    let store_root = normalize_path(store_root);
    ensure_store_root(&store_root)?;

    let objects_dir = objects_root(&store_root);
    fs::create_dir_all(&objects_dir).map_err(|error| {
        StoreRebuildError::new(format!(
            "failed to create store objects directory {}: {error}",
            objects_dir.display()
        ))
    })?;

    let indexes_dir = indexes_root(&store_root);
    fs::create_dir_all(&indexes_dir).map_err(|error| {
        StoreRebuildError::new(format!(
            "failed to create store index directory {}: {error}",
            indexes_dir.display()
        ))
    })?;
    let local_policy_path = ensure_local_store_policy_file(&store_root)?;

    let manifest = StoreIndexManifest {
        version: "mycel-store-index/0.1".to_string(),
        stored_object_count: 0,
        object_ids_by_type: BTreeMap::new(),
        doc_revisions: BTreeMap::new(),
        revision_parents: BTreeMap::new(),
        author_patches: BTreeMap::new(),
        view_governance: Vec::new(),
        maintainer_views: BTreeMap::new(),
        profile_views: BTreeMap::new(),
        document_views: BTreeMap::new(),
        profile_heads: BTreeMap::new(),
        doc_heads: BTreeMap::new(),
    };
    let manifest_path = write_store_index_manifest(&store_root, &manifest)?;

    Ok(StoreInitSummary {
        store_root,
        status: "ok".to_string(),
        index_manifest_path: manifest_path,
        local_policy_path,
        notes: Vec::new(),
        errors: Vec::new(),
    })
}

pub fn load_store_object_index(
    store_root: &Path,
) -> Result<HashMap<String, Value>, StoreRebuildError> {
    let store_root = normalize_path(store_root);
    load_object_index_from_store(&store_root)
}

pub fn write_object_value_to_store(
    store_root: &Path,
    value: &Value,
) -> Result<StoreWriteSummary, StoreRebuildError> {
    let store_root = normalize_path(store_root);
    ensure_store_root(&store_root)?;
    ensure_local_store_policy_file(&store_root)?;

    let loaded_object = loaded_object_from_inline_value(value)?;
    let object_index = load_object_index_from_store(&store_root)?;
    let verification = verify_object_value_with_object_index(value, Some(&object_index));
    if !verification.is_ok() {
        return Err(StoreRebuildError::new(verification.errors.join("; ")));
    }

    let record = stored_record_from_loaded(&loaded_object)?.ok_or_else(|| {
        StoreRebuildError::new(format!(
            "object '{}' does not expose a storable object ID",
            loaded_object.object_type
        ))
    })?;

    let created = match write_stored_object(&store_root, &loaded_object, &record)? {
        WriteOutcome::Written(_) => true,
        WriteOutcome::AlreadyPresent(_) => false,
    };
    let rebuilt = rebuild_store_from_path(&store_root)?;

    Ok(StoreWriteSummary {
        store_root: store_root.clone(),
        record: StoredObjectRecord {
            path: store_path_for_record(&store_root, &record)?,
            ..record
        },
        created,
        index_manifest_path: rebuilt.index_manifest_path,
    })
}

#[derive(Debug, Clone)]
struct LoadedObject {
    path: PathBuf,
    value: Value,
    object_type: String,
    object_id: Option<String>,
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

fn local_root(store_root: &Path) -> PathBuf {
    store_root.join("local")
}

fn store_index_manifest_path(store_root: &Path) -> PathBuf {
    indexes_root(store_root).join("manifest.json")
}

pub fn local_store_policy_path(store_root: &Path) -> PathBuf {
    local_root(store_root).join("policy.json")
}

pub fn persist_local_store_policy(
    store_root: &Path,
    policy: &LocalStorePolicy,
) -> Result<PathBuf, StoreRebuildError> {
    let store_root = normalize_path(store_root);
    ensure_store_root(&store_root)?;

    let local_dir = local_root(&store_root);
    fs::create_dir_all(&local_dir).map_err(|error| {
        StoreRebuildError::new(format!(
            "failed to create local policy directory {}: {error}",
            local_dir.display()
        ))
    })?;

    let policy_path = local_store_policy_path(&store_root);
    let rendered = serde_json::to_string_pretty(policy).map_err(|error| {
        StoreRebuildError::new(format!(
            "failed to serialize local store policy {}: {error}",
            policy_path.display()
        ))
    })?;
    fs::write(&policy_path, rendered).map_err(|error| {
        StoreRebuildError::new(format!(
            "failed to write local store policy {}: {error}",
            policy_path.display()
        ))
    })?;

    Ok(policy_path)
}

pub fn load_local_store_policy(store_root: &Path) -> Result<LocalStorePolicy, StoreRebuildError> {
    let store_root = normalize_path(store_root);
    let policy_path = local_store_policy_path(&store_root);
    let content = fs::read_to_string(&policy_path).map_err(|error| {
        StoreRebuildError::new(format!(
            "failed to read local store policy {}: {error}",
            policy_path.display()
        ))
    })?;
    parse_json_strict(&content).map_err(|error| {
        StoreRebuildError::new(format!(
            "failed to parse local store policy {}: {error}",
            policy_path.display()
        ))
    })
}

fn ensure_local_store_policy_file(store_root: &Path) -> Result<PathBuf, StoreRebuildError> {
    let policy_path = local_store_policy_path(store_root);
    if policy_path.exists() {
        return Ok(policy_path);
    }

    persist_local_store_policy(store_root, &LocalStorePolicy::default())
}

fn write_store_index_manifest(
    store_root: &Path,
    manifest: &StoreIndexManifest,
) -> Result<PathBuf, StoreRebuildError> {
    let manifest_path = store_index_manifest_path(store_root);
    let rendered = serde_json::to_string_pretty(manifest).map_err(|error| {
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

fn loaded_object_from_inline_value(value: &Value) -> Result<LoadedObject, StoreRebuildError> {
    let object_type = crate::protocol::parse_object_envelope(value)
        .map_err(|error| {
            StoreRebuildError::new(format!("failed to parse object envelope: {error}"))
        })?
        .object_type()
        .to_string();
    let object_id = object_id_from_value(value)
        .map_err(|error| StoreRebuildError::new(format!("failed to resolve object ID: {error}")))?;

    Ok(LoadedObject {
        path: PathBuf::from("<inline-object>"),
        value: value.clone(),
        object_type,
        object_id,
    })
}

fn object_id_from_value(value: &Value) -> Result<Option<String>, String> {
    let envelope =
        crate::protocol::parse_object_envelope(value).map_err(|error| error.to_string())?;
    if let Some(derived_id) = envelope
        .declared_id()
        .map_err(|error| format!("{error:?}"))?
    {
        return Ok(Some(derived_id.to_string()));
    }
    if let Some(logical_id) = envelope
        .logical_id()
        .map_err(|error| format!("{error:?}"))?
    {
        return Ok(Some(logical_id.to_string()));
    }
    Ok(None)
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
        let existing_value: Value = parse_json_value_strict(&existing).map_err(|error| {
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

    let looks_like_store_root = objects_root(target).exists()
        || indexes_root(target).exists()
        || local_root(target).exists()
        || local_store_policy_path(target).exists();
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

fn load_object_index_from_store(
    store_root: &Path,
) -> Result<HashMap<String, Value>, StoreRebuildError> {
    let discovery = discover_store_paths(store_root, "store root")?;
    let mut summary = StoreRebuildSummary::new(store_root);
    let loaded = load_objects(&discovery.json_paths, SummaryAdapter::Rebuild(&mut summary))?;
    let mut object_index = HashMap::new();

    for loaded_object in loaded {
        if let Some(object_id) = loaded_object.object_id {
            if object_index
                .insert(object_id.clone(), loaded_object.value.clone())
                .is_some()
            {
                return Err(StoreRebuildError::new(format!(
                    "duplicate stored object ID '{}' found while loading store object index",
                    object_id
                )));
            }
        }
    }

    Ok(object_index)
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
        let value: Value = parse_json_value_strict(&content).map_err(|error| {
            StoreRebuildError::new(format!(
                "failed to parse object JSON {}: {error}",
                path.display()
            ))
        })?;
        let (object_type, declared_id) = {
            let object_id = object_id_from_value(&value).map_err(|error| {
                StoreRebuildError::new(format!(
                    "{}: failed to resolve object ID: {error}",
                    path.display()
                ))
            })?;
            let envelope = crate::protocol::parse_object_envelope(&value).map_err(|error| {
                StoreRebuildError::new(format!(
                    "failed to parse object envelope {}: {error}",
                    path.display()
                ))
            })?;
            (envelope.object_type().to_string(), object_id)
        };
        loaded.push(LoadedObject {
            path: path.clone(),
            value,
            object_type,
            object_id: declared_id,
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
        if let Some(object_id) = &loaded_object.object_id {
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
    let Some(object_id) = &loaded_object.object_id else {
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
            summary
                .maintainer_views
                .entry(view.maintainer.clone())
                .or_default()
                .push(view.view_id.clone());
            summary
                .profile_views
                .entry(profile_id.clone())
                .or_default()
                .push(view.view_id.clone());
            for (doc_id, revision_id) in &documents {
                summary
                    .document_views
                    .entry(doc_id.clone())
                    .or_default()
                    .push(view.view_id.clone());
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
    prefixed_canonical_hash(value, "hash")
        .map_err(|error| StoreRebuildError::new(format!("failed to canonicalize value: {error}")))
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
            maintainer_views: summary.maintainer_views.clone(),
            profile_views: summary.profile_views.clone(),
            document_views: summary.document_views.clone(),
            profile_heads: summary.profile_heads.clone(),
            doc_heads: summary.doc_heads.clone(),
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
    write_store_index_manifest(store_root, &manifest)
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
    parse_json_strict(&content).map_err(|error| {
        StoreRebuildError::new(format!(
            "failed to parse store index manifest {}: {error}",
            manifest_path.display()
        ))
    })
}

/// Load all objects needed to replay any revision of a document from the store.
///
/// Uses the store index manifest to find all revision IDs for the document,
/// then loads those revisions and their referenced patches by object ID. The
/// returned index is scoped to the document and is suitable for passing to
/// `replay_revision_from_index`.
///
/// This is more efficient than `load_store_object_index` for single-document
/// reader and render workflows because it loads only the objects relevant to
/// the requested document instead of the entire store.
pub fn load_doc_replay_objects_from_store(
    store_root: &Path,
    doc_id: &str,
) -> Result<HashMap<String, Value>, StoreRebuildError> {
    let manifest = load_store_index_manifest(store_root)?;
    let revision_ids = manifest
        .doc_revisions
        .get(doc_id)
        .cloned()
        .unwrap_or_default();

    let mut objects: HashMap<String, Value> = HashMap::new();

    for revision_id in &revision_ids {
        let revision_value = load_stored_object_value(store_root, revision_id)?;

        if let Some(patches) = revision_value.get("patches").and_then(|v| v.as_array()) {
            for patch_id_value in patches {
                if let Some(patch_id) = patch_id_value.as_str() {
                    if !objects.contains_key(patch_id) {
                        // Skip patches that are not yet stored; replay will detect
                        // the gap and emit the canonical "missing patch for replay"
                        // error rather than failing here with a bare I/O error.
                        if let Ok(patch) = load_stored_object_value(store_root, patch_id) {
                            objects.insert(patch_id.to_string(), patch);
                        }
                    }
                }
            }
        }

        objects.insert(revision_id.clone(), revision_value);
    }

    Ok(objects)
}

pub fn load_stored_object_value(
    store_root: &Path,
    object_id: &str,
) -> Result<Value, StoreRebuildError> {
    let (object_prefix, object_hash) = object_id.split_once(':').ok_or_else(|| {
        StoreRebuildError::new(format!(
            "stored object ID '{}' is missing type prefix separator",
            object_id
        ))
    })?;
    let object_type = match object_prefix {
        "rev" => "revision",
        "doc" => "document",
        "blk" => "block",
        other => other,
    };
    let object_path = objects_root(store_root)
        .join(object_type)
        .join(format!("{object_hash}.json"));
    let content = fs::read_to_string(&object_path).map_err(|error| {
        StoreRebuildError::new(format!(
            "failed to read stored object {}: {error}",
            object_path.display()
        ))
    })?;
    parse_json_value_strict(&content).map_err(|error| {
        StoreRebuildError::new(format!(
            "failed to parse stored object {}: {error}",
            object_path.display()
        ))
    })
}

#[cfg(test)]
mod tests {
    use std::collections::BTreeMap;
    use std::fs;
    use std::path::PathBuf;

    use base64::Engine;
    use ed25519_dalek::{Signer, SigningKey};
    use serde_json::{json, Value};

    use super::{
        ingest_store_from_path, initialize_store_root, load_local_store_policy,
        load_store_index_manifest, load_store_object_index, load_stored_object_value,
        local_store_policy_path, persist_local_store_policy, rebuild_store_from_path,
        LocalStorePolicy,
    };
    use crate::canonical::{prefixed_canonical_hash, signed_payload_bytes};
    use crate::protocol::recompute_object_id;
    use crate::replay::{compute_state_hash, replay_revision_from_index, DocumentState};

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

    fn block(block_id: &str, content: &str) -> Value {
        json!({
            "block_id": block_id,
            "block_type": "paragraph",
            "content": content,
            "attrs": {},
            "children": []
        })
    }

    fn recompute_id(value: &Value, id_field: &str, prefix: &str) -> String {
        recompute_object_id(value, id_field, prefix).expect("test object ID should recompute")
    }

    fn sign_value(signing_key: &SigningKey, value: &Value) -> String {
        let payload = signed_payload_bytes(value).expect("payload should canonicalize");
        let signature = signing_key.sign(&payload);
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
        let state_hash =
            prefixed_canonical_hash(&state, "hash").expect("state hash should compute");

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
        assert_eq!(summary.maintainer_views.len(), 1);
        assert_eq!(summary.profile_views.len(), 1);
        assert_eq!(summary.document_views.len(), 1);
        assert_eq!(summary.profile_heads.len(), 1);

        let _ = fs::remove_dir_all(dir);
    }

    #[test]
    fn rebuild_store_reports_duplicate_declared_object_ids() {
        let dir = write_temp_dir("rebuild-duplicate-ids");
        let patch = signed_object(
            json!({
                "type": "patch",
                "version": "mycel/0.1",
                "doc_id": "doc:duplicate",
                "base_revision": "rev:genesis-null",
                "timestamp": 1u64,
                "ops": []
            }),
            "author",
            "patch_id",
            "patch",
        );
        fs::write(
            dir.join("patch-a.json"),
            serde_json::to_string_pretty(&patch).expect("patch should serialize"),
        )
        .expect("first patch should write");
        fs::write(
            dir.join("patch-b.json"),
            serde_json::to_string_pretty(&patch).expect("patch should serialize"),
        )
        .expect("second patch should write");

        let summary = rebuild_store_from_path(&dir).expect("store rebuild should complete");

        assert!(!summary.is_ok(), "expected failed summary, got {summary:?}");
        assert!(
            summary.errors.iter().any(|message| {
                message.contains("duplicate declared object ID")
                    && message.contains(patch["patch_id"].as_str().unwrap())
            }),
            "expected duplicate object ID error, got {summary:?}"
        );

        let _ = fs::remove_dir_all(dir);
    }

    #[test]
    fn rebuild_store_reports_missing_revision_patch_dependency() {
        let dir = write_temp_dir("rebuild-missing-patch");
        let revision = signed_object(
            json!({
                "type": "revision",
                "version": "mycel/0.1",
                "doc_id": "doc:missing-patch",
                "parents": [],
                "patches": ["patch:missing"],
                "state_hash": "hash:missing",
                "timestamp": 2u64
            }),
            "author",
            "revision_id",
            "rev",
        );
        fs::write(
            dir.join("revision.json"),
            serde_json::to_string_pretty(&revision).expect("revision should serialize"),
        )
        .expect("revision should write");

        let summary = rebuild_store_from_path(&dir).expect("store rebuild should complete");

        assert!(!summary.is_ok(), "expected failed summary, got {summary:?}");
        assert!(
            summary.errors.iter().any(|message| {
                message.contains("revision replay failed: missing patch 'patch:missing' for replay")
            }),
            "expected missing patch replay error, got {summary:?}"
        );
        assert_eq!(summary.verified_object_count, 0);
        assert_eq!(summary.stored_object_count, 0);

        let _ = fs::remove_dir_all(dir);
    }

    #[test]
    fn rebuild_store_reports_missing_parent_revision_dependency() {
        let dir = write_temp_dir("rebuild-missing-parent");
        let revision = signed_object(
            json!({
                "type": "revision",
                "version": "mycel/0.1",
                "doc_id": "doc:missing-parent",
                "parents": ["rev:missing-parent"],
                "patches": [],
                "state_hash": "hash:missing-parent",
                "timestamp": 2u64
            }),
            "author",
            "revision_id",
            "rev",
        );
        fs::write(
            dir.join("revision.json"),
            serde_json::to_string_pretty(&revision).expect("revision should serialize"),
        )
        .expect("revision should write");

        let summary = rebuild_store_from_path(&dir).expect("store rebuild should complete");

        assert!(!summary.is_ok(), "expected failed summary, got {summary:?}");
        assert!(
            summary.errors.iter().any(|message| {
                message.contains(
                    "revision replay failed: missing parent revision 'rev:missing-parent' for replay",
                )
            }),
            "expected missing parent replay error, got {summary:?}"
        );
        assert_eq!(summary.verified_object_count, 0);
        assert_eq!(summary.stored_object_count, 0);

        let _ = fs::remove_dir_all(dir);
    }

    #[test]
    fn rebuild_store_reports_cross_document_parent_revision_dependency() {
        let dir = write_temp_dir("rebuild-cross-doc-parent");
        let parent_state = json!({
            "doc_id": "doc:parent",
            "blocks": []
        });
        let parent_state_hash = prefixed_canonical_hash(&parent_state, "hash")
            .expect("parent state hash should compute");

        let parent_revision = signed_object(
            json!({
                "type": "revision",
                "version": "mycel/0.1",
                "doc_id": "doc:parent",
                "parents": [],
                "patches": [],
                "state_hash": parent_state_hash,
                "timestamp": 1u64
            }),
            "author",
            "revision_id",
            "rev",
        );
        let parent_revision_id = parent_revision["revision_id"]
            .as_str()
            .expect("parent revision id should exist")
            .to_string();
        fs::write(
            dir.join("parent-revision.json"),
            serde_json::to_string_pretty(&parent_revision)
                .expect("parent revision should serialize"),
        )
        .expect("parent revision should write");

        let child_revision = signed_object(
            json!({
                "type": "revision",
                "version": "mycel/0.1",
                "doc_id": "doc:child",
                "parents": [parent_revision_id.clone()],
                "patches": [],
                "state_hash": "hash:child",
                "timestamp": 2u64
            }),
            "author",
            "revision_id",
            "rev",
        );
        fs::write(
            dir.join("child-revision.json"),
            serde_json::to_string_pretty(&child_revision).expect("child revision should serialize"),
        )
        .expect("child revision should write");

        let summary = rebuild_store_from_path(&dir).expect("store rebuild should complete");

        assert!(!summary.is_ok(), "expected failed summary, got {summary:?}");
        assert!(
            summary.errors.iter().any(|message| {
                message.contains("revision replay failed: parent revision")
                    && message.contains("belongs to 'doc:parent' instead of 'doc:child'")
            }),
            "expected cross-document parent replay error, got {summary:?}"
        );
        assert_eq!(summary.verified_object_count, 1);
        assert_eq!(summary.stored_object_count, 1);

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
        let state_hash =
            prefixed_canonical_hash(&state, "hash").expect("state hash should compute");

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
    fn replay_rebuilds_document_state_from_stored_objects_only() {
        let source_dir = write_temp_dir("replay-store-source");
        let store_dir = write_temp_dir("replay-store-target");
        let signer = signing_key();
        let author = signer_id(&signer);

        let base_patch = signed_object(
            json!({
                "type": "patch",
                "version": "mycel/0.1",
                "doc_id": "doc:store-replay",
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
        let base_patch_id = base_patch["patch_id"]
            .as_str()
            .expect("base patch id should exist")
            .to_string();
        fs::write(
            source_dir.join("patch-base.json"),
            serde_json::to_string_pretty(&base_patch).expect("base patch should serialize"),
        )
        .expect("base patch should write");

        let base_state = DocumentState {
            doc_id: "doc:store-replay".to_string(),
            blocks: vec![
                crate::protocol::parse_block_object(&block("blk:001", "Hello"))
                    .expect("base block should parse"),
            ],
            metadata: serde_json::Map::new(),
        };
        let base_revision = signed_object(
            json!({
                "type": "revision",
                "version": "mycel/0.1",
                "doc_id": "doc:store-replay",
                "parents": [],
                "patches": [base_patch_id.clone()],
                "state_hash": compute_state_hash(&base_state).expect("base state hash should compute"),
                "timestamp": 2u64
            }),
            "author",
            "revision_id",
            "rev",
        );
        let base_revision_id = base_revision["revision_id"]
            .as_str()
            .expect("base revision id should exist")
            .to_string();
        fs::write(
            source_dir.join("revision-base.json"),
            serde_json::to_string_pretty(&base_revision).expect("base revision should serialize"),
        )
        .expect("base revision should write");

        let child_patch = signed_object(
            json!({
                "type": "patch",
                "version": "mycel/0.1",
                "doc_id": "doc:store-replay",
                "base_revision": base_revision_id.clone(),
                "timestamp": 3u64,
                "ops": [
                    {
                        "op": "replace_block",
                        "block_id": "blk:001",
                        "new_content": "Hello from store"
                    }
                ]
            }),
            "author",
            "patch_id",
            "patch",
        );
        let child_patch_id = child_patch["patch_id"]
            .as_str()
            .expect("child patch id should exist")
            .to_string();
        fs::write(
            source_dir.join("patch-child.json"),
            serde_json::to_string_pretty(&child_patch).expect("child patch should serialize"),
        )
        .expect("child patch should write");

        let expected_state = DocumentState {
            doc_id: "doc:store-replay".to_string(),
            blocks: vec![crate::protocol::parse_block_object(&block(
                "blk:001",
                "Hello from store",
            ))
            .expect("expected block should parse")],
            metadata: serde_json::Map::new(),
        };
        let child_revision = signed_object(
            json!({
                "type": "revision",
                "version": "mycel/0.1",
                "doc_id": "doc:store-replay",
                "parents": [base_revision_id.clone()],
                "patches": [child_patch_id.clone()],
                "state_hash": compute_state_hash(&expected_state).expect("child state hash should compute"),
                "timestamp": 4u64
            }),
            "author",
            "revision_id",
            "rev",
        );
        let child_revision_id = child_revision["revision_id"]
            .as_str()
            .expect("child revision id should exist")
            .to_string();
        fs::write(
            source_dir.join("revision-child.json"),
            serde_json::to_string_pretty(&child_revision).expect("child revision should serialize"),
        )
        .expect("child revision should write");

        let ingest =
            ingest_store_from_path(&source_dir, &store_dir).expect("store ingest should succeed");
        assert!(ingest.is_ok(), "expected ok ingest summary, got {ingest:?}");

        let manifest =
            load_store_index_manifest(&store_dir).expect("store manifest should be readable");
        let mut object_index = std::collections::HashMap::new();
        for object_ids in manifest.object_ids_by_type.values() {
            for object_id in object_ids {
                let value = load_stored_object_value(&store_dir, object_id)
                    .expect("stored object should be readable");
                object_index.insert(object_id.clone(), value);
            }
        }

        let replay_revision = load_stored_object_value(&store_dir, &child_revision_id)
            .expect("child revision should load from store");
        let replay = replay_revision_from_index(&replay_revision, &object_index)
            .expect("replay should work");

        assert_eq!(replay.revision_id, child_revision_id);
        assert_eq!(replay.state, expected_state);
        assert_eq!(
            replay.recomputed_state_hash,
            compute_state_hash(&expected_state).expect("expected state hash should compute")
        );
        assert_eq!(
            manifest.doc_revisions.get("doc:store-replay"),
            Some(&vec![base_revision_id, child_revision_id.clone()])
        );
        assert_eq!(
            manifest
                .author_patches
                .get(&author)
                .expect("author patch index should exist"),
            &vec![base_patch_id, child_patch_id]
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

    #[test]
    fn initialize_store_root_persists_local_policy_outside_manifest() {
        let store_dir = write_temp_dir("init-local-policy-store");

        let summary = initialize_store_root(&store_dir).expect("store init should succeed");
        assert!(summary.is_ok(), "expected ok init summary, got {summary:?}");
        assert!(summary.local_policy_path.exists());
        assert_eq!(
            summary.local_policy_path,
            local_store_policy_path(&store_dir)
        );

        let policy =
            load_local_store_policy(&store_dir).expect("default local policy should be readable");
        assert_eq!(policy, LocalStorePolicy::default());

        let manifest =
            load_store_index_manifest(&store_dir).expect("manifest should be readable after init");
        let manifest_value =
            serde_json::to_value(&manifest).expect("manifest should serialize to JSON");
        assert!(manifest_value.get("transport").is_none());
        assert!(manifest_value.get("safety").is_none());

        let _ = fs::remove_dir_all(store_dir);
    }

    #[test]
    fn rebuild_store_preserves_existing_local_policy_file() {
        let source_dir = write_temp_dir("rebuild-local-policy-source");
        let store_dir = write_temp_dir("rebuild-local-policy-store");
        let patch = signed_object(
            json!({
                "type": "patch",
                "version": "mycel/0.1",
                "doc_id": "doc:local-policy",
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

        initialize_store_root(&store_dir).expect("store init should succeed");
        let custom_policy = LocalStorePolicy {
            version: "mycel-local-policy/0.1".to_string(),
            transport: BTreeMap::from([(
                "preferred_peer".to_string(),
                Value::String("node:relay-a".to_string()),
            )]),
            safety: BTreeMap::from([("require_manual_review".to_string(), Value::Bool(true))]),
        };
        persist_local_store_policy(&store_dir, &custom_policy)
            .expect("custom local policy should persist");
        let original_policy_bytes = fs::read(local_store_policy_path(&store_dir))
            .expect("local policy should be readable before rebuild");

        ingest_store_from_path(&source_dir, &store_dir).expect("ingest should succeed");
        rebuild_store_from_path(&store_dir).expect("rebuild should succeed");

        let reloaded_policy =
            load_local_store_policy(&store_dir).expect("custom local policy should still load");
        assert_eq!(reloaded_policy, custom_policy);
        let rebuilt_policy_bytes = fs::read(local_store_policy_path(&store_dir))
            .expect("local policy should be readable after rebuild");
        assert_eq!(rebuilt_policy_bytes, original_policy_bytes);

        let manifest = load_store_index_manifest(&store_dir)
            .expect("manifest should be readable after rebuild");
        let manifest_value =
            serde_json::to_value(&manifest).expect("manifest should serialize to JSON");
        assert!(manifest_value.get("transport").is_none());
        assert!(manifest_value.get("safety").is_none());

        let _ = fs::remove_dir_all(source_dir);
        let _ = fs::remove_dir_all(store_dir);
    }

    #[test]
    fn load_store_object_index_ignores_local_policy_before_objects_exist() {
        let store_dir = write_temp_dir("load-empty-store-index");

        initialize_store_root(&store_dir).expect("store init should succeed");
        let object_index =
            load_store_object_index(&store_dir).expect("store object index should load");
        assert!(object_index.is_empty());

        let _ = fs::remove_dir_all(store_dir);
    }

    #[test]
    fn load_doc_replay_objects_loads_revisions_and_patches_for_doc() {
        use crate::author::{
            commit_revision_to_store, create_document_in_store, create_patch_in_store,
            parse_signing_key_seed, DocumentCreateParams, PatchCreateParams, RevisionCommitParams,
        };

        let store_dir = write_temp_dir("doc-replay-objects");
        let key_seed = base64::engine::general_purpose::STANDARD.encode([7u8; 32]);
        let signing_key = parse_signing_key_seed(&key_seed).expect("signing key should parse");

        let doc = create_document_in_store(
            &store_dir,
            &signing_key,
            &DocumentCreateParams {
                doc_id: "doc:replay-index".to_string(),
                title: "Replay Index".to_string(),
                language: "en".to_string(),
                timestamp: 1,
            },
        )
        .expect("document should be created");

        let patch = create_patch_in_store(
            &store_dir,
            &signing_key,
            &PatchCreateParams {
                doc_id: "doc:replay-index".to_string(),
                base_revision: doc.genesis_revision_id.clone(),
                timestamp: 2,
                ops: serde_json::json!([{
                    "op": "insert_block",
                    "new_block": {
                        "block_id": "blk:ri-001",
                        "block_type": "paragraph",
                        "content": "Hello",
                        "attrs": {},
                        "children": []
                    }
                }]),
            },
        )
        .expect("patch should be created");

        let revision = commit_revision_to_store(
            &store_dir,
            &signing_key,
            &RevisionCommitParams {
                doc_id: "doc:replay-index".to_string(),
                parents: vec![doc.genesis_revision_id.clone()],
                patches: vec![patch.patch_id.clone()],
                merge_strategy: None,
                timestamp: 3,
            },
        )
        .expect("revision should be committed");

        let objects = super::load_doc_replay_objects_from_store(&store_dir, "doc:replay-index")
            .expect("doc replay objects should load");

        assert!(
            objects.contains_key(&doc.genesis_revision_id),
            "should contain genesis revision"
        );
        assert!(
            objects.contains_key(&revision.revision_id),
            "should contain committed revision"
        );
        assert!(
            objects.contains_key(&patch.patch_id),
            "should contain patch referenced by revision"
        );

        let _ = fs::remove_dir_all(store_dir);
    }

    #[test]
    fn load_doc_replay_objects_supports_replay_of_stored_revision() {
        use crate::author::{
            commit_revision_to_store, create_document_in_store, create_patch_in_store,
            parse_signing_key_seed, DocumentCreateParams, PatchCreateParams, RevisionCommitParams,
        };

        let store_dir = write_temp_dir("doc-replay-verify");
        let key_seed = base64::engine::general_purpose::STANDARD.encode([7u8; 32]);
        let signing_key = parse_signing_key_seed(&key_seed).expect("signing key should parse");

        let doc = create_document_in_store(
            &store_dir,
            &signing_key,
            &DocumentCreateParams {
                doc_id: "doc:replay-verify".to_string(),
                title: "Replay Verify".to_string(),
                language: "en".to_string(),
                timestamp: 10,
            },
        )
        .expect("document should be created");

        let patch = create_patch_in_store(
            &store_dir,
            &signing_key,
            &PatchCreateParams {
                doc_id: "doc:replay-verify".to_string(),
                base_revision: doc.genesis_revision_id.clone(),
                timestamp: 11,
                ops: serde_json::json!([{
                    "op": "insert_block",
                    "new_block": {
                        "block_id": "blk:rv-001",
                        "block_type": "paragraph",
                        "content": "Replay content",
                        "attrs": {},
                        "children": []
                    }
                }]),
            },
        )
        .expect("patch should be created");

        let revision = commit_revision_to_store(
            &store_dir,
            &signing_key,
            &RevisionCommitParams {
                doc_id: "doc:replay-verify".to_string(),
                parents: vec![doc.genesis_revision_id.clone()],
                patches: vec![patch.patch_id.clone()],
                merge_strategy: None,
                timestamp: 12,
            },
        )
        .expect("revision should be committed");

        let objects = super::load_doc_replay_objects_from_store(&store_dir, "doc:replay-verify")
            .expect("doc replay objects should load");

        let revision_value = load_stored_object_value(&store_dir, &revision.revision_id)
            .expect("revision should load");
        let replay = replay_revision_from_index(&revision_value, &objects)
            .expect("replay should succeed with doc-scoped object index");

        assert_eq!(replay.revision_id, revision.revision_id);
        assert_eq!(replay.state.doc_id, "doc:replay-verify");
        assert_eq!(replay.state.blocks.len(), 1);
        assert_eq!(replay.state.blocks[0].content, "Replay content");
        assert_eq!(replay.recomputed_state_hash, revision.recomputed_state_hash);

        let _ = fs::remove_dir_all(store_dir);
    }

    #[test]
    fn load_doc_replay_objects_returns_empty_for_unknown_doc() {
        let store_dir = write_temp_dir("doc-replay-unknown");
        initialize_store_root(&store_dir).expect("store init should succeed");

        let objects = super::load_doc_replay_objects_from_store(&store_dir, "doc:nonexistent")
            .expect("unknown doc should return empty objects without error");

        assert!(
            objects.is_empty(),
            "unknown doc should yield empty object map"
        );

        let _ = fs::remove_dir_all(store_dir);
    }

    #[test]
    fn rebuild_store_indexes_two_independent_documents() {
        use crate::author::{
            commit_revision_to_store, create_document_in_store, create_patch_in_store,
            parse_signing_key_seed, DocumentCreateParams, PatchCreateParams, RevisionCommitParams,
        };

        let store_dir = write_temp_dir("rebuild-two-docs");
        let key_seed = base64::engine::general_purpose::STANDARD.encode([9u8; 32]);
        let signing_key = parse_signing_key_seed(&key_seed).expect("signing key should parse");

        // Document A with one patch and one revision
        let doc_a = create_document_in_store(
            &store_dir,
            &signing_key,
            &DocumentCreateParams {
                doc_id: "doc:multi-a".to_string(),
                title: "Doc A".to_string(),
                language: "en".to_string(),
                timestamp: 1,
            },
        )
        .expect("doc A should be created");

        let patch_a = create_patch_in_store(
            &store_dir,
            &signing_key,
            &PatchCreateParams {
                doc_id: "doc:multi-a".to_string(),
                base_revision: doc_a.genesis_revision_id.clone(),
                timestamp: 2,
                ops: serde_json::json!([{
                    "op": "insert_block",
                    "new_block": {
                        "block_id": "blk:ma-001",
                        "block_type": "paragraph",
                        "content": "Doc A content",
                        "attrs": {},
                        "children": []
                    }
                }]),
            },
        )
        .expect("patch A should be created");

        let rev_a = commit_revision_to_store(
            &store_dir,
            &signing_key,
            &RevisionCommitParams {
                doc_id: "doc:multi-a".to_string(),
                parents: vec![doc_a.genesis_revision_id.clone()],
                patches: vec![patch_a.patch_id.clone()],
                merge_strategy: None,
                timestamp: 3,
            },
        )
        .expect("revision A should be committed");

        // Document B with one patch and one revision
        let doc_b = create_document_in_store(
            &store_dir,
            &signing_key,
            &DocumentCreateParams {
                doc_id: "doc:multi-b".to_string(),
                title: "Doc B".to_string(),
                language: "en".to_string(),
                timestamp: 4,
            },
        )
        .expect("doc B should be created");

        let patch_b = create_patch_in_store(
            &store_dir,
            &signing_key,
            &PatchCreateParams {
                doc_id: "doc:multi-b".to_string(),
                base_revision: doc_b.genesis_revision_id.clone(),
                timestamp: 5,
                ops: serde_json::json!([{
                    "op": "insert_block",
                    "new_block": {
                        "block_id": "blk:mb-001",
                        "block_type": "paragraph",
                        "content": "Doc B content",
                        "attrs": {},
                        "children": []
                    }
                }]),
            },
        )
        .expect("patch B should be created");

        let rev_b = commit_revision_to_store(
            &store_dir,
            &signing_key,
            &RevisionCommitParams {
                doc_id: "doc:multi-b".to_string(),
                parents: vec![doc_b.genesis_revision_id.clone()],
                patches: vec![patch_b.patch_id.clone()],
                merge_strategy: None,
                timestamp: 6,
            },
        )
        .expect("revision B should be committed");

        let summary = rebuild_store_from_path(&store_dir).expect("store rebuild should succeed");

        assert!(summary.is_ok(), "expected ok summary, got {summary:?}");

        // Both documents should be indexed independently
        let doc_a_revs = summary
            .doc_revisions
            .get("doc:multi-a")
            .expect("doc A revisions should be indexed");
        assert!(
            doc_a_revs.contains(&doc_a.genesis_revision_id),
            "doc A genesis revision should be indexed"
        );
        assert!(
            doc_a_revs.contains(&rev_a.revision_id),
            "doc A child revision should be indexed"
        );

        let doc_b_revs = summary
            .doc_revisions
            .get("doc:multi-b")
            .expect("doc B revisions should be indexed");
        assert!(
            doc_b_revs.contains(&doc_b.genesis_revision_id),
            "doc B genesis revision should be indexed"
        );
        assert!(
            doc_b_revs.contains(&rev_b.revision_id),
            "doc B child revision should be indexed"
        );

        // Doc A revisions should not appear in Doc B's index and vice versa
        assert!(
            !doc_a_revs.contains(&rev_b.revision_id),
            "doc B revision should not appear in doc A index"
        );
        assert!(
            !doc_b_revs.contains(&rev_a.revision_id),
            "doc A revision should not appear in doc B index"
        );

        // Both patches should be indexed under the author
        let author_patches = summary
            .author_patches
            .values()
            .flatten()
            .cloned()
            .collect::<Vec<_>>();
        assert!(
            author_patches.contains(&patch_a.patch_id),
            "patch A should be indexed under author"
        );
        assert!(
            author_patches.contains(&patch_b.patch_id),
            "patch B should be indexed under author"
        );

        let _ = fs::remove_dir_all(store_dir);
    }

    #[test]
    fn load_doc_replay_objects_does_not_cross_contaminate_documents() {
        use crate::author::{
            commit_revision_to_store, create_document_in_store, create_patch_in_store,
            parse_signing_key_seed, DocumentCreateParams, PatchCreateParams, RevisionCommitParams,
        };

        let store_dir = write_temp_dir("doc-replay-no-cross-contamination");
        let key_seed = base64::engine::general_purpose::STANDARD.encode([11u8; 32]);
        let signing_key = parse_signing_key_seed(&key_seed).expect("signing key should parse");

        // Populate doc A
        let doc_a = create_document_in_store(
            &store_dir,
            &signing_key,
            &DocumentCreateParams {
                doc_id: "doc:cc-alpha".to_string(),
                title: "Alpha".to_string(),
                language: "en".to_string(),
                timestamp: 1,
            },
        )
        .expect("doc alpha should be created");

        let patch_a = create_patch_in_store(
            &store_dir,
            &signing_key,
            &PatchCreateParams {
                doc_id: "doc:cc-alpha".to_string(),
                base_revision: doc_a.genesis_revision_id.clone(),
                timestamp: 2,
                ops: serde_json::json!([{
                    "op": "insert_block",
                    "new_block": {
                        "block_id": "blk:alpha-001",
                        "block_type": "paragraph",
                        "content": "Alpha text",
                        "attrs": {},
                        "children": []
                    }
                }]),
            },
        )
        .expect("patch alpha should be created");

        let rev_a = commit_revision_to_store(
            &store_dir,
            &signing_key,
            &RevisionCommitParams {
                doc_id: "doc:cc-alpha".to_string(),
                parents: vec![doc_a.genesis_revision_id.clone()],
                patches: vec![patch_a.patch_id.clone()],
                merge_strategy: None,
                timestamp: 3,
            },
        )
        .expect("revision alpha should be committed");

        // Populate doc B
        let doc_b = create_document_in_store(
            &store_dir,
            &signing_key,
            &DocumentCreateParams {
                doc_id: "doc:cc-beta".to_string(),
                title: "Beta".to_string(),
                language: "en".to_string(),
                timestamp: 4,
            },
        )
        .expect("doc beta should be created");

        let patch_b = create_patch_in_store(
            &store_dir,
            &signing_key,
            &PatchCreateParams {
                doc_id: "doc:cc-beta".to_string(),
                base_revision: doc_b.genesis_revision_id.clone(),
                timestamp: 5,
                ops: serde_json::json!([{
                    "op": "insert_block",
                    "new_block": {
                        "block_id": "blk:beta-001",
                        "block_type": "paragraph",
                        "content": "Beta text",
                        "attrs": {},
                        "children": []
                    }
                }]),
            },
        )
        .expect("patch beta should be created");

        let rev_b = commit_revision_to_store(
            &store_dir,
            &signing_key,
            &RevisionCommitParams {
                doc_id: "doc:cc-beta".to_string(),
                parents: vec![doc_b.genesis_revision_id.clone()],
                patches: vec![patch_b.patch_id.clone()],
                merge_strategy: None,
                timestamp: 6,
            },
        )
        .expect("revision beta should be committed");

        // Load objects scoped to doc alpha — must not include doc beta objects
        let alpha_objects = super::load_doc_replay_objects_from_store(&store_dir, "doc:cc-alpha")
            .expect("alpha replay objects should load");

        assert!(
            alpha_objects.contains_key(&doc_a.genesis_revision_id),
            "alpha genesis revision should be in alpha objects"
        );
        assert!(
            alpha_objects.contains_key(&rev_a.revision_id),
            "alpha child revision should be in alpha objects"
        );
        assert!(
            alpha_objects.contains_key(&patch_a.patch_id),
            "alpha patch should be in alpha objects"
        );
        assert!(
            !alpha_objects.contains_key(&doc_b.genesis_revision_id),
            "beta genesis revision must NOT appear in alpha objects"
        );
        assert!(
            !alpha_objects.contains_key(&rev_b.revision_id),
            "beta revision must NOT appear in alpha objects"
        );
        assert!(
            !alpha_objects.contains_key(&patch_b.patch_id),
            "beta patch must NOT appear in alpha objects"
        );

        // Load objects scoped to doc beta — must not include doc alpha objects
        let beta_objects = super::load_doc_replay_objects_from_store(&store_dir, "doc:cc-beta")
            .expect("beta replay objects should load");

        assert!(
            beta_objects.contains_key(&doc_b.genesis_revision_id),
            "beta genesis revision should be in beta objects"
        );
        assert!(
            beta_objects.contains_key(&rev_b.revision_id),
            "beta child revision should be in beta objects"
        );
        assert!(
            beta_objects.contains_key(&patch_b.patch_id),
            "beta patch should be in beta objects"
        );
        assert!(
            !beta_objects.contains_key(&doc_a.genesis_revision_id),
            "alpha genesis revision must NOT appear in beta objects"
        );
        assert!(
            !beta_objects.contains_key(&rev_a.revision_id),
            "alpha revision must NOT appear in beta objects"
        );
        assert!(
            !beta_objects.contains_key(&patch_a.patch_id),
            "alpha patch must NOT appear in beta objects"
        );

        let _ = fs::remove_dir_all(store_dir);
    }

    #[test]
    fn load_doc_replay_objects_supports_deep_revision_chain() {
        use crate::author::{
            commit_revision_to_store, create_document_in_store, create_patch_in_store,
            parse_signing_key_seed, DocumentCreateParams, PatchCreateParams, RevisionCommitParams,
        };

        let store_dir = write_temp_dir("doc-replay-deep-chain");
        let key_seed = base64::engine::general_purpose::STANDARD.encode([13u8; 32]);
        let signing_key = parse_signing_key_seed(&key_seed).expect("signing key should parse");

        // Create document with genesis revision
        let doc = create_document_in_store(
            &store_dir,
            &signing_key,
            &DocumentCreateParams {
                doc_id: "doc:deep-chain".to_string(),
                title: "Deep Chain".to_string(),
                language: "en".to_string(),
                timestamp: 1,
            },
        )
        .expect("document should be created");

        // Revision 1: insert block
        let patch1 = create_patch_in_store(
            &store_dir,
            &signing_key,
            &PatchCreateParams {
                doc_id: "doc:deep-chain".to_string(),
                base_revision: doc.genesis_revision_id.clone(),
                timestamp: 2,
                ops: serde_json::json!([{
                    "op": "insert_block",
                    "new_block": {
                        "block_id": "blk:dc-001",
                        "block_type": "paragraph",
                        "content": "Initial content",
                        "attrs": {},
                        "children": []
                    }
                }]),
            },
        )
        .expect("patch 1 should be created");

        let rev1 = commit_revision_to_store(
            &store_dir,
            &signing_key,
            &RevisionCommitParams {
                doc_id: "doc:deep-chain".to_string(),
                parents: vec![doc.genesis_revision_id.clone()],
                patches: vec![patch1.patch_id.clone()],
                merge_strategy: None,
                timestamp: 3,
            },
        )
        .expect("revision 1 should be committed");

        // Revision 2: replace block content
        let patch2 = create_patch_in_store(
            &store_dir,
            &signing_key,
            &PatchCreateParams {
                doc_id: "doc:deep-chain".to_string(),
                base_revision: rev1.revision_id.clone(),
                timestamp: 4,
                ops: serde_json::json!([{
                    "op": "replace_block",
                    "block_id": "blk:dc-001",
                    "new_content": "Updated content"
                }]),
            },
        )
        .expect("patch 2 should be created");

        let rev2 = commit_revision_to_store(
            &store_dir,
            &signing_key,
            &RevisionCommitParams {
                doc_id: "doc:deep-chain".to_string(),
                parents: vec![rev1.revision_id.clone()],
                patches: vec![patch2.patch_id.clone()],
                merge_strategy: None,
                timestamp: 5,
            },
        )
        .expect("revision 2 should be committed");

        // Load all objects for the document
        let objects = super::load_doc_replay_objects_from_store(&store_dir, "doc:deep-chain")
            .expect("deep chain replay objects should load");

        // All revisions and patches in the chain must be present
        assert!(
            objects.contains_key(&doc.genesis_revision_id),
            "genesis revision should be present"
        );
        assert!(
            objects.contains_key(&patch1.patch_id),
            "patch 1 should be present"
        );
        assert!(
            objects.contains_key(&rev1.revision_id),
            "revision 1 should be present"
        );
        assert!(
            objects.contains_key(&patch2.patch_id),
            "patch 2 should be present"
        );
        assert!(
            objects.contains_key(&rev2.revision_id),
            "revision 2 should be present"
        );

        // Replay the leaf revision (rev2) using the doc-scoped object index
        let rev2_value = load_stored_object_value(&store_dir, &rev2.revision_id)
            .expect("revision 2 should load from store");
        let replay = replay_revision_from_index(&rev2_value, &objects)
            .expect("deep-chain replay should succeed with doc-scoped object index");

        assert_eq!(replay.revision_id, rev2.revision_id);
        assert_eq!(replay.state.doc_id, "doc:deep-chain");
        assert_eq!(replay.state.blocks.len(), 1);
        assert_eq!(replay.state.blocks[0].content, "Updated content");
        assert_eq!(replay.recomputed_state_hash, rev2.recomputed_state_hash);

        let _ = fs::remove_dir_all(store_dir);
    }
}
