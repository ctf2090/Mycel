use std::collections::{BTreeSet, HashMap};
use std::path::{Path, PathBuf};

use base64::Engine;
use ed25519_dalek::{Signer, SigningKey};
use serde::Serialize;
use serde_json::{json, Value};

use crate::protocol::{
    canonical_json, recompute_object_id, signed_payload_bytes, BlockObject, PatchObject,
    PatchOperation, RevisionObject, CORE_PROTOCOL_VERSION,
};
use crate::replay::{
    apply_patch_ops, compute_state_hash, replay_revision, replay_revision_from_index,
    DocumentState, GENESIS_BASE_REVISION,
};
use crate::store::{
    load_store_object_index, load_stored_object_value, write_object_value_to_store,
    StoreRebuildError, StoredObjectRecord,
};

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

#[derive(Debug, Clone)]
struct MergeAssessment {
    outcome: MergeOutcome,
    reasons: Vec<String>,
}

#[derive(Debug, Clone)]
struct BlockPlacement {
    block: BlockObject,
    parent_block_id: Option<String>,
    depth: usize,
}

pub fn parse_signing_key_seed(seed: &str) -> Result<SigningKey, String> {
    let decoded = base64::engine::general_purpose::STANDARD
        .decode(seed.trim())
        .map_err(|error| format!("failed to decode base64 signing key seed: {error}"))?;
    let bytes: [u8; 32] = decoded
        .try_into()
        .map_err(|_| "signing key seed must decode to exactly 32 bytes".to_string())?;
    Ok(SigningKey::from_bytes(&bytes))
}

pub fn signer_id(signing_key: &SigningKey) -> String {
    format!(
        "pk:ed25519:{}",
        base64::engine::general_purpose::STANDARD.encode(signing_key.verifying_key().as_bytes())
    )
}

pub fn create_document_in_store(
    store_root: &Path,
    signing_key: &SigningKey,
    params: &DocumentCreateParams,
) -> Result<DocumentCreateSummary, StoreRebuildError> {
    if params.doc_id.is_empty() {
        return Err(StoreRebuildError::new("document doc_id must not be empty"));
    }
    if params.title.is_empty() {
        return Err(StoreRebuildError::new("document title must not be empty"));
    }
    if params.language.is_empty() {
        return Err(StoreRebuildError::new(
            "document language must not be empty",
        ));
    }
    if load_stored_object_value(store_root, &params.doc_id).is_ok() {
        return Err(StoreRebuildError::new(format!(
            "document '{}' already exists in the store",
            params.doc_id
        )));
    }

    let created_by = signer_id(signing_key);
    let empty_state = DocumentState {
        doc_id: params.doc_id.clone(),
        blocks: Vec::new(),
        metadata: serde_json::Map::new(),
    };
    let state_hash = compute_state_hash(&empty_state).map_err(|error| {
        StoreRebuildError::new(format!("failed to compute genesis state_hash: {error}"))
    })?;
    let mut genesis_revision = json!({
        "type": "revision",
        "version": CORE_PROTOCOL_VERSION,
        "doc_id": params.doc_id,
        "parents": [],
        "patches": [],
        "state_hash": state_hash,
        "author": created_by,
        "timestamp": params.timestamp
    });
    let genesis_revision_id = recompute_object_id(&genesis_revision, "revision_id", "rev")
        .map_err(StoreRebuildError::new)?;
    genesis_revision["revision_id"] = Value::String(genesis_revision_id.clone());
    genesis_revision["signature"] =
        Value::String(sign_object_value(signing_key, &genesis_revision)?);

    let document = json!({
        "type": "document",
        "version": CORE_PROTOCOL_VERSION,
        "doc_id": params.doc_id,
        "title": params.title,
        "language": params.language,
        "content_model": "block-tree",
        "created_at": params.timestamp,
        "created_by": signer_id(signing_key),
        "genesis_revision": genesis_revision_id
    });

    let revision_write = write_object_value_to_store(store_root, &genesis_revision)?;
    let document_write = write_object_value_to_store(store_root, &document)?;
    let written_object_count =
        usize::from(revision_write.created) + usize::from(document_write.created);
    let existing_object_count =
        usize::from(!revision_write.created) + usize::from(!document_write.created);
    let index_manifest_path = document_write
        .index_manifest_path
        .or(revision_write.index_manifest_path);

    Ok(DocumentCreateSummary {
        store_root: store_root.to_path_buf(),
        status: "ok".to_string(),
        doc_id: params.doc_id.clone(),
        document_object_id: params.doc_id.clone(),
        genesis_revision_id: genesis_revision["revision_id"]
            .as_str()
            .expect("generated revision_id should be string")
            .to_string(),
        written_object_count,
        existing_object_count,
        stored_objects: vec![document_write.record, revision_write.record],
        index_manifest_path,
        notes: Vec::new(),
        errors: Vec::new(),
    })
}

pub fn create_patch_in_store(
    store_root: &Path,
    signing_key: &SigningKey,
    params: &PatchCreateParams,
) -> Result<PatchCreateSummary, StoreRebuildError> {
    ensure_document_exists(store_root, &params.doc_id)?;
    if params.base_revision != GENESIS_BASE_REVISION {
        ensure_object_exists(store_root, &params.base_revision, "base revision")?;
    }

    let mut patch = json!({
        "type": "patch",
        "version": CORE_PROTOCOL_VERSION,
        "doc_id": params.doc_id,
        "base_revision": params.base_revision,
        "author": signer_id(signing_key),
        "timestamp": params.timestamp,
        "ops": params.ops
    });
    let patch_id =
        recompute_object_id(&patch, "patch_id", "patch").map_err(StoreRebuildError::new)?;
    patch["patch_id"] = Value::String(patch_id.clone());
    patch["signature"] = Value::String(sign_object_value(signing_key, &patch)?);

    let write = write_object_value_to_store(store_root, &patch)?;

    Ok(PatchCreateSummary {
        store_root: store_root.to_path_buf(),
        status: "ok".to_string(),
        doc_id: params.doc_id.clone(),
        patch_id,
        base_revision: params.base_revision.clone(),
        written_object_count: usize::from(write.created),
        existing_object_count: usize::from(!write.created),
        stored_object: write.record,
        index_manifest_path: write.index_manifest_path,
        notes: Vec::new(),
        errors: Vec::new(),
    })
}

