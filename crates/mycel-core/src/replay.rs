use std::collections::HashMap;

use serde::Serialize;
use serde_json::{Map, Value};
use sha2::{Digest, Sha256};

use crate::protocol::{
    parse_patch_object, parse_revision_object, BlockObject, PatchObject, PatchOperation,
    RevisionObject,
};
use crate::verify::{canonical_json, hex_encode};

pub const GENESIS_BASE_REVISION: &str = "rev:genesis-null";

#[derive(Debug, Clone, PartialEq, Serialize)]
pub struct DocumentState {
    pub doc_id: String,
    pub blocks: Vec<BlockObject>,
    #[serde(skip_serializing_if = "Map::is_empty")]
    pub metadata: Map<String, Value>,
}

#[derive(Debug, Clone, PartialEq, Serialize)]
pub struct RevisionReplaySummary {
    pub revision_id: String,
    pub recomputed_state_hash: String,
    pub state: DocumentState,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ReplayError {
    message: String,
}

impl ReplayError {
    fn new(message: impl Into<String>) -> Self {
        Self {
            message: message.into(),
        }
    }
}

impl std::fmt::Display for ReplayError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(&self.message)
    }
}

impl std::error::Error for ReplayError {}

pub fn apply_patch_ops(state: &mut DocumentState, patch: &PatchObject) -> Result<(), ReplayError> {
    for op in &patch.ops {
        apply_patch_op(state, op)?;
    }

    Ok(())
}

pub fn compute_state_hash(state: &DocumentState) -> Result<String, ReplayError> {
    let canonical = canonical_json(&state_to_value(state))
        .map_err(|error| ReplayError::new(format!("failed to canonicalize state: {error}")))?;
    let mut hasher = Sha256::new();
    hasher.update(canonical.as_bytes());
    let digest = hasher.finalize();
    Ok(format!("hash:{}", hex_encode(&digest)))
}

pub fn replay_revision_from_index(
    revision_value: &Value,
    object_values_by_id: &HashMap<String, Value>,
) -> Result<RevisionReplaySummary, ReplayError> {
    let revision = parse_revision_object(revision_value)
        .map_err(|error| ReplayError::new(format!("failed to parse revision object: {error}")))?;
    let mut cache = HashMap::new();
    let state = replay_revision_object(&revision, object_values_by_id, &mut cache)?;
    let recomputed_state_hash = compute_state_hash(&state)?;

    Ok(RevisionReplaySummary {
        revision_id: revision.revision_id,
        recomputed_state_hash,
        state,
    })
}

fn replay_revision_object(
    revision: &RevisionObject,
    object_values_by_id: &HashMap<String, Value>,
    cache: &mut HashMap<String, DocumentState>,
) -> Result<DocumentState, ReplayError> {
    if let Some(state) = cache.get(&revision.revision_id) {
        return Ok(state.clone());
    }

    let mut state = if revision.parents.is_empty() {
        DocumentState {
            doc_id: revision.doc_id.clone(),
            blocks: Vec::new(),
            metadata: Map::new(),
        }
    } else {
        let parent_id = &revision.parents[0];
        let parent_value = object_values_by_id.get(parent_id).ok_or_else(|| {
            ReplayError::new(format!("missing parent revision '{parent_id}' for replay"))
        })?;
        let parent_revision = parse_revision_object(parent_value).map_err(|error| {
            ReplayError::new(format!(
                "failed to parse parent revision '{parent_id}': {error}"
            ))
        })?;
        let parent_state = replay_revision_object(&parent_revision, object_values_by_id, cache)?;
        if parent_state.doc_id != revision.doc_id {
            return Err(ReplayError::new(format!(
                "parent revision '{parent_id}' belongs to '{}' instead of '{}'",
                parent_state.doc_id, revision.doc_id
            )));
        }
        parent_state
    };

    let expected_base_revision = revision
        .parents
        .first()
        .map(String::as_str)
        .unwrap_or(GENESIS_BASE_REVISION);

    for patch_id in &revision.patches {
        let patch_value = object_values_by_id
            .get(patch_id)
            .ok_or_else(|| ReplayError::new(format!("missing patch '{patch_id}' for replay")))?;
        let patch = parse_patch_object(patch_value).map_err(|error| {
            ReplayError::new(format!("failed to parse patch '{patch_id}': {error}"))
        })?;
        if patch.doc_id != revision.doc_id {
            return Err(ReplayError::new(format!(
                "patch '{patch_id}' belongs to '{}' instead of '{}'",
                patch.doc_id, revision.doc_id
            )));
        }
        if patch.base_revision != expected_base_revision {
            return Err(ReplayError::new(format!(
                "patch '{patch_id}' base_revision '{}' does not match expected '{}'",
                patch.base_revision, expected_base_revision
            )));
        }
        apply_patch_ops(&mut state, &patch)?;
    }

    cache.insert(revision.revision_id.clone(), state.clone());
    Ok(state)
}

