use std::path::PathBuf;

use serde::Serialize;
use serde_json::Value;

use crate::protocol::BlockObject;
use crate::replay::DocumentState;
use crate::store::StoredObjectRecord;

#[derive(Debug, Clone)]
pub struct DocumentCreateParams {
    pub doc_id: String,
    pub title: String,
    pub language: String,
    pub timestamp: u64,
}

#[derive(Debug, Clone)]
pub struct PatchCreateParams {
    pub doc_id: String,
    pub base_revision: String,
    pub timestamp: u64,
    pub ops: Value,
}

#[derive(Debug, Clone)]
pub struct RevisionCommitParams {
    pub doc_id: String,
    pub parents: Vec<String>,
    pub patches: Vec<String>,
    pub merge_strategy: Option<String>,
    pub timestamp: u64,
}

#[derive(Debug, Clone)]
pub struct MergeRevisionCreateParams {
    pub doc_id: String,
    pub parents: Vec<String>,
    pub resolved_state: DocumentState,
    pub merge_strategy: String,
    pub timestamp: u64,
}

#[derive(Debug, Clone, Copy, Serialize, PartialEq, Eq)]
#[serde(rename_all = "kebab-case")]
pub enum MergeOutcome {
    AutoMerged,
    MultiVariant,
    ManualCurationRequired,
}

impl MergeOutcome {
    pub fn as_str(self) -> &'static str {
        match self {
            Self::AutoMerged => "auto-merged",
            Self::MultiVariant => "multi-variant",
            Self::ManualCurationRequired => "manual-curation-required",
        }
    }
}

#[derive(Debug, Clone, Copy, Serialize, PartialEq, Eq)]
#[serde(rename_all = "kebab-case")]
pub enum MergeReasonSubjectKind {
    Block,
    MetadataKey,
}

#[derive(Debug, Clone, Copy, Serialize, PartialEq, Eq)]
#[serde(rename_all = "kebab-case")]
pub enum MergeReasonVariantKind {
    Content,
    Metadata,
    ParentPlacement,
    SiblingPlacement,
}

#[derive(Debug, Clone, Copy, Serialize, PartialEq, Eq)]
#[serde(rename_all = "kebab-case")]
pub enum MergeReasonKind {
    SelectedNonPrimaryParentVariant,
    KeptPrimaryParentVariantOverCompetingNonPrimaryAlternative,
    MultipleCompetingAlternativesRemainAfterSelectedVariant,
    MultipleCompetingAlternativesRemainAfterKeepingPrimaryVariant,
    NoMatchingParentPlacement,
    NoMatchingSiblingPlacement,
}

#[derive(Debug, Clone, Copy, Serialize, PartialEq, Eq)]
#[serde(rename_all = "kebab-case")]
pub enum MergeReasonBranchKind {
    AdoptedNonPrimaryAddition,
    AdoptedNonPrimaryReplacement,
    AdoptedNonPrimaryRemoval,
    AdoptedNonPrimaryReplacementWhileCompetingRemovalRemains,
    AdoptedNonPrimaryReplacementWhileCompetingReplacementsAndRemovalRemain,
    AdoptedNonPrimaryRemovalWhileCompetingReplacementRemains,
    AdoptedNonPrimaryRemovalWhileCompetingReplacementAndRemovalsRemain,
    KeptPrimaryAbsenceOverNonPrimaryAddition,
    KeptPrimaryVariantOverNonPrimaryReplacement,
    KeptPrimaryVariantOverNonPrimaryRemoval,
    KeptPrimaryVariantOverMixedNonPrimaryAlternatives,
    KeptPrimaryVariantOverMultipleCompetingNonPrimaryReplacementsAndRemovals,
    MultipleCompetingNonPrimaryAdditions,
    MultipleCompetingNonPrimaryReplacements,
    MultipleCompetingNonPrimaryRemovals,
    MultipleCompetingMixedNonPrimaryAlternatives,
    MultipleCompetingNonPrimaryReplacementsAndRemovals,
}

#[derive(Debug, Clone, Serialize, PartialEq, Eq)]
pub struct MergeReasonDetail {
    pub subject_kind: MergeReasonSubjectKind,
    pub subject_id: String,
    pub variant_kind: MergeReasonVariantKind,
    pub reason_kind: MergeReasonKind,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub branch_kind: Option<MergeReasonBranchKind>,
    pub primary_variant: String,
    pub resolved_variant: String,
    pub competing_variants: Vec<String>,
}

#[derive(Debug, Clone, Serialize)]
pub struct DocumentCreateSummary {
    pub store_root: PathBuf,
    pub status: String,
    pub doc_id: String,
    pub document_object_id: String,
    pub genesis_revision_id: String,
    pub written_object_count: usize,
    pub existing_object_count: usize,
    pub stored_objects: Vec<StoredObjectRecord>,
    pub index_manifest_path: Option<PathBuf>,
    pub notes: Vec<String>,
    pub errors: Vec<String>,
}

#[derive(Debug, Clone, Serialize)]
pub struct PatchCreateSummary {
    pub store_root: PathBuf,
    pub status: String,
    pub doc_id: String,
    pub patch_id: String,
    pub base_revision: String,
    pub written_object_count: usize,
    pub existing_object_count: usize,
    pub stored_object: StoredObjectRecord,
    pub index_manifest_path: Option<PathBuf>,
    pub notes: Vec<String>,
    pub errors: Vec<String>,
}

#[derive(Debug, Clone, Serialize)]
pub struct RevisionCommitSummary {
    pub store_root: PathBuf,
    pub status: String,
    pub doc_id: String,
    pub revision_id: String,
    pub parent_revision_ids: Vec<String>,
    pub patch_ids: Vec<String>,
    pub recomputed_state_hash: String,
    pub written_object_count: usize,
    pub existing_object_count: usize,
    pub stored_object: StoredObjectRecord,
    pub index_manifest_path: Option<PathBuf>,
    pub notes: Vec<String>,
    pub errors: Vec<String>,
}

#[derive(Debug, Clone, Serialize)]
pub struct MergeRevisionCreateSummary {
    pub store_root: PathBuf,
    pub status: String,
    pub doc_id: String,
    pub merge_outcome: MergeOutcome,
    pub merge_reasons: Vec<String>,
    pub merge_reason_details: Vec<MergeReasonDetail>,
    pub parent_revision_ids: Vec<String>,
    pub patch_id: String,
    pub patch_op_count: usize,
    pub revision_id: String,
    pub recomputed_state_hash: String,
    pub written_object_count: usize,
    pub existing_object_count: usize,
    pub stored_objects: Vec<StoredObjectRecord>,
    pub index_manifest_path: Option<PathBuf>,
    pub notes: Vec<String>,
    pub errors: Vec<String>,
}

#[derive(Debug, Clone, Serialize)]
pub struct ManualCurationSummary {
    pub status: String,
    pub doc_id: String,
    pub merge_outcome: MergeOutcome,
    pub merge_reasons: Vec<String>,
    pub merge_reason_details: Vec<MergeReasonDetail>,
    pub parent_revision_ids: Vec<String>,
    pub errors: Vec<String>,
}

#[derive(Debug, Clone)]
pub(crate) struct MergeAssessment {
    pub(crate) outcome: MergeOutcome,
    pub(crate) reasons: Vec<String>,
    pub(crate) reason_details: Vec<MergeReasonDetail>,
}

#[derive(Debug, Clone)]
pub(crate) struct BlockPlacement {
    pub(crate) block: BlockObject,
    pub(crate) parent_block_id: Option<String>,
    pub(crate) previous_sibling_id: Option<String>,
}