pub fn commit_revision_to_store(
    store_root: &Path,
    signing_key: &SigningKey,
    params: &RevisionCommitParams,
) -> Result<RevisionCommitSummary, StoreRebuildError> {
    ensure_document_exists(store_root, &params.doc_id)?;
    for parent_id in &params.parents {
        ensure_object_exists(store_root, parent_id, "parent revision")?;
    }
    for patch_id in &params.patches {
        ensure_object_exists(store_root, patch_id, "patch")?;
    }

    let object_index = load_store_object_index(store_root)?;
    let author = signer_id(signing_key);
    let replay_revision_object = RevisionObject {
        revision_id: "rev:pending".to_string(),
        doc_id: params.doc_id.clone(),
        parents: params.parents.clone(),
        patches: params.patches.clone(),
        merge_strategy: params.merge_strategy.clone(),
        state_hash: "hash:pending".to_string(),
        author: author.clone(),
        timestamp: params.timestamp,
    };
    let state = replay_revision(&replay_revision_object, &object_index).map_err(|error| {
        StoreRebuildError::new(format!("failed to replay committed revision: {error}"))
    })?;
    let recomputed_state_hash = compute_state_hash(&state).map_err(|error| {
        StoreRebuildError::new(format!("failed to compute revision state_hash: {error}"))
    })?;

    let mut revision = json!({
        "type": "revision",
        "version": CORE_PROTOCOL_VERSION,
        "doc_id": params.doc_id,
        "parents": params.parents,
        "patches": params.patches,
        "state_hash": recomputed_state_hash,
        "author": author,
        "timestamp": params.timestamp
    });
    if let Some(merge_strategy) = &params.merge_strategy {
        revision["merge_strategy"] = Value::String(merge_strategy.clone());
    }
    let revision_id =
        recompute_object_id(&revision, "revision_id", "rev").map_err(StoreRebuildError::new)?;
    revision["revision_id"] = Value::String(revision_id.clone());
    revision["signature"] = Value::String(sign_object_value(signing_key, &revision)?);

    let write = write_object_value_to_store(store_root, &revision)?;

    Ok(RevisionCommitSummary {
        store_root: store_root.to_path_buf(),
        status: "ok".to_string(),
        doc_id: params.doc_id.clone(),
        revision_id,
        parent_revision_ids: params.parents.clone(),
        patch_ids: params.patches.clone(),
        recomputed_state_hash,
        written_object_count: usize::from(write.created),
        existing_object_count: usize::from(!write.created),
        stored_object: write.record,
        index_manifest_path: write.index_manifest_path,
        notes: Vec::new(),
        errors: Vec::new(),
    })
}

pub fn create_merge_revision_in_store(
    store_root: &Path,
    signing_key: &SigningKey,
    params: &MergeRevisionCreateParams,
) -> Result<MergeRevisionCreateSummary, StoreRebuildError> {
    ensure_document_exists(store_root, &params.doc_id)?;
    if params.parents.len() < 2 {
        return Err(StoreRebuildError::new(
            "merge authoring requires at least two parent revisions",
        ));
    }
    if params.merge_strategy.is_empty() {
        return Err(StoreRebuildError::new(
            "merge_strategy must not be empty for merge authoring",
        ));
    }
    if params.resolved_state.doc_id != params.doc_id {
        return Err(StoreRebuildError::new(format!(
            "resolved state doc_id '{}' does not match requested '{}'",
            params.resolved_state.doc_id, params.doc_id
        )));
    }

    let object_index = load_store_object_index(store_root)?;
    let mut parent_states = Vec::new();
    for parent_id in &params.parents {
        ensure_object_exists(store_root, parent_id, "parent revision")?;
        let parent_value = object_index.get(parent_id).ok_or_else(|| {
            StoreRebuildError::new(format!(
                "parent revision '{}' was not found in the store object index",
                parent_id
            ))
        })?;
        let replay = replay_revision_from_index(parent_value, &object_index).map_err(|error| {
            StoreRebuildError::new(format!(
                "failed to replay parent revision '{parent_id}': {error}"
            ))
        })?;
        if replay.state.doc_id != params.doc_id {
            return Err(StoreRebuildError::new(format!(
                "parent revision '{}' belongs to '{}' instead of '{}'",
                parent_id, replay.state.doc_id, params.doc_id
            )));
        }
        parent_states.push((parent_id.clone(), replay.state));
    }

    let primary_parent_id = params
        .parents
        .first()
        .expect("merge parents should contain at least one parent")
        .clone();
    let primary_state = parent_states
        .first()
        .expect("merge parent states should contain the primary parent")
        .1
        .clone();
    let assessment = assess_merge_resolution(&parent_states, &params.resolved_state)?;
    if assessment.outcome == MergeOutcome::ManualCurationRequired {
        return Err(StoreRebuildError::new(format!(
            "merge resolution is manual-curation-required: {}",
            assessment.reasons.join("; ")
        )));
    }

    let ops = build_conservative_merge_ops(&primary_state, &params.resolved_state)?;
    let patch_summary = create_patch_in_store(
        store_root,
        signing_key,
        &PatchCreateParams {
            doc_id: params.doc_id.clone(),
            base_revision: primary_parent_id,
            timestamp: params.timestamp,
            ops: patch_ops_to_value(&ops),
        },
    )?;
    let revision_summary = commit_revision_to_store(
        store_root,
        signing_key,
        &RevisionCommitParams {
            doc_id: params.doc_id.clone(),
            parents: params.parents.clone(),
            patches: vec![patch_summary.patch_id.clone()],
            merge_strategy: Some(params.merge_strategy.clone()),
            timestamp: params.timestamp,
        },
    )?;

    let written_object_count =
        patch_summary.written_object_count + revision_summary.written_object_count;
    let existing_object_count =
        patch_summary.existing_object_count + revision_summary.existing_object_count;
    let index_manifest_path = revision_summary
        .index_manifest_path
        .clone()
        .or_else(|| patch_summary.index_manifest_path.clone());

    Ok(MergeRevisionCreateSummary {
        store_root: store_root.to_path_buf(),
        status: "ok".to_string(),
        doc_id: params.doc_id.clone(),
        merge_outcome: assessment.outcome,
        merge_reasons: assessment.reasons,
        parent_revision_ids: params.parents.clone(),
        patch_id: patch_summary.patch_id,
        patch_op_count: ops.len(),
        revision_id: revision_summary.revision_id,
        recomputed_state_hash: revision_summary.recomputed_state_hash,
        written_object_count,
        existing_object_count,
        stored_objects: vec![patch_summary.stored_object, revision_summary.stored_object],
        index_manifest_path,
        notes: Vec::new(),
        errors: Vec::new(),
    })
}

