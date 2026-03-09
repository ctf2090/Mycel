use std::collections::{BTreeMap, BTreeSet, HashMap};
use std::fs;
use std::path::{Path, PathBuf};

use serde::{Deserialize, Serialize};
use serde_json::Value;
use sha2::{Digest, Sha256};

use crate::verify::verify_object_value;
use crate::verify::{canonical_json, hex_encode};

#[derive(Debug, Clone, Serialize)]
pub struct HeadInspectSummary {
    pub input_path: PathBuf,
    pub status: String,
    pub doc_id: String,
    pub profile_id: Option<String>,
    pub effective_selection_time: Option<u64>,
    pub selector_epoch: Option<i64>,
    pub selected_head: Option<String>,
    pub tie_break_reason: Option<String>,
    pub eligible_heads: Vec<EligibleHeadSummary>,
    pub verified_revision_count: usize,
    pub verified_view_count: usize,
    pub notes: Vec<String>,
    pub errors: Vec<String>,
}

#[derive(Debug, Clone, Serialize)]
pub struct EligibleHeadSummary {
    pub revision_id: String,
    pub revision_timestamp: u64,
    pub weighted_support: u64,
    pub supporter_count: u64,
    pub selector_score: u64,
}

#[derive(Debug, Deserialize)]
struct HeadInspectInput {
    profile: HeadInspectProfile,
    revisions: Vec<Value>,
    #[serde(default)]
    views: Vec<Value>,
}

#[derive(Debug, Deserialize)]
struct HeadInspectProfile {
    policy_hash: String,
    effective_selection_time: u64,
    #[serde(default = "default_epoch_seconds")]
    epoch_seconds: u64,
    #[serde(default)]
    epoch_zero_timestamp: i64,
}

fn default_epoch_seconds() -> u64 {
    3600
}

#[derive(Debug, Clone)]
struct VerifiedRevision {
    revision_id: String,
    doc_id: String,
    timestamp: u64,
    parents: Vec<String>,
}

#[derive(Debug, Clone)]
struct VerifiedView {
    view_id: String,
    maintainer: String,
    timestamp: u64,
    documents: BTreeMap<String, String>,
}

