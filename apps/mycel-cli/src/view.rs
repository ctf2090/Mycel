use std::collections::BTreeMap;
use std::fs;
use std::path::{Path, PathBuf};

use clap::{Args, Subcommand, ValueEnum};
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
    #[command(about = "List persisted governance View records with optional filters")]
    List(ViewListCliArgs),
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
struct ViewListSummary {
    store_root: PathBuf,
    manifest_path: PathBuf,
    status: String,
    sort: ViewListSort,
    summary_only: bool,
    latest_per_profile: bool,
    limit: Option<usize>,
    record_count: usize,
    filters: ViewListFilters,
    group_by: Vec<ViewListGroupBy>,
    records: Vec<ViewListRecord>,
    groups: Vec<ViewListGroupSummary>,
    notes: Vec<String>,
    errors: Vec<String>,
}

#[derive(Debug, Clone, Serialize)]
struct ViewListFilters {
    view_id: Option<String>,
    profile_id: Option<String>,
    maintainer: Option<String>,
    doc_id: Option<String>,
    revision_id: Option<String>,
    timestamp_min: Option<u64>,
    timestamp_max: Option<u64>,
}

#[derive(Debug, Clone, Serialize)]
struct ViewListRecord {
    view_id: String,
    maintainer: String,
    profile_id: String,
    timestamp: u64,
    documents: BTreeMap<String, String>,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, ValueEnum)]
#[serde(rename_all = "kebab-case")]
enum ViewListSort {
    ViewId,
    TimestampAsc,
    TimestampDesc,
    ProfileId,
    Maintainer,
}

impl ViewListSort {
    fn as_str(self) -> &'static str {
        match self {
            Self::ViewId => "view-id",
            Self::TimestampAsc => "timestamp-asc",
            Self::TimestampDesc => "timestamp-desc",
            Self::ProfileId => "profile-id",
            Self::Maintainer => "maintainer",
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, ValueEnum)]
#[serde(rename_all = "kebab-case")]
enum ViewListGroupBy {
    ProfileId,
    Maintainer,
    DocId,
}

impl ViewListGroupBy {
    fn as_str(self) -> &'static str {
        match self {
            Self::ProfileId => "profile-id",
            Self::Maintainer => "maintainer",
            Self::DocId => "doc-id",
        }
    }
}

#[derive(Debug, Clone, Serialize)]
struct ViewListGroupSummary {
    group_by: ViewListGroupBy,
    groups: Vec<ViewListGroupBucket>,
}

#[derive(Debug, Clone, Serialize)]
struct ViewListGroupBucket {
    key: String,
    record_count: usize,
    latest_timestamp: u64,
    view_ids: Vec<String>,
}

#[derive(Args)]
struct ViewListCliArgs {
    #[arg(
        long,
        value_name = "STORE_ROOT",
        help = "Store root directory to read governance indexes from",
        required = true
    )]
    store_root: String,
    #[arg(long, help = "Only return one persisted view ID")]
    view_id: Option<String>,
    #[arg(long, help = "Only return one governance profile ID")]
    profile_id: Option<String>,
    #[arg(long, help = "Only return one governance maintainer key")]
    maintainer: Option<String>,
    #[arg(long, help = "Only return views that mention one document ID")]
    doc_id: Option<String>,
    #[arg(long, help = "Only return views that mention one revision ID")]
    revision_id: Option<String>,
    #[arg(
        long,
        value_name = "TIMESTAMP",
        help = "Only return views at or after one timestamp"
    )]
    timestamp_min: Option<u64>,
    #[arg(
        long,
        value_name = "TIMESTAMP",
        help = "Only return views at or before one timestamp"
    )]
    timestamp_max: Option<u64>,
    #[arg(
        long,
        value_enum,
        default_value_t = ViewListSort::TimestampDesc,
        help = "Sort listed governance records"
    )]
    sort: ViewListSort,
    #[arg(
        long,
        value_name = "COUNT",
        help = "Return at most this many records after projection"
    )]
    limit: Option<usize>,
    #[arg(
        long,
        help = "Keep only the latest governance record for each profile ID"
    )]
    latest_per_profile: bool,
    #[arg(long, help = "Omit per-record output and emit only summary metadata")]
    summary_only: bool,
    #[arg(
        long,
        value_name = "GROUP_BY",
        value_enum,
        help = "Emit grouped summaries by one governance field"
    )]
    group_by: Vec<ViewListGroupBy>,
    #[arg(long, help = "Emit machine-readable view listing output")]
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

