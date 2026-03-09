use std::collections::{BTreeMap, BTreeSet, HashMap};
use std::env;
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
    pub critical_violations: Vec<CriticalViolationSummary>,
    pub effective_weights: Vec<EffectiveWeightSummary>,
    pub maintainer_support: Vec<MaintainerSupportSummary>,
    pub decision_trace: Vec<DecisionTraceEntry>,
    pub notes: Vec<String>,
    pub errors: Vec<String>,
}

#[derive(Debug, Clone, Serialize)]
pub struct DecisionTraceEntry {
    pub step: String,
    pub detail: String,
}

#[derive(Debug, Clone, Serialize)]
pub struct CriticalViolationSummary {
    pub maintainer: String,
    pub timestamp: u64,
    pub selector_epoch: i64,
    pub reason: Option<String>,
}

#[derive(Debug, Clone, Serialize)]
pub struct EffectiveWeightSummary {
    pub maintainer: String,
    pub admitted: bool,
    pub effective_weight: u64,
    pub valid_view_counts: Vec<EpochCountSummary>,
    pub critical_violation_counts: Vec<EpochCountSummary>,
}

#[derive(Debug, Clone, Serialize)]
pub struct MaintainerSupportSummary {
    pub maintainer: String,
    pub revision_id: String,
    pub effective_weight: u64,
}

#[derive(Debug, Clone, Serialize)]
pub struct EpochCountSummary {
    pub epoch: i64,
    pub count: u64,
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
    #[serde(default)]
    critical_violations: Vec<HeadInspectCriticalViolation>,
}

#[derive(Debug, Deserialize)]
struct HeadInspectProfile {
    policy_hash: String,
    effective_selection_time: u64,
    epoch_seconds: u64,
    epoch_zero_timestamp: i64,
    admission_window_epochs: u64,
    min_valid_views_for_admission: u64,
    min_valid_views_per_epoch: u64,
    weight_cap_per_key: u64,
}

#[derive(Debug, Clone, Deserialize)]
struct HeadInspectCriticalViolation {
    maintainer: String,
    timestamp: u64,
    #[serde(default)]
    reason: Option<String>,
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

#[derive(Debug, Clone)]
struct MaintainerSupport {
    revision_id: String,
    effective_weight: u64,
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
            critical_violations: Vec::new(),
            effective_weights: Vec::new(),
            maintainer_support: Vec::new(),
            decision_trace: Vec::new(),
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