fn apply_patch_op(state: &mut DocumentState, op: &PatchOperation) -> Result<(), ReplayError> {
    match op {
        PatchOperation::InsertBlock {
            parent_block_id,
            index,
            new_block,
        } => insert_block(state, parent_block_id.as_deref(), *index, new_block.clone()),
        PatchOperation::InsertBlockAfter {
            after_block_id,
            new_block,
        } => insert_block_after(&mut state.blocks, after_block_id, new_block.clone()),
        PatchOperation::DeleteBlock { block_id } => {
            delete_block(&mut state.blocks, block_id).map(|deleted| {
                if !deleted {
                    return Err(ReplayError::new(format!(
                        "delete_block target '{block_id}' was not found"
                    )));
                }
                Ok(())
            })?
        }
        PatchOperation::ReplaceBlock {
            block_id,
            new_content,
        } => replace_block(&mut state.blocks, block_id, new_content),
        PatchOperation::MoveBlock {
            block_id,
            parent_block_id,
            after_block_id,
        } => move_block(
            &mut state.blocks,
            block_id,
            parent_block_id.as_deref(),
            after_block_id.as_deref(),
        ),
        PatchOperation::AnnotateBlock {
            block_id,
            annotation,
        } => annotate_block(&mut state.blocks, block_id, annotation.clone()),
        PatchOperation::SetMetadata { entries } => {
            for (key, value) in entries {
                state.metadata.insert(key.clone(), value.clone());
            }
            Ok(())
        }
    }
}

fn insert_block(
    state: &mut DocumentState,
    parent_block_id: Option<&str>,
    index: Option<usize>,
    new_block: BlockObject,
) -> Result<(), ReplayError> {
    if block_exists(&state.blocks, &new_block.block_id) {
        return Err(ReplayError::new(format!(
            "insert_block would duplicate block_id '{}'",
            new_block.block_id
        )));
    }

    match parent_block_id {
        Some(parent_block_id) => {
            let children =
                find_children_mut(&mut state.blocks, parent_block_id).ok_or_else(|| {
                    ReplayError::new(format!(
                        "insert_block parent '{parent_block_id}' was not found"
                    ))
                })?;
            insert_at(children, index, new_block)
        }
        None => insert_at(&mut state.blocks, index, new_block),
    }
}

fn insert_block_after(
    blocks: &mut Vec<BlockObject>,
    after_block_id: &str,
    new_block: BlockObject,
) -> Result<(), ReplayError> {
    if block_exists(blocks, &new_block.block_id) {
        return Err(ReplayError::new(format!(
            "insert_block_after would duplicate block_id '{}'",
            new_block.block_id
        )));
    }

    if insert_after_in_blocks(blocks, after_block_id, new_block.clone()) {
        Ok(())
    } else {
        Err(ReplayError::new(format!(
            "insert_block_after target '{after_block_id}' was not found"
        )))
    }
}

fn delete_block(blocks: &mut Vec<BlockObject>, block_id: &str) -> Result<bool, ReplayError> {
    if let Some(index) = blocks.iter().position(|block| block.block_id == block_id) {
        blocks.remove(index);
        return Ok(true);
    }

    for block in blocks {
        if delete_block(&mut block.children, block_id)? {
            return Ok(true);
        }
    }

    Ok(false)
}

fn replace_block(
    blocks: &mut Vec<BlockObject>,
    block_id: &str,
    new_content: &str,
) -> Result<(), ReplayError> {
    let block = find_block_mut(blocks, block_id).ok_or_else(|| {
        ReplayError::new(format!("replace_block target '{block_id}' was not found"))
    })?;
    block.content = new_content.to_string();
    Ok(())
}