fn ensure_document_exists(store_root: &Path, doc_id: &str) -> Result<(), StoreRebuildError> {
    ensure_object_exists(store_root, doc_id, "document")
}

fn ensure_object_exists(
    store_root: &Path,
    object_id: &str,
    label: &str,
) -> Result<(), StoreRebuildError> {
    load_stored_object_value(store_root, object_id)
        .map(|_| ())
        .map_err(|_| {
            StoreRebuildError::new(format!(
                "{label} '{}' was not found in the store",
                object_id
            ))
        })
}

fn sign_object_value(signing_key: &SigningKey, value: &Value) -> Result<String, StoreRebuildError> {
    let payload = signed_payload_bytes(value).map_err(|error| {
        StoreRebuildError::new(format!("failed to compute signed payload: {error}"))
    })?;
    let signature = signing_key.sign(&payload);
    Ok(format!(
        "sig:ed25519:{}",
        base64::engine::general_purpose::STANDARD.encode(signature.to_bytes())
    ))
}

fn assess_merge_resolution(
    parent_states: &[(String, DocumentState)],
    resolved_state: &DocumentState,
) -> Result<MergeAssessment, StoreRebuildError> {
    let primary_state = &parent_states
        .first()
        .expect("merge parent states should not be empty")
        .1;
    let primary_blocks = flatten_blocks(&primary_state.blocks);
    let resolved_blocks = flatten_blocks(&resolved_state.blocks);
    let mut reasons = Vec::new();
    let mut saw_multi_variant = false;

    let block_ids = primary_blocks
        .keys()
        .cloned()
        .chain(
            parent_states
                .iter()
                .skip(1)
                .flat_map(|(_, state)| flatten_blocks(&state.blocks).into_keys()),
        )
        .chain(resolved_blocks.keys().cloned())
        .collect::<BTreeSet<_>>();

    for block_id in block_ids {
        let primary_variant = block_variant(primary_blocks.get(&block_id))?;
        let resolved_variant = block_variant(resolved_blocks.get(&block_id))?;
        let alternative_variants = parent_states
            .iter()
            .skip(1)
            .map(|(_, state)| flatten_blocks(&state.blocks))
            .map(|blocks| block_variant(blocks.get(&block_id)))
            .collect::<Result<BTreeSet<_>, _>>()?
            .into_iter()
            .filter(|variant| variant != &primary_variant)
            .collect::<BTreeSet<_>>();

        if resolved_variant != primary_variant && !alternative_variants.contains(&resolved_variant)
        {
            reasons.push(format!(
                "resolved block '{}' does not match any parent variant",
                block_id
            ));
        } else if primary_variant != "<absent>"
            && resolved_variant != primary_variant
            && alternative_variants.contains(&resolved_variant)
        {
            saw_multi_variant = true;
            reasons.push(format!(
                "block '{}' selected a non-primary parent variant",
                block_id
            ));
        } else if alternative_variants.len() > 1 {
            saw_multi_variant = true;
            reasons.push(format!(
                "block '{}' has multiple competing parent variants",
                block_id
            ));
        }
    }

    let metadata_keys = primary_state
        .metadata
        .keys()
        .cloned()
        .chain(
            parent_states
                .iter()
                .skip(1)
                .flat_map(|(_, state)| state.metadata.keys().cloned()),
        )
        .chain(resolved_state.metadata.keys().cloned())
        .collect::<BTreeSet<_>>();

    for key in metadata_keys {
        let primary_variant = metadata_variant(primary_state.metadata.get(&key))?;
        let resolved_variant = metadata_variant(resolved_state.metadata.get(&key))?;
        let alternative_variants = parent_states
            .iter()
            .skip(1)
            .map(|(_, state)| metadata_variant(state.metadata.get(&key)))
            .collect::<Result<BTreeSet<_>, _>>()?
            .into_iter()
            .filter(|variant| variant != &primary_variant)
            .collect::<BTreeSet<_>>();

        if resolved_variant != primary_variant && !alternative_variants.contains(&resolved_variant)
        {
            reasons.push(format!(
                "resolved metadata key '{}' does not match any parent variant",
                key
            ));
        } else if primary_variant != "<absent>"
            && resolved_variant != primary_variant
            && alternative_variants.contains(&resolved_variant)
        {
            saw_multi_variant = true;
            reasons.push(format!(
                "metadata key '{}' selected a non-primary parent variant",
                key
            ));
        } else if alternative_variants.len() > 1 {
            saw_multi_variant = true;
            reasons.push(format!(
                "metadata key '{}' has multiple competing parent variants",
                key
            ));
        }
    }

    let outcome = if reasons
        .iter()
        .any(|reason| reason.contains("does not match any parent variant"))
    {
        MergeOutcome::ManualCurationRequired
    } else if saw_multi_variant {
        MergeOutcome::MultiVariant
    } else {
        MergeOutcome::AutoMerged
    };

    Ok(MergeAssessment { outcome, reasons })
}

fn block_variant(block: Option<&BlockPlacement>) -> Result<String, StoreRebuildError> {
    match block {
        Some(placement) => {
            let mut object = serde_json::Map::new();
            object.insert(
                "block".to_string(),
                serde_json::to_value(&placement.block).map_err(|error| {
                    StoreRebuildError::new(format!("failed to serialize block variant: {error}"))
                })?,
            );
            if let Some(parent_block_id) = &placement.parent_block_id {
                object.insert(
                    "parent_block_id".to_string(),
                    Value::String(parent_block_id.clone()),
                );
            }
            canonical_json(&Value::Object(object)).map_err(|error| {
                StoreRebuildError::new(format!("failed to canonicalize block variant: {error}"))
            })
        }
        None => Ok("<absent>".to_string()),
    }
}

fn metadata_variant(value: Option<&Value>) -> Result<String, StoreRebuildError> {
    match value {
        Some(value) => canonical_json(value).map_err(|error| {
            StoreRebuildError::new(format!("failed to canonicalize metadata variant: {error}"))
        }),
        None => Ok("<absent>".to_string()),
    }
}

