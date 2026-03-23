use std::collections::BTreeMap;
use std::fs;
use std::path::Path;

use mycel_core::protocol::parse_json_strict;
use mycel_core::store::{StoreIndexManifest, ViewGovernanceRecord};
use serde_json::Value;

use super::list::{ViewListGroupBucket, ViewListGroupSummary, ViewListRecord};
use super::{ViewListFilters, ViewListGroupBy, ViewListSort};

pub(super) fn find_view_record(
    manifest: &StoreIndexManifest,
    view_id: &str,
) -> Option<ViewGovernanceRecord> {
    manifest
        .view_governance
        .iter()
        .find(|record| record.view_id == view_id)
        .cloned()
}

pub(super) fn related_document_view_ids(
    manifest: &StoreIndexManifest,
    documents: &BTreeMap<String, String>,
) -> BTreeMap<String, Vec<String>> {
    documents
        .keys()
        .map(|doc_id| {
            (
                doc_id.clone(),
                manifest
                    .document_views
                    .get(doc_id)
                    .cloned()
                    .unwrap_or_default(),
            )
        })
        .collect()
}

pub(super) fn load_source_value(source_path: &Path) -> Result<Value, String> {
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

pub(super) fn matches_manifest_filters(
    record: &ViewGovernanceRecord,
    filters: &ViewListFilters,
) -> bool {
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

pub(super) fn matches_timestamp_filters(timestamp: u64, filters: &ViewListFilters) -> bool {
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

pub(super) fn sort_view_records(records: &mut [ViewListRecord], sort: ViewListSort) {
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

pub(super) fn latest_profile_records(records: Vec<ViewListRecord>) -> Vec<ViewListRecord> {
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

pub(super) fn build_view_group_summary(
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
