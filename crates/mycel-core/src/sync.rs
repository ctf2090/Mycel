use std::collections::{BTreeMap, BTreeSet, HashMap};
use std::path::{Path, PathBuf};

use base64::Engine;
use ed25519_dalek::{Signer, SigningKey};
use serde::{Deserialize, Serialize};
use serde_json::{json, Value};

use crate::canonical::wire_envelope_signed_payload_bytes;
use crate::replay::GENESIS_BASE_REVISION;
use crate::store::{
    load_store_index_manifest, load_store_object_index, write_object_value_to_store,
    StoreIndexManifest, StoreRebuildError, StoredObjectRecord,
};
use crate::wire::{
    derive_wire_object_payload_identity, discover_reachable_object_ids_from_value, WireMessageType,
    WirePeerDirectory, WireSession,
};

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct SyncPeer {
    pub node_id: String,
    pub public_key: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SyncPullTranscript {
    pub peer: SyncPeer,
    #[serde(default)]
    pub messages: Vec<Value>,
}

#[derive(Debug, Clone, Serialize)]
pub struct SyncPullSummary {
    pub peer_node_id: String,
    pub store_root: PathBuf,
    pub status: String,
    pub message_count: usize,
    pub verified_message_count: usize,
    pub object_message_count: usize,
    pub verified_object_count: usize,
    pub written_object_count: usize,
    pub existing_object_count: usize,
    pub stored_objects: Vec<StoredObjectRecord>,
    pub index_manifest_path: Option<PathBuf>,
    pub notes: Vec<String>,
    pub errors: Vec<String>,
}

impl SyncPullSummary {
    fn new(peer_node_id: &str, store_root: &Path, message_count: usize) -> Self {
        Self {
            peer_node_id: peer_node_id.to_string(),
            store_root: store_root.to_path_buf(),
            status: "ok".to_string(),
            message_count,
            verified_message_count: 0,
            object_message_count: 0,
            verified_object_count: 0,
            written_object_count: 0,
            existing_object_count: 0,
            stored_objects: Vec::new(),
            index_manifest_path: None,
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

    fn push_note(&mut self, message: impl Into<String>) {
        self.notes.push(message.into());
        if self.status != "failed" {
            self.status = "warning".to_string();
        }
    }
}

#[derive(Debug, Clone)]
struct PendingSyncObject {
    object_id: String,
    body: Value,
}

fn sender_public_key(signing_key: &SigningKey) -> String {
    format!(
        "pk:ed25519:{}",
        base64::engine::general_purpose::STANDARD.encode(signing_key.verifying_key().as_bytes())
    )
}

fn sign_wire_value(signing_key: &SigningKey, value: &Value) -> Result<String, StoreRebuildError> {
    let payload = wire_envelope_signed_payload_bytes(value).map_err(|error| {
        StoreRebuildError::new(format!("failed to canonicalize wire payload: {error}"))
    })?;
    let signature = signing_key.sign(&payload);
    Ok(format!(
        "sig:ed25519:{}",
        base64::engine::general_purpose::STANDARD.encode(signature.to_bytes())
    ))
}

fn fixed_wire_timestamp() -> &'static str {
    "2026-03-08T20:00:00+08:00"
}

fn next_wire_msg_id(sequence: &mut usize, label: &str) -> String {
    let current = *sequence;
    *sequence += 1;
    format!("msg:peer-sync-{label}-{current:04}")
}

fn signed_sync_wire_message(
    signing_key: &SigningKey,
    sender: &str,
    message_type: &str,
    msg_id: String,
    payload: Value,
) -> Result<Value, StoreRebuildError> {
    let mut value = json!({
        "type": message_type,
        "version": "mycel-wire/0.1",
        "msg_id": msg_id,
        "timestamp": fixed_wire_timestamp(),
        "from": sender,
        "payload": payload,
        "sig": "sig:placeholder"
    });
    value["sig"] = Value::String(sign_wire_value(signing_key, &value)?);
    Ok(value)
}

fn signed_hello_message_with_capabilities(
    signing_key: &SigningKey,
    sender: &str,
    capabilities: &[&str],
) -> Result<Value, StoreRebuildError> {
    signed_sync_wire_message(
        signing_key,
        sender,
        "HELLO",
        "msg:peer-sync-hello-0000".to_string(),
        json!({
            "node_id": sender,
            "capabilities": capabilities,
            "nonce": "n:peer-sync"
        }),
    )
}

fn signed_manifest_message_with_capabilities(
    signing_key: &SigningKey,
    sender: &str,
    msg_id: String,
    heads: &BTreeMap<String, Vec<String>>,
    capabilities: &[&str],
) -> Result<Value, StoreRebuildError> {
    signed_sync_wire_message(
        signing_key,
        sender,
        "MANIFEST",
        msg_id,
        json!({
            "node_id": sender,
            "capabilities": capabilities,
            "heads": heads
        }),
    )
}

fn signed_heads_message(
    signing_key: &SigningKey,
    sender: &str,
    msg_id: String,
    heads: &BTreeMap<String, Vec<String>>,
) -> Result<Value, StoreRebuildError> {
    signed_sync_wire_message(
        signing_key,
        sender,
        "HEADS",
        msg_id,
        json!({
            "documents": heads,
            "replace": true
        }),
    )
}

fn signed_want_message(
    signing_key: &SigningKey,
    sender: &str,
    msg_id: String,
    object_ids: &[String],
) -> Result<Value, StoreRebuildError> {
    signed_sync_wire_message(
        signing_key,
        sender,
        "WANT",
        msg_id,
        json!({
            "objects": object_ids
        }),
    )
}

fn signed_object_message(
    signing_key: &SigningKey,
    sender: &str,
    msg_id: String,
    body: &Value,
) -> Result<Value, StoreRebuildError> {
    let identity = derive_wire_object_payload_identity(body).map_err(|error| {
        StoreRebuildError::new(format!(
            "failed to derive peer store OBJECT payload identity: {error}"
        ))
    })?;

    signed_sync_wire_message(
        signing_key,
        sender,
        "OBJECT",
        msg_id,
        json!({
            "object_id": identity.object_id,
            "object_type": identity.object_type,
            "encoding": "json",
            "hash_alg": "sha256",
            "hash": identity.hash,
            "body": body
        }),
    )
}

fn signed_snapshot_offer_message(
    signing_key: &SigningKey,
    sender: &str,
    msg_id: String,
    snapshot_id: &str,
    body: &Value,
) -> Result<Value, StoreRebuildError> {
    let documents = body
        .get("documents")
        .and_then(Value::as_object)
        .ok_or_else(|| {
            StoreRebuildError::new(
                "failed to build SNAPSHOT_OFFER payload: snapshot is missing 'documents'",
            )
        })?;
    let root_hash = body
        .get("root_hash")
        .and_then(Value::as_str)
        .ok_or_else(|| {
            StoreRebuildError::new(
                "failed to build SNAPSHOT_OFFER payload: snapshot is missing string field 'root_hash'",
            )
        })?;
    let document_ids = documents.keys().cloned().collect::<Vec<_>>();

    signed_sync_wire_message(
        signing_key,
        sender,
        "SNAPSHOT_OFFER",
        msg_id,
        json!({
            "snapshot_id": snapshot_id,
            "root_hash": root_hash,
            "documents": document_ids
        }),
    )
}

fn signed_view_announce_message(
    signing_key: &SigningKey,
    sender: &str,
    msg_id: String,
    view_id: &str,
    body: &Value,
) -> Result<Value, StoreRebuildError> {
    let documents = body
        .get("documents")
        .and_then(Value::as_object)
        .ok_or_else(|| {
            StoreRebuildError::new(
                "failed to build VIEW_ANNOUNCE payload: view is missing 'documents'",
            )
        })?;
    let maintainer = body
        .get("maintainer")
        .and_then(Value::as_str)
        .ok_or_else(|| {
            StoreRebuildError::new(
                "failed to build VIEW_ANNOUNCE payload: view is missing string field 'maintainer'",
            )
        })?;

    signed_sync_wire_message(
        signing_key,
        sender,
        "VIEW_ANNOUNCE",
        msg_id,
        json!({
            "view_id": view_id,
            "maintainer": maintainer,
            "documents": documents
        }),
    )
}

fn signed_bye_message(signing_key: &SigningKey, sender: &str) -> Result<Value, StoreRebuildError> {
    signed_sync_wire_message(
        signing_key,
        sender,
        "BYE",
        "msg:peer-sync-bye-final".to_string(),
        json!({
            "reason": "done"
        }),
    )
}

fn local_store_object_index_or_empty(
    store_root: &Path,
) -> Result<HashMap<String, Value>, StoreRebuildError> {
    if store_root.exists() {
        load_store_object_index(store_root)
    } else {
        Ok(HashMap::new())
    }
}

fn head_map_from_manifest(manifest: &StoreIndexManifest) -> BTreeMap<String, Vec<String>> {
    if !manifest.doc_heads.is_empty() {
        return manifest.doc_heads.clone();
    }

    // Fallback for stores with older manifests that lack doc_heads.
    let mut heads = BTreeMap::new();
    for (doc_id, revision_ids) in &manifest.doc_revisions {
        let parent_ids = revision_ids
            .iter()
            .filter_map(|revision_id| manifest.revision_parents.get(revision_id))
            .flatten()
            .cloned()
            .collect::<BTreeSet<_>>();
        let mut doc_heads = revision_ids
            .iter()
            .filter(|revision_id| !parent_ids.contains(*revision_id))
            .cloned()
            .collect::<Vec<_>>();
        doc_heads.sort();
        doc_heads.dedup();
        if !doc_heads.is_empty() {
            heads.insert(doc_id.clone(), doc_heads);
        }
    }

    heads
}

fn missing_object_ids(
    candidates: impl IntoIterator<Item = String>,
    known_local_ids: &BTreeSet<String>,
) -> Vec<String> {
    let mut missing = candidates
        .into_iter()
        .filter(|object_id| object_id != GENESIS_BASE_REVISION)
        .filter(|object_id| !known_local_ids.contains(object_id))
        .collect::<Vec<_>>();
    missing.sort();
    missing.dedup();
    missing
}

fn advertised_sync_capabilities(remote_manifest: &StoreIndexManifest) -> Vec<&'static str> {
    let mut capabilities = vec!["patch-sync"];
    if remote_manifest
        .object_ids_by_type
        .get("snapshot")
        .is_some_and(|ids| !ids.is_empty())
    {
        capabilities.push("snapshot-sync");
    }
    if remote_manifest
        .object_ids_by_type
        .get("view")
        .is_some_and(|ids| !ids.is_empty())
    {
        capabilities.push("view-sync");
    }
    capabilities
}

fn sorted_object_ids_for_type(manifest: &StoreIndexManifest, object_type: &str) -> Vec<String> {
    let mut object_ids = manifest
        .object_ids_by_type
        .get(object_type)
        .cloned()
        .unwrap_or_default();
    object_ids.sort();
    object_ids.dedup();
    object_ids
}

fn generate_sync_pull_transcript_filtered(
    peer: &SyncPeer,
    peer_signing_key: &SigningKey,
    peer_store_root: &Path,
    local_store_root: &Path,
    requested_doc_ids: Option<&[String]>,
) -> Result<SyncPullTranscript, StoreRebuildError> {
    let derived_public_key = sender_public_key(peer_signing_key);
    if derived_public_key != peer.public_key {
        return Err(StoreRebuildError::new(format!(
            "peer public key '{}' does not match provided signing key '{}'",
            peer.public_key, derived_public_key
        )));
    }

    let remote_manifest = load_store_index_manifest(peer_store_root)?;
    let all_remote_heads = head_map_from_manifest(&remote_manifest);
    let remote_heads: BTreeMap<String, Vec<String>> = match requested_doc_ids {
        Some(ids) => {
            let ids_set: BTreeSet<&str> = ids.iter().map(String::as_str).collect();
            all_remote_heads
                .into_iter()
                .filter(|(doc_id, _)| ids_set.contains(doc_id.as_str()))
                .collect()
        }
        None => all_remote_heads,
    };
    let remote_snapshot_ids = sorted_object_ids_for_type(&remote_manifest, "snapshot");
    let remote_view_ids = sorted_object_ids_for_type(&remote_manifest, "view");
    let advertised_capabilities = advertised_sync_capabilities(&remote_manifest);
    if remote_heads.is_empty() {
        return Err(StoreRebuildError::new(format!(
            "peer store {} does not expose any document heads for the requested scope",
            peer_store_root.display()
        )));
    }

    let remote_object_index = load_store_object_index(peer_store_root)?;
    let local_object_index = local_store_object_index_or_empty(local_store_root)?;
    let mut known_local_ids = local_object_index.keys().cloned().collect::<BTreeSet<_>>();
    let local_store_was_empty = known_local_ids.is_empty();

    let mut messages = Vec::new();
    messages.push(signed_hello_message_with_capabilities(
        peer_signing_key,
        &peer.node_id,
        &advertised_capabilities,
    )?);
    if local_store_was_empty {
        messages.push(signed_manifest_message_with_capabilities(
            peer_signing_key,
            &peer.node_id,
            "msg:peer-sync-manifest-0001".to_string(),
            &remote_heads,
            &advertised_capabilities,
        )?);
    } else {
        messages.push(signed_heads_message(
            peer_signing_key,
            &peer.node_id,
            "msg:peer-sync-heads-0001".to_string(),
            &remote_heads,
        )?);
    }

    let mut sequence = 2usize;
    for snapshot_id in &remote_snapshot_ids {
        let body = remote_object_index.get(snapshot_id).ok_or_else(|| {
            StoreRebuildError::new(format!(
                "peer store {} is missing offered snapshot '{}'",
                peer_store_root.display(),
                snapshot_id
            ))
        })?;
        messages.push(signed_snapshot_offer_message(
            peer_signing_key,
            &peer.node_id,
            next_wire_msg_id(&mut sequence, "snapshot-offer"),
            snapshot_id,
            body,
        )?);
    }
    for view_id in &remote_view_ids {
        let body = remote_object_index.get(view_id).ok_or_else(|| {
            StoreRebuildError::new(format!(
                "peer store {} is missing announced view '{}'",
                peer_store_root.display(),
                view_id
            ))
        })?;
        messages.push(signed_view_announce_message(
            peer_signing_key,
            &peer.node_id,
            next_wire_msg_id(&mut sequence, "view-announce"),
            view_id,
            body,
        )?);
    }

    let mut next_batch = missing_object_ids(
        remote_heads
            .values()
            .flatten()
            .cloned()
            .chain(remote_snapshot_ids.iter().cloned())
            .chain(remote_view_ids.iter().cloned())
            .collect::<Vec<_>>(),
        &known_local_ids,
    );

    while !next_batch.is_empty() {
        messages.push(signed_want_message(
            peer_signing_key,
            &peer.node_id,
            next_wire_msg_id(&mut sequence, "want"),
            &next_batch,
        )?);

        let mut newly_reachable = BTreeSet::new();
        for object_id in &next_batch {
            let body = remote_object_index.get(object_id).ok_or_else(|| {
                StoreRebuildError::new(format!(
                    "peer store {} is missing advertised object '{}'",
                    peer_store_root.display(),
                    object_id
                ))
            })?;
            messages.push(signed_object_message(
                peer_signing_key,
                &peer.node_id,
                next_wire_msg_id(&mut sequence, "object"),
                body,
            )?);
            known_local_ids.insert(object_id.clone());
            newly_reachable.extend(discover_reachable_object_ids_from_value(body).map_err(
                |error| {
                    StoreRebuildError::new(format!(
                        "failed to discover reachable IDs for '{}': {error}",
                        object_id
                    ))
                },
            )?);
        }

        next_batch = missing_object_ids(newly_reachable, &known_local_ids);
    }

    messages.push(signed_bye_message(peer_signing_key, &peer.node_id)?);

    Ok(SyncPullTranscript {
        peer: peer.clone(),
        messages,
    })
}

fn flush_pending_sync_objects(
    store_root: &Path,
    pending_objects: &mut Vec<PendingSyncObject>,
    summary: &mut SyncPullSummary,
) -> Result<(), StoreRebuildError> {
    loop {
        let mut wrote_any = false;
        let mut next_round = Vec::new();

        for pending in pending_objects.drain(..) {
            match write_object_value_to_store(store_root, &pending.body) {
                Ok(write) => {
                    summary.index_manifest_path = write.index_manifest_path.clone();
                    if write.created {
                        summary.written_object_count += 1;
                    } else {
                        summary.existing_object_count += 1;
                    }
                    summary.stored_objects.push(write.record);
                    wrote_any = true;
                }
                Err(_) => next_round.push(pending),
            }
        }

        *pending_objects = next_round;
        if !wrote_any {
            break;
        }
    }

    Ok(())
}

fn sync_pull_from_transcript_with_policy(
    transcript: &SyncPullTranscript,
    store_root: &Path,
    require_object_messages: bool,
) -> Result<SyncPullSummary, StoreRebuildError> {
    std::fs::create_dir_all(store_root).map_err(|error| {
        StoreRebuildError::new(format!(
            "failed to create sync store root {}: {error}",
            store_root.display()
        ))
    })?;

    let mut known_peers = WirePeerDirectory::new();
    known_peers
        .register_known_peer(&transcript.peer.node_id, &transcript.peer.public_key)
        .map_err(StoreRebuildError::new)?;

    let mut session = WireSession::from_store_root(known_peers, store_root)?;
    let mut summary = SyncPullSummary::new(
        &transcript.peer.node_id,
        store_root,
        transcript.messages.len(),
    );
    let mut pending_objects = Vec::new();

    if transcript.messages.is_empty() {
        summary.push_error("sync transcript must include at least one wire message");
        return Ok(summary);
    }

    for (index, message) in transcript.messages.iter().enumerate() {
        let envelope = match session.verify_incoming(message) {
            Ok(envelope) => envelope,
            Err(error) => {
                summary.push_error(format!(
                    "message {} failed verification: {error}",
                    index + 1
                ));
                break;
            }
        };
        summary.verified_message_count += 1;

        if !matches!(envelope.message_type(), WireMessageType::Object) {
            continue;
        }

        summary.object_message_count += 1;
        let Some(body) = envelope.payload().get("body") else {
            summary.push_error(format!(
                "message {} OBJECT payload is missing object field 'body'",
                index + 1
            ));
            break;
        };
        let object_id = envelope
            .payload()
            .get("object_id")
            .and_then(Value::as_str)
            .unwrap_or("<unknown>");
        summary.verified_object_count += 1;
        pending_objects.push(PendingSyncObject {
            object_id: object_id.to_string(),
            body: body.clone(),
        });
        flush_pending_sync_objects(store_root, &mut pending_objects, &mut summary)?;
    }

    flush_pending_sync_objects(store_root, &mut pending_objects, &mut summary)?;
    if !pending_objects.is_empty() {
        for pending in pending_objects {
            let error = write_object_value_to_store(store_root, &pending.body)
                .err()
                .map(|error| error.to_string())
                .unwrap_or_else(|| "unknown store failure".to_string());
            summary.push_error(format!(
                "verified OBJECT '{}' could not be stored after pull completion: {error}",
                pending.object_id
            ));
        }
    }

    summary.stored_objects.sort_by(|left, right| {
        left.object_id
            .cmp(&right.object_id)
            .then_with(|| left.path.cmp(&right.path))
    });

    let Some(peer_session) = session.peer_session(&transcript.peer.node_id) else {
        summary.push_error(format!(
            "sync transcript did not establish a wire session for '{}'",
            transcript.peer.node_id
        ));
        return Ok(summary);
    };

    if summary.is_ok() {
        if !peer_session.hello_received() {
            summary.push_error(format!(
                "sync transcript did not include HELLO from '{}'",
                transcript.peer.node_id
            ));
        }
        if !peer_session.has_head_context() {
            summary.push_error(format!(
                "sync transcript did not include MANIFEST or HEADS from '{}'",
                transcript.peer.node_id
            ));
        }
        if require_object_messages && summary.object_message_count == 0 {
            summary.push_error("sync transcript did not include any OBJECT messages");
        }
        if peer_session.pending_object_count() > 0 {
            summary.push_error(format!(
                "sync transcript ended with {} pending requested object(s)",
                peer_session.pending_object_count()
            ));
        }
        if !peer_session.is_closed() {
            summary.push_note(format!(
                "sync transcript ended without BYE from '{}'",
                transcript.peer.node_id
            ));
        }
    }

    Ok(summary)
}

pub fn sync_pull_from_transcript(
    transcript: &SyncPullTranscript,
    store_root: &Path,
) -> Result<SyncPullSummary, StoreRebuildError> {
    sync_pull_from_transcript_with_policy(transcript, store_root, true)
}

pub fn generate_sync_pull_transcript_from_peer_store(
    peer: &SyncPeer,
    peer_signing_key: &SigningKey,
    peer_store_root: &Path,
    local_store_root: &Path,
) -> Result<SyncPullTranscript, StoreRebuildError> {
    generate_sync_pull_transcript_filtered(
        peer,
        peer_signing_key,
        peer_store_root,
        local_store_root,
        None,
    )
}

pub fn sync_pull_from_peer_store(
    peer: &SyncPeer,
    peer_signing_key: &SigningKey,
    peer_store_root: &Path,
    local_store_root: &Path,
) -> Result<SyncPullSummary, StoreRebuildError> {
    let transcript = generate_sync_pull_transcript_filtered(
        peer,
        peer_signing_key,
        peer_store_root,
        local_store_root,
        None,
    )?;
    let generated_object_messages = transcript
        .messages
        .iter()
        .filter(|message| message.get("type").and_then(Value::as_str) == Some("OBJECT"))
        .count();
    let mut summary = sync_pull_from_transcript_with_policy(
        &transcript,
        local_store_root,
        generated_object_messages > 0,
    )?;
    if generated_object_messages == 0 && summary.is_ok() {
        summary.notes.push(
            "local store already satisfied the peer's advertised heads; no WANT messages were generated"
                .to_string(),
        );
    }
    Ok(summary)
}

pub fn sync_pull_from_peer_store_with_doc_filter(
    peer: &SyncPeer,
    peer_signing_key: &SigningKey,
    peer_store_root: &Path,
    local_store_root: &Path,
    requested_doc_ids: &[String],
) -> Result<SyncPullSummary, StoreRebuildError> {
    let transcript = generate_sync_pull_transcript_filtered(
        peer,
        peer_signing_key,
        peer_store_root,
        local_store_root,
        Some(requested_doc_ids),
    )?;
    let generated_object_messages = transcript
        .messages
        .iter()
        .filter(|message| message.get("type").and_then(Value::as_str) == Some("OBJECT"))
        .count();
    let mut summary = sync_pull_from_transcript_with_policy(
        &transcript,
        local_store_root,
        generated_object_messages > 0,
    )?;
    if generated_object_messages == 0 && summary.is_ok() {
        summary.notes.push(
            "partial-doc sync: local store already satisfied the requested document heads; no WANT messages were generated"
                .to_string(),
        );
    }
    Ok(summary)
}

#[cfg(test)]
mod tests {
    use std::collections::BTreeMap;
    use std::fs;
    use std::path::PathBuf;

    use base64::Engine;
    use ed25519_dalek::{Signer, SigningKey};
    use serde_json::{json, Value};

    use crate::canonical::{signed_payload_bytes, wire_envelope_signed_payload_bytes};
    use crate::protocol::recompute_object_id;
    use crate::replay::{compute_state_hash, DocumentState};
    use crate::store::{
        load_store_index_manifest, write_object_value_to_store, StoreIndexManifest,
    };

    use super::{
        generate_sync_pull_transcript_from_peer_store, head_map_from_manifest,
        sync_pull_from_peer_store, sync_pull_from_transcript, SyncPeer, SyncPullTranscript,
    };

    fn signing_key() -> SigningKey {
        SigningKey::from_bytes(&[5u8; 32])
    }

    fn temp_dir(prefix: &str) -> PathBuf {
        let unique = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .expect("system time should be after unix epoch")
            .as_nanos();
        let path = std::env::temp_dir().join(format!("mycel-sync-{prefix}-{unique}"));
        fs::create_dir_all(&path).expect("temp dir should be created");
        path
    }

    fn sender_public_key(signing_key: &SigningKey) -> String {
        format!(
            "pk:ed25519:{}",
            base64::engine::general_purpose::STANDARD
                .encode(signing_key.verifying_key().as_bytes())
        )
    }

    fn sign_wire_value(signing_key: &SigningKey, value: &Value) -> String {
        let payload =
            wire_envelope_signed_payload_bytes(value).expect("wire payload should canonicalize");
        let signature = signing_key.sign(&payload);
        format!(
            "sig:ed25519:{}",
            base64::engine::general_purpose::STANDARD.encode(signature.to_bytes())
        )
    }

    fn sign_object_value(
        signing_key: &SigningKey,
        mut value: Value,
        signer_field: &str,
        id_field: &str,
        prefix: &str,
    ) -> Value {
        value[signer_field] = Value::String(sender_public_key(signing_key));
        let object_id =
            recompute_object_id(&value, id_field, prefix).expect("test object ID should recompute");
        value[id_field] = Value::String(object_id);
        let payload = signed_payload_bytes(&value).expect("object payload should canonicalize");
        let signature = signing_key.sign(&payload);
        value["signature"] = Value::String(format!(
            "sig:ed25519:{}",
            base64::engine::general_purpose::STANDARD.encode(signature.to_bytes())
        ));
        value
    }

    fn empty_state_hash(doc_id: &str) -> String {
        compute_state_hash(&DocumentState {
            doc_id: doc_id.to_string(),
            blocks: Vec::new(),
            metadata: serde_json::Map::new(),
        })
        .expect("empty state hash should compute")
    }

    fn signed_hello_message(signing_key: &SigningKey, sender: &str) -> Value {
        signed_hello_message_with_capabilities(signing_key, sender, json!(["patch-sync"]))
    }

    fn signed_hello_message_with_capabilities(
        signing_key: &SigningKey,
        sender: &str,
        capabilities: Value,
    ) -> Value {
        let mut value = json!({
            "type": "HELLO",
            "version": "mycel-wire/0.1",
            "msg_id": "msg:hello-sync-001",
            "timestamp": "2026-03-08T20:00:00+08:00",
            "from": sender,
            "payload": {
                "node_id": sender,
                "capabilities": capabilities,
                "nonce": "n:sync-test"
            },
            "sig": "sig:placeholder"
        });
        value["sig"] = Value::String(sign_wire_value(signing_key, &value));
        value
    }

    fn signed_manifest_message(signing_key: &SigningKey, sender: &str, revision_id: &str) -> Value {
        signed_manifest_message_with_capabilities(
            signing_key,
            sender,
            revision_id,
            json!(["patch-sync"]),
        )
    }

    fn signed_manifest_message_with_capabilities(
        signing_key: &SigningKey,
        sender: &str,
        revision_id: &str,
        capabilities: Value,
    ) -> Value {
        let mut value = json!({
            "type": "MANIFEST",
            "version": "mycel-wire/0.1",
            "msg_id": "msg:manifest-sync-001",
            "timestamp": "2026-03-08T20:00:10+08:00",
            "from": sender,
            "payload": {
                "node_id": sender,
                "capabilities": capabilities,
                "heads": {
                    "doc:test": [revision_id]
                }
            },
            "sig": "sig:placeholder"
        });
        value["sig"] = Value::String(sign_wire_value(signing_key, &value));
        value
    }

    fn signed_snapshot_offer_message(
        signing_key: &SigningKey,
        sender: &str,
        snapshot_id: &str,
    ) -> Value {
        let mut value = json!({
            "type": "SNAPSHOT_OFFER",
            "version": "mycel-wire/0.1",
            "msg_id": "msg:snapshot-offer-sync-001",
            "timestamp": "2026-03-08T20:00:30+08:00",
            "from": sender,
            "payload": {
                "snapshot_id": snapshot_id,
                "root_hash": "hash:snapshot-root",
                "documents": ["doc:test"]
            },
            "sig": "sig:placeholder"
        });
        value["sig"] = Value::String(sign_wire_value(signing_key, &value));
        value
    }

    fn signed_view_announce_message(
        signing_key: &SigningKey,
        sender: &str,
        view_id: &str,
    ) -> Value {
        let mut value = json!({
            "type": "VIEW_ANNOUNCE",
            "version": "mycel-wire/0.1",
            "msg_id": "msg:view-announce-sync-001",
            "timestamp": "2026-03-08T20:00:45+08:00",
            "from": sender,
            "payload": {
                "view_id": view_id,
                "maintainer": sender_public_key(signing_key),
                "documents": {
                    "doc:test": "rev:test"
                }
            },
            "sig": "sig:placeholder"
        });
        value["sig"] = Value::String(sign_wire_value(signing_key, &value));
        value
    }

    fn signed_heads_message(
        signing_key: &SigningKey,
        sender: &str,
        revision_id: &str,
        replace: bool,
    ) -> Value {
        let mut value = json!({
            "type": "HEADS",
            "version": "mycel-wire/0.1",
            "msg_id": "msg:heads-sync-001",
            "timestamp": "2026-03-08T20:00:20+08:00",
            "from": sender,
            "payload": {
                "documents": {
                    "doc:test": [revision_id]
                },
                "replace": replace
            },
            "sig": "sig:placeholder"
        });
        value["sig"] = Value::String(sign_wire_value(signing_key, &value));
        value
    }

    fn signed_want_message(signing_key: &SigningKey, sender: &str, object_ids: &[&str]) -> Value {
        let mut value = json!({
            "type": "WANT",
            "version": "mycel-wire/0.1",
            "msg_id": "msg:want-sync-001",
            "timestamp": "2026-03-08T20:01:00+08:00",
            "from": sender,
            "payload": {
                "objects": object_ids
            },
            "sig": "sig:placeholder"
        });
        value["sig"] = Value::String(sign_wire_value(signing_key, &value));
        value
    }

    fn signed_patch_object_message(
        signing_key: &SigningKey,
        sender: &str,
        base_revision: &str,
    ) -> Value {
        let body = sign_object_value(
            signing_key,
            json!({
                "type": "patch",
                "version": "mycel/0.1",
                "patch_id": "patch:placeholder",
                "doc_id": "doc:test",
                "base_revision": base_revision,
                "author": "pk:ed25519:placeholder",
                "timestamp": 1u64,
                "ops": [],
                "signature": "sig:placeholder"
            }),
            "author",
            "patch_id",
            "patch",
        );
        let object_id = body["patch_id"]
            .as_str()
            .expect("signed patch body should include patch_id")
            .to_owned();
        let object_hash = object_id
            .split_once(':')
            .map(|(_, hash)| hash.to_string())
            .expect("wire object ID should contain hash");

        let mut value = json!({
            "type": "OBJECT",
            "version": "mycel-wire/0.1",
            "msg_id": "msg:object-sync-patch-001",
            "timestamp": "2026-03-08T20:01:10+08:00",
            "from": sender,
            "payload": {
                "object_id": object_id,
                "object_type": "patch",
                "encoding": "json",
                "hash_alg": "sha256",
                "hash": format!("hash:{object_hash}"),
                "body": body
            },
            "sig": "sig:placeholder"
        });
        value["sig"] = Value::String(sign_wire_value(signing_key, &value));
        value
    }

    fn signed_revision_object_message(
        signing_key: &SigningKey,
        sender: &str,
        parents: &[&str],
        patches: &[&str],
    ) -> Value {
        let body = sign_object_value(
            signing_key,
            json!({
                "type": "revision",
                "version": "mycel/0.1",
                "revision_id": "rev:placeholder",
                "doc_id": "doc:test",
                "parents": parents,
                "patches": patches,
                "state_hash": empty_state_hash("doc:test"),
                "author": "pk:ed25519:placeholder",
                "timestamp": 2u64,
                "signature": "sig:placeholder"
            }),
            "author",
            "revision_id",
            "rev",
        );
        let object_id = body["revision_id"]
            .as_str()
            .expect("signed revision body should include revision_id")
            .to_owned();
        let object_hash = object_id
            .split_once(':')
            .map(|(_, hash)| hash.to_string())
            .expect("wire revision ID should contain hash");

        let mut value = json!({
            "type": "OBJECT",
            "version": "mycel-wire/0.1",
            "msg_id": "msg:object-sync-rev-001",
            "timestamp": "2026-03-08T20:01:12+08:00",
            "from": sender,
            "payload": {
                "object_id": object_id,
                "object_type": "revision",
                "encoding": "json",
                "hash_alg": "sha256",
                "hash": format!("hash:{object_hash}"),
                "body": body
            },
            "sig": "sig:placeholder"
        });
        value["sig"] = Value::String(sign_wire_value(signing_key, &value));
        value
    }

    fn signed_snapshot_object_message(
        signing_key: &SigningKey,
        sender: &str,
        revision_id: &str,
    ) -> Value {
        let body = sign_object_value(
            signing_key,
            json!({
                "type": "snapshot",
                "version": "mycel/0.1",
                "snapshot_id": "snap:placeholder",
                "documents": {
                    "doc:test": revision_id
                },
                "included_objects": [revision_id],
                "root_hash": "hash:snapshot-root",
                "created_by": "pk:ed25519:placeholder",
                "timestamp": 3u64,
                "signature": "sig:placeholder"
            }),
            "created_by",
            "snapshot_id",
            "snap",
        );
        let object_id = body["snapshot_id"]
            .as_str()
            .expect("signed snapshot body should include snapshot_id")
            .to_owned();
        let object_hash = object_id
            .split_once(':')
            .map(|(_, hash)| hash.to_string())
            .expect("wire snapshot ID should contain hash");

        let mut value = json!({
            "type": "OBJECT",
            "version": "mycel-wire/0.1",
            "msg_id": "msg:object-sync-snapshot-001",
            "timestamp": "2026-03-08T20:01:14+08:00",
            "from": sender,
            "payload": {
                "object_id": object_id,
                "object_type": "snapshot",
                "encoding": "json",
                "hash_alg": "sha256",
                "hash": format!("hash:{object_hash}"),
                "body": body
            },
            "sig": "sig:placeholder"
        });
        value["sig"] = Value::String(sign_wire_value(signing_key, &value));
        value
    }

    fn signed_view_object_message(
        signing_key: &SigningKey,
        sender: &str,
        revision_id: &str,
    ) -> Value {
        let body = sign_object_value(
            signing_key,
            json!({
                "type": "view",
                "version": "mycel/0.1",
                "view_id": "view:placeholder",
                "maintainer": "pk:ed25519:placeholder",
                "documents": {
                    "doc:test": revision_id
                },
                "policy": {
                    "accept_keys": [sender_public_key(signing_key)],
                    "merge_rule": "manual-reviewed",
                    "preferred_branches": ["main"]
                },
                "timestamp": 4u64,
                "signature": "sig:placeholder"
            }),
            "maintainer",
            "view_id",
            "view",
        );
        let object_id = body["view_id"]
            .as_str()
            .expect("signed view body should include view_id")
            .to_owned();
        let object_hash = object_id
            .split_once(':')
            .map(|(_, hash)| hash.to_string())
            .expect("wire view ID should contain hash");

        let mut value = json!({
            "type": "OBJECT",
            "version": "mycel-wire/0.1",
            "msg_id": "msg:object-sync-view-001",
            "timestamp": "2026-03-08T20:01:16+08:00",
            "from": sender,
            "payload": {
                "object_id": object_id,
                "object_type": "view",
                "encoding": "json",
                "hash_alg": "sha256",
                "hash": format!("hash:{object_hash}"),
                "body": body
            },
            "sig": "sig:placeholder"
        });
        value["sig"] = Value::String(sign_wire_value(signing_key, &value));
        value
    }

    fn signed_bye_message(signing_key: &SigningKey, sender: &str) -> Value {
        let mut value = json!({
            "type": "BYE",
            "version": "mycel-wire/0.1",
            "msg_id": "msg:bye-sync-001",
            "timestamp": "2026-03-08T20:02:00+08:00",
            "from": sender,
            "payload": {
                "reason": "done"
            },
            "sig": "sig:placeholder"
        });
        value["sig"] = Value::String(sign_wire_value(signing_key, &value));
        value
    }

    fn sync_peer(sender: &str, signing_key: &SigningKey) -> SyncPeer {
        SyncPeer {
            node_id: sender.to_string(),
            public_key: sender_public_key(signing_key),
        }
    }

    #[test]
    fn generate_sync_pull_transcript_from_peer_store_builds_first_time_manifest_flow() {
        let signing_key = signing_key();
        let sender = "node:alpha";
        let remote_store_root = temp_dir("peer-driver-remote-first-time");
        let local_store_root = temp_dir("peer-driver-local-first-time");

        let patch_object = signed_patch_object_message(&signing_key, sender, "rev:genesis-null");
        let patch_id = patch_object["payload"]["object_id"]
            .as_str()
            .expect("patch object should include object id")
            .to_string();
        let revision_object =
            signed_revision_object_message(&signing_key, sender, &[], &[&patch_id]);

        write_object_value_to_store(&remote_store_root, &patch_object["payload"]["body"])
            .expect("patch should write to remote store");
        write_object_value_to_store(&remote_store_root, &revision_object["payload"]["body"])
            .expect("revision should write to remote store");

        let transcript = generate_sync_pull_transcript_from_peer_store(
            &sync_peer(sender, &signing_key),
            &signing_key,
            &remote_store_root,
            &local_store_root,
        )
        .expect("peer-store transcript should generate");

        let message_types = transcript
            .messages
            .iter()
            .map(|message| {
                message["type"]
                    .as_str()
                    .expect("generated message should include type")
                    .to_string()
            })
            .collect::<Vec<_>>();
        assert_eq!(
            message_types,
            vec!["HELLO", "MANIFEST", "WANT", "OBJECT", "WANT", "OBJECT", "BYE"]
        );
    }

    #[test]
    fn generate_sync_pull_transcript_from_peer_store_announces_views_when_present() {
        let signing_key = signing_key();
        let sender = "node:alpha";
        let remote_store_root = temp_dir("peer-driver-remote-views");
        let local_store_root = temp_dir("peer-driver-local-views");

        let patch_object = signed_patch_object_message(&signing_key, sender, "rev:genesis-null");
        let patch_id = patch_object["payload"]["object_id"]
            .as_str()
            .expect("patch object should include object id")
            .to_string();
        let revision_object =
            signed_revision_object_message(&signing_key, sender, &[], &[&patch_id]);
        let revision_id = revision_object["payload"]["object_id"]
            .as_str()
            .expect("revision object should include object id")
            .to_string();
        let view_object = signed_view_object_message(&signing_key, sender, &revision_id);
        let view_id = view_object["payload"]["object_id"]
            .as_str()
            .expect("view object should include object id")
            .to_string();

        for body in [
            &patch_object["payload"]["body"],
            &revision_object["payload"]["body"],
            &view_object["payload"]["body"],
        ] {
            write_object_value_to_store(&remote_store_root, body)
                .expect("object should write to remote store");
        }

        let transcript = generate_sync_pull_transcript_from_peer_store(
            &sync_peer(sender, &signing_key),
            &signing_key,
            &remote_store_root,
            &local_store_root,
        )
        .expect("peer-store transcript should generate");

        let message_types = transcript
            .messages
            .iter()
            .map(|message| {
                message["type"]
                    .as_str()
                    .expect("generated message should include type")
                    .to_string()
            })
            .collect::<Vec<_>>();
        assert_eq!(
            message_types,
            vec![
                "HELLO",
                "MANIFEST",
                "VIEW_ANNOUNCE",
                "WANT",
                "OBJECT",
                "OBJECT",
                "WANT",
                "OBJECT",
                "BYE"
            ]
        );
        assert_eq!(
            transcript.messages[0]["payload"]["capabilities"],
            json!(["patch-sync", "view-sync"])
        );
        assert_eq!(transcript.messages[2]["payload"]["view_id"], view_id);
    }

    #[test]
    fn sync_pull_from_peer_store_verifies_and_stores_first_time_sync() {
        let signing_key = signing_key();
        let sender = "node:alpha";
        let remote_store_root = temp_dir("peer-driver-remote-sync");
        let local_store_root = temp_dir("peer-driver-local-sync");

        let patch_object = signed_patch_object_message(&signing_key, sender, "rev:genesis-null");
        let patch_id = patch_object["payload"]["object_id"]
            .as_str()
            .expect("patch object should include object id")
            .to_string();
        let revision_object =
            signed_revision_object_message(&signing_key, sender, &[], &[&patch_id]);
        let revision_id = revision_object["payload"]["object_id"]
            .as_str()
            .expect("revision object should include object id")
            .to_string();

        write_object_value_to_store(&remote_store_root, &patch_object["payload"]["body"])
            .expect("patch should write to remote store");
        write_object_value_to_store(&remote_store_root, &revision_object["payload"]["body"])
            .expect("revision should write to remote store");

        let summary = sync_pull_from_peer_store(
            &sync_peer(sender, &signing_key),
            &signing_key,
            &remote_store_root,
            &local_store_root,
        )
        .expect("peer-store sync should run");

        assert!(summary.is_ok(), "expected ok summary, got {summary:?}");
        assert_eq!(summary.status, "ok");
        assert_eq!(summary.object_message_count, 2);
        assert_eq!(summary.written_object_count, 2);

        let manifest =
            load_store_index_manifest(&local_store_root).expect("local manifest should exist");
        assert_eq!(manifest.stored_object_count, 2);
        assert_eq!(
            manifest.doc_revisions.get("doc:test"),
            Some(&vec![revision_id])
        );
    }

    #[test]
    fn head_map_from_manifest_prefers_persisted_doc_heads() {
        let manifest = StoreIndexManifest {
            version: "mycel-store-index/0.1".to_string(),
            stored_object_count: 2,
            object_ids_by_type: BTreeMap::new(),
            doc_revisions: BTreeMap::from([(
                "doc:test".to_string(),
                vec!["rev:older".to_string(), "rev:newer".to_string()],
            )]),
            revision_parents: BTreeMap::from([(
                "rev:newer".to_string(),
                vec!["rev:older".to_string()],
            )]),
            author_patches: BTreeMap::new(),
            view_governance: Vec::new(),
            maintainer_views: BTreeMap::new(),
            profile_views: BTreeMap::new(),
            document_views: BTreeMap::new(),
            latest_profile_views: BTreeMap::new(),
            latest_document_profile_views: BTreeMap::new(),
            current_governance: BTreeMap::new(),
            profile_heads: BTreeMap::new(),
            doc_heads: BTreeMap::from([(
                "doc:test".to_string(),
                vec!["rev:persisted-head".to_string()],
            )]),
        };

        let heads = head_map_from_manifest(&manifest);

        assert_eq!(
            heads.get("doc:test"),
            Some(&vec!["rev:persisted-head".to_string()])
        );
    }

    #[test]
    fn head_map_from_manifest_falls_back_when_doc_heads_missing() {
        let manifest = StoreIndexManifest {
            version: "mycel-store-index/0.1".to_string(),
            stored_object_count: 3,
            object_ids_by_type: BTreeMap::new(),
            doc_revisions: BTreeMap::from([(
                "doc:test".to_string(),
                vec![
                    "rev:base".to_string(),
                    "rev:left".to_string(),
                    "rev:right".to_string(),
                ],
            )]),
            revision_parents: BTreeMap::from([
                ("rev:left".to_string(), vec!["rev:base".to_string()]),
                ("rev:right".to_string(), vec!["rev:base".to_string()]),
            ]),
            author_patches: BTreeMap::new(),
            view_governance: Vec::new(),
            maintainer_views: BTreeMap::new(),
            profile_views: BTreeMap::new(),
            document_views: BTreeMap::new(),
            latest_profile_views: BTreeMap::new(),
            latest_document_profile_views: BTreeMap::new(),
            current_governance: BTreeMap::new(),
            profile_heads: BTreeMap::new(),
            doc_heads: BTreeMap::new(),
        };

        let heads = head_map_from_manifest(&manifest);

        assert_eq!(
            heads.get("doc:test"),
            Some(&vec!["rev:left".to_string(), "rev:right".to_string()])
        );
    }

    #[test]
    fn sync_pull_from_peer_store_fetches_announced_views_as_governance_state() {
        let signing_key = signing_key();
        let sender = "node:alpha";
        let remote_store_root = temp_dir("peer-driver-remote-view-sync");
        let local_store_root = temp_dir("peer-driver-local-view-sync");

        let patch_object = signed_patch_object_message(&signing_key, sender, "rev:genesis-null");
        let patch_id = patch_object["payload"]["object_id"]
            .as_str()
            .expect("patch object should include object id")
            .to_string();
        let revision_object =
            signed_revision_object_message(&signing_key, sender, &[], &[&patch_id]);
        let revision_id = revision_object["payload"]["object_id"]
            .as_str()
            .expect("revision object should include object id")
            .to_string();
        let view_object = signed_view_object_message(&signing_key, sender, &revision_id);
        let view_id = view_object["payload"]["object_id"]
            .as_str()
            .expect("view object should include object id")
            .to_string();

        for body in [
            &patch_object["payload"]["body"],
            &revision_object["payload"]["body"],
            &view_object["payload"]["body"],
        ] {
            write_object_value_to_store(&remote_store_root, body)
                .expect("object should write to remote store");
        }

        let summary = sync_pull_from_peer_store(
            &sync_peer(sender, &signing_key),
            &signing_key,
            &remote_store_root,
            &local_store_root,
        )
        .expect("peer-store sync should run");

        assert!(summary.is_ok(), "expected ok summary, got {summary:?}");
        assert_eq!(summary.object_message_count, 3);
        assert_eq!(summary.written_object_count, 3);

        let manifest =
            load_store_index_manifest(&local_store_root).expect("local manifest should exist");
        assert_eq!(manifest.stored_object_count, 3);
        assert_eq!(manifest.view_governance.len(), 1);
        assert_eq!(manifest.view_governance[0].view_id, view_id);
        assert_eq!(
            manifest
                .document_views
                .get("doc:test")
                .expect("document views should be indexed"),
            &vec![view_id]
        );
    }

    #[test]
    fn sync_pull_from_peer_store_uses_heads_for_incremental_and_skips_up_to_date_sync() {
        let signing_key = signing_key();
        let sender = "node:alpha";
        let remote_store_root = temp_dir("peer-driver-remote-incremental");
        let local_store_root = temp_dir("peer-driver-local-incremental");

        let base_patch_object =
            signed_patch_object_message(&signing_key, sender, "rev:genesis-null");
        let base_patch_id = base_patch_object["payload"]["object_id"]
            .as_str()
            .expect("base patch object should include object id")
            .to_string();
        let base_revision_object =
            signed_revision_object_message(&signing_key, sender, &[], &[&base_patch_id]);
        let base_revision_id = base_revision_object["payload"]["object_id"]
            .as_str()
            .expect("base revision object should include object id")
            .to_string();
        let follow_patch_object =
            signed_patch_object_message(&signing_key, sender, &base_revision_id);
        let follow_patch_id = follow_patch_object["payload"]["object_id"]
            .as_str()
            .expect("follow patch object should include object id")
            .to_string();
        let follow_revision_object = signed_revision_object_message(
            &signing_key,
            sender,
            &[&base_revision_id],
            &[&follow_patch_id],
        );
        let follow_revision_id = follow_revision_object["payload"]["object_id"]
            .as_str()
            .expect("follow revision object should include object id")
            .to_string();

        for body in [
            &base_patch_object["payload"]["body"],
            &base_revision_object["payload"]["body"],
            &follow_patch_object["payload"]["body"],
            &follow_revision_object["payload"]["body"],
        ] {
            write_object_value_to_store(&remote_store_root, body)
                .expect("object should write to remote store");
        }

        write_object_value_to_store(&local_store_root, &base_patch_object["payload"]["body"])
            .expect("base patch should write to local store");
        write_object_value_to_store(&local_store_root, &base_revision_object["payload"]["body"])
            .expect("base revision should write to local store");

        let transcript = generate_sync_pull_transcript_from_peer_store(
            &sync_peer(sender, &signing_key),
            &signing_key,
            &remote_store_root,
            &local_store_root,
        )
        .expect("incremental peer-store transcript should generate");
        assert_eq!(
            transcript.messages[1]["type"].as_str(),
            Some("HEADS"),
            "incremental sync should advertise HEADS"
        );

        let summary = sync_pull_from_peer_store(
            &sync_peer(sender, &signing_key),
            &signing_key,
            &remote_store_root,
            &local_store_root,
        )
        .expect("incremental peer-store sync should run");

        assert!(summary.is_ok(), "expected ok summary, got {summary:?}");
        assert_eq!(summary.object_message_count, 2);
        let manifest =
            load_store_index_manifest(&local_store_root).expect("local manifest should exist");
        let revisions = manifest
            .doc_revisions
            .get("doc:test")
            .expect("expected local revisions after sync");
        assert!(revisions.contains(&follow_revision_id));

        let up_to_date_summary = sync_pull_from_peer_store(
            &sync_peer(sender, &signing_key),
            &signing_key,
            &remote_store_root,
            &local_store_root,
        )
        .expect("up-to-date sync should run");
        assert!(
            up_to_date_summary.is_ok(),
            "expected ok summary, got {up_to_date_summary:?}"
        );
        assert_eq!(up_to_date_summary.object_message_count, 0);
        assert!(
            up_to_date_summary
                .notes
                .iter()
                .any(|note| note.contains("no WANT messages")),
            "expected no-op sync note, got {up_to_date_summary:?}"
        );
    }

    #[test]
    fn sync_pull_from_transcript_verifies_and_stores_requested_objects() {
        let signing_key = signing_key();
        let sender = "node:alpha";
        let patch_object = signed_patch_object_message(&signing_key, sender, "rev:genesis-null");
        let patch_id = patch_object["payload"]["object_id"]
            .as_str()
            .expect("patch object should include object id")
            .to_string();
        let revision_object =
            signed_revision_object_message(&signing_key, sender, &[], &[&patch_id]);
        let revision_id = revision_object["payload"]["object_id"]
            .as_str()
            .expect("revision object should include object id")
            .to_string();
        let transcript = SyncPullTranscript {
            peer: SyncPeer {
                node_id: sender.to_string(),
                public_key: sender_public_key(&signing_key),
            },
            messages: vec![
                signed_hello_message(&signing_key, sender),
                signed_manifest_message(&signing_key, sender, &revision_id),
                signed_want_message(&signing_key, sender, &[&revision_id]),
                revision_object,
                signed_want_message(&signing_key, sender, &[&patch_id]),
                patch_object,
                signed_bye_message(&signing_key, sender),
            ],
        };
        let store_root = temp_dir("pull-ok");

        let summary =
            sync_pull_from_transcript(&transcript, &store_root).expect("sync pull should run");

        assert!(summary.is_ok(), "expected ok summary, got {summary:?}");
        assert_eq!(summary.status, "ok");
        assert_eq!(summary.message_count, 7);
        assert_eq!(summary.verified_message_count, 7);
        assert_eq!(summary.object_message_count, 2);
        assert_eq!(summary.verified_object_count, 2);
        assert_eq!(summary.written_object_count, 2);
        assert_eq!(summary.existing_object_count, 0);
        assert!(
            summary.errors.is_empty(),
            "summary errors: {:?}",
            summary.errors
        );
        assert!(
            summary
                .index_manifest_path
                .as_ref()
                .is_some_and(|path| path.ends_with("indexes/manifest.json")),
            "expected manifest path in summary: {summary:?}"
        );

        let manifest =
            load_store_index_manifest(&store_root).expect("store manifest should be readable");
        assert_eq!(manifest.stored_object_count, 2);
        assert_eq!(
            manifest.doc_revisions.get("doc:test"),
            Some(&vec![revision_id]),
            "expected revision to be indexed"
        );
    }

    #[test]
    fn sync_pull_from_transcript_verifies_first_time_sync_from_heads() {
        let signing_key = signing_key();
        let sender = "node:alpha";
        let patch_object = signed_patch_object_message(&signing_key, sender, "rev:genesis-null");
        let patch_id = patch_object["payload"]["object_id"]
            .as_str()
            .expect("patch object should include object id")
            .to_string();
        let revision_object =
            signed_revision_object_message(&signing_key, sender, &[], &[&patch_id]);
        let revision_id = revision_object["payload"]["object_id"]
            .as_str()
            .expect("revision object should include object id")
            .to_string();
        let transcript = SyncPullTranscript {
            peer: SyncPeer {
                node_id: sender.to_string(),
                public_key: sender_public_key(&signing_key),
            },
            messages: vec![
                signed_hello_message(&signing_key, sender),
                signed_heads_message(&signing_key, sender, &revision_id, true),
                signed_want_message(&signing_key, sender, &[&revision_id]),
                revision_object,
                signed_want_message(&signing_key, sender, &[&patch_id]),
                patch_object,
                signed_bye_message(&signing_key, sender),
            ],
        };
        let store_root = temp_dir("pull-heads-ok");

        let summary =
            sync_pull_from_transcript(&transcript, &store_root).expect("sync pull should run");

        assert!(summary.is_ok(), "expected ok summary, got {summary:?}");
        assert_eq!(summary.status, "ok");
        assert_eq!(summary.message_count, 7);
        assert_eq!(summary.verified_message_count, 7);
        assert_eq!(summary.object_message_count, 2);
        assert_eq!(summary.verified_object_count, 2);
        assert_eq!(summary.written_object_count, 2);
        assert_eq!(summary.existing_object_count, 0);
        assert!(
            summary.notes.is_empty(),
            "expected closed first-time sync without warnings: {summary:?}"
        );

        let manifest =
            load_store_index_manifest(&store_root).expect("store manifest should be readable");
        assert_eq!(manifest.stored_object_count, 2);
        assert_eq!(
            manifest.doc_revisions.get("doc:test"),
            Some(&vec![revision_id]),
            "expected revision to be indexed"
        );
    }

    #[test]
    fn sync_pull_from_transcript_verifies_incremental_sync_from_existing_store() {
        let signing_key = signing_key();
        let sender = "node:alpha";

        let base_patch_object =
            signed_patch_object_message(&signing_key, sender, "rev:genesis-null");
        let base_patch_id = base_patch_object["payload"]["object_id"]
            .as_str()
            .expect("base patch object should include object id")
            .to_string();
        let base_revision_object =
            signed_revision_object_message(&signing_key, sender, &[], &[&base_patch_id]);
        let base_revision_id = base_revision_object["payload"]["object_id"]
            .as_str()
            .expect("base revision object should include object id")
            .to_string();

        let follow_patch_object =
            signed_patch_object_message(&signing_key, sender, &base_revision_id);
        let follow_patch_id = follow_patch_object["payload"]["object_id"]
            .as_str()
            .expect("follow patch object should include object id")
            .to_string();
        let follow_revision_object = signed_revision_object_message(
            &signing_key,
            sender,
            &[&base_revision_id],
            &[&follow_patch_id],
        );
        let follow_revision_id = follow_revision_object["payload"]["object_id"]
            .as_str()
            .expect("follow revision object should include object id")
            .to_string();

        let store_root = temp_dir("pull-incremental");
        write_object_value_to_store(&store_root, &base_patch_object["payload"]["body"])
            .expect("base patch should write to store");
        write_object_value_to_store(&store_root, &base_revision_object["payload"]["body"])
            .expect("base revision should write to store");

        let transcript = SyncPullTranscript {
            peer: SyncPeer {
                node_id: sender.to_string(),
                public_key: sender_public_key(&signing_key),
            },
            messages: vec![
                signed_hello_message(&signing_key, sender),
                signed_manifest_message(&signing_key, sender, &follow_revision_id),
                signed_want_message(&signing_key, sender, &[&follow_revision_id]),
                follow_revision_object,
                signed_want_message(&signing_key, sender, &[&follow_patch_id]),
                follow_patch_object,
                signed_bye_message(&signing_key, sender),
            ],
        };

        let summary =
            sync_pull_from_transcript(&transcript, &store_root).expect("sync pull should run");

        assert!(summary.is_ok(), "expected ok summary, got {summary:?}");
        assert_eq!(summary.status, "ok");
        assert_eq!(summary.message_count, 7);
        assert_eq!(summary.verified_message_count, 7);
        assert_eq!(summary.object_message_count, 2);
        assert_eq!(summary.verified_object_count, 2);
        assert_eq!(summary.written_object_count, 2);
        assert_eq!(summary.existing_object_count, 0);
        assert!(
            summary.notes.is_empty(),
            "expected closed incremental sync without warnings: {summary:?}"
        );

        let manifest =
            load_store_index_manifest(&store_root).expect("store manifest should be readable");
        assert_eq!(manifest.stored_object_count, 4);
        let revisions = manifest
            .doc_revisions
            .get("doc:test")
            .expect("expected synced document revisions");
        assert_eq!(revisions.len(), 2);
        assert!(revisions.contains(&base_revision_id));
        assert!(revisions.contains(&follow_revision_id));
    }

    #[test]
    fn sync_pull_from_transcript_accepts_snapshot_offer_when_capability_is_advertised() {
        let signing_key = signing_key();
        let sender = "node:alpha";
        let snapshot_object = signed_snapshot_object_message(&signing_key, sender, "rev:test");
        let snapshot_id = snapshot_object["payload"]["object_id"]
            .as_str()
            .expect("snapshot object should include object id")
            .to_string();
        let transcript = SyncPullTranscript {
            peer: SyncPeer {
                node_id: sender.to_string(),
                public_key: sender_public_key(&signing_key),
            },
            messages: vec![
                signed_hello_message_with_capabilities(
                    &signing_key,
                    sender,
                    json!(["patch-sync", "snapshot-sync"]),
                ),
                signed_manifest_message_with_capabilities(
                    &signing_key,
                    sender,
                    "rev:test",
                    json!(["patch-sync", "snapshot-sync"]),
                ),
                signed_snapshot_offer_message(&signing_key, sender, &snapshot_id),
                signed_want_message(&signing_key, sender, &[&snapshot_id]),
                snapshot_object,
                signed_bye_message(&signing_key, sender),
            ],
        };
        let store_root = temp_dir("pull-snapshot-offer");

        let summary =
            sync_pull_from_transcript(&transcript, &store_root).expect("sync pull should run");

        assert!(summary.is_ok(), "expected ok summary, got {summary:?}");
        assert_eq!(summary.message_count, 6);
        assert_eq!(summary.verified_message_count, 6);
        assert_eq!(summary.object_message_count, 1);
        assert_eq!(summary.verified_object_count, 1);
        assert_eq!(summary.written_object_count, 1);

        let manifest =
            load_store_index_manifest(&store_root).expect("store manifest should be readable");
        assert_eq!(manifest.stored_object_count, 1);
    }

    #[test]
    fn generate_sync_pull_transcript_from_peer_store_offers_snapshots_when_present() {
        let signing_key = signing_key();
        let sender = "node:alpha";
        let remote_store_root = temp_dir("peer-driver-remote-snapshots");
        let local_store_root = temp_dir("peer-driver-local-snapshots");

        let patch_object = signed_patch_object_message(&signing_key, sender, "rev:genesis-null");
        let patch_id = patch_object["payload"]["object_id"]
            .as_str()
            .expect("patch object should include object id")
            .to_string();
        let revision_object =
            signed_revision_object_message(&signing_key, sender, &[], &[&patch_id]);
        let revision_id = revision_object["payload"]["object_id"]
            .as_str()
            .expect("revision object should include object id")
            .to_string();
        let snapshot_object = signed_snapshot_object_message(&signing_key, sender, &revision_id);
        let snapshot_id = snapshot_object["payload"]["object_id"]
            .as_str()
            .expect("snapshot object should include object id")
            .to_string();

        for body in [
            &patch_object["payload"]["body"],
            &revision_object["payload"]["body"],
            &snapshot_object["payload"]["body"],
        ] {
            write_object_value_to_store(&remote_store_root, body)
                .expect("object should write to remote store");
        }

        let transcript = generate_sync_pull_transcript_from_peer_store(
            &sync_peer(sender, &signing_key),
            &signing_key,
            &remote_store_root,
            &local_store_root,
        )
        .expect("peer-store transcript should generate");

        let message_types = transcript
            .messages
            .iter()
            .map(|message| {
                message["type"]
                    .as_str()
                    .expect("generated message should include type")
                    .to_string()
            })
            .collect::<Vec<_>>();
        assert_eq!(
            message_types,
            vec![
                "HELLO",
                "MANIFEST",
                "SNAPSHOT_OFFER",
                "WANT",
                "OBJECT",
                "OBJECT",
                "WANT",
                "OBJECT",
                "BYE"
            ]
        );
        assert_eq!(
            transcript.messages[0]["payload"]["capabilities"],
            json!(["patch-sync", "snapshot-sync"])
        );
        assert_eq!(
            transcript.messages[2]["payload"]["snapshot_id"],
            snapshot_id
        );
    }

    #[test]
    fn sync_pull_from_transcript_accepts_view_announce_when_capability_is_advertised() {
        let signing_key = signing_key();
        let sender = "node:alpha";
        let patch_object = signed_patch_object_message(&signing_key, sender, "rev:genesis-null");
        let patch_id = patch_object["payload"]["object_id"]
            .as_str()
            .expect("patch object should include object id")
            .to_string();
        let revision_object =
            signed_revision_object_message(&signing_key, sender, &[], &[&patch_id]);
        let revision_id = revision_object["payload"]["object_id"]
            .as_str()
            .expect("revision object should include object id")
            .to_string();
        let view_object = signed_view_object_message(&signing_key, sender, &revision_id);
        let view_id = view_object["payload"]["object_id"]
            .as_str()
            .expect("view object should include object id")
            .to_string();
        let transcript = SyncPullTranscript {
            peer: SyncPeer {
                node_id: sender.to_string(),
                public_key: sender_public_key(&signing_key),
            },
            messages: vec![
                signed_hello_message_with_capabilities(
                    &signing_key,
                    sender,
                    json!(["patch-sync", "view-sync"]),
                ),
                signed_manifest_message_with_capabilities(
                    &signing_key,
                    sender,
                    &revision_id,
                    json!(["patch-sync", "view-sync"]),
                ),
                signed_view_announce_message(&signing_key, sender, &view_id),
                signed_want_message(&signing_key, sender, &[&view_id]),
                view_object,
                signed_want_message(&signing_key, sender, &[&revision_id]),
                revision_object,
                signed_want_message(&signing_key, sender, &[&patch_id]),
                patch_object,
                signed_bye_message(&signing_key, sender),
            ],
        };
        let store_root = temp_dir("pull-view-announce");

        let summary =
            sync_pull_from_transcript(&transcript, &store_root).expect("sync pull should run");

        assert!(summary.is_ok(), "expected ok summary, got {summary:?}");
        assert_eq!(summary.object_message_count, 3);
        assert_eq!(summary.written_object_count, 3);

        let manifest =
            load_store_index_manifest(&store_root).expect("store manifest should be readable");
        assert_eq!(manifest.view_governance.len(), 1);
        assert_eq!(manifest.view_governance[0].view_id, view_id);
    }

    #[test]
    fn sync_pull_from_peer_store_fetches_offered_snapshots() {
        let signing_key = signing_key();
        let sender = "node:alpha";
        let remote_store_root = temp_dir("peer-driver-remote-snapshot-sync");
        let local_store_root = temp_dir("peer-driver-local-snapshot-sync");

        let patch_object = signed_patch_object_message(&signing_key, sender, "rev:genesis-null");
        let patch_id = patch_object["payload"]["object_id"]
            .as_str()
            .expect("patch object should include object id")
            .to_string();
        let revision_object =
            signed_revision_object_message(&signing_key, sender, &[], &[&patch_id]);
        let revision_id = revision_object["payload"]["object_id"]
            .as_str()
            .expect("revision object should include object id")
            .to_string();
        let snapshot_object = signed_snapshot_object_message(&signing_key, sender, &revision_id);
        let snapshot_id = snapshot_object["payload"]["object_id"]
            .as_str()
            .expect("snapshot object should include object id")
            .to_string();

        for body in [
            &patch_object["payload"]["body"],
            &revision_object["payload"]["body"],
            &snapshot_object["payload"]["body"],
        ] {
            write_object_value_to_store(&remote_store_root, body)
                .expect("object should write to remote store");
        }

        let summary = sync_pull_from_peer_store(
            &sync_peer(sender, &signing_key),
            &signing_key,
            &remote_store_root,
            &local_store_root,
        )
        .expect("peer-store sync should run");

        assert!(summary.is_ok(), "expected ok summary, got {summary:?}");
        assert_eq!(summary.object_message_count, 3);
        assert_eq!(summary.written_object_count, 3);

        let manifest =
            load_store_index_manifest(&local_store_root).expect("local manifest should exist");
        assert_eq!(manifest.stored_object_count, 3);
        assert_eq!(
            manifest.object_ids_by_type.get("snapshot"),
            Some(&vec![snapshot_id])
        );
    }

    #[test]
    fn sync_pull_from_transcript_fails_before_storing_invalid_object_message() {
        let signing_key = signing_key();
        let sender = "node:alpha";
        let patch_object = signed_patch_object_message(&signing_key, sender, "rev:genesis-null");
        let revision_object = signed_revision_object_message(
            &signing_key,
            sender,
            &[],
            &[patch_object["payload"]["object_id"]
                .as_str()
                .expect("patch id")],
        );
        let revision_id = revision_object["payload"]["object_id"]
            .as_str()
            .expect("revision object should include object id")
            .to_string();
        let mut invalid_object = revision_object.clone();
        invalid_object["payload"]["hash"] = Value::String("hash:tampered".to_string());
        let transcript = SyncPullTranscript {
            peer: SyncPeer {
                node_id: sender.to_string(),
                public_key: sender_public_key(&signing_key),
            },
            messages: vec![
                signed_hello_message(&signing_key, sender),
                signed_manifest_message(&signing_key, sender, &revision_id),
                signed_want_message(&signing_key, sender, &[&revision_id]),
                invalid_object,
            ],
        };
        let store_root = temp_dir("pull-invalid");

        let summary =
            sync_pull_from_transcript(&transcript, &store_root).expect("sync pull should run");

        assert!(!summary.is_ok(), "expected failed summary, got {summary:?}");
        assert_eq!(summary.verified_message_count, 3);
        assert_eq!(summary.object_message_count, 0);
        assert_eq!(summary.written_object_count, 0);
        assert!(
            summary
                .errors
                .iter()
                .any(|error| error.contains("failed verification")),
            "expected verification failure, got {summary:?}"
        );
        assert!(
            load_store_index_manifest(&store_root).is_err(),
            "unexpected manifest for failed pull"
        );
    }

    #[test]
    fn sync_pull_from_transcript_fails_before_counting_semantically_invalid_object_body() {
        let signing_key = signing_key();
        let sender = "node:alpha";
        let invalid_view_body = sign_object_value(
            &signing_key,
            json!({
                "type": "view",
                "version": "mycel/0.1",
                "view_id": "view:placeholder",
                "maintainer": "pk:ed25519:placeholder",
                "documents": {
                    "doc:test": "rev:test"
                },
                "policy": {
                    "accept_keys": [""],
                    "merge_rule": "manual-reviewed"
                },
                "timestamp": 4u64,
                "signature": "sig:placeholder"
            }),
            "maintainer",
            "view_id",
            "view",
        );
        let invalid_object = super::signed_object_message(
            &signing_key,
            sender,
            "msg:object-sync-invalid-view-001".to_string(),
            &invalid_view_body,
        )
        .expect("invalid view OBJECT should still serialize");
        let transcript = SyncPullTranscript {
            peer: SyncPeer {
                node_id: sender.to_string(),
                public_key: sender_public_key(&signing_key),
            },
            messages: vec![
                signed_hello_message_with_capabilities(
                    &signing_key,
                    sender,
                    json!(["patch-sync", "view-sync"]),
                ),
                signed_manifest_message_with_capabilities(
                    &signing_key,
                    sender,
                    "rev:test",
                    json!(["patch-sync", "view-sync"]),
                ),
                signed_view_announce_message(
                    &signing_key,
                    sender,
                    invalid_view_body["view_id"]
                        .as_str()
                        .expect("view id should exist"),
                ),
                signed_want_message(
                    &signing_key,
                    sender,
                    &[invalid_view_body["view_id"]
                        .as_str()
                        .expect("view id should exist")],
                ),
                invalid_object,
            ],
        };
        let store_root = temp_dir("pull-invalid-view-body");

        let summary =
            sync_pull_from_transcript(&transcript, &store_root).expect("sync pull should run");

        assert!(!summary.is_ok(), "expected failed summary, got {summary:?}");
        assert_eq!(summary.verified_message_count, 4);
        assert_eq!(summary.object_message_count, 0);
        assert_eq!(summary.verified_object_count, 0);
        assert_eq!(summary.written_object_count, 0);
        assert!(
            summary.errors.iter().any(|error| error.contains(
                "OBJECT body failed shared verification: top-level 'policy.accept_keys[0]' must not be an empty string"
            )),
            "expected shared semantic-edge verification failure, got {summary:?}"
        );
        assert!(
            load_store_index_manifest(&store_root).is_err(),
            "unexpected manifest for failed pull"
        );
    }

    #[test]
    fn sync_pull_from_transcript_fails_for_unrequested_object_message() {
        let signing_key = signing_key();
        let sender = "node:alpha";
        let patch_object = signed_patch_object_message(&signing_key, sender, "rev:genesis-null");
        let revision_object = signed_revision_object_message(
            &signing_key,
            sender,
            &[],
            &[patch_object["payload"]["object_id"]
                .as_str()
                .expect("patch id")],
        );
        let revision_id = revision_object["payload"]["object_id"]
            .as_str()
            .expect("revision object should include object id")
            .to_string();
        let transcript = SyncPullTranscript {
            peer: SyncPeer {
                node_id: sender.to_string(),
                public_key: sender_public_key(&signing_key),
            },
            messages: vec![
                signed_hello_message(&signing_key, sender),
                signed_manifest_message(&signing_key, sender, &revision_id),
                revision_object,
            ],
        };
        let store_root = temp_dir("pull-unrequested");

        let summary =
            sync_pull_from_transcript(&transcript, &store_root).expect("sync pull should run");

        assert!(!summary.is_ok(), "expected failed summary, got {summary:?}");
        assert_eq!(summary.verified_message_count, 2);
        assert_eq!(summary.object_message_count, 0);
        assert_eq!(summary.written_object_count, 0);
        assert!(
            summary
                .errors
                .iter()
                .any(|error| error.contains("was not requested")),
            "expected unrequested-object failure, got {summary:?}"
        );
        assert!(
            load_store_index_manifest(&store_root).is_err(),
            "unexpected manifest for failed pull"
        );
    }

    #[test]
    fn sync_pull_from_transcript_reports_pending_requested_objects_at_end() {
        let signing_key = signing_key();
        let sender = "node:alpha";
        let patch_object = signed_patch_object_message(&signing_key, sender, "rev:genesis-null");
        let patch_id = patch_object["payload"]["object_id"]
            .as_str()
            .expect("patch id")
            .to_string();
        let revision_object =
            signed_revision_object_message(&signing_key, sender, &[], &[&patch_id]);
        let revision_id = revision_object["payload"]["object_id"]
            .as_str()
            .expect("revision object should include object id")
            .to_string();
        let transcript = SyncPullTranscript {
            peer: SyncPeer {
                node_id: sender.to_string(),
                public_key: sender_public_key(&signing_key),
            },
            messages: vec![
                signed_hello_message(&signing_key, sender),
                signed_manifest_message(&signing_key, sender, &revision_id),
                signed_want_message(&signing_key, sender, &[&revision_id]),
                signed_bye_message(&signing_key, sender),
            ],
        };
        let store_root = temp_dir("pull-pending");

        let summary =
            sync_pull_from_transcript(&transcript, &store_root).expect("sync pull should run");

        assert!(!summary.is_ok(), "expected failed summary, got {summary:?}");
        assert_eq!(summary.verified_message_count, 4);
        assert_eq!(summary.object_message_count, 0);
        assert_eq!(summary.written_object_count, 0);
        assert!(
            summary
                .errors
                .iter()
                .any(|error| error.contains("did not include any OBJECT messages")),
            "expected missing-object failure, got {summary:?}"
        );
        assert!(
            summary
                .errors
                .iter()
                .any(|error| error.contains("pending requested object(s)")),
            "expected pending-request failure, got {summary:?}"
        );
        assert!(
            summary.notes.is_empty(),
            "BYE should suppress end-of-session warning: {summary:?}"
        );
        assert!(
            load_store_index_manifest(&store_root).is_err(),
            "unexpected manifest for failed pull"
        );
    }
}
