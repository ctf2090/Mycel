use std::path::Path;

use ed25519_dalek::SigningKey;
use serde_json::{json, Value};

use crate::protocol::{recompute_object_id, RevisionObject, CORE_PROTOCOL_VERSION};
use crate::replay::{compute_state_hash, replay_revision, DocumentState, GENESIS_BASE_REVISION};
use crate::store::{
    load_store_object_index, load_stored_object_value, write_object_value_to_store,
    StoreRebuildError,
};

use super::shared::{ensure_document_exists, ensure_object_exists, sign_object_value, signer_id};
use super::types::{
    DocumentCreateParams, DocumentCreateSummary, PatchCreateParams, PatchCreateSummary,
    RevisionCommitParams, RevisionCommitSummary,
};

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