fn build_conservative_merge_ops(
    primary_state: &DocumentState,
    resolved_state: &DocumentState,
) -> Result<Vec<PatchOperation>, StoreRebuildError> {
    let primary_blocks = flatten_blocks(&primary_state.blocks);
    let resolved_blocks = flatten_blocks(&resolved_state.blocks);
    let primary_ids = primary_blocks.keys().cloned().collect::<BTreeSet<_>>();
    let resolved_ids = resolved_blocks.keys().cloned().collect::<BTreeSet<_>>();
    let deleted_ids = primary_ids
        .difference(&resolved_ids)
        .cloned()
        .collect::<BTreeSet<_>>();
    let new_ids = resolved_ids
        .difference(&primary_ids)
        .cloned()
        .collect::<BTreeSet<_>>();
    let mut ops = Vec::new();
    let mut simulated = primary_state.clone();

    for key in primary_state.metadata.keys() {
        if !resolved_state.metadata.contains_key(key) {
            return Err(StoreRebuildError::new(format!(
                "manual-curation-required: resolved state removes metadata key '{}'",
                key
            )));
        }
    }
    let changed_metadata = resolved_state
        .metadata
        .iter()
        .filter(|(key, value)| primary_state.metadata.get(*key) != Some(*value))
        .map(|(key, value)| (key.clone(), value.clone()))
        .collect::<serde_json::Map<_, _>>();
    if !changed_metadata.is_empty() {
        let op = PatchOperation::SetMetadata {
            entries: changed_metadata,
        };
        apply_generated_op(&mut simulated, &op)?;
        ops.push(op);
    }

    let mut deletions = deleted_ids
        .iter()
        .filter_map(|block_id| {
            let placement = primary_blocks.get(block_id)?;
            let parent_is_deleted = placement
                .parent_block_id
                .as_ref()
                .is_some_and(|parent_id| deleted_ids.contains(parent_id));
            (!parent_is_deleted).then(|| (placement.depth, block_id.clone()))
        })
        .collect::<Vec<_>>();
    deletions.sort_by(|left, right| right.0.cmp(&left.0).then_with(|| left.1.cmp(&right.1)));
    for (_, block_id) in deletions {
        let op = PatchOperation::DeleteBlock { block_id };
        apply_generated_op(&mut simulated, &op)?;
        ops.push(op);
    }

    sync_child_list(
        &mut simulated,
        None,
        &resolved_state.blocks,
        &new_ids,
        &mut ops,
    )?;

    if simulated != *resolved_state {
        return Err(StoreRebuildError::new(
            "manual-curation-required: resolved state requires unsupported structural edits"
                .to_string(),
        ));
    }

    Ok(ops)
}

fn sync_child_list(
    simulated: &mut DocumentState,
    parent_block_id: Option<&str>,
    resolved_children: &[BlockObject],
    new_ids: &BTreeSet<String>,
    ops: &mut Vec<PatchOperation>,
) -> Result<(), StoreRebuildError> {
    let mut previous_sibling_id: Option<String> = None;

    for resolved_block in resolved_children {
        let current_blocks = flatten_blocks(&simulated.blocks);
        if let Some(current) = current_blocks.get(&resolved_block.block_id) {
            if current.block.block_type != resolved_block.block_type {
                return Err(StoreRebuildError::new(format!(
                    "manual-curation-required: block '{}' changes block_type from '{}' to '{}'",
                    resolved_block.block_id, current.block.block_type, resolved_block.block_type
                )));
            }
            if current.block.attrs != resolved_block.attrs {
                return Err(StoreRebuildError::new(format!(
                    "manual-curation-required: block '{}' changes attrs in an unsupported way",
                    resolved_block.block_id
                )));
            }

            let desired_parent = parent_block_id.map(str::to_string);
            if !block_is_in_desired_position(
                simulated,
                &resolved_block.block_id,
                parent_block_id,
                previous_sibling_id.as_deref(),
            ) {
                let maybe_move = match previous_sibling_id.as_ref() {
                    Some(after_block_id) => Some(PatchOperation::MoveBlock {
                        block_id: resolved_block.block_id.clone(),
                        parent_block_id: desired_parent.clone(),
                        after_block_id: Some(after_block_id.clone()),
                    }),
                    None if current.parent_block_id != desired_parent => {
                        Some(PatchOperation::MoveBlock {
                            block_id: resolved_block.block_id.clone(),
                            parent_block_id: desired_parent.clone(),
                            after_block_id: None,
                        })
                    }
                    None => None,
                };
                if let Some(op) = maybe_move {
                    apply_generated_op(simulated, &op)?;
                    ops.push(op);
                }
            }

            let current_content = flatten_blocks(&simulated.blocks)
                .get(&resolved_block.block_id)
                .expect("existing block should remain indexed after move")
                .block
                .content
                .clone();
            if current_content != resolved_block.content {
                let op = PatchOperation::ReplaceBlock {
                    block_id: resolved_block.block_id.clone(),
                    new_content: resolved_block.content.clone(),
                };
                apply_generated_op(simulated, &op)?;
                ops.push(op);
            }

            sync_child_list(
                simulated,
                Some(&resolved_block.block_id),
                &resolved_block.children,
                new_ids,
                ops,
            )?;
        } else {
            let op = match previous_sibling_id.as_ref() {
                Some(after_block_id) => PatchOperation::InsertBlockAfter {
                    after_block_id: after_block_id.clone(),
                    new_block: resolved_block.clone(),
                },
                None => PatchOperation::InsertBlock {
                    parent_block_id: parent_block_id.map(str::to_string),
                    index: Some(0),
                    new_block: resolved_block.clone(),
                },
            };
            apply_generated_op(simulated, &op)?;
            ops.push(op);

            if !new_ids.contains(&resolved_block.block_id) {
                return Err(StoreRebuildError::new(format!(
                    "manual-curation-required: block '{}' is missing from the primary state without appearing as a new resolved block",
                    resolved_block.block_id
                )));
            }
        }

        previous_sibling_id = Some(resolved_block.block_id.clone());
    }

    let resolved_ids = resolved_children
        .iter()
        .map(|block| block.block_id.as_str())
        .collect::<BTreeSet<_>>();
    for current_id in sibling_block_ids(simulated, parent_block_id)? {
        if !resolved_ids.contains(current_id.as_str()) {
            return Err(StoreRebuildError::new(format!(
                "manual-curation-required: unresolved extra block '{}' remained under '{}'",
                current_id,
                parent_block_id.unwrap_or("<root>")
            )));
        }
    }
    Ok(())
}

fn apply_generated_op(
    simulated: &mut DocumentState,
    op: &PatchOperation,
) -> Result<(), StoreRebuildError> {
    let patch = PatchObject {
        patch_id: "patch:pending".to_string(),
        doc_id: simulated.doc_id.clone(),
        base_revision: "rev:pending".to_string(),
        author: "pk:pending".to_string(),
        timestamp: 0,
        ops: vec![op.clone()],
    };
    apply_patch_ops(simulated, &patch).map_err(|error| {
        StoreRebuildError::new(format!(
            "manual-curation-required: generated merge patch did not apply cleanly: {error}"
        ))
    })
}