fn move_block(
    blocks: &mut Vec<BlockObject>,
    block_id: &str,
    parent_block_id: Option<&str>,
    after_block_id: Option<&str>,
) -> Result<(), ReplayError> {
    let block = detach_block(blocks, block_id)?
        .ok_or_else(|| ReplayError::new(format!("move_block target '{block_id}' was not found")))?;

    match (parent_block_id, after_block_id) {
        (Some(parent_block_id), Some(after_block_id)) => {
            let children = find_children_mut(blocks, parent_block_id).ok_or_else(|| {
                ReplayError::new(format!(
                    "move_block parent '{parent_block_id}' was not found"
                ))
            })?;
            if !insert_after_in_blocks(children, after_block_id, block.clone()) {
                return Err(ReplayError::new(format!(
                    "move_block after target '{after_block_id}' was not found"
                )));
            }
            Ok(())
        }
        (Some(parent_block_id), None) => {
            let children = find_children_mut(blocks, parent_block_id).ok_or_else(|| {
                ReplayError::new(format!(
                    "move_block parent '{parent_block_id}' was not found"
                ))
            })?;
            children.push(block);
            Ok(())
        }
        (None, Some(after_block_id)) => {
            if !insert_after_in_blocks(blocks, after_block_id, block.clone()) {
                return Err(ReplayError::new(format!(
                    "move_block after target '{after_block_id}' was not found"
                )));
            }
            Ok(())
        }
        (None, None) => {
            blocks.push(block);
            Ok(())
        }
    }
}

fn annotate_block(
    blocks: &mut Vec<BlockObject>,
    block_id: &str,
    annotation: BlockObject,
) -> Result<(), ReplayError> {
    let block = find_block_mut(blocks, block_id).ok_or_else(|| {
        ReplayError::new(format!("annotate_block target '{block_id}' was not found"))
    })?;
    if block_exists(&block.children, &annotation.block_id) {
        return Err(ReplayError::new(format!(
            "annotate_block would duplicate child block_id '{}'",
            annotation.block_id
        )));
    }
    block.children.push(annotation);
    Ok(())
}

fn insert_at(
    blocks: &mut Vec<BlockObject>,
    index: Option<usize>,
    new_block: BlockObject,
) -> Result<(), ReplayError> {
    match index {
        Some(index) if index <= blocks.len() => {
            blocks.insert(index, new_block);
            Ok(())
        }
        Some(index) => Err(ReplayError::new(format!(
            "insert index {index} is out of range for {} blocks",
            blocks.len()
        ))),
        None => {
            blocks.push(new_block);
            Ok(())
        }
    }
}

fn insert_after_in_blocks(
    blocks: &mut Vec<BlockObject>,
    after_block_id: &str,
    new_block: BlockObject,
) -> bool {
    if let Some(index) = blocks
        .iter()
        .position(|block| block.block_id == after_block_id)
    {
        blocks.insert(index + 1, new_block);
        return true;
    }

    for block in blocks {
        if insert_after_in_blocks(&mut block.children, after_block_id, new_block.clone()) {
            return true;
        }
    }

    false
}

fn find_block_mut<'a>(
    blocks: &'a mut [BlockObject],
    block_id: &str,
) -> Option<&'a mut BlockObject> {
    for block in blocks {
        if block.block_id == block_id {
            return Some(block);
        }
        if let Some(child) = find_block_mut(&mut block.children, block_id) {
            return Some(child);
        }
    }
    None
}

fn find_children_mut<'a>(
    blocks: &'a mut [BlockObject],
    block_id: &str,
) -> Option<&'a mut Vec<BlockObject>> {
    for block in blocks {
        if block.block_id == block_id {
            return Some(&mut block.children);
        }
        if let Some(children) = find_children_mut(&mut block.children, block_id) {
            return Some(children);
        }
    }
    None
}

fn detach_block(
    blocks: &mut Vec<BlockObject>,
    block_id: &str,
) -> Result<Option<BlockObject>, ReplayError> {
    if let Some(index) = blocks.iter().position(|block| block.block_id == block_id) {
        return Ok(Some(blocks.remove(index)));
    }

    for block in blocks {
        if let Some(found) = detach_block(&mut block.children, block_id)? {
            return Ok(Some(found));
        }
    }

    Ok(None)
}

fn block_exists(blocks: &[BlockObject], block_id: &str) -> bool {
    blocks
        .iter()
        .any(|block| block.block_id == block_id || block_exists(&block.children, block_id))
}

fn state_to_value(state: &DocumentState) -> Value {
    let mut object = Map::new();
    object.insert("doc_id".to_string(), Value::String(state.doc_id.clone()));
    object.insert(
        "blocks".to_string(),
        Value::Array(state.blocks.iter().map(block_to_value).collect::<Vec<_>>()),
    );
    if !state.metadata.is_empty() {
        object.insert(
            "metadata".to_string(),
            Value::Object(state.metadata.clone()),
        );
    }
    Value::Object(object)
}

