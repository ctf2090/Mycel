use std::collections::BTreeMap;
use std::path::PathBuf;

use mycel_core::store::load_store_index_manifest;
use serde::Serialize;

use crate::{emit_error_line, CliError};

use super::shared::{
    build_view_group_summary, latest_profile_records, matches_manifest_filters,
    matches_timestamp_filters, related_document_view_ids, sort_view_records,
};
use super::{ViewListCliArgs, ViewListFilters, ViewListGroupBy, ViewListSort};

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
pub(super) struct ViewListRecord {
    pub(super) view_id: String,
    pub(super) maintainer: String,
    pub(super) profile_id: String,
    pub(super) timestamp: u64,
    pub(super) documents: BTreeMap<String, String>,
    maintainer_view_ids: Vec<String>,
    profile_view_ids: Vec<String>,
    document_view_ids: BTreeMap<String, Vec<String>>,
}

#[derive(Debug, Clone, Serialize)]
pub(super) struct ViewListGroupSummary {
    pub(super) group_by: ViewListGroupBy,
    pub(super) groups: Vec<ViewListGroupBucket>,
}

#[derive(Debug, Clone, Serialize)]
pub(super) struct ViewListGroupBucket {
    pub(super) key: String,
    pub(super) record_count: usize,
    pub(super) latest_timestamp: u64,
    pub(super) view_ids: Vec<String>,
}

impl ViewListSummary {
    fn new(
        store_root: PathBuf,
        filters: ViewListFilters,
        sort: ViewListSort,
        summary_only: bool,
        latest_per_profile: bool,
        limit: Option<usize>,
        group_by: Vec<ViewListGroupBy>,
    ) -> Self {
        Self {
            manifest_path: store_root.join("indexes").join("manifest.json"),
            store_root,
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
        println!(
            "  related views: maintainer={} profile={} docs={}",
            record.maintainer_view_ids.len(),
            record.profile_view_ids.len(),
            record.document_view_ids.len()
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

pub(super) fn handle(args: ViewListCliArgs) -> Result<i32, CliError> {
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

    let mut summary = ViewListSummary::new(
        PathBuf::from(&store_root),
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
    );
    let manifest = match load_store_index_manifest(&summary.store_root) {
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
        if !matches_timestamp_filters(record.timestamp, &summary.filters) {
            continue;
        }
        let maintainer_view_ids = manifest
            .maintainer_views
            .get(&record.maintainer)
            .cloned()
            .unwrap_or_default();
        let profile_view_ids = manifest
            .profile_views
            .get(&record.profile_id)
            .cloned()
            .unwrap_or_default();
        let document_view_ids = related_document_view_ids(&manifest, &record.documents);
        summary.records.push(ViewListRecord {
            view_id: record.view_id,
            maintainer: record.maintainer,
            profile_id: record.profile_id,
            timestamp: record.timestamp,
            documents: record.documents,
            maintainer_view_ids,
            profile_view_ids,
            document_view_ids,
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