fn block_is_in_desired_position(
    simulated: &DocumentState,
    block_id: &str,
    parent_block_id: Option<&str>,
    previous_sibling_id: Option<&str>,
) -> bool {
    let sibling_ids = match sibling_block_ids(simulated, parent_block_id) {
        Ok(sibling_ids) => sibling_ids,
        Err(_) => return false,
    };
    let Some(index) = sibling_ids
        .iter()
        .position(|candidate| candidate == block_id)
    else {
        return false;
    };

    match previous_sibling_id {
        Some(previous_sibling_id) => index > 0 && sibling_ids[index - 1] == previous_sibling_id,
        None => index == 0,
    }
}

fn sibling_block_ids(
    state: &DocumentState,
    parent_block_id: Option<&str>,
) -> Result<Vec<String>, StoreRebuildError> {
    match parent_block_id {
        Some(parent_block_id) => find_children(&state.blocks, parent_block_id)
            .map(|children| {
                children
                    .iter()
                    .map(|block| block.block_id.clone())
                    .collect()
            })
            .ok_or_else(|| {
                StoreRebuildError::new(format!(
                    "manual-curation-required: parent block '{}' was not found during merge sync",
                    parent_block_id
                ))
            }),
        None => Ok(state
            .blocks
            .iter()
            .map(|block| block.block_id.clone())
            .collect()),
    }
}

fn find_children<'a>(
    blocks: &'a [BlockObject],
    parent_block_id: &str,
) -> Option<&'a [BlockObject]> {
    for block in blocks {
        if block.block_id == parent_block_id {
            return Some(&block.children);
        }
        if let Some(children) = find_children(&block.children, parent_block_id) {
            return Some(children);
        }
    }
    None
}

fn flatten_blocks(blocks: &[BlockObject]) -> HashMap<String, BlockPlacement> {
    let mut placements = HashMap::new();
    flatten_blocks_into(blocks, None, 0, &mut placements);
    placements
}

fn flatten_blocks_into(
    blocks: &[BlockObject],
    parent_block_id: Option<&str>,
    depth: usize,
    placements: &mut HashMap<String, BlockPlacement>,
) {
    for block in blocks {
        placements.insert(
            block.block_id.clone(),
            BlockPlacement {
                block: block.clone(),
                parent_block_id: parent_block_id.map(str::to_string),
                depth,
            },
        );
        flatten_blocks_into(
            &block.children,
            Some(&block.block_id),
            depth + 1,
            placements,
        );
    }
}

fn patch_ops_to_value(ops: &[PatchOperation]) -> Value {
    Value::Array(
        ops.iter()
            .map(|op| match op {
                PatchOperation::InsertBlock {
                    parent_block_id,
                    index,
                    new_block,
                } => {
                    let mut object = serde_json::Map::new();
                    object.insert("op".to_string(), Value::String("insert_block".to_string()));
                    if let Some(parent_block_id) = parent_block_id {
                        object.insert(
                            "parent_block_id".to_string(),
                            Value::String(parent_block_id.clone()),
                        );
                    }
                    if let Some(index) = index {
                        object.insert(
                            "index".to_string(),
                            Value::Number(serde_json::Number::from(*index)),
                        );
                    }
                    object.insert(
                        "new_block".to_string(),
                        serde_json::to_value(new_block)
                            .expect("generated new_block should serialize"),
                    );
                    Value::Object(object)
                }
                PatchOperation::DeleteBlock { block_id } => json!({
                    "op": "delete_block",
                    "block_id": block_id
                }),
                PatchOperation::ReplaceBlock {
                    block_id,
                    new_content,
                } => json!({
                    "op": "replace_block",
                    "block_id": block_id,
                    "new_content": new_content
                }),
                PatchOperation::SetMetadata { entries } => json!({
                    "op": "set_metadata",
                    "entries": entries
                }),
                PatchOperation::InsertBlockAfter {
                    after_block_id,
                    new_block,
                } => json!({
                    "op": "insert_block_after",
                    "after_block_id": after_block_id,
                    "new_block": new_block
                }),
                PatchOperation::MoveBlock {
                    block_id,
                    parent_block_id,
                    after_block_id,
                } => {
                    let mut object = serde_json::Map::new();
                    object.insert("op".to_string(), Value::String("move_block".to_string()));
                    object.insert("block_id".to_string(), Value::String(block_id.clone()));
                    if let Some(parent_block_id) = parent_block_id {
                        object.insert(
                            "parent_block_id".to_string(),
                            Value::String(parent_block_id.clone()),
                        );
                    }
                    if let Some(after_block_id) = after_block_id {
                        object.insert(
                            "after_block_id".to_string(),
                            Value::String(after_block_id.clone()),
                        );
                    }
                    Value::Object(object)
                }
                PatchOperation::AnnotateBlock {
                    block_id,
                    annotation,
                } => json!({
                    "op": "annotate_block",
                    "block_id": block_id,
                    "annotation": annotation
                }),
            })
            .collect(),
    )
}

#[cfg(test)]
mod tests {
    use std::fs;
    use std::path::PathBuf;

    use base64::Engine;
    use ed25519_dalek::SigningKey;
    use serde_json::{json, Value};

    use super::{
        commit_revision_to_store, create_document_in_store, create_merge_revision_in_store,
        create_patch_in_store, parse_signing_key_seed, signer_id, DocumentCreateParams,
        MergeOutcome, MergeRevisionCreateParams, PatchCreateParams, RevisionCommitParams,
    };
    use crate::protocol::{parse_patch_object, BlockObject, PatchOperation};
    use crate::replay::replay_revision_from_index;
    use crate::store::{load_store_index_manifest, load_stored_object_value};

    fn temp_dir(prefix: &str) -> PathBuf {
        let unique = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .expect("system time should be after unix epoch")
            .as_nanos();
        let path = std::env::temp_dir().join(format!("mycel-author-{prefix}-{unique}"));
        fs::create_dir_all(&path).expect("temp dir should be created");
        path
    }

    fn signing_key() -> SigningKey {
        parse_signing_key_seed(&base64::engine::general_purpose::STANDARD.encode([7u8; 32]))
            .expect("signing key seed should parse")
    }

    fn paragraph_block(block_id: &str, content: &str) -> BlockObject {
        BlockObject {
            block_id: block_id.to_string(),
            block_type: "paragraph".to_string(),
            content: content.to_string(),
            attrs: serde_json::Map::new(),
            children: Vec::new(),
        }
    }

