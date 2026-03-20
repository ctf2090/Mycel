use std::collections::{BTreeSet, HashMap};
use std::path::Path;

use ed25519_dalek::SigningKey;
use serde_json::{json, Value};

use crate::canonical::canonical_json;
use crate::protocol::{BlockObject, PatchObject, PatchOperation};
use crate::replay::{apply_patch_ops, replay_revision_from_index, DocumentState};
use crate::store::{load_doc_replay_objects_from_store, StoreRebuildError};

use super::shared::{ensure_document_exists, ensure_object_exists};
use super::types::{
    BlockPlacement, MergeAssessment, MergeOutcome, MergeRevisionCreateParams,
    MergeRevisionCreateSummary, PatchCreateParams, RevisionCommitParams,
};
use super::write::{commit_revision_to_store, create_patch_in_store};

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

    let object_index = load_doc_replay_objects_from_store(store_root, &params.doc_id)?;
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
        let primary_content_variant =
            block_content_variant(primary_blocks.get(&block_id).map(|entry| &entry.block))?;
        let resolved_content_variant =
            block_content_variant(resolved_blocks.get(&block_id).map(|entry| &entry.block))?;
        let alternative_content_variants = parent_states
            .iter()
            .skip(1)
            .map(|(_, state)| flatten_blocks(&state.blocks))
            .map(|blocks| block_content_variant(blocks.get(&block_id).map(|entry| &entry.block)))
            .collect::<Result<BTreeSet<_>, _>>()?
            .into_iter()
            .filter(|variant| variant != &primary_content_variant)
            .collect::<BTreeSet<_>>();

        if resolved_content_variant != primary_content_variant
            && !alternative_content_variants.contains(&resolved_content_variant)
        {
            reasons.push(format!(
                "resolved block '{}' does not match any parent variant",
                block_id
            ));
        } else if primary_content_variant != "<absent>"
            && resolved_content_variant != primary_content_variant
            && alternative_content_variants.contains(&resolved_content_variant)
        {
            saw_multi_variant = true;
            reasons.push(format!(
                "block '{}' selected a non-primary parent variant",
                block_id
            ));
            if alternative_content_variants.len() > 1 {
                reasons.push(format!(
                    "block '{}' has multiple competing parent variants",
                    block_id
                ));
            }
        } else if alternative_content_variants.len() > 1 {
            saw_multi_variant = true;
            reasons.push(format!(
                "block '{}' has multiple competing parent variants",
                block_id
            ));
        }

        if primary_content_variant == "<absent>" || resolved_content_variant == "<absent>" {
            continue;
        }

        let primary_parent_variant = block_parent_variant(primary_blocks.get(&block_id));
        let resolved_parent_variant = block_parent_variant(resolved_blocks.get(&block_id));
        let alternative_parent_variants = parent_states
            .iter()
            .skip(1)
            .map(|(_, state)| flatten_blocks(&state.blocks))
            .map(|blocks| block_parent_variant(blocks.get(&block_id)))
            .collect::<BTreeSet<_>>()
            .into_iter()
            .filter(|variant| variant != &primary_parent_variant)
            .collect::<BTreeSet<_>>();

        let resolved_parent_anchor_variant = (resolved_parent_variant != primary_parent_variant
            && !alternative_parent_variants.contains(&resolved_parent_variant))
        .then(|| resolved_parent_anchor_variant(&block_id, &primary_blocks, &resolved_blocks))
        .flatten();
        let primary_sibling_variant = block_sibling_variant(primary_blocks.get(&block_id));
        let resolved_sibling_variant = block_sibling_variant(resolved_blocks.get(&block_id));
        let alternative_sibling_variants = parent_states
            .iter()
            .skip(1)
            .map(|(_, state)| flatten_blocks(&state.blocks))
            .filter(|blocks| block_parent_variant(blocks.get(&block_id)) == resolved_parent_variant)
            .map(|blocks| block_sibling_variant(blocks.get(&block_id)))
            .collect::<BTreeSet<_>>()
            .into_iter()
            .filter(|variant| variant != &primary_sibling_variant)
            .collect::<BTreeSet<_>>();
        let accepted_sibling_variants = std::iter::once(primary_sibling_variant.clone())
            .chain(alternative_sibling_variants.iter().cloned())
            .collect::<BTreeSet<_>>();
        let resolved_sibling_anchor = (!accepted_sibling_variants
            .contains(&resolved_sibling_variant))
        .then(|| {
            resolved_sibling_anchor_variant(&block_id, &resolved_blocks, &accepted_sibling_variants)
        })
        .flatten();
        let root_or_absent_only_alternatives = alternative_parent_variants.is_empty()
            || alternative_parent_variants
                .iter()
                .all(|variant| variant == "<root>" || variant == "<absent>");

        if let Some(anchor_variant) = resolved_parent_anchor_variant.as_deref() {
            if anchor_variant == "<root>" && root_or_absent_only_alternatives {
                if alternative_parent_variants.contains("<absent>") {
                    saw_multi_variant = true;
                    reasons.push(format!(
                        "block '{}' selected a non-primary parent placement",
                        block_id
                    ));
                    if alternative_parent_variants.len() > 1 {
                        reasons.push(format!(
                            "block '{}' has multiple competing parent placements",
                            block_id
                        ));
                    }
                }
                continue;
            }

            if anchor_variant != "<root>"
                && anchor_variant == primary_parent_variant
                && (resolved_sibling_variant == primary_sibling_variant
                    || resolved_sibling_anchor.as_deref() == Some(primary_sibling_variant.as_str()))
            {
                continue;
            }

            let anchor_sibling_variants = parent_states
                .iter()
                .skip(1)
                .map(|(_, state)| flatten_blocks(&state.blocks))
                .filter(|blocks| block_parent_variant(blocks.get(&block_id)) == anchor_variant)
                .map(|blocks| block_sibling_variant(blocks.get(&block_id)))
                .collect::<BTreeSet<_>>();

            if anchor_sibling_variants.contains(&resolved_sibling_variant)
                || resolved_sibling_anchor
                    .as_ref()
                    .is_some_and(|variant| anchor_sibling_variants.contains(variant))
            {
                saw_multi_variant = true;
                reasons.push(format!(
                    "block '{}' selected a non-primary parent placement",
                    block_id
                ));
                if alternative_parent_variants.len() > 1 {
                    reasons.push(format!(
                        "block '{}' has multiple competing parent placements",
                        block_id
                    ));
                }
                continue;
            }

            if alternative_parent_variants.contains(anchor_variant) {
                saw_multi_variant = true;
                reasons.push(format!(
                    "block '{}' selected a non-primary parent placement",
                    block_id
                ));
                if alternative_parent_variants.len() > 1 {
                    reasons.push(format!(
                        "block '{}' has multiple competing parent placements",
                        block_id
                    ));
                }
                continue;
            }
        }

        if resolved_parent_variant != primary_parent_variant
            && !alternative_parent_variants.contains(&resolved_parent_variant)
        {
            reasons.push(format!(
                "resolved block '{}' does not match any parent placement",
                block_id
            ));
        } else if resolved_parent_variant != primary_parent_variant
            && alternative_parent_variants.contains(&resolved_parent_variant)
        {
            saw_multi_variant = true;
            reasons.push(format!(
                "block '{}' selected a non-primary parent placement",
                block_id
            ));
            if alternative_parent_variants.len() > 1 {
                reasons.push(format!(
                    "block '{}' has multiple competing parent placements",
                    block_id
                ));
            }
        } else if alternative_parent_variants.len() > 1 {
            saw_multi_variant = true;
            reasons.push(format!(
                "block '{}' has multiple competing parent placements",
                block_id
            ));
        }

        if resolved_parent_variant == primary_parent_variant
            && resolved_sibling_variant != primary_sibling_variant
            && !alternative_sibling_variants.contains(&resolved_sibling_variant)
        {
            if resolved_sibling_anchor.as_deref() == Some(primary_sibling_variant.as_str()) {
                continue;
            }
            if primary_sibling_variant != "<absent>"
                && resolved_sibling_anchor
                    .as_ref()
                    .is_some_and(|variant| alternative_sibling_variants.contains(variant))
            {
                saw_multi_variant = true;
                reasons.push(format!(
                    "block '{}' selected a non-primary sibling placement",
                    block_id
                ));
                if alternative_sibling_variants.len() > 1 {
                    reasons.push(format!(
                        "block '{}' has multiple competing sibling placements",
                        block_id
                    ));
                }
                continue;
            }
            reasons.push(format!(
                "resolved block '{}' does not match any parent sibling placement",
                block_id
            ));
        } else if resolved_parent_variant == primary_parent_variant
            && primary_sibling_variant != "<absent>"
            && resolved_sibling_variant != primary_sibling_variant
            && alternative_sibling_variants.contains(&resolved_sibling_variant)
        {
            saw_multi_variant = true;
            reasons.push(format!(
                "block '{}' selected a non-primary sibling placement",
                block_id
            ));
            if alternative_sibling_variants.len() > 1 {
                reasons.push(format!(
                    "block '{}' has multiple competing sibling placements",
                    block_id
                ));
            }
        } else if resolved_parent_variant == primary_parent_variant
            && alternative_sibling_variants.len() > 1
        {
            saw_multi_variant = true;
            reasons.push(format!(
                "block '{}' has multiple competing sibling placements",
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

        if resolved_variant == "<absent>" && primary_variant != "<absent>" {
            reasons.push(format!(
                "resolved metadata key '{}' removes primary metadata but v0.1 patch ops cannot express metadata deletion",
                key
            ));
        } else if resolved_variant != primary_variant
            && !alternative_variants.contains(&resolved_variant)
        {
            reasons.push(format!(
                "resolved metadata key '{}' does not match any parent variant",
                key
            ));
        } else if resolved_variant != primary_variant
            && alternative_variants.contains(&resolved_variant)
        {
            saw_multi_variant = true;
            reasons.push(format!(
                "metadata key '{}' selected a non-primary parent variant",
                key
            ));
            if alternative_variants.len() > 1 {
                reasons.push(format!(
                    "metadata key '{}' has multiple competing parent variants",
                    key
                ));
            }
        } else if !alternative_variants.is_empty() {
            saw_multi_variant = true;
            if alternative_variants.len() > 1 {
                reasons.push(format!(
                    "metadata key '{}' has multiple competing parent variants",
                    key
                ));
            } else {
                reasons.push(format!(
                    "metadata key '{}' kept the primary parent variant over a competing non-primary alternative",
                    key
                ));
            }
        }
    }

    let outcome = if reasons.iter().any(|reason| {
        reason.contains("does not match any parent")
            || reason.contains("cannot express metadata deletion")
    }) {
        MergeOutcome::ManualCurationRequired
    } else if saw_multi_variant {
        MergeOutcome::MultiVariant
    } else {
        MergeOutcome::AutoMerged
    };

    Ok(MergeAssessment { outcome, reasons })
}

fn block_content_variant(block: Option<&BlockObject>) -> Result<String, StoreRebuildError> {
    match block {
        Some(block) => canonical_json(&json!({
            "block_id": block.block_id,
            "block_type": block.block_type,
            "content": block.content,
            "attrs": block.attrs
        }))
        .map_err(|error| {
            StoreRebuildError::new(format!("failed to canonicalize block variant: {error}"))
        }),
        None => Ok("<absent>".to_string()),
    }
}

fn block_parent_variant(block: Option<&BlockPlacement>) -> String {
    match block {
        Some(placement) => placement
            .parent_block_id
            .clone()
            .unwrap_or_else(|| "<root>".to_string()),
        None => "<absent>".to_string(),
    }
}

fn block_sibling_variant(block: Option<&BlockPlacement>) -> String {
    match block {
        Some(placement) => placement
            .previous_sibling_id
            .clone()
            .unwrap_or_else(|| "<start>".to_string()),
        None => "<absent>".to_string(),
    }
}

fn resolved_parent_anchor_variant(
    block_id: &str,
    primary_blocks: &HashMap<String, BlockPlacement>,
    resolved_blocks: &HashMap<String, BlockPlacement>,
) -> Option<String> {
    let mut parent_block_id = resolved_blocks
        .get(block_id)
        .and_then(|placement| placement.parent_block_id.as_ref())
        .cloned()?;

    loop {
        if let Some(primary_parent) = primary_blocks.get(&parent_block_id) {
            let resolved_parent = resolved_blocks.get(&parent_block_id)?;
            if block_parent_variant(Some(resolved_parent))
                == block_parent_variant(Some(primary_parent))
            {
                return Some(parent_block_id);
            }
        }

        let resolved_parent = resolved_blocks.get(&parent_block_id)?;
        let Some(next_parent_id) = resolved_parent.parent_block_id.as_ref() else {
            return Some("<root>".to_string());
        };
        parent_block_id = next_parent_id.clone();
    }
}

fn resolved_sibling_anchor_variant(
    block_id: &str,
    resolved_blocks: &HashMap<String, BlockPlacement>,
    accepted_variants: &BTreeSet<String>,
) -> Option<String> {
    let mut previous_sibling_id = resolved_blocks
        .get(block_id)
        .and_then(|placement| placement.previous_sibling_id.as_ref())
        .cloned()?;

    loop {
        if accepted_variants.contains(&previous_sibling_id) {
            return Some(previous_sibling_id);
        }

        let previous_sibling = resolved_blocks.get(&previous_sibling_id)?;
        let Some(next_previous_sibling_id) = previous_sibling.previous_sibling_id.as_ref() else {
            return accepted_variants
                .contains("<start>")
                .then(|| "<start>".to_string());
        };
        previous_sibling_id = next_previous_sibling_id.clone();
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
                "manual-curation-required: resolved state removes metadata key '{}' but v0.1 patch ops cannot express metadata deletion",
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

    sync_child_list(
        &mut simulated,
        None,
        &resolved_state.blocks,
        &resolved_blocks,
        &deleted_ids,
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
    resolved_blocks: &HashMap<String, BlockPlacement>,
    deleted_ids: &BTreeSet<String>,
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
                resolved_blocks,
                deleted_ids,
                new_ids,
                ops,
            )?;
        } else {
            if !new_ids.contains(&resolved_block.block_id) {
                return Err(StoreRebuildError::new(format!(
                    "manual-curation-required: block '{}' is missing from the primary state without appearing as a new resolved block",
                    resolved_block.block_id
                )));
            }

            let insertable_block = build_insertable_block(resolved_block, &current_blocks);
            let op = match previous_sibling_id.as_ref() {
                Some(after_block_id) => PatchOperation::InsertBlockAfter {
                    after_block_id: after_block_id.clone(),
                    new_block: insertable_block,
                },
                None => PatchOperation::InsertBlock {
                    parent_block_id: parent_block_id.map(str::to_string),
                    index: Some(0),
                    new_block: insertable_block,
                },
            };
            apply_generated_op(simulated, &op)?;
            ops.push(op);
            sync_child_list(
                simulated,
                Some(&resolved_block.block_id),
                &resolved_block.children,
                resolved_blocks,
                deleted_ids,
                new_ids,
                ops,
            )?;
        }

        previous_sibling_id = Some(resolved_block.block_id.clone());
    }

    let resolved_ids = resolved_children
        .iter()
        .map(|block| block.block_id.as_str())
        .collect::<BTreeSet<_>>();
    for current_id in sibling_block_ids(simulated, parent_block_id)? {
        if !resolved_ids.contains(current_id.as_str())
            && !resolved_blocks.contains_key(current_id.as_str())
        {
            if deleted_ids.contains(&current_id) {
                let op = PatchOperation::DeleteBlock {
                    block_id: current_id,
                };
                apply_generated_op(simulated, &op)?;
                ops.push(op);
                continue;
            }
            return Err(StoreRebuildError::new(format!(
                "manual-curation-required: unresolved extra block '{}' remained under '{}'",
                current_id,
                parent_block_id.unwrap_or("<root>")
            )));
        }
    }
    Ok(())
}

fn build_insertable_block(
    resolved_block: &BlockObject,
    current_blocks: &HashMap<String, BlockPlacement>,
) -> BlockObject {
    let mut insertable = resolved_block.clone();
    insertable.children = resolved_block
        .children
        .iter()
        .filter(|child| !current_blocks.contains_key(&child.block_id))
        .map(|child| build_insertable_block(child, current_blocks))
        .collect();
    insertable
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
    for (index, block) in blocks.iter().enumerate() {
        placements.insert(
            block.block_id.clone(),
            BlockPlacement {
                block: block.clone(),
                parent_block_id: parent_block_id.map(str::to_string),
                previous_sibling_id: (index > 0).then(|| blocks[index - 1].block_id.clone()),
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
                    "metadata": entries
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