fn block_to_value(block: &BlockObject) -> Value {
    let mut object = Map::new();
    object.insert(
        "block_id".to_string(),
        Value::String(block.block_id.clone()),
    );
    object.insert(
        "block_type".to_string(),
        Value::String(block.block_type.clone()),
    );
    object.insert("content".to_string(), Value::String(block.content.clone()));
    object.insert("attrs".to_string(), Value::Object(block.attrs.clone()));
    object.insert(
        "children".to_string(),
        Value::Array(
            block
                .children
                .iter()
                .map(block_to_value)
                .collect::<Vec<_>>(),
        ),
    );
    Value::Object(object)
}

#[cfg(test)]
mod tests {
    use std::collections::HashMap;

    use serde_json::json;
    use serde_json::Map;

    use super::{apply_patch_ops, compute_state_hash, replay_revision_from_index, DocumentState};
    use crate::protocol::{parse_patch_object, BlockObject};

    fn block(block_id: &str, content: &str) -> BlockObject {
        BlockObject {
            block_id: block_id.to_string(),
            block_type: "paragraph".to_string(),
            content: content.to_string(),
            attrs: Map::new(),
            children: Vec::new(),
        }
    }

    #[test]
    fn apply_patch_ops_inserts_and_replaces_blocks() {
        let patch = parse_patch_object(&json!({
            "type": "patch",
            "version": "mycel/0.1",
            "patch_id": "patch:test",
            "doc_id": "doc:test",
            "base_revision": "rev:genesis-null",
            "author": "pk:ed25519:test",
            "timestamp": 1u64,
            "ops": [
                {
                    "op": "insert_block",
                    "new_block": {
                        "block_id": "blk:001",
                        "block_type": "paragraph",
                        "content": "Alpha",
                        "attrs": {},
                        "children": []
                    }
                },
                {
                    "op": "replace_block",
                    "block_id": "blk:001",
                    "new_content": "Beta"
                }
            ]
        }))
        .expect("patch should parse");

        let mut state = DocumentState {
            doc_id: "doc:test".to_string(),
            blocks: Vec::new(),
            metadata: Map::new(),
        };
        apply_patch_ops(&mut state, &patch).expect("patch should apply");

        assert_eq!(state.blocks, vec![block("blk:001", "Beta")]);
    }

    #[test]
    fn compute_state_hash_is_deterministic() {
        let state = DocumentState {
            doc_id: "doc:test".to_string(),
            blocks: vec![block("blk:001", "Hello")],
            metadata: Map::new(),
        };

        let left = compute_state_hash(&state).expect("hash should compute");
        let right = compute_state_hash(&state).expect("hash should compute");

        assert_eq!(left, right);
        assert!(left.starts_with("hash:"));
    }

    #[test]
    fn replay_revision_from_index_recomputes_state_hash() {
        let patch = json!({
            "type": "patch",
            "version": "mycel/0.1",
            "patch_id": "patch:seed",
            "doc_id": "doc:test",
            "base_revision": "rev:genesis-null",
            "author": "pk:ed25519:test",
            "timestamp": 1u64,
            "ops": [
                {
                    "op": "insert_block",
                    "new_block": {
                        "block_id": "blk:001",
                        "block_type": "paragraph",
                        "content": "Hello",
                        "attrs": {},
                        "children": []
                    }
                }
            ]
        });
        let expected_state = DocumentState {
            doc_id: "doc:test".to_string(),
            blocks: vec![block("blk:001", "Hello")],
            metadata: Map::new(),
        };
        let expected_hash = compute_state_hash(&expected_state).expect("hash should compute");
        let revision = json!({
            "type": "revision",
            "version": "mycel/0.1",
            "revision_id": "rev:test",
            "doc_id": "doc:test",
            "parents": [],
            "patches": ["patch:seed"],
            "state_hash": expected_hash,
            "author": "pk:ed25519:test",
            "timestamp": 2u64
        });

        let mut index = HashMap::new();
        index.insert("patch:seed".to_string(), patch.clone());
        index.insert("rev:test".to_string(), revision.clone());

        let summary = replay_revision_from_index(&revision, &index).expect("replay should work");

        assert_eq!(summary.state, expected_state);
        assert_eq!(
            summary.recomputed_state_hash,
            compute_state_hash(&summary.state).unwrap()
        );
    }
}