    fn paragraph_block_with_children(
        block_id: &str,
        content: &str,
        children: Vec<BlockObject>,
    ) -> BlockObject {
        BlockObject {
            children,
            ..paragraph_block(block_id, content)
        }
    }

    fn paragraph_block_with_attrs(
        block_id: &str,
        content: &str,
        attrs: serde_json::Map<String, Value>,
    ) -> BlockObject {
        BlockObject {
            attrs,
            ..paragraph_block(block_id, content)
        }
    }

    fn commit_ops_revision(
        store_root: &std::path::Path,
        signing_key: &SigningKey,
        doc_id: &str,
        base_revision: &str,
        patch_timestamp: u64,
        revision_timestamp: u64,
        ops: Value,
    ) -> String {
        let patch = create_patch_in_store(
            store_root,
            signing_key,
            &PatchCreateParams {
                doc_id: doc_id.to_string(),
                base_revision: base_revision.to_string(),
                timestamp: patch_timestamp,
                ops,
            },
        )
        .expect("patch should be created");
        commit_revision_to_store(
            store_root,
            signing_key,
            &RevisionCommitParams {
                doc_id: doc_id.to_string(),
                parents: vec![base_revision.to_string()],
                patches: vec![patch.patch_id],
                merge_strategy: None,
                timestamp: revision_timestamp,
            },
        )
        .expect("revision should be committed")
        .revision_id
    }

    #[test]
    fn authoring_flow_creates_document_patch_and_revision_in_store() {
        let store_root = temp_dir("flow");
        let signing_key = signing_key();
        let document = create_document_in_store(
            &store_root,
            &signing_key,
            &DocumentCreateParams {
                doc_id: "doc:author-flow".to_string(),
                title: "Author Flow".to_string(),
                language: "en".to_string(),
                timestamp: 10,
            },
        )
        .expect("document should be created");
        assert_eq!(document.written_object_count, 2);

        let patch = create_patch_in_store(
            &store_root,
            &signing_key,
            &PatchCreateParams {
                doc_id: "doc:author-flow".to_string(),
                base_revision: document.genesis_revision_id.clone(),
                timestamp: 11,
                ops: json!([
                    {
                        "op": "insert_block",
                        "new_block": {
                            "block_id": "blk:001",
                            "block_type": "paragraph",
                            "content": "Hello authoring",
                            "attrs": {},
                            "children": []
                        }
                    }
                ]),
            },
        )
        .expect("patch should be created");
        assert_eq!(patch.written_object_count, 1);

        let revision = commit_revision_to_store(
            &store_root,
            &signing_key,
            &RevisionCommitParams {
                doc_id: "doc:author-flow".to_string(),
                parents: vec![document.genesis_revision_id.clone()],
                patches: vec![patch.patch_id.clone()],
                merge_strategy: None,
                timestamp: 12,
            },
        )
        .expect("revision should be committed");
        assert_eq!(revision.written_object_count, 1);

        let manifest = load_store_index_manifest(&store_root).expect("manifest should load");
        assert_eq!(
            manifest.doc_revisions.get("doc:author-flow").map(Vec::len),
            Some(2)
        );
        assert_eq!(
            manifest
                .author_patches
                .get(&signer_id(&signing_key))
                .map(Vec::len),
            Some(1)
        );

        let mut object_index =
            crate::store::load_store_object_index(&store_root).expect("object index should load");
        object_index.insert(
            "doc:author-flow".to_string(),
            load_stored_object_value(&store_root, "doc:author-flow").expect("document should load"),
        );
        let replay = replay_revision_from_index(
            &load_stored_object_value(&store_root, &revision.revision_id)
                .expect("revision should load"),
            &object_index,
        )
        .expect("revision replay should succeed");
        assert_eq!(replay.revision_id, revision.revision_id);
        assert_eq!(replay.state.doc_id, "doc:author-flow");
        assert_eq!(replay.state.blocks.len(), 1);

        let _ = fs::remove_dir_all(store_root);
    }

    #[test]
    fn merge_authoring_reports_multi_variant_when_parents_disagree() {
        let store_root = temp_dir("merge-multi-variant");
        let signing_key = signing_key();
        let document = create_document_in_store(
            &store_root,
            &signing_key,
            &DocumentCreateParams {
                doc_id: "doc:merge-variant".to_string(),
                title: "Merge Variant".to_string(),
                language: "en".to_string(),
                timestamp: 10,
            },
        )
        .expect("document should be created");

        let base_patch = create_patch_in_store(
            &store_root,
            &signing_key,
            &PatchCreateParams {
                doc_id: "doc:merge-variant".to_string(),
                base_revision: document.genesis_revision_id.clone(),
                timestamp: 11,
                ops: json!([
                    {
                        "op": "insert_block",
                        "new_block": {
                            "block_id": "blk:merge-001",
                            "block_type": "paragraph",
                            "content": "Base",
                            "attrs": {},
                            "children": []
                        }
                    }
                ]),
            },
        )
        .expect("base patch should be created");
        let base_revision = commit_revision_to_store(
            &store_root,
            &signing_key,
            &RevisionCommitParams {
                doc_id: "doc:merge-variant".to_string(),
                parents: vec![document.genesis_revision_id.clone()],
                patches: vec![base_patch.patch_id],
                merge_strategy: None,
                timestamp: 12,
            },
        )
        .expect("base revision should be committed");

        let left_patch = create_patch_in_store(
            &store_root,
            &signing_key,
            &PatchCreateParams {
                doc_id: "doc:merge-variant".to_string(),
                base_revision: base_revision.revision_id.clone(),
                timestamp: 13,
                ops: json!([
                    {
                        "op": "replace_block",
                        "block_id": "blk:merge-001",
                        "new_content": "Left variant"
                    }
                ]),
            },
        )
        .expect("left patch should be created");
        let left_revision = commit_revision_to_store(
            &store_root,
            &signing_key,
            &RevisionCommitParams {
                doc_id: "doc:merge-variant".to_string(),
                parents: vec![base_revision.revision_id.clone()],
                patches: vec![left_patch.patch_id],
                merge_strategy: None,
                timestamp: 14,
            },
        )
        .expect("left revision should be committed");

        let right_patch = create_patch_in_store(
            &store_root,
            &signing_key,
            &PatchCreateParams {
                doc_id: "doc:merge-variant".to_string(),
                base_revision: base_revision.revision_id.clone(),
                timestamp: 15,
                ops: json!([
                    {
                        "op": "replace_block",
                        "block_id": "blk:merge-001",
                        "new_content": "Right variant"
                    }
                ]),
            },
        )
        .expect("right patch should be created");
        let right_revision = commit_revision_to_store(
            &store_root,
            &signing_key,
            &RevisionCommitParams {
                doc_id: "doc:merge-variant".to_string(),
                parents: vec![base_revision.revision_id.clone()],
                patches: vec![right_patch.patch_id],
                merge_strategy: None,
                timestamp: 16,
            },
        )
        .expect("right revision should be committed");

        let summary = create_merge_revision_in_store(
            &store_root,
            &signing_key,
            &MergeRevisionCreateParams {
                doc_id: "doc:merge-variant".to_string(),
                parents: vec![left_revision.revision_id, right_revision.revision_id],
                resolved_state: crate::replay::DocumentState {
                    doc_id: "doc:merge-variant".to_string(),
                    blocks: vec![crate::protocol::BlockObject {
                        block_id: "blk:merge-001".to_string(),
                        block_type: "paragraph".to_string(),
                        content: "Right variant".to_string(),
                        attrs: serde_json::Map::new(),
                        children: Vec::new(),
                    }],
                    metadata: serde_json::Map::new(),
                },
                merge_strategy: "semantic-block-merge".to_string(),
                timestamp: 17,
            },
        )
        .expect("merge revision should be created");

        assert_eq!(summary.merge_outcome, MergeOutcome::MultiVariant);
        assert_eq!(summary.patch_op_count, 1);
        assert!(
            summary
                .merge_reasons
                .iter()
                .any(|reason| reason.contains("selected a non-primary parent variant")),
            "expected multi-variant reason, got {summary:?}"
        );

        let _ = fs::remove_dir_all(store_root);
    }