impl ViewListSummary {
    fn new(
        store_root: &Path,
        filters: ViewListFilters,
        sort: ViewListSort,
        summary_only: bool,
        latest_per_profile: bool,
        limit: Option<usize>,
        group_by: Vec<ViewListGroupBy>,
    ) -> Self {
        Self {
            store_root: store_root.to_path_buf(),
            manifest_path: store_root.join("indexes").join("manifest.json"),
            status: "ok".to_string(),
            sort,
            summary_only,
            latest_per_profile,
            limit,
            record_count: 0,
            filters,
            group_by,
            records: Vec::new(),
            groups: Vec::new(),
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

fn print_view_list_text(summary: &ViewListSummary) -> i32 {
    println!("store root: {}", summary.store_root.display());
    println!("manifest path: {}", summary.manifest_path.display());
    println!("sort: {}", summary.sort.as_str());
    println!(
        "latest per profile: {}",
        if summary.latest_per_profile {
            "yes"
        } else {
            "no"
        }
    );
    println!(
        "summary only: {}",
        if summary.summary_only { "yes" } else { "no" }
    );
    if let Some(limit) = summary.limit {
        println!("limit: {limit}");
    }
    println!("record count: {}", summary.record_count);
    if let Some(view_id) = &summary.filters.view_id {
        println!("filter view_id: {view_id}");
    }
    if let Some(profile_id) = &summary.filters.profile_id {
        println!("filter profile_id: {profile_id}");
    }
    if let Some(maintainer) = &summary.filters.maintainer {
        println!("filter maintainer: {maintainer}");
    }
    if let Some(doc_id) = &summary.filters.doc_id {
        println!("filter doc_id: {doc_id}");
    }
    if let Some(revision_id) = &summary.filters.revision_id {
        println!("filter revision_id: {revision_id}");
    }
    if let Some(timestamp_min) = summary.filters.timestamp_min {
        println!("filter timestamp_min: {timestamp_min}");
    }
    if let Some(timestamp_max) = summary.filters.timestamp_max {
        println!("filter timestamp_max: {timestamp_max}");
    }
    if !summary.group_by.is_empty() {
        println!(
            "group by: {}",
            summary
                .group_by
                .iter()
                .map(|group_by| group_by.as_str())
                .collect::<Vec<_>>()
                .join(", ")
        );
    }
    for record in &summary.records {
        println!(
            "view: {} maintainer={} profile={} timestamp={} docs={}",
            record.view_id,
            record.maintainer,
            record.profile_id,
            record.timestamp,
            record.documents.len()
        );
    }
    for grouped in &summary.groups {
        println!("group summary: {}", grouped.group_by.as_str());
        for bucket in &grouped.groups {
            println!(
                "group: {} records={} latest_timestamp={} views={}",
                bucket.key,
                bucket.record_count,
                bucket.latest_timestamp,
                bucket.view_ids.join(", ")
            );
        }
    }
    for note in &summary.notes {
        println!("note: {note}");
    }

    if summary.is_ok() {
        println!("view list: ok");
        0
    } else {
        println!("view list: failed");
        for error in &summary.errors {
            emit_error_line(error);
        }
        1
    }
}

fn print_view_list_json(summary: &ViewListSummary) -> Result<i32, CliError> {
    match serde_json::to_string_pretty(summary) {
        Ok(json) => {
            println!("{json}");
            if summary.is_ok() {
                Ok(0)
            } else {
                Ok(1)
            }
        }
        Err(source) => Err(CliError::serialization("view list summary", source)),
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

fn matches_manifest_filters(record: &ViewGovernanceRecord, filters: &ViewListFilters) -> bool {
    if filters
        .view_id
        .as_ref()
        .is_some_and(|requested| requested != &record.view_id)
    {
        return false;
    }
    if filters
        .profile_id
        .as_ref()
        .is_some_and(|requested| requested != &record.profile_id)
    {
        return false;
    }
    if filters
        .maintainer
        .as_ref()
        .is_some_and(|requested| requested != &record.maintainer)
    {
        return false;
    }
    if let Some(doc_id) = &filters.doc_id {
        if !record.documents.contains_key(doc_id) {
            return false;
        }
    }
    if let Some(revision_id) = &filters.revision_id {
        if !record
            .documents
            .values()
            .any(|current_revision_id| current_revision_id == revision_id)
        {
            return false;
        }
    }

    true
}

fn matches_timestamp_filters(timestamp: u64, filters: &ViewListFilters) -> bool {
    if filters
        .timestamp_min
        .is_some_and(|timestamp_min| timestamp < timestamp_min)
    {
        return false;
    }
    if filters
        .timestamp_max
        .is_some_and(|timestamp_max| timestamp > timestamp_max)
    {
        return false;
    }

    true
}

fn sort_view_records(records: &mut [ViewListRecord], sort: ViewListSort) {
    records.sort_by(|left, right| match sort {
        ViewListSort::ViewId => left
            .view_id
            .cmp(&right.view_id)
            .then_with(|| left.profile_id.cmp(&right.profile_id)),
        ViewListSort::TimestampAsc => left
            .timestamp
            .cmp(&right.timestamp)
            .then_with(|| left.view_id.cmp(&right.view_id)),
        ViewListSort::TimestampDesc => right
            .timestamp
            .cmp(&left.timestamp)
            .then_with(|| left.view_id.cmp(&right.view_id)),
        ViewListSort::ProfileId => left
            .profile_id
            .cmp(&right.profile_id)
            .then_with(|| right.timestamp.cmp(&left.timestamp))
            .then_with(|| left.view_id.cmp(&right.view_id)),
        ViewListSort::Maintainer => left
            .maintainer
            .cmp(&right.maintainer)
            .then_with(|| right.timestamp.cmp(&left.timestamp))
            .then_with(|| left.view_id.cmp(&right.view_id)),
    });
}

fn latest_profile_records(records: Vec<ViewListRecord>) -> Vec<ViewListRecord> {
    let mut latest_by_profile = BTreeMap::<String, ViewListRecord>::new();

    for record in records {
        latest_by_profile
            .entry(record.profile_id.clone())
            .and_modify(|current| {
                let is_newer = record.timestamp > current.timestamp;
                let breaks_tie =
                    record.timestamp == current.timestamp && record.view_id < current.view_id;
                if is_newer || breaks_tie {
                    *current = record.clone();
                }
            })
            .or_insert(record);
    }

    latest_by_profile.into_values().collect()
}

fn build_view_group_summary(
    records: &[ViewListRecord],
    group_by: ViewListGroupBy,
) -> ViewListGroupSummary {
    let mut buckets = BTreeMap::<String, Vec<&ViewListRecord>>::new();
    for record in records {
        match group_by {
            ViewListGroupBy::ProfileId => {
                buckets
                    .entry(record.profile_id.clone())
                    .or_default()
                    .push(record);
            }
            ViewListGroupBy::Maintainer => {
                buckets
                    .entry(record.maintainer.clone())
                    .or_default()
                    .push(record);
            }
            ViewListGroupBy::DocId => {
                for doc_id in record.documents.keys() {
                    buckets.entry(doc_id.clone()).or_default().push(record);
                }
            }
        }
    }

    let mut groups = buckets
        .into_iter()
        .map(|(key, group_records)| {
            let latest_timestamp = group_records
                .iter()
                .map(|record| record.timestamp)
                .max()
                .unwrap_or_default();
            let mut view_ids = group_records
                .iter()
                .map(|record| record.view_id.clone())
                .collect::<Vec<_>>();
            view_ids.sort();
            view_ids.dedup();

            ViewListGroupBucket {
                key,
                record_count: group_records.len(),
                latest_timestamp,
                view_ids,
            }
        })
        .collect::<Vec<_>>();
    groups.sort_by(|left, right| {
        right
            .record_count
            .cmp(&left.record_count)
            .then_with(|| right.latest_timestamp.cmp(&left.latest_timestamp))
            .then_with(|| left.key.cmp(&right.key))
    });

    ViewListGroupSummary { group_by, groups }
}

fn view_list(
    filters: ViewListFilters,
    sort: ViewListSort,
    summary_only: bool,
    latest_per_profile: bool,
    limit: Option<usize>,
    group_by: Vec<ViewListGroupBy>,
    store_root: PathBuf,
    json: bool,
) -> Result<i32, CliError> {
    let mut summary = ViewListSummary::new(
        &store_root,
        filters,
        sort,
        summary_only,
        latest_per_profile,
        limit,
        group_by,
    );
    let manifest = match load_store_index_manifest(&store_root) {
        Ok(manifest) => manifest,
        Err(error) => {
            summary.push_error(format!("failed to read store index manifest: {error}"));
            return if json {
                print_view_list_json(&summary)
            } else {
                Ok(print_view_list_text(&summary))
            };
        }
    };

    let matching_records = manifest
        .view_governance
        .iter()
        .filter(|record| matches_manifest_filters(record, &summary.filters))
        .cloned()
        .collect::<Vec<_>>();

    for record in matching_records {
        let value = match load_stored_object_value(&store_root, &record.view_id) {
            Ok(value) => value,
            Err(error) => {
                summary.push_error(format!(
                    "failed to load stored view '{}' while listing governance records: {error}",
                    record.view_id
                ));
                return if json {
                    print_view_list_json(&summary)
                } else {
                    Ok(print_view_list_text(&summary))
                };
            }
        };
        let view = match parse_view_object(&value) {
            Ok(view) => view,
            Err(error) => {
                summary.push_error(format!(
                    "failed to parse stored view '{}' while listing governance records: {error}",
                    record.view_id
                ));
                return if json {
                    print_view_list_json(&summary)
                } else {
                    Ok(print_view_list_text(&summary))
                };
            }
        };
        if !matches_timestamp_filters(view.timestamp, &summary.filters) {
            continue;
        }
        summary.records.push(ViewListRecord {
            view_id: record.view_id,
            maintainer: record.maintainer,
            profile_id: record.profile_id,
            timestamp: view.timestamp,
            documents: record.documents,
        });
    }

    if summary.latest_per_profile {
        summary.records = latest_profile_records(std::mem::take(&mut summary.records));
    }
    sort_view_records(&mut summary.records, summary.sort);
    if let Some(limit) = summary.limit {
        summary.records.truncate(limit);
    }
    summary.record_count = summary.records.len();
    summary.groups = summary
        .group_by
        .iter()
        .copied()
        .map(|group_by| build_view_group_summary(&summary.records, group_by))
        .collect();
    summary.notes.push(
        "governance record listing is separate from reader-facing accepted-head workflows"
            .to_string(),
    );
    if summary.latest_per_profile {
        summary.notes.push(
            "records were projected to the latest persisted view for each profile".to_string(),
        );
    }
    if summary.summary_only {
        summary
            .notes
            .push("per-record output was omitted because summary-only was requested".to_string());
        summary.records.clear();
    }

    if json {
        print_view_list_json(&summary)
    } else {
        Ok(print_view_list_text(&summary))
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
        Some(ViewSubcommand::List(args)) => {
            if let Some(message) = unexpected_extra(&args.extra, "view list") {
                return Err(CliError::usage(message));
            }
            let ViewListCliArgs {
                store_root,
                view_id,
                profile_id,
                maintainer,
                doc_id,
                revision_id,
                timestamp_min,
                timestamp_max,
                sort,
                limit,
                latest_per_profile,
                summary_only,
                group_by: raw_group_by,
                json,
                extra: _,
            } = args;

            if let (Some(timestamp_min), Some(timestamp_max)) = (timestamp_min, timestamp_max) {
                if timestamp_min > timestamp_max {
                    return Err(CliError::usage(
                        "view list timestamp-min cannot be greater than timestamp-max".to_string(),
                    ));
                }
            }
            let mut group_by = Vec::new();
            for current in raw_group_by {
                if !group_by.contains(&current) {
                    group_by.push(current);
                }
            }

            view_list(
                ViewListFilters {
                    view_id,
                    profile_id,
                    maintainer,
                    doc_id,
                    revision_id,
                    timestamp_min,
                    timestamp_max,
                },
                sort,
                summary_only,
                latest_per_profile,
                limit,
                group_by,
                PathBuf::from(store_root),
                json,
            )
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
