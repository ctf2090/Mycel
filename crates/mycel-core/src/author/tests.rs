use std::fs;
use std::path::PathBuf;

use base64::Engine;
use ed25519_dalek::SigningKey;
use serde_json::{json, Value};

use super::{
    commit_revision_to_store, create_document_in_store, create_merge_revision_in_store,
    create_patch_in_store, parse_signing_key_seed, signer_id, DocumentCreateParams, MergeOutcome,
    MergeRevisionCreateParams, PatchCreateParams, RevisionCommitParams,
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

#[path = "tests/authoring.rs"]
mod authoring;
#[path = "tests/manual.rs"]
mod manual;
#[path = "tests/structural.rs"]
mod structural;
#[path = "tests/variants.rs"]
mod variants;