    #[test]
    fn merge_authoring_supports_structural_move_and_insert_ops() {
        let store_root = temp_dir("merge-structural");
        let signing_key = signing_key();
        let document = create_document_in_store(
            &store_root,
            &signing_key,
            &DocumentCreateParams {
                doc_id: "doc:merge-structural".to_string(),
                title: "Merge Structural".to_string(),
                language: "en".to_string(),
                timestamp: 20,
            },
        )
        .expect("document should be created");

        let base_patch = create_patch_in_store(
            &store_root,
            &signing_key,
            &PatchCreateParams {
                doc_id: "doc:merge-structural".to_string(),
                base_revision: document.genesis_revision_id.clone(),
                timestamp: 21,
                ops: json!([
                    {
                        "op": "insert_block",
                        "new_block": {
                            "block_id": "blk:merge-a",
                            "block_type": "paragraph",
                            "content": "A",
                            "attrs": {},
                            "children": []
                        }
                    },
                    {
                        "op": "insert_block",
                        "new_block": {
                            "block_id": "blk:merge-b",
                            "block_type": "paragraph",
                            "content": "B",
                            "attrs": {},
                            "children": []
                        }
                    }
                ]),
            },
        )
        .expect("base patch should be created");
        let base_revision = commit_revision_to_store(
            &store_root,
            &signing_key,
            &RevisionCommitParams {
                doc_id: "doc:merge-structural".to_string(),
                parents: vec![document.genesis_revision_id.clone()],
                patches: vec![base_patch.patch_id],
                merge_strategy: None,
                timestamp: 22,
            },
        )
        .expect("base revision should be committed");

        let move_patch = create_patch_in_store(
            &store_root,
            &signing_key,
            &PatchCreateParams {
                doc_id: "doc:merge-structural".to_string(),
                base_revision: base_revision.revision_id.clone(),
                timestamp: 23,
                ops: json!([
                    {
                        "op": "move_block",
                        "block_id": "blk:merge-a",
                        "after_block_id": "blk:merge-b"
                    }
                ]),
            },
        )
        .expect("move patch should be created");
        let move_revision = commit_revision_to_store(
            &store_root,
            &signing_key,
            &RevisionCommitParams {
                doc_id: "doc:merge-structural".to_string(),
                parents: vec![base_revision.revision_id.clone()],
                patches: vec![move_patch.patch_id],
                merge_strategy: None,
                timestamp: 24,
            },
        )
        .expect("move revision should be committed");

        let insert_patch = create_patch_in_store(
            &store_root,
            &signing_key,
            &PatchCreateParams {
                doc_id: "doc:merge-structural".to_string(),
                base_revision: base_revision.revision_id.clone(),
                timestamp: 25,
                ops: json!([
                    {
                        "op": "insert_block",
                        "new_block": {
                            "block_id": "blk:merge-c",
                            "block_type": "paragraph",
                            "content": "C",
                            "attrs": {},
                            "children": []
                        }
                    }
                ]),
            },
        )
        .expect("insert patch should be created");
        let insert_revision = commit_revision_to_store(
            &store_root,
            &signing_key,
            &RevisionCommitParams {
                doc_id: "doc:merge-structural".to_string(),
                parents: vec![base_revision.revision_id.clone()],
                patches: vec![insert_patch.patch_id],
                merge_strategy: None,
                timestamp: 26,
            },
        )
        .expect("insert revision should be committed");

        let summary = create_merge_revision_in_store(
            &store_root,
            &signing_key,
            &MergeRevisionCreateParams {
                doc_id: "doc:merge-structural".to_string(),
                parents: vec![
                    base_revision.revision_id.clone(),
                    move_revision.revision_id.clone(),
                    insert_revision.revision_id.clone(),
                ],
                resolved_state: crate::replay::DocumentState {
                    doc_id: "doc:merge-structural".to_string(),
                    blocks: vec![
                        crate::protocol::BlockObject {
                            block_id: "blk:merge-b".to_string(),
                            block_type: "paragraph".to_string(),
                            content: "B".to_string(),
                            attrs: serde_json::Map::new(),
                            children: Vec::new(),
                        },
                        crate::protocol::BlockObject {
                            block_id: "blk:merge-a".to_string(),
                            block_type: "paragraph".to_string(),
                            content: "A".to_string(),
                            attrs: serde_json::Map::new(),
                            children: Vec::new(),
                        },
                        crate::protocol::BlockObject {
                            block_id: "blk:merge-c".to_string(),
                            block_type: "paragraph".to_string(),
                            content: "C".to_string(),
                            attrs: serde_json::Map::new(),
                            children: Vec::new(),
                        },
                    ],
                    metadata: serde_json::Map::new(),
                },
                merge_strategy: "semantic-block-merge".to_string(),
                timestamp: 27,
            },
        )
        .expect("merge revision should be created");

        assert_eq!(summary.merge_outcome, MergeOutcome::AutoMerged);
        assert_eq!(summary.patch_op_count, 2);
        let patch_value = load_stored_object_value(&store_root, &summary.patch_id)
            .expect("generated merge patch should be stored");
        let patch = parse_patch_object(&patch_value).expect("generated patch should parse");
        assert_eq!(patch.ops.len(), 2);
        assert!(patch.ops.iter().any(|op| matches!(
            op,
            PatchOperation::MoveBlock { block_id, after_block_id: Some(after_block_id), .. }
            if block_id == "blk:merge-a" && after_block_id == "blk:merge-b"
        )));
        assert!(patch.ops.iter().any(|op| matches!(
            op,
            PatchOperation::InsertBlockAfter { after_block_id, new_block }
            if after_block_id == "blk:merge-a" && new_block.block_id == "blk:merge-c"
        )));

        let _ = fs::remove_dir_all(store_root);
    }