impl HeadInspectSummary {
    fn new(input_path: &Path, doc_id: &str) -> Self {
        Self {
            input_path: input_path.to_path_buf(),
            status: "ok".to_string(),
            doc_id: doc_id.to_string(),
            profile_id: None,
            effective_selection_time: None,
            selector_epoch: None,
            selected_head: None,
            tie_break_reason: None,
            eligible_heads: Vec::new(),
            verified_revision_count: 0,
            verified_view_count: 0,
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
}

pub fn inspect_heads_from_path(input_path: &Path, doc_id: &str) -> HeadInspectSummary {
    let mut summary = HeadInspectSummary::new(input_path, doc_id);

    let content = match fs::read_to_string(input_path) {
        Ok(content) => content,
        Err(err) => {
            summary.push_error(format!("failed to read head-inspect input: {err}"));
            return summary;
        }
    };

    let input: HeadInspectInput = match serde_json::from_str(&content) {
        Ok(input) => input,
        Err(err) => {
            summary.push_error(format!("failed to parse head-inspect input JSON: {err}"));
            return summary;
        }
    };

    summary.profile_id = Some(input.profile.policy_hash.clone());
    summary.effective_selection_time = Some(input.profile.effective_selection_time);
    summary.selector_epoch = Some(selector_epoch(
        input.profile.effective_selection_time,
        input.profile.epoch_seconds,
        input.profile.epoch_zero_timestamp,
    ));
    summary.notes.push(
        "minimal selector mode: view-maintainer admission and weighted governance are not implemented yet; each matching maintainer contributes weight 1".to_string(),
    );

    let verified_revisions = collect_verified_revisions(
        &input.revisions,
        doc_id,
        input.profile.effective_selection_time,
        &mut summary,
    );
    let verified_views = collect_verified_views(
        &input.views,
        &input.profile,
        input.profile.effective_selection_time,
        &mut summary,
    );

    summary.verified_revision_count = verified_revisions.len();
    summary.verified_view_count = verified_views.len();

    if !summary.errors.is_empty() {
        return summary;
    }

    let eligible_heads = compute_eligible_heads(&verified_revisions);
    if eligible_heads.is_empty() {
        summary.push_error("NO_ELIGIBLE_HEAD");
        return summary;
    }

    let support_map = latest_support_by_maintainer(
        &verified_views,
        doc_id,
        &eligible_heads,
        summary
            .selector_epoch
            .expect("selector epoch should be set"),
        &input.profile,
    );

    let mut eligible_summaries = eligible_heads
        .iter()
        .map(|revision| {
            let supporter_count = support_map
                .values()
                .filter(|candidate| candidate.as_str() == revision.revision_id.as_str())
                .count() as u64;

            EligibleHeadSummary {
                revision_id: revision.revision_id.clone(),
                revision_timestamp: revision.timestamp,
                weighted_support: supporter_count,
                supporter_count,
                selector_score: supporter_count,
            }
        })
        .collect::<Vec<_>>();

    eligible_summaries.sort_by(|left, right| left.revision_id.cmp(&right.revision_id));
    summary.eligible_heads = eligible_summaries;

    let selected = summary
        .eligible_heads
        .iter()
        .max_by(|left, right| {
            left.selector_score
                .cmp(&right.selector_score)
                .then(left.revision_timestamp.cmp(&right.revision_timestamp))
                .then_with(|| right.revision_id.cmp(&left.revision_id))
        })
        .expect("eligible heads should not be empty");

    summary.selected_head = Some(selected.revision_id.clone());
    summary.tie_break_reason = Some(if selected.selector_score > 0 {
        "higher_selector_score".to_string()
    } else {
        "newer_revision_timestamp_or_lexicographic_tiebreak".to_string()
    });

    summary
}

fn collect_verified_revisions(
    values: &[Value],
    doc_id: &str,
    effective_selection_time: u64,
    summary: &mut HeadInspectSummary,
) -> Vec<VerifiedRevision> {
    let mut revisions = Vec::new();

    for value in values {
        let verification = verify_object_value(value);
        if !verification.is_ok() {
            summary.push_error(format!(
                "revision candidate failed verification: {}",
                verification.errors.join("; ")
            ));
            continue;
        }
        if verification.object_type.as_deref() != Some("revision") {
            summary.push_error(
                "head-inspect input 'revisions' array must contain only revision objects",
            );
            continue;
        }

        let object = value.as_object().expect("verified object should be object");
        let revision_doc_id = match object.get("doc_id").and_then(Value::as_str) {
            Some(value) => value,
            None => {
                summary.push_error("revision is missing string field 'doc_id'");
                continue;
            }
        };
        if revision_doc_id != doc_id {
            continue;
        }

        let timestamp = match object.get("timestamp").and_then(Value::as_u64) {
            Some(value) => value,
            None => {
                summary.push_error("revision is missing integer field 'timestamp'");
                continue;
            }
        };
        if timestamp > effective_selection_time {
            continue;
        }

        let revision_id = match object.get("revision_id").and_then(Value::as_str) {
            Some(value) => value,
            None => {
                summary.push_error("revision is missing string field 'revision_id'");
                continue;
            }
        };

        let parents = object
            .get("parents")
            .and_then(Value::as_array)
            .map(|parents| {
                parents
                    .iter()
                    .filter_map(Value::as_str)
                    .map(ToString::to_string)
                    .collect::<Vec<_>>()
            })
            .unwrap_or_default();

        revisions.push(VerifiedRevision {
            revision_id: revision_id.to_string(),
            doc_id: revision_doc_id.to_string(),
            timestamp,
            parents,
        });
    }

    revisions
}

fn collect_verified_views(
    values: &[Value],
    profile: &HeadInspectProfile,
    effective_selection_time: u64,
    summary: &mut HeadInspectSummary,
) -> Vec<VerifiedView> {
    let mut views = Vec::new();

    for value in values {
        let verification = verify_object_value(value);
        if !verification.is_ok() {
            summary.push_error(format!(
                "view candidate failed verification: {}",
                verification.errors.join("; ")
            ));
            continue;
        }
        if verification.object_type.as_deref() != Some("view") {
            summary.push_error("head-inspect input 'views' array must contain only view objects");
            continue;
        }

        let object = value.as_object().expect("verified object should be object");
        let timestamp = match object.get("timestamp").and_then(Value::as_u64) {
            Some(value) => value,
            None => {
                summary.push_error("view is missing integer field 'timestamp'");
                continue;
            }
        };
        if timestamp > effective_selection_time {
            continue;
        }

        let policy = match object.get("policy") {
            Some(policy) => policy,
            None => {
                summary.push_error("view is missing object field 'policy'");
                continue;
            }
        };
        let policy_hash = match hash_json(policy) {
            Ok(hash) => hash,
            Err(err) => {
                summary.push_error(format!("failed to hash view policy: {err}"));
                continue;
            }
        };
        if policy_hash != profile.policy_hash {
            continue;
        }

        let documents = object
            .get("documents")
            .and_then(Value::as_object)
            .map(|documents| {
                documents
                    .iter()
                    .filter_map(|(key, value)| {
                        value.as_str().map(|value| (key.clone(), value.to_string()))
                    })
                    .collect::<BTreeMap<_, _>>()
            })
            .unwrap_or_default();

        let view_id = match object.get("view_id").and_then(Value::as_str) {
            Some(value) => value,
            None => {
                summary.push_error("view is missing string field 'view_id'");
                continue;
            }
        };
        let maintainer = match object.get("maintainer").and_then(Value::as_str) {
            Some(value) => value,
            None => {
                summary.push_error("view is missing string field 'maintainer'");
                continue;
            }
        };

        views.push(VerifiedView {
            view_id: view_id.to_string(),
            maintainer: maintainer.to_string(),
            timestamp,
            documents,
        });
    }

    views
}

fn compute_eligible_heads(revisions: &[VerifiedRevision]) -> Vec<VerifiedRevision> {
    let revision_ids = revisions
        .iter()
        .map(|revision| revision.revision_id.clone())
        .collect::<BTreeSet<_>>();
    let children = build_children_map(revisions, &revision_ids);

    revisions
        .iter()
        .filter(|candidate| {
            !revisions.iter().any(|other| {
                other.revision_id != candidate.revision_id
                    && other.doc_id == candidate.doc_id
                    && is_descendant(&candidate.revision_id, &other.revision_id, &children)
            })
        })
        .cloned()
        .collect()
}

fn build_children_map(
    revisions: &[VerifiedRevision],
    known_ids: &BTreeSet<String>,
) -> HashMap<String, Vec<String>> {
    let mut children: HashMap<String, Vec<String>> = HashMap::new();

    for revision in revisions {
        for parent in &revision.parents {
            if known_ids.contains(parent) {
                children
                    .entry(parent.clone())
                    .or_default()
                    .push(revision.revision_id.clone());
            }
        }
    }

    children
}

fn is_descendant(
    ancestor_id: &str,
    candidate_descendant_id: &str,
    children: &HashMap<String, Vec<String>>,
) -> bool {
    let mut stack = children.get(ancestor_id).cloned().unwrap_or_default();
    let mut visited = BTreeSet::new();

    while let Some(current) = stack.pop() {
        if current == candidate_descendant_id {
            return true;
        }
        if !visited.insert(current.clone()) {
            continue;
        }
        if let Some(next_children) = children.get(&current) {
            stack.extend(next_children.iter().cloned());
        }
    }

    false
}

fn latest_support_by_maintainer(
    views: &[VerifiedView],
    doc_id: &str,
    eligible_heads: &[VerifiedRevision],
    selector_epoch: i64,
    profile: &HeadInspectProfile,
) -> HashMap<String, String> {
    let eligible_ids = eligible_heads
        .iter()
        .map(|revision| revision.revision_id.clone())
        .collect::<BTreeSet<_>>();
    let mut latest_by_maintainer: HashMap<String, &VerifiedView> = HashMap::new();

    for view in views.iter().filter(|view| {
        selector_epoch_for_view(
            view.timestamp,
            profile.epoch_seconds,
            profile.epoch_zero_timestamp,
        ) == selector_epoch
    }) {
        let replace = match latest_by_maintainer.get(&view.maintainer) {
            Some(current) => {
                view.timestamp > current.timestamp
                    || (view.timestamp == current.timestamp && view.view_id < current.view_id)
            }
            None => true,
        };

        if replace {
            latest_by_maintainer.insert(view.maintainer.clone(), view);
        }
    }

    latest_by_maintainer
        .into_iter()
        .filter_map(|(maintainer, view)| {
            view.documents
                .get(doc_id)
                .filter(|revision_id| eligible_ids.contains(*revision_id))
                .map(|revision_id| (maintainer, revision_id.clone()))
        })
        .collect()
}

fn selector_epoch(
    effective_selection_time: u64,
    epoch_seconds: u64,
    epoch_zero_timestamp: i64,
) -> i64 {
    selector_epoch_for_view(
        effective_selection_time,
        epoch_seconds,
        epoch_zero_timestamp,
    )
}

fn selector_epoch_for_view(timestamp: u64, epoch_seconds: u64, epoch_zero_timestamp: i64) -> i64 {
    ((timestamp as i64) - epoch_zero_timestamp) / (epoch_seconds as i64)
}

fn hash_json(value: &Value) -> Result<String, String> {
    let canonical = canonical_json(value)?;
    let mut hasher = Sha256::new();
    hasher.update(canonical.as_bytes());
    let digest = hasher.finalize();
    Ok(format!("hash:{}", hex_encode(&digest)))
}