    fn push_trace(&mut self, step: impl Into<String>, detail: impl Into<String>) {
        self.decision_trace.push(DecisionTraceEntry {
            step: step.into(),
            detail: detail.into(),
        });
    }
}

pub fn inspect_heads_from_path(input_path: &Path, doc_id: &str) -> HeadInspectSummary {
    let resolved_input_path = match resolve_head_inspect_input_path(input_path) {
        Ok(path) => path,
        Err(message) => {
            let mut summary = HeadInspectSummary::new(input_path, doc_id);
            summary.push_error(message);
            return summary;
        }
    };

    let mut summary = HeadInspectSummary::new(&resolved_input_path, doc_id);

    let content = match fs::read_to_string(&resolved_input_path) {
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
    summary.push_trace(
        "selector_epoch",
        format!(
            "effective_selection_time={} epoch_seconds={} epoch_zero_timestamp={} selector_epoch={}",
            input.profile.effective_selection_time,
            input.profile.epoch_seconds,
            input.profile.epoch_zero_timestamp,
            summary.selector_epoch.expect("selector epoch should be set")
        ),
    );
    summary.notes.push(
        "minimal selector mode: critical violations are bundle-provided fixture evidence; external dispute / penalty objects are not implemented yet".to_string(),
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
    summary.push_trace(
        "verified_inputs",
        format!(
            "verified_revisions={} verified_views={}",
            summary.verified_revision_count, summary.verified_view_count
        ),
    );
    summary.push_trace(
        "critical_violations",
        if input.critical_violations.is_empty() {
            "count=0 affected_maintainers=0".to_string()
        } else {
            let affected_maintainers = input
                .critical_violations
                .iter()
                .map(|violation| violation.maintainer.clone())
                .collect::<BTreeSet<_>>()
                .len();
            format!(
                "count={} affected_maintainers={affected_maintainers}",
                input.critical_violations.len()
            )
        },
    );
    summary.critical_violations = input
        .critical_violations
        .iter()
        .map(|violation| CriticalViolationSummary {
            maintainer: violation.maintainer.clone(),
            timestamp: violation.timestamp,
            selector_epoch: selector_epoch_for_view(
                violation.timestamp,
                input.profile.epoch_seconds,
                input.profile.epoch_zero_timestamp,
            ),
            reason: violation.reason.clone(),
        })
        .collect();

    if !summary.errors.is_empty() {
        return summary;
    }

    let eligible_heads = compute_eligible_heads(&verified_revisions);
    summary.push_trace("eligible_heads", format!("count={}", eligible_heads.len()));
    if eligible_heads.is_empty() {
        summary.push_error("NO_ELIGIBLE_HEAD");
        return summary;
    }

    let (effective_weights, effective_weight_summaries, weight_trace) = compute_effective_weights(
        &verified_views,
        &input.critical_violations,
        summary
            .selector_epoch
            .expect("selector epoch should be set"),
        &input.profile,
    );
    summary.effective_weights = effective_weight_summaries;
    for entry in weight_trace {
        summary.push_trace(entry.step, entry.detail);
    }

    let (support_map, support_summaries, support_trace) = latest_support_by_maintainer(
        &verified_views,
        doc_id,
        &eligible_heads,
        summary
            .selector_epoch
            .expect("selector epoch should be set"),
        &input.profile,
        &effective_weights,
    );
    summary.maintainer_support = support_summaries;
    for entry in support_trace {
        summary.push_trace(entry.step, entry.detail);
    }

    let mut eligible_summaries = eligible_heads
        .iter()
        .map(|revision| {
            let supporting_entries = support_map
                .values()
                .filter(|candidate| candidate.revision_id.as_str() == revision.revision_id.as_str())
                .collect::<Vec<_>>();
            let supporter_count = supporting_entries
                .iter()
                .filter(|candidate| candidate.effective_weight > 0)
                .count() as u64;
            let weighted_support = supporting_entries
                .iter()
                .map(|candidate| candidate.effective_weight)
                .sum::<u64>();

            EligibleHeadSummary {
                revision_id: revision.revision_id.clone(),
                revision_timestamp: revision.timestamp,
                weighted_support,
                supporter_count,
                selector_score: weighted_support,
            }
        })
        .collect::<Vec<_>>();

    eligible_summaries.sort_by(|left, right| left.revision_id.cmp(&right.revision_id));
    summary.eligible_heads = eligible_summaries;
    summary.push_trace(
        "selector_scores",
        format!(
            "head_count={} max_selector_score={} supported_head_count={}",
            summary.eligible_heads.len(),
            summary
                .eligible_heads
                .iter()
                .map(|head| head.selector_score)
                .max()
                .unwrap_or(0),
            summary
                .eligible_heads
                .iter()
                .filter(|head| head.supporter_count > 0)
                .count()
        ),
    );

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
    summary.push_trace(
        "selected_head",
        format!(
            "selected={} tie_break_reason={}",
            selected.revision_id,
            summary
                .tie_break_reason
                .as_deref()
                .expect("tie break reason should be set")
        ),
    );

    summary
}

fn resolve_head_inspect_input_path(input_path: &Path) -> Result<PathBuf, String> {
    if input_path.exists() {
        return resolve_existing_head_inspect_input(input_path);
    }

    let repo_root = find_repo_root_from_current_dir()?;
    let fixture_root = repo_root.join("fixtures/head-inspect");
    let candidates = [
        repo_root.join(input_path),
        fixture_root.join(input_path),
        fixture_root.join(format!("{}.json", input_path.display())),
        fixture_root.join(format!("{}.example.json", input_path.display())),
    ];

    for candidate in candidates {
        if candidate.exists() {
            return resolve_existing_head_inspect_input(&candidate);
        }
    }

    Err(format!(
        "could not resolve head-inspect input '{candidate}' from the current directory or fixtures/head-inspect/",
        candidate = input_path.display()
    ))
}

fn resolve_existing_head_inspect_input(input_path: &Path) -> Result<PathBuf, String> {
    if input_path.is_file() {
        return Ok(input_path.to_path_buf());
    }

    if input_path.is_dir() {
        for candidate_name in ["bundle.json", "head-inspect.json", "input.json"] {
            let candidate = input_path.join(candidate_name);
            if candidate.is_file() {
                return Ok(candidate);
            }
        }

        return Err(format!(
            "head-inspect input directory '{}' must contain one of: bundle.json, head-inspect.json, input.json",
            input_path.display()
        ));
    }

    Err(format!(
        "head-inspect input '{}' is neither a file nor a directory",
        input_path.display()
    ))
}

fn find_repo_root_from_current_dir() -> Result<PathBuf, String> {
    let current_dir =
        env::current_dir().map_err(|err| format!("failed to read current directory: {err}"))?;

    for candidate in current_dir.ancestors() {
        if candidate.join("Cargo.toml").is_file() && candidate.join("fixtures").is_dir() {
            return Ok(candidate.to_path_buf());
        }
    }

    Err("could not find repository root containing Cargo.toml and fixtures/".to_string())
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
    effective_weights: &HashMap<String, u64>,
) -> (
    HashMap<String, MaintainerSupport>,
    Vec<MaintainerSupportSummary>,
    Vec<DecisionTraceEntry>,
) {
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

    let support_map = latest_by_maintainer
        .into_iter()
        .filter_map(|(maintainer, view)| {
            let effective_weight = effective_weights.get(&maintainer).copied().unwrap_or(0);
            view.documents
                .get(doc_id)
                .filter(|revision_id| eligible_ids.contains(*revision_id))
                .map(|revision_id| {
                    (
                        maintainer,
                        MaintainerSupport {
                            revision_id: revision_id.clone(),
                            effective_weight,
                        },
                    )
                })
        })
        .collect::<HashMap<_, _>>();

    let mut support_summaries = support_map
        .iter()
        .map(|(maintainer, support)| MaintainerSupportSummary {
            maintainer: maintainer.clone(),
            revision_id: support.revision_id.clone(),
            effective_weight: support.effective_weight,
        })
        .collect::<Vec<_>>();
    support_summaries.sort_by(|left, right| left.maintainer.cmp(&right.maintainer));

    let trace = vec![DecisionTraceEntry {
        step: "maintainer_support".to_string(),
        detail: format!(
            "supporting_maintainers={} supported_heads={} active_epoch={}",
            support_summaries.len(),
            support_summaries
                .iter()
                .map(|support| support.revision_id.clone())
                .collect::<BTreeSet<_>>()
                .len(),
            selector_epoch
        ),
    }];

    (support_map, support_summaries, trace)
}

fn compute_effective_weights(
    views: &[VerifiedView],
    critical_violations: &[HeadInspectCriticalViolation],
    selector_epoch: i64,
    profile: &HeadInspectProfile,
) -> (
    HashMap<String, u64>,
    Vec<EffectiveWeightSummary>,
    Vec<DecisionTraceEntry>,
) {
    let mut per_epoch_counts: HashMap<String, BTreeMap<i64, u64>> = HashMap::new();
    let mut per_epoch_violations: HashMap<String, BTreeMap<i64, u64>> = HashMap::new();

    for view in views {
        let epoch = selector_epoch_for_view(
            view.timestamp,
            profile.epoch_seconds,
            profile.epoch_zero_timestamp,
        );
        per_epoch_counts
            .entry(view.maintainer.clone())
            .or_default()
            .entry(epoch)
            .and_modify(|count| *count += 1)
            .or_insert(1);
    }

    for violation in critical_violations {
        let epoch = selector_epoch_for_view(
            violation.timestamp,
            profile.epoch_seconds,
            profile.epoch_zero_timestamp,
        );
        per_epoch_violations
            .entry(violation.maintainer.clone())
            .or_default()
            .entry(epoch)
            .and_modify(|count| *count += 1)
            .or_insert(1);
    }

    let mut maintainers = per_epoch_counts
        .keys()
        .chain(per_epoch_violations.keys())
        .cloned()
        .collect::<BTreeSet<_>>()
        .into_iter()
        .collect::<Vec<_>>();
    maintainers.sort();

    let mut weights = HashMap::new();
    let mut summaries = Vec::new();
    let mut admitted_maintainers = 0_usize;
    let mut penalized_maintainers = 0_usize;
    let mut zero_weight_maintainers = 0_usize;
    let mut max_effective_weight = 0_u64;

    for maintainer in maintainers {
        let counts = per_epoch_counts
            .get(&maintainer)
            .cloned()
            .unwrap_or_default();
        let violations = per_epoch_violations
            .get(&maintainer)
            .cloned()
            .unwrap_or_default();
        let effective_weight =
            effective_weight_for_epoch(&counts, &violations, selector_epoch, profile);
        let admitted = is_admitted_in_epoch(&counts, &violations, selector_epoch, profile);
        if admitted {
            admitted_maintainers += 1;
        }
        if !violations.is_empty() {
            penalized_maintainers += 1;
        }
        if effective_weight == 0 {
            zero_weight_maintainers += 1;
        }
        max_effective_weight = max_effective_weight.max(effective_weight);
        weights.insert(maintainer.clone(), effective_weight);
        summaries.push(EffectiveWeightSummary {
            maintainer: maintainer.clone(),
            admitted,
            effective_weight,
            valid_view_counts: to_epoch_count_summaries(&counts),
            critical_violation_counts: to_epoch_count_summaries(&violations),
        });
    }

    let trace = vec![DecisionTraceEntry {
        step: "effective_weight".to_string(),
        detail: format!(
            "maintainers={} admitted={} penalized={} zero_weight={} max_effective_weight={}",
            summaries.len(),
            admitted_maintainers,
            penalized_maintainers,
            zero_weight_maintainers,
            max_effective_weight
        ),
    }];

    (weights, summaries, trace)
}

fn to_epoch_count_summaries(counts: &BTreeMap<i64, u64>) -> Vec<EpochCountSummary> {
    counts
        .iter()
        .map(|(epoch, count)| EpochCountSummary {
            epoch: *epoch,
            count: *count,
        })
        .collect()
}

fn effective_weight_for_epoch(
    counts: &BTreeMap<i64, u64>,
    violations: &BTreeMap<i64, u64>,
    epoch: i64,
    profile: &HeadInspectProfile,
) -> u64 {
    if !is_admitted_in_epoch(counts, violations, epoch, profile) {
        return 0;
    }

    if epoch <= 0 || !is_admitted_in_epoch(counts, violations, epoch - 1, profile) {
        return 1;
    }

    let previous_weight = effective_weight_for_epoch(counts, violations, epoch - 1, profile);
    let previous_valid_views = counts.get(&(epoch - 1)).copied().unwrap_or(0);
    let previous_critical_violations = violations.get(&(epoch - 1)).copied().unwrap_or(0);
    let delta = if previous_critical_violations > 0 {
        -1_i64
    } else if previous_valid_views >= profile.min_valid_views_per_epoch {
        1_i64
    } else {
        0_i64
    };

    clamp_weight(
        previous_weight as i64 + delta,
        0,
        profile.weight_cap_per_key as i64,
    )
}

fn is_admitted_in_epoch(
    counts: &BTreeMap<i64, u64>,
    violations: &BTreeMap<i64, u64>,
    epoch: i64,
    profile: &HeadInspectProfile,
) -> bool {
    if profile.admission_window_epochs == 0 {
        return profile.min_valid_views_for_admission == 0;
    }

    let window = profile.admission_window_epochs as i64;
    let start_epoch = epoch - window;
    let end_epoch = epoch - 1;

    let valid_view_sum = (start_epoch..=end_epoch)
        .map(|candidate_epoch| counts.get(&candidate_epoch).copied().unwrap_or(0))
        .sum::<u64>();
    let critical_violation_sum = (start_epoch..=end_epoch)
        .map(|candidate_epoch| violations.get(&candidate_epoch).copied().unwrap_or(0))
        .sum::<u64>();

    valid_view_sum >= profile.min_valid_views_for_admission && critical_violation_sum == 0
}

fn clamp_weight(value: i64, lo: i64, hi: i64) -> u64 {
    value.clamp(lo, hi) as u64
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