    #[test]
    fn merge_authoring_marks_non_primary_structural_parent_choice_as_multi_variant() {
        let store_root = temp_dir("merge-parent-choice");
        let signing_key = signing_key();
        let document = create_document_in_store(
            &store_root,
            &signing_key,
            &DocumentCreateParams {
                doc_id: "doc:merge-parent-choice".to_string(),
                title: "Merge Parent Choice".to_string(),
                language: "en".to_string(),
                timestamp: 30,
            },
        )
        .expect("document should be created");

        let base_revision_id = commit_ops_revision(
            &store_root,
            &signing_key,
            "doc:merge-parent-choice",
            &document.genesis_revision_id,
            31,
            32,
            json!([
                {
                    "op": "insert_block",
                    "new_block": {
                        "block_id": "blk:merge-parent",
                        "block_type": "paragraph",
                        "content": "Parent",
                        "attrs": {},
                        "children": []
                    }
                },
                {
                    "op": "insert_block",
                    "new_block": {
                        "block_id": "blk:merge-leaf",
                        "block_type": "paragraph",
                        "content": "Leaf",
                        "attrs": {},
                        "children": []
                    }
                }
            ]),
        );

        let moved_revision_id = commit_ops_revision(
            &store_root,
            &signing_key,
            "doc:merge-parent-choice",
            &base_revision_id,
            33,
            34,
            json!([
                {
                    "op": "move_block",
                    "block_id": "blk:merge-leaf",
                    "parent_block_id": "blk:merge-parent"
                }
            ]),
        );

        let summary = create_merge_revision_in_store(
            &store_root,
            &signing_key,
            &MergeRevisionCreateParams {
                doc_id: "doc:merge-parent-choice".to_string(),
                parents: vec![base_revision_id, moved_revision_id],
                resolved_state: crate::replay::DocumentState {
                    doc_id: "doc:merge-parent-choice".to_string(),
                    blocks: vec![paragraph_block_with_children(
                        "blk:merge-parent",
                        "Parent",
                        vec![paragraph_block("blk:merge-leaf", "Leaf")],
                    )],
                    metadata: serde_json::Map::new(),
                },
                merge_strategy: "semantic-block-merge".to_string(),
                timestamp: 35,
            },
        )
        .expect("merge revision should be created");

        assert_eq!(summary.merge_outcome, MergeOutcome::MultiVariant);
        assert!(
            summary
                .merge_reasons
                .iter()
                .any(|reason| reason.contains("selected a non-primary parent variant")),
            "expected structural multi-variant reason, got {summary:?}"
        );
        let patch_value = load_stored_object_value(&store_root, &summary.patch_id)
            .expect("generated merge patch should be stored");
        let patch = parse_patch_object(&patch_value).expect("generated patch should parse");
        assert!(patch.ops.iter().any(|op| matches!(
            op,
            PatchOperation::MoveBlock { block_id, parent_block_id: Some(parent_block_id), after_block_id: None }
            if block_id == "blk:merge-leaf" && parent_block_id == "blk:merge-parent"
        )));

        let _ = fs::remove_dir_all(store_root);
    }

    #[test]
    fn merge_authoring_rejects_parent_matched_attr_variant_as_manual_curation_required() {
        let store_root = temp_dir("merge-attrs-manual");
        let signing_key = signing_key();
        let document = create_document_in_store(
            &store_root,
            &signing_key,
            &DocumentCreateParams {
                doc_id: "doc:merge-attrs".to_string(),
                title: "Merge Attrs".to_string(),
                language: "en".to_string(),
                timestamp: 40,
            },
        )
        .expect("document should be created");

        let base_revision_id = commit_ops_revision(
            &store_root,
            &signing_key,
            "doc:merge-attrs",
            &document.genesis_revision_id,
            41,
            42,
            json!([
                {
                    "op": "insert_block",
                    "new_block": {
                        "block_id": "blk:merge-attrs",
                        "block_type": "paragraph",
                        "content": "Attrs",
                        "attrs": {},
                        "children": []
                    }
                }
            ]),
        );

        let attrs_revision_id = commit_ops_revision(
            &store_root,
            &signing_key,
            "doc:merge-attrs",
            &base_revision_id,
            43,
            44,
            json!([
                {
                    "op": "delete_block",
                    "block_id": "blk:merge-attrs"
                },
                {
                    "op": "insert_block",
                    "new_block": {
                        "block_id": "blk:merge-attrs",
                        "block_type": "paragraph",
                        "content": "Attrs",
                        "attrs": {
                            "style": "note"
                        },
                        "children": []
                    }
                }
            ]),
        );

        let mut attrs = serde_json::Map::new();
        attrs.insert("style".to_string(), Value::String("note".to_string()));
        let error = create_merge_revision_in_store(
            &store_root,
            &signing_key,
            &MergeRevisionCreateParams {
                doc_id: "doc:merge-attrs".to_string(),
                parents: vec![base_revision_id, attrs_revision_id],
                resolved_state: crate::replay::DocumentState {
                    doc_id: "doc:merge-attrs".to_string(),
                    blocks: vec![paragraph_block_with_attrs(
                        "blk:merge-attrs",
                        "Attrs",
                        attrs,
                    )],
                    metadata: serde_json::Map::new(),
                },
                merge_strategy: "semantic-block-merge".to_string(),
                timestamp: 45,
            },
        )
        .expect_err("merge revision should require manual curation");

        assert!(
            error
                .to_string()
                .contains("manual-curation-required: block 'blk:merge-attrs' changes attrs in an unsupported way"),
            "expected attrs manual-curation error, got {error}"
        );

        let _ = fs::remove_dir_all(store_root);
    }
}
