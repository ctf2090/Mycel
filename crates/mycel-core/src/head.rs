use std::collections::{BTreeMap, BTreeSet, HashMap};
use std::env;
use std::fs;
use std::path::{Path, PathBuf};

use serde::{Deserialize, Serialize};
use serde_json::Value;

use crate::canonical::prefixed_canonical_hash;
use crate::protocol::{parse_json_strict, parse_object_envelope, BlockObject, StringFieldError};
use crate::replay::{replay_revision_from_index, DocumentState};
use crate::store::{
    load_doc_replay_objects_from_store, load_store_index_manifest, load_stored_object_value,
};
use crate::verify::{verify_object_value, verify_object_value_with_object_index};

#[derive(Debug, Clone, Serialize)]
pub struct HeadInspectSummary {
    pub input_path: PathBuf,
    pub status: String,
    pub doc_id: String,
    pub profile_id: Option<String>,
    pub available_profile_ids: Vec<String>,
    pub effective_selection_time: Option<u64>,
    pub selector_epoch: Option<i64>,
    pub selected_head: Option<String>,
    pub tie_break_reason: Option<String>,
    pub eligible_heads: Vec<EligibleHeadSummary>,
    pub verified_revision_count: usize,
    pub verified_view_count: usize,
    pub viewer_signal_count: usize,
    pub critical_violations: Vec<CriticalViolationSummary>,
    pub editor_candidates: Vec<EditorCandidateSummary>,
    pub effective_weights: Vec<EffectiveWeightSummary>,
    pub maintainer_support: Vec<MaintainerSupportSummary>,
    pub viewer_signals: Vec<ViewerSignalSummary>,
    pub viewer_score_channels: Vec<ViewerScoreChannelSummary>,
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
pub struct EditorCandidateSummary {
    pub revision_id: String,
    pub author: String,
    pub editor_admitted: bool,
    pub candidate_eligible: bool,
    pub formal_candidate: bool,
}

#[derive(Debug, Clone, Serialize)]
pub struct EffectiveWeightSummary {
    pub maintainer: String,
    pub view_admitted: bool,
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
    pub author: String,
    pub editor_admitted: bool,
    pub formal_candidate: bool,
    pub revision_timestamp: u64,
    pub maintainer_score: u64,
    pub weighted_support: u64,
    pub supporter_count: u64,
    pub viewer_bonus: u64,
    pub viewer_penalty: u64,
    pub selector_score: u64,
}

#[derive(Debug, Clone, Serialize)]
pub struct ViewerSignalSummary {
    pub signal_id: String,
    pub viewer_id: String,
    pub candidate_revision_id: String,
    pub signal_type: ViewerSignalType,
    pub reason_code: Option<String>,
    pub confidence_level: ViewerConfidenceLevel,
    pub evidence_ref: Option<String>,
    pub created_at: u64,
    pub expires_at: u64,
    pub signal_status: ViewerSignalStatus,
    pub viewer_identity_tier: ViewerIdentityTier,
    pub viewer_admission_status: ViewerAdmissionStatus,
    pub viewer_reputation_band: ViewerReputationBand,
    pub selector_eligible: bool,
    pub effective_signal_weight: u64,
}

#[derive(Debug, Clone, Serialize)]
pub struct ViewerScoreChannelSummary {
    pub revision_id: String,
    pub maintainer_score: u64,
    pub viewer_bonus: u64,
    pub viewer_penalty: u64,
    pub approval_signal_count: u64,
    pub objection_signal_count: u64,
    pub challenge_signal_count: u64,
    pub challenge_review_pressure: u64,
    pub challenge_freeze_pressure: u64,
    pub viewer_review_state: ViewerReviewState,
    pub selector_score: u64,
}

#[derive(Debug, Clone, Serialize, PartialEq, Eq)]
#[serde(rename_all = "kebab-case")]
pub enum ViewerReviewState {
    None,
    ReviewPressure,
    FreezePressure,
}

#[derive(Debug, Clone, Serialize)]
pub struct HeadRenderSummary {
    pub input_path: PathBuf,
    pub store_root: Option<PathBuf>,
    pub status: String,
    pub doc_id: String,
    pub profile_id: Option<String>,
    pub available_profile_ids: Vec<String>,
    pub effective_selection_time: Option<u64>,
    pub selected_head: Option<String>,
    pub recomputed_state_hash: Option<String>,
    pub rendered_block_count: usize,
    pub rendered_blocks: Vec<RenderedBlockSummary>,
    pub rendered_text: String,
    pub notes: Vec<String>,
    pub errors: Vec<String>,
}

#[derive(Debug, Clone, Serialize)]
pub struct RenderedBlockSummary {
    pub block_id: String,
    pub block_type: String,
    pub depth: usize,
    pub content: String,
    pub child_count: usize,
}

#[derive(Debug, Deserialize)]
struct HeadInspectInput {
    #[serde(default)]
    profile: Option<HeadInspectProfile>,
    #[serde(default)]
    profiles: BTreeMap<String, HeadInspectProfile>,
    revisions: Vec<Value>,
    #[serde(default)]
    objects: Vec<Value>,
    #[serde(default)]
    views: Vec<Value>,
    #[serde(default)]
    viewer_signals: Vec<HeadInspectViewerSignal>,
    #[serde(default)]
    critical_violations: Vec<HeadInspectCriticalViolation>,
}

#[derive(Debug, Clone, Deserialize)]
struct HeadInspectProfile {
    policy_hash: String,
    effective_selection_time: u64,
    epoch_seconds: u64,
    epoch_zero_timestamp: i64,
    admission_window_epochs: u64,
    min_valid_views_for_admission: u64,
    min_valid_views_per_epoch: u64,
    weight_cap_per_key: u64,
    #[serde(default)]
    editor_admission: EditorAdmissionProfile,
    #[serde(default)]
    view_admission: ViewAdmissionProfile,
    #[serde(default)]
    viewer_score: ViewerScoreProfile,
}

#[derive(Debug, Clone, Deserialize, Default)]
struct EditorAdmissionProfile {
    #[serde(default)]
    mode: EditorCandidateMode,
    #[serde(default)]
    admitted_keys: Vec<String>,
}

#[derive(Debug, Clone, Deserialize, Default)]
#[serde(rename_all = "kebab-case")]
enum EditorCandidateMode {
    #[default]
    Open,
    AdmittedOnly,
    Mixed,
}

#[derive(Debug, Clone, Deserialize, Default)]
struct ViewAdmissionProfile {
    #[serde(default)]
    mode: ViewMaintainerAdmissionMode,
    #[serde(default)]
    admitted_keys: Vec<String>,
}

#[derive(Debug, Clone, Deserialize, Default)]
#[serde(rename_all = "kebab-case")]
enum ViewMaintainerAdmissionMode {
    #[default]
    Open,
    AdmittedOnly,
}

#[derive(Debug, Clone, Deserialize, Default)]
struct ViewerScoreProfile {
    #[serde(default)]
    mode: ViewerScoreMode,
    #[serde(default)]
    bonus_cap: u64,
    #[serde(default)]
    penalty_cap: u64,
    #[serde(default)]
    signal_weight_cap: u64,
    #[serde(default = "default_true")]
    admission_required: bool,
    #[serde(default)]
    min_identity_tier: ViewerIdentityTier,
    #[serde(default)]
    min_reputation_band: ViewerReputationBand,
}

#[derive(Debug, Clone, Deserialize, Default, Serialize, PartialEq, Eq)]
#[serde(rename_all = "kebab-case")]
pub enum ViewerScoreMode {
    #[default]
    Disabled,
    BoundedBonusPenalty,
}

#[derive(Debug, Clone, Deserialize)]
struct HeadInspectViewerSignal {
    signal_id: String,
    viewer_id: String,
    candidate_revision_id: String,
    signal_type: ViewerSignalType,
    #[serde(default)]
    reason_code: Option<String>,
    #[serde(default)]
    confidence_level: ViewerConfidenceLevel,
    #[serde(default)]
    evidence_ref: Option<String>,
    created_at: u64,
    expires_at: u64,
    #[serde(default)]
    signal_status: ViewerSignalStatus,
    #[serde(default)]
    viewer_identity_tier: ViewerIdentityTier,
    #[serde(default)]
    viewer_admission_status: ViewerAdmissionStatus,
    #[serde(default)]
    viewer_reputation_band: ViewerReputationBand,
}

#[derive(Debug, Clone, Deserialize, Serialize, PartialEq, Eq)]
#[serde(rename_all = "kebab-case")]
pub enum ViewerSignalType {
    Approval,
    Objection,
    Challenge,
}

#[derive(Debug, Clone, Deserialize, Serialize, PartialEq, Eq, Default)]
#[serde(rename_all = "kebab-case")]
pub enum ViewerConfidenceLevel {
    #[default]
    Low,
    Medium,
    High,
}

#[derive(Debug, Clone, Deserialize, Serialize, PartialEq, Eq, Default)]
#[serde(rename_all = "kebab-case")]
pub enum ViewerSignalStatus {
    #[default]
    Active,
    Expired,
    Withdrawn,
    Resolved,
}

#[derive(Debug, Clone, Deserialize, Serialize, PartialEq, Eq, PartialOrd, Ord, Default)]
#[serde(rename_all = "kebab-case")]
pub enum ViewerIdentityTier {
    #[default]
    None,
    Basic,
    Strong,
}

#[derive(Debug, Clone, Deserialize, Serialize, PartialEq, Eq, Default)]
#[serde(rename_all = "kebab-case")]
pub enum ViewerAdmissionStatus {
    #[default]
    Pending,
    Admitted,
    Restricted,
    Revoked,
}

#[derive(Debug, Clone, Deserialize, Serialize, PartialEq, Eq, PartialOrd, Ord, Default)]
#[serde(rename_all = "kebab-case")]
pub enum ViewerReputationBand {
    #[default]
    New,
    Established,
    Trusted,
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
    author: String,
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

#[derive(Debug, Clone, Default)]
struct ViewerHeadScore {
    viewer_bonus: u64,
    viewer_penalty: u64,
    approval_signal_count: u64,
    objection_signal_count: u64,
    challenge_signal_count: u64,
    challenge_review_pressure: u64,
    challenge_freeze_pressure: u64,
}

struct LoadedHeadInspectContext {
    selected_profile_id: String,
    profile: HeadInspectProfile,
    revision_values: Vec<Value>,
    view_values: Vec<Value>,
    viewer_signals: Vec<HeadInspectViewerSignal>,
    critical_violations: Vec<HeadInspectCriticalViolation>,
}

struct ViewerGatingSummary {
    review_delayed_revision_ids: BTreeSet<String>,
    frozen_revision_ids: BTreeSet<String>,
    viewer_gating: Option<&'static str>,
}

struct SelectedHeadRenderContext {
    selected_head: String,
    object_index: HashMap<String, Value>,
    revision_value: Value,
    replay_note: String,
    replay_error_context: &'static str,
}

fn default_true() -> bool {
    true
}

impl HeadInspectSummary {
    fn new(input_path: &Path, doc_id: &str) -> Self {
        Self {
            input_path: input_path.to_path_buf(),
            status: "ok".to_string(),
            doc_id: doc_id.to_string(),
            profile_id: None,
            available_profile_ids: Vec::new(),
            effective_selection_time: None,
            selector_epoch: None,
            selected_head: None,
            tie_break_reason: None,
            eligible_heads: Vec::new(),
            verified_revision_count: 0,
            verified_view_count: 0,
            viewer_signal_count: 0,
            critical_violations: Vec::new(),
            editor_candidates: Vec::new(),
            effective_weights: Vec::new(),
            maintainer_support: Vec::new(),
            viewer_signals: Vec::new(),
            viewer_score_channels: Vec::new(),
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

impl HeadRenderSummary {
    fn new(input_path: &Path, store_root: Option<&Path>, doc_id: &str) -> Self {
        Self {
            input_path: input_path.to_path_buf(),
            store_root: store_root.map(Path::to_path_buf),
            status: "ok".to_string(),
            doc_id: doc_id.to_string(),
            profile_id: None,
            available_profile_ids: Vec::new(),
            effective_selection_time: None,
            selected_head: None,
            recomputed_state_hash: None,
            rendered_block_count: 0,
            rendered_blocks: Vec::new(),
            rendered_text: String::new(),
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

pub fn inspect_heads_from_path(
    input_path: &Path,
    doc_id: &str,
    requested_profile_id: Option<&str>,
) -> HeadInspectSummary {
    let (resolved_input_path, input) = match load_head_inspect_input(input_path, doc_id) {
        Ok(loaded) => loaded,
        Err(summary) => return *summary,
    };

    inspect_heads_from_loaded_input(
        resolved_input_path,
        input,
        doc_id,
        requested_profile_id,
        None,
    )
}

pub fn inspect_heads_from_store_path(
    input_path: &Path,
    store_root: &Path,
    doc_id: &str,
    requested_profile_id: Option<&str>,
) -> HeadInspectSummary {
    let (resolved_input_path, input) = match load_head_inspect_input(input_path, doc_id) {
        Ok(loaded) => loaded,
        Err(summary) => return *summary,
    };

    inspect_heads_from_loaded_input(
        resolved_input_path,
        input,
        doc_id,
        requested_profile_id,
        Some(store_root),
    )
}

pub fn render_head_from_store_path(
    input_path: &Path,
    store_root: &Path,
    doc_id: &str,
    requested_profile_id: Option<&str>,
) -> HeadRenderSummary {
    let inspect_summary =
        inspect_heads_from_store_path(input_path, store_root, doc_id, requested_profile_id);
    let mut summary =
        render_summary_from_inspect_summary(&inspect_summary, Some(store_root), doc_id);
    let Some(context) =
        build_store_render_context(store_root, doc_id, &inspect_summary, &mut summary)
    else {
        return summary;
    };
    finalize_render_summary(&mut summary, context);
    summary
}

pub fn render_head_from_path(
    input_path: &Path,
    doc_id: &str,
    requested_profile_id: Option<&str>,
) -> HeadRenderSummary {
    let inspect_summary = inspect_heads_from_path(input_path, doc_id, requested_profile_id);
    let mut summary = render_summary_from_inspect_summary(&inspect_summary, None, doc_id);
    let (resolved_input_path, input) = match load_head_inspect_input(input_path, doc_id) {
        Ok(loaded) => loaded,
        Err(inspect_failure) => {
            summary.status = "failed".to_string();
            summary.errors = inspect_failure.errors;
            return summary;
        }
    };
    summary.input_path = resolved_input_path;

    let Some(context) = build_bundle_render_context(input, &inspect_summary, &mut summary) else {
        return summary;
    };
    finalize_render_summary(&mut summary, context);
    summary
}

fn render_summary_from_inspect_summary(
    inspect_summary: &HeadInspectSummary,
    store_root: Option<&Path>,
    doc_id: &str,
) -> HeadRenderSummary {
    let mut summary = HeadRenderSummary::new(&inspect_summary.input_path, store_root, doc_id);
    summary.profile_id = inspect_summary.profile_id.clone();
    summary.available_profile_ids = inspect_summary.available_profile_ids.clone();
    summary.effective_selection_time = inspect_summary.effective_selection_time;
    summary.selected_head = inspect_summary.selected_head.clone();
    summary.notes = inspect_summary.notes.clone();

    if !inspect_summary.is_ok() {
        summary.status = "failed".to_string();
        summary.errors = inspect_summary.errors.clone();
    }

    summary
}

fn selected_head_for_render(
    inspect_summary: &HeadInspectSummary,
    summary: &mut HeadRenderSummary,
) -> Option<String> {
    if !inspect_summary.is_ok() {
        return None;
    }

    let Some(selected_head) = &inspect_summary.selected_head else {
        summary.push_error("accepted-head inspection did not select a head");
        return None;
    };

    Some(selected_head.clone())
}

fn build_store_render_context(
    store_root: &Path,
    doc_id: &str,
    inspect_summary: &HeadInspectSummary,
    summary: &mut HeadRenderSummary,
) -> Option<SelectedHeadRenderContext> {
    let selected_head = selected_head_for_render(inspect_summary, summary)?;
    let object_index = match load_doc_replay_objects_from_store(store_root, doc_id) {
        Ok(index) => index,
        Err(error) => {
            summary.push_error(format!(
                "failed to load store objects for doc '{doc_id}': {error}"
            ));
            return None;
        }
    };
    let revision_value = match load_stored_object_value(store_root, &selected_head) {
        Ok(value) => value,
        Err(error) => {
            summary.push_error(format!(
                "failed to load selected head '{selected_head}': {error}"
            ));
            return None;
        }
    };

    Some(SelectedHeadRenderContext {
        selected_head: selected_head.clone(),
        object_index,
        revision_value,
        replay_note: format!(
            "rendered accepted head '{}' from store-backed replay",
            selected_head
        ),
        replay_error_context: "failed to replay selected head",
    })
}

fn build_bundle_render_context(
    input: HeadInspectInput,
    inspect_summary: &HeadInspectSummary,
    summary: &mut HeadRenderSummary,
) -> Option<SelectedHeadRenderContext> {
    let selected_head = selected_head_for_render(inspect_summary, summary)?;
    let object_index = match build_bundle_object_index(&input.revisions, &input.objects) {
        Ok(index) => index,
        Err(error) => {
            summary.push_error(error);
            return None;
        }
    };
    let Some(revision_value) = object_index.get(&selected_head).cloned() else {
        summary.push_error(format!(
            "selected head '{}' is not available in the bundle replay object set",
            selected_head
        ));
        return None;
    };

    Some(SelectedHeadRenderContext {
        selected_head: selected_head.clone(),
        object_index,
        revision_value,
        replay_note: format!(
            "rendered accepted head '{}' from bundle-backed replay objects",
            selected_head
        ),
        replay_error_context: "failed to replay selected head from bundle objects",
    })
}

fn finalize_render_summary(summary: &mut HeadRenderSummary, context: SelectedHeadRenderContext) {
    if let Err(error) = verify_selected_head_for_render(
        &context.selected_head,
        &context.revision_value,
        &context.object_index,
    ) {
        summary.push_error(error);
        return;
    }
    let replay = match replay_revision_from_index(&context.revision_value, &context.object_index) {
        Ok(replay) => replay,
        Err(error) => {
            summary.push_error(format!(
                "{} '{}': {error}",
                context.replay_error_context, context.selected_head
            ));
            return;
        }
    };

    summary.recomputed_state_hash = Some(replay.recomputed_state_hash);
    summary.rendered_blocks = summarize_rendered_blocks(&replay.state);
    summary.rendered_block_count = summary.rendered_blocks.len();
    summary.rendered_text = render_document_text(&replay.state);
    summary.notes.push(context.replay_note);
}

fn verify_selected_head_for_render(
    selected_head: &str,
    revision_value: &Value,
    object_index: &HashMap<String, Value>,
) -> Result<(), String> {
    let verification = verify_object_value_with_object_index(revision_value, Some(object_index));
    if verification.is_ok() {
        return Ok(());
    }

    Err(format!(
        "selected head '{selected_head}' failed verification before render replay: {}",
        verification.errors.join("; ")
    ))
}

fn load_head_inspect_input(
    input_path: &Path,
    doc_id: &str,
) -> Result<(PathBuf, HeadInspectInput), Box<HeadInspectSummary>> {
    let resolved_input_path = match resolve_head_inspect_input_path(input_path) {
        Ok(path) => path,
        Err(message) => {
            let mut summary = HeadInspectSummary::new(input_path, doc_id);
            summary.push_error(message);
            return Err(Box::new(summary));
        }
    };

    let mut summary = HeadInspectSummary::new(&resolved_input_path, doc_id);
    let content = match fs::read_to_string(&resolved_input_path) {
        Ok(content) => content,
        Err(err) => {
            summary.push_error(format!("failed to read head-inspect input: {err}"));
            return Err(Box::new(summary));
        }
    };

    let input = match parse_json_strict(&content) {
        Ok(input) => input,
        Err(err) => {
            summary.push_error(format!("failed to parse head-inspect input JSON: {err}"));
            return Err(Box::new(summary));
        }
    };

    Ok((resolved_input_path, input))
}

fn inspect_heads_from_loaded_input(
    resolved_input_path: PathBuf,
    input: HeadInspectInput,
    doc_id: &str,
    requested_profile_id: Option<&str>,
    store_root: Option<&Path>,
) -> HeadInspectSummary {
    let mut summary = HeadInspectSummary::new(&resolved_input_path, doc_id);
    summary.available_profile_ids = collect_available_profile_ids(&input.profiles);
    let Some(context) = load_head_inspect_context(
        input,
        doc_id,
        requested_profile_id,
        store_root,
        &mut summary,
    ) else {
        return summary;
    };
    populate_head_inspect_profile_metadata(&mut summary, &context);

    let verified_revisions = collect_verified_revisions(
        &context.revision_values,
        doc_id,
        context.profile.effective_selection_time,
        &mut summary,
    );
    let verified_views = collect_verified_views(
        &context.view_values,
        &context.profile,
        context.profile.effective_selection_time,
        &mut summary,
    );

    summary.verified_revision_count = verified_revisions.len();
    summary.verified_view_count = verified_views.len();
    summary.viewer_signal_count = context.viewer_signals.len();
    summary.push_trace(
        "verified_inputs",
        format!(
            "verified_revisions={} verified_views={} viewer_signals={}",
            summary.verified_revision_count,
            summary.verified_view_count,
            summary.viewer_signal_count
        ),
    );
    populate_critical_violation_summary(&mut summary, &context);

    if !summary.errors.is_empty() {
        return summary;
    }

    let structural_heads = compute_eligible_heads(&verified_revisions);
    let (eligible_heads, editor_candidate_summaries, editor_trace) =
        apply_editor_admission(&structural_heads, &context.profile.editor_admission);
    summary.editor_candidates = editor_candidate_summaries;
    summary.push_trace(
        "eligible_heads",
        format!("count={}", structural_heads.len()),
    );
    for entry in editor_trace {
        summary.push_trace(entry.step, entry.detail);
    }
    if eligible_heads.is_empty() {
        summary.push_error("NO_ELIGIBLE_HEAD");
        return summary;
    }

    let (effective_weights, effective_weight_summaries, weight_trace) = compute_effective_weights(
        &verified_views,
        &context.critical_violations,
        summary
            .selector_epoch
            .expect("selector epoch should be set"),
        &context.profile,
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
        &context.profile,
        &effective_weights,
    );
    summary.maintainer_support = support_summaries;
    for entry in support_trace {
        summary.push_trace(entry.step, entry.detail);
    }

    let (viewer_head_scores, viewer_signal_summaries, viewer_score_channels, viewer_trace) =
        compute_viewer_score_channels(
            &context.viewer_signals,
            &eligible_heads,
            &context.profile,
            context.profile.effective_selection_time,
        );
    summary.viewer_signals = viewer_signal_summaries;
    summary.viewer_score_channels = viewer_score_channels;
    for entry in viewer_trace {
        summary.push_trace(entry.step, entry.detail);
    }

    populate_eligible_head_summaries(
        &mut summary,
        &eligible_heads,
        &support_map,
        &viewer_head_scores,
    );
    let viewer_gating_summary = summarize_viewer_gating(&mut summary);
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
    record_viewer_gating_trace(&mut summary, &viewer_gating_summary);

    let Some(selected) = select_head_from_eligible_summaries(&summary, &viewer_gating_summary)
    else {
        summary.push_error("NO_ACTIVE_HEAD_AFTER_VIEWER_REVIEW_OR_FREEZE");
        summary.status =
            blocked_status_for_viewer_gating(viewer_gating_summary.viewer_gating).to_string();
        return summary;
    };

    summary.selected_head = Some(selected.revision_id.clone());
    summary.status =
        success_status_for_viewer_gating(viewer_gating_summary.viewer_gating).to_string();
    summary.tie_break_reason = Some(selection_tie_break_reason(
        selected.selector_score > 0,
        viewer_gating_summary.viewer_gating,
    ));
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

fn load_head_inspect_context(
    input: HeadInspectInput,
    doc_id: &str,
    requested_profile_id: Option<&str>,
    store_root: Option<&Path>,
    summary: &mut HeadInspectSummary,
) -> Option<LoadedHeadInspectContext> {
    let (selected_profile_id, profile) =
        match resolve_head_inspect_profile(&input, requested_profile_id) {
            Ok(selected) => selected,
            Err(message) => {
                summary.push_error(message);
                return None;
            }
        };
    let HeadInspectInput {
        profile: _,
        profiles: _,
        revisions,
        objects: _,
        views,
        viewer_signals,
        critical_violations,
    } = input;
    let (revision_values, view_values) = match store_root {
        Some(store_root) => {
            match load_store_backed_selector_objects(store_root, doc_id, &profile.policy_hash) {
                Ok(values) => {
                    summary.notes.push(format!(
                        "store-backed selector inputs loaded from {} using persisted store index",
                        store_root.display()
                    ));
                    values
                }
                Err(message) => {
                    summary.push_error(message);
                    return None;
                }
            }
        }
        None => (revisions, views),
    };

    Some(LoadedHeadInspectContext {
        selected_profile_id,
        profile,
        revision_values,
        view_values,
        viewer_signals,
        critical_violations,
    })
}

fn populate_head_inspect_profile_metadata(
    summary: &mut HeadInspectSummary,
    context: &LoadedHeadInspectContext,
) {
    summary.profile_id = Some(context.selected_profile_id.clone());
    summary.effective_selection_time = Some(context.profile.effective_selection_time);
    summary.notes.push(format!(
        "selected reader profile '{}' with policy_hash={}",
        context.selected_profile_id, context.profile.policy_hash
    ));
    summary.selector_epoch = Some(selector_epoch(
        context.profile.effective_selection_time,
        context.profile.epoch_seconds,
        context.profile.epoch_zero_timestamp,
    ));
    summary.push_trace(
        "selector_epoch",
        format!(
            "effective_selection_time={} epoch_seconds={} epoch_zero_timestamp={} selector_epoch={}",
            context.profile.effective_selection_time,
            context.profile.epoch_seconds,
            context.profile.epoch_zero_timestamp,
            summary.selector_epoch.expect("selector epoch should be set")
        ),
    );
    summary.notes.push(
        "minimal selector mode: critical violations are bundle-provided fixture evidence; external dispute / penalty objects are not implemented yet".to_string(),
    );
}

fn populate_critical_violation_summary(
    summary: &mut HeadInspectSummary,
    context: &LoadedHeadInspectContext,
) {
    summary.push_trace(
        "critical_violations",
        if context.critical_violations.is_empty() {
            "count=0 affected_maintainers=0".to_string()
        } else {
            let affected_maintainers = context
                .critical_violations
                .iter()
                .map(|violation| violation.maintainer.clone())
                .collect::<BTreeSet<_>>()
                .len();
            format!(
                "count={} affected_maintainers={affected_maintainers}",
                context.critical_violations.len()
            )
        },
    );
    summary.critical_violations = context
        .critical_violations
        .iter()
        .map(|violation| CriticalViolationSummary {
            maintainer: violation.maintainer.clone(),
            timestamp: violation.timestamp,
            selector_epoch: selector_epoch_for_view(
                violation.timestamp,
                context.profile.epoch_seconds,
                context.profile.epoch_zero_timestamp,
            ),
            reason: violation.reason.clone(),
        })
        .collect();
}

fn populate_eligible_head_summaries(
    summary: &mut HeadInspectSummary,
    eligible_heads: &[VerifiedRevision],
    support_map: &HashMap<String, MaintainerSupport>,
    viewer_head_scores: &HashMap<String, ViewerHeadScore>,
) {
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
            let viewer_score = viewer_head_scores
                .get(revision.revision_id.as_str())
                .cloned()
                .unwrap_or_default();
            let selector_score = weighted_support
                .saturating_add(viewer_score.viewer_bonus)
                .saturating_sub(viewer_score.viewer_penalty);

            EligibleHeadSummary {
                revision_id: revision.revision_id.clone(),
                author: revision.author.clone(),
                editor_admitted: summary
                    .editor_candidates
                    .iter()
                    .find(|candidate| candidate.revision_id == revision.revision_id)
                    .map(|candidate| candidate.editor_admitted)
                    .unwrap_or(false),
                formal_candidate: summary
                    .editor_candidates
                    .iter()
                    .find(|candidate| candidate.revision_id == revision.revision_id)
                    .map(|candidate| candidate.formal_candidate)
                    .unwrap_or(false),
                revision_timestamp: revision.timestamp,
                maintainer_score: weighted_support,
                weighted_support,
                supporter_count,
                viewer_bonus: viewer_score.viewer_bonus,
                viewer_penalty: viewer_score.viewer_penalty,
                selector_score,
            }
        })
        .collect::<Vec<_>>();

    eligible_summaries.sort_by(|left, right| left.revision_id.cmp(&right.revision_id));
    summary.eligible_heads = eligible_summaries;
    for channel in &mut summary.viewer_score_channels {
        if let Some(head) = summary
            .eligible_heads
            .iter()
            .find(|entry| entry.revision_id == channel.revision_id)
        {
            channel.maintainer_score = head.maintainer_score;
            channel.selector_score = head.selector_score;
        }
    }
}

fn summarize_viewer_gating(summary: &mut HeadInspectSummary) -> ViewerGatingSummary {
    let review_delayed_revision_ids = summary
        .viewer_score_channels
        .iter()
        .filter(|channel| channel.viewer_review_state == ViewerReviewState::ReviewPressure)
        .map(|channel| channel.revision_id.clone())
        .collect::<BTreeSet<_>>();
    let frozen_revision_ids = summary
        .viewer_score_channels
        .iter()
        .filter(|channel| channel.viewer_review_state == ViewerReviewState::FreezePressure)
        .map(|channel| channel.revision_id.clone())
        .collect::<BTreeSet<_>>();
    if !review_delayed_revision_ids.is_empty() {
        let delayed_list = review_delayed_revision_ids
            .iter()
            .cloned()
            .collect::<Vec<_>>()
            .join(", ");
        summary.notes.push(format!(
            "review pressure delays candidate activation for: {delayed_list}"
        ));
    }
    if !frozen_revision_ids.is_empty() {
        let frozen_list = frozen_revision_ids
            .iter()
            .cloned()
            .collect::<Vec<_>>()
            .join(", ");
        summary.notes.push(format!(
            "temporary freeze blocks candidate activation for: {frozen_list}"
        ));
    }

    ViewerGatingSummary {
        viewer_gating: viewer_selection_gating(
            !review_delayed_revision_ids.is_empty(),
            !frozen_revision_ids.is_empty(),
        ),
        review_delayed_revision_ids,
        frozen_revision_ids,
    }
}

fn record_viewer_gating_trace(
    summary: &mut HeadInspectSummary,
    viewer_gating_summary: &ViewerGatingSummary,
) {
    let active_candidate_count = summary
        .eligible_heads
        .iter()
        .filter(|head| {
            !viewer_gating_summary
                .review_delayed_revision_ids
                .contains(head.revision_id.as_str())
                && !viewer_gating_summary
                    .frozen_revision_ids
                    .contains(head.revision_id.as_str())
        })
        .count();
    if !viewer_gating_summary.review_delayed_revision_ids.is_empty() {
        summary.push_trace(
            "viewer_review",
            format!(
                "delayed_candidates={} active_candidates={} delayed_revision_ids={}",
                viewer_gating_summary.review_delayed_revision_ids.len(),
                active_candidate_count,
                viewer_gating_summary
                    .review_delayed_revision_ids
                    .iter()
                    .cloned()
                    .collect::<Vec<_>>()
                    .join(", ")
            ),
        );
    }
    if !viewer_gating_summary.frozen_revision_ids.is_empty() {
        summary.push_trace(
            "viewer_freeze",
            format!(
                "blocked_candidates={} active_candidates={} blocked_revision_ids={}",
                viewer_gating_summary.frozen_revision_ids.len(),
                active_candidate_count,
                viewer_gating_summary
                    .frozen_revision_ids
                    .iter()
                    .cloned()
                    .collect::<Vec<_>>()
                    .join(", ")
            ),
        );
    }
}

fn select_head_from_eligible_summaries(
    summary: &HeadInspectSummary,
    viewer_gating_summary: &ViewerGatingSummary,
) -> Option<EligibleHeadSummary> {
    summary
        .eligible_heads
        .iter()
        .filter(|head| {
            !viewer_gating_summary
                .review_delayed_revision_ids
                .contains(head.revision_id.as_str())
                && !viewer_gating_summary
                    .frozen_revision_ids
                    .contains(head.revision_id.as_str())
        })
        .max_by(|left, right| {
            left.selector_score
                .cmp(&right.selector_score)
                .then(left.revision_timestamp.cmp(&right.revision_timestamp))
                .then_with(|| right.revision_id.cmp(&left.revision_id))
        })
        .cloned()
}

fn load_store_backed_selector_objects(
    store_root: &Path,
    doc_id: &str,
    policy_hash: &str,
) -> Result<(Vec<Value>, Vec<Value>), String> {
    let manifest = load_store_index_manifest(store_root).map_err(|error| error.to_string())?;
    let revision_ids = manifest
        .doc_revisions
        .get(doc_id)
        .cloned()
        .unwrap_or_default();
    let revisions = revision_ids
        .iter()
        .map(|revision_id| {
            load_stored_object_value(store_root, revision_id).map_err(|error| error.to_string())
        })
        .collect::<Result<Vec<_>, _>>()?;
    let mut view_ids = manifest
        .profile_views
        .get(policy_hash)
        .cloned()
        .unwrap_or_default();
    if view_ids.is_empty() {
        view_ids = manifest
            .view_governance
            .iter()
            .filter(|record| record.profile_id == policy_hash)
            .map(|record| record.view_id.clone())
            .collect::<BTreeSet<_>>()
            .into_iter()
            .collect();
    }
    let views = view_ids
        .iter()
        .map(|view_id| {
            load_stored_object_value(store_root, view_id).map_err(|error| error.to_string())
        })
        .collect::<Result<Vec<_>, _>>()?;

    Ok((revisions, views))
}

fn resolve_head_inspect_profile(
    input: &HeadInspectInput,
    requested_profile_id: Option<&str>,
) -> Result<(String, HeadInspectProfile), String> {
    if input.profile.is_some() && !input.profiles.is_empty() {
        return Err("head input cannot declare both 'profile' and 'profiles'".to_string());
    }

    if let Some(requested_profile_id) = requested_profile_id {
        if let Some(profile) = input.profiles.get(requested_profile_id) {
            return Ok((requested_profile_id.to_string(), profile.clone()));
        }

        if input.profile.is_some() {
            return Err(format!(
                "head input does not declare named profiles; remove --profile-id '{}'",
                requested_profile_id
            ));
        }

        if input.profiles.is_empty() {
            return Err("head input must declare either 'profile' or 'profiles'".to_string());
        }

        return Err(format!(
            "unknown --profile-id '{}' for head input; available profiles: {}",
            requested_profile_id,
            available_profile_ids(&input.profiles)
        ));
    }

    if let Some(profile) = &input.profile {
        return Ok((profile.policy_hash.clone(), profile.clone()));
    }

    match input.profiles.len() {
        0 => Err("head input must declare either 'profile' or 'profiles'".to_string()),
        1 => {
            let (profile_id, profile) = input
                .profiles
                .iter()
                .next()
                .expect("single profile should exist");
            Ok((profile_id.clone(), profile.clone()))
        }
        _ => Err(format!(
            "head input declares multiple named profiles; pass --profile-id ({})",
            available_profile_ids(&input.profiles)
        )),
    }
}

fn collect_available_profile_ids(profiles: &BTreeMap<String, HeadInspectProfile>) -> Vec<String> {
    profiles.keys().cloned().collect()
}

fn available_profile_ids(profiles: &BTreeMap<String, HeadInspectProfile>) -> String {
    collect_available_profile_ids(profiles).join(", ")
}

fn build_bundle_object_index(
    revisions: &[Value],
    objects: &[Value],
) -> Result<HashMap<String, Value>, String> {
    let mut object_index = HashMap::new();

    for value in revisions.iter().chain(objects.iter()) {
        let verification = verify_object_value(value);
        if !verification.is_ok() {
            return Err(format!(
                "bundle replay object failed verification: {}",
                verification.errors.join("; ")
            ));
        }

        let object_id = bundle_object_id(value)?;
        if let Some(existing) = object_index.insert(object_id.clone(), value.clone()) {
            let _ = existing;
            return Err(format!(
                "duplicate bundle replay object ID '{}' found while building render object index",
                object_id
            ));
        }
    }

    Ok(object_index)
}

fn bundle_object_id(value: &Value) -> Result<String, String> {
    let envelope = parse_object_envelope(value).map_err(|error| error.to_string())?;
    match envelope.declared_id() {
        Ok(Some(object_id)) => Ok(object_id.to_string()),
        Ok(None) => match envelope.logical_id() {
            Ok(Some(object_id)) => Ok(object_id.to_string()),
            Ok(None) => {
                Err("bundle replay object does not expose a declared or logical ID".to_string())
            }
            Err(StringFieldError::Missing) => {
                Err("bundle replay object is missing its logical ID field".to_string())
            }
            Err(StringFieldError::WrongType) => {
                Err("bundle replay object logical ID field must be a string".to_string())
            }
        },
        Err(StringFieldError::Missing) => {
            Err("bundle replay object is missing its derived ID field".to_string())
        }
        Err(StringFieldError::WrongType) => {
            Err("bundle replay object derived ID field must be a string".to_string())
        }
    }
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
        let author = match object.get("author").and_then(Value::as_str) {
            Some(value) => value,
            None => {
                summary.push_error("revision is missing string field 'author'");
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
            author: author.to_string(),
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

fn apply_editor_admission(
    revisions: &[VerifiedRevision],
    admission: &EditorAdmissionProfile,
) -> (
    Vec<VerifiedRevision>,
    Vec<EditorCandidateSummary>,
    Vec<DecisionTraceEntry>,
) {
    let admitted_keys = admission
        .admitted_keys
        .iter()
        .map(String::as_str)
        .collect::<BTreeSet<_>>();
    let mut eligible = Vec::new();
    let mut summaries = revisions
        .iter()
        .map(|revision| {
            let editor_admitted = admitted_keys.contains(revision.author.as_str());
            let candidate_eligible = match admission.mode {
                EditorCandidateMode::Open | EditorCandidateMode::Mixed => true,
                EditorCandidateMode::AdmittedOnly => editor_admitted,
            };
            let formal_candidate = match admission.mode {
                EditorCandidateMode::Open => true,
                EditorCandidateMode::AdmittedOnly | EditorCandidateMode::Mixed => editor_admitted,
            };
            if candidate_eligible {
                eligible.push(revision.clone());
            }
            EditorCandidateSummary {
                revision_id: revision.revision_id.clone(),
                author: revision.author.clone(),
                editor_admitted,
                candidate_eligible,
                formal_candidate,
            }
        })
        .collect::<Vec<_>>();
    summaries.sort_by(|left, right| left.revision_id.cmp(&right.revision_id));

    let mode = match admission.mode {
        EditorCandidateMode::Open => "open",
        EditorCandidateMode::AdmittedOnly => "admitted-only",
        EditorCandidateMode::Mixed => "mixed",
    };
    let trace = vec![DecisionTraceEntry {
        step: "editor_admission".to_string(),
        detail: format!(
            "mode={} structural_heads={} eligible={} formal={} admitted={}",
            mode,
            revisions.len(),
            summaries
                .iter()
                .filter(|entry| entry.candidate_eligible)
                .count(),
            summaries
                .iter()
                .filter(|entry| entry.formal_candidate)
                .count(),
            summaries
                .iter()
                .filter(|entry| entry.editor_admitted)
                .count(),
        ),
    }];

    (eligible, summaries, trace)
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
    let mut view_admitted_maintainers = 0_usize;
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
        let view_admitted = is_view_maintainer_admitted(&maintainer, &profile.view_admission);
        let effective_weight =
            effective_weight_for_epoch(&maintainer, &counts, &violations, selector_epoch, profile);
        let admitted =
            view_admitted && is_admitted_in_epoch(&counts, &violations, selector_epoch, profile);
        if view_admitted {
            view_admitted_maintainers += 1;
        }
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
            view_admitted,
            admitted,
            effective_weight,
            valid_view_counts: to_epoch_count_summaries(&counts),
            critical_violation_counts: to_epoch_count_summaries(&violations),
        });
    }

    let trace = vec![
        DecisionTraceEntry {
            step: "view_admission".to_string(),
            detail: format!(
                "mode={} maintainers={} view_admitted={}",
                view_admission_mode_label(&profile.view_admission.mode),
                summaries.len(),
                view_admitted_maintainers
            ),
        },
        DecisionTraceEntry {
            step: "effective_weight".to_string(),
            detail: format!(
                "maintainers={} admitted={} penalized={} zero_weight={} max_effective_weight={}",
                summaries.len(),
                admitted_maintainers,
                penalized_maintainers,
                zero_weight_maintainers,
                max_effective_weight
            ),
        },
    ];

    (weights, summaries, trace)
}

fn compute_viewer_score_channels(
    signals: &[HeadInspectViewerSignal],
    eligible_heads: &[VerifiedRevision],
    profile: &HeadInspectProfile,
    effective_selection_time: u64,
) -> (
    HashMap<String, ViewerHeadScore>,
    Vec<ViewerSignalSummary>,
    Vec<ViewerScoreChannelSummary>,
    Vec<DecisionTraceEntry>,
) {
    let eligible_ids = eligible_heads
        .iter()
        .map(|revision| revision.revision_id.as_str())
        .collect::<BTreeSet<_>>();
    let score_mode_enabled = profile.viewer_score.mode == ViewerScoreMode::BoundedBonusPenalty;
    let effective_weight_cap = if profile.viewer_score.signal_weight_cap == 0 {
        u64::MAX
    } else {
        profile.viewer_score.signal_weight_cap
    };

    let mut per_head_scores: HashMap<String, ViewerHeadScore> = HashMap::new();
    let mut signal_summaries = Vec::new();
    let mut eligible_signal_count = 0_u64;
    let mut contributing_signal_count = 0_u64;

    for signal in signals {
        let base_selector_eligible = score_mode_enabled
            && signal.signal_status == ViewerSignalStatus::Active
            && signal.created_at <= effective_selection_time
            && signal.expires_at > effective_selection_time
            && eligible_ids.contains(signal.candidate_revision_id.as_str())
            && (!profile.viewer_score.admission_required
                || signal.viewer_admission_status == ViewerAdmissionStatus::Admitted)
            && signal.viewer_identity_tier >= profile.viewer_score.min_identity_tier
            && signal.viewer_reputation_band >= profile.viewer_score.min_reputation_band;
        let selector_eligible = base_selector_eligible
            && match signal.signal_type {
                ViewerSignalType::Challenge => {
                    signal.evidence_ref.is_some()
                        && matches!(
                            signal.confidence_level,
                            ViewerConfidenceLevel::Medium | ViewerConfidenceLevel::High
                        )
                }
                ViewerSignalType::Approval | ViewerSignalType::Objection => true,
            };

        let signal_weight = confidence_weight(&signal.confidence_level).min(effective_weight_cap);
        let effective_signal_weight = if selector_eligible
            && matches!(
                signal.signal_type,
                ViewerSignalType::Approval | ViewerSignalType::Objection
            ) {
            signal_weight
        } else {
            0
        };

        if selector_eligible {
            eligible_signal_count += 1;
        }
        if effective_signal_weight > 0 {
            contributing_signal_count += 1;
            let entry = per_head_scores
                .entry(signal.candidate_revision_id.clone())
                .or_default();
            match signal.signal_type {
                ViewerSignalType::Approval => {
                    entry.viewer_bonus = entry.viewer_bonus.saturating_add(effective_signal_weight);
                    entry.approval_signal_count += 1;
                }
                ViewerSignalType::Objection => {
                    entry.viewer_penalty =
                        entry.viewer_penalty.saturating_add(effective_signal_weight);
                    entry.objection_signal_count += 1;
                }
                ViewerSignalType::Challenge => {}
            }
        }

        if selector_eligible && signal.signal_type == ViewerSignalType::Challenge {
            let entry = per_head_scores
                .entry(signal.candidate_revision_id.clone())
                .or_default();
            entry.challenge_signal_count += 1;
            if signal.evidence_ref.is_some() {
                let challenge_weight =
                    confidence_weight(&signal.confidence_level).min(effective_weight_cap);
                entry.challenge_review_pressure = entry
                    .challenge_review_pressure
                    .saturating_add(challenge_weight);
                if signal.confidence_level == ViewerConfidenceLevel::High {
                    entry.challenge_freeze_pressure = entry
                        .challenge_freeze_pressure
                        .saturating_add(challenge_weight);
                }
            }
        }

        signal_summaries.push(ViewerSignalSummary {
            signal_id: signal.signal_id.clone(),
            viewer_id: signal.viewer_id.clone(),
            candidate_revision_id: signal.candidate_revision_id.clone(),
            signal_type: signal.signal_type.clone(),
            reason_code: signal.reason_code.clone(),
            confidence_level: signal.confidence_level.clone(),
            evidence_ref: signal.evidence_ref.clone(),
            created_at: signal.created_at,
            expires_at: signal.expires_at,
            signal_status: signal.signal_status.clone(),
            viewer_identity_tier: signal.viewer_identity_tier.clone(),
            viewer_admission_status: signal.viewer_admission_status.clone(),
            viewer_reputation_band: signal.viewer_reputation_band.clone(),
            selector_eligible,
            effective_signal_weight,
        });
    }

    for score in per_head_scores.values_mut() {
        score.viewer_bonus = score.viewer_bonus.min(profile.viewer_score.bonus_cap);
        score.viewer_penalty = score.viewer_penalty.min(profile.viewer_score.penalty_cap);
    }

    signal_summaries.sort_by(|left, right| left.signal_id.cmp(&right.signal_id));

    let mut channel_summaries = eligible_heads
        .iter()
        .map(|revision| {
            let score = per_head_scores
                .get(revision.revision_id.as_str())
                .cloned()
                .unwrap_or_default();
            ViewerScoreChannelSummary {
                revision_id: revision.revision_id.clone(),
                maintainer_score: 0,
                viewer_bonus: score.viewer_bonus,
                viewer_penalty: score.viewer_penalty,
                approval_signal_count: score.approval_signal_count,
                objection_signal_count: score.objection_signal_count,
                challenge_signal_count: score.challenge_signal_count,
                challenge_review_pressure: score.challenge_review_pressure,
                challenge_freeze_pressure: score.challenge_freeze_pressure,
                viewer_review_state: viewer_review_state_for_score(&score),
                selector_score: 0,
            }
        })
        .collect::<Vec<_>>();
    channel_summaries.sort_by(|left, right| left.revision_id.cmp(&right.revision_id));

    let trace = if score_mode_enabled || !signals.is_empty() {
        vec![DecisionTraceEntry {
            step: "viewer_score_channels".to_string(),
            detail: format!(
                "mode={} signals={} eligible={} contributing={} affected_heads={} bonus_cap={} penalty_cap={}",
                viewer_score_mode_label(&profile.viewer_score.mode),
                signals.len(),
                eligible_signal_count,
                contributing_signal_count,
                channel_summaries
                    .iter()
                    .filter(|entry| entry.viewer_bonus > 0 || entry.viewer_penalty > 0)
                    .count(),
                profile.viewer_score.bonus_cap,
                profile.viewer_score.penalty_cap
            ),
        }]
    } else {
        Vec::new()
    };

    (per_head_scores, signal_summaries, channel_summaries, trace)
}

fn confidence_weight(confidence_level: &ViewerConfidenceLevel) -> u64 {
    match confidence_level {
        ViewerConfidenceLevel::Low => 1,
        ViewerConfidenceLevel::Medium => 2,
        ViewerConfidenceLevel::High => 3,
    }
}

fn viewer_score_mode_label(mode: &ViewerScoreMode) -> &'static str {
    match mode {
        ViewerScoreMode::Disabled => "disabled",
        ViewerScoreMode::BoundedBonusPenalty => "bounded-bonus-penalty",
    }
}

fn viewer_review_state_for_score(score: &ViewerHeadScore) -> ViewerReviewState {
    if score.challenge_freeze_pressure > 0 {
        ViewerReviewState::FreezePressure
    } else if score.challenge_review_pressure > 0 {
        ViewerReviewState::ReviewPressure
    } else {
        ViewerReviewState::None
    }
}

fn viewer_selection_gating(has_review_delay: bool, has_freeze_block: bool) -> Option<&'static str> {
    match (has_review_delay, has_freeze_block) {
        (false, false) => None,
        (true, false) => Some("viewer-review-delay"),
        (false, true) => Some("viewer-freeze-block"),
        (true, true) => Some("viewer-review-delay-and-freeze-block"),
    }
}

fn success_status_for_viewer_gating(viewer_gating: Option<&str>) -> &'static str {
    match viewer_gating {
        None => "ok",
        Some("viewer-review-delay") => "ok-with-viewer-review-delay",
        Some("viewer-freeze-block") => "ok-with-viewer-freeze-block",
        Some("viewer-review-delay-and-freeze-block") => {
            "ok-with-viewer-review-delay-and-freeze-block"
        }
        Some(_) => "ok",
    }
}

fn blocked_status_for_viewer_gating(viewer_gating: Option<&str>) -> &'static str {
    match viewer_gating {
        Some("viewer-review-delay") => "blocked-by-viewer-review-delay",
        Some("viewer-freeze-block") => "blocked-by-viewer-freeze-block",
        Some("viewer-review-delay-and-freeze-block") => {
            "blocked-by-viewer-review-delay-and-freeze-block"
        }
        _ => "failed",
    }
}

fn selection_tie_break_reason(selected_by_score: bool, viewer_gating: Option<&str>) -> String {
    let base = if selected_by_score {
        "higher_selector_score"
    } else {
        "newer_revision_timestamp_or_lexicographic_tiebreak"
    };
    match viewer_gating {
        None => base.to_string(),
        Some(gating) => format!("{base}_after_{gating}"),
    }
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
    maintainer: &str,
    counts: &BTreeMap<i64, u64>,
    violations: &BTreeMap<i64, u64>,
    epoch: i64,
    profile: &HeadInspectProfile,
) -> u64 {
    if !is_view_maintainer_admitted(maintainer, &profile.view_admission) {
        return 0;
    }

    if !is_admitted_in_epoch(counts, violations, epoch, profile) {
        return 0;
    }

    if epoch <= 0 || !is_admitted_in_epoch(counts, violations, epoch - 1, profile) {
        return 1;
    }

    let previous_weight =
        effective_weight_for_epoch(maintainer, counts, violations, epoch - 1, profile);
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

fn is_view_maintainer_admitted(maintainer: &str, admission: &ViewAdmissionProfile) -> bool {
    match admission.mode {
        ViewMaintainerAdmissionMode::Open => true,
        ViewMaintainerAdmissionMode::AdmittedOnly => admission
            .admitted_keys
            .iter()
            .any(|candidate| candidate == maintainer),
    }
}

fn view_admission_mode_label(mode: &ViewMaintainerAdmissionMode) -> &'static str {
    match mode {
        ViewMaintainerAdmissionMode::Open => "open",
        ViewMaintainerAdmissionMode::AdmittedOnly => "admitted-only",
    }
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
    prefixed_canonical_hash(value, "hash")
}

fn summarize_rendered_blocks(state: &DocumentState) -> Vec<RenderedBlockSummary> {
    let mut blocks = Vec::new();
    collect_rendered_blocks(&state.blocks, 0, &mut blocks);
    blocks
}

fn collect_rendered_blocks(
    source: &[BlockObject],
    depth: usize,
    blocks: &mut Vec<RenderedBlockSummary>,
) {
    for block in source {
        blocks.push(RenderedBlockSummary {
            block_id: block.block_id.clone(),
            block_type: block.block_type.clone(),
            depth,
            content: block.content.clone(),
            child_count: block.children.len(),
        });
        collect_rendered_blocks(&block.children, depth + 1, blocks);
    }
}

fn render_document_text(state: &DocumentState) -> String {
    let mut lines = Vec::new();
    render_block_lines(&state.blocks, 0, &mut lines);
    lines.join("\n")
}

fn render_block_lines(blocks: &[BlockObject], depth: usize, lines: &mut Vec<String>) {
    for block in blocks {
        let indent = "  ".repeat(depth);
        let line = if block.content.is_empty() {
            format!("{indent}[{}]", block.block_type)
        } else {
            format!("{indent}{}", block.content)
        };
        lines.push(line);
        render_block_lines(&block.children, depth + 1, lines);
    }
}
