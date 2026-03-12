use std::path::{Path, PathBuf};

use serde::{Deserialize, Serialize};
use serde_json::Value;

use crate::store::{write_object_value_to_store, StoreRebuildError, StoredObjectRecord};
use crate::wire::{WireMessageType, WirePeerDirectory, WireSession};

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

pub fn sync_pull_from_transcript(
    transcript: &SyncPullTranscript,
    store_root: &Path,
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
        if summary.object_message_count == 0 {
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

#[cfg(test)]
mod tests {
    use std::fs;
    use std::path::PathBuf;

    use base64::Engine;
    use ed25519_dalek::{Signer, SigningKey};
    use serde_json::{json, Value};

    use crate::canonical::{signed_payload_bytes, wire_envelope_signed_payload_bytes};
    use crate::protocol::recompute_object_id;
    use crate::replay::{compute_state_hash, DocumentState};
    use crate::store::load_store_index_manifest;

    use super::{sync_pull_from_transcript, SyncPeer, SyncPullTranscript};

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
        let mut value = json!({
            "type": "HELLO",
            "version": "mycel-wire/0.1",
            "msg_id": "msg:hello-sync-001",
            "timestamp": "2026-03-08T20:00:00+08:00",
            "from": sender,
            "payload": {
                "node_id": sender,
                "capabilities": ["patch-sync"],
                "nonce": "n:sync-test"
            },
            "sig": "sig:placeholder"
        });
        value["sig"] = Value::String(sign_wire_value(signing_key, &value));
        value
    }

    fn signed_manifest_message(signing_key: &SigningKey, sender: &str, revision_id: &str) -> Value {
        let mut value = json!({
            "type": "MANIFEST",
            "version": "mycel-wire/0.1",
            "msg_id": "msg:manifest-sync-001",
            "timestamp": "2026-03-08T20:00:10+08:00",
            "from": sender,
            "payload": {
                "node_id": sender,
                "capabilities": ["patch-sync"],
                "heads": {
                    "doc:test": [revision_id]
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
        patches: &[&str],
    ) -> Value {
        let body = sign_object_value(
            signing_key,
            json!({
                "type": "revision",
                "version": "mycel/0.1",
                "revision_id": "rev:placeholder",
                "doc_id": "doc:test",
                "parents": [],
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

    #[test]
    fn sync_pull_from_transcript_verifies_and_stores_requested_objects() {
        let signing_key = signing_key();
        let sender = "node:alpha";
        let patch_object = signed_patch_object_message(&signing_key, sender, "rev:genesis-null");
        let patch_id = patch_object["payload"]["object_id"]
            .as_str()
            .expect("patch object should include object id")
            .to_string();
        let revision_object = signed_revision_object_message(&signing_key, sender, &[&patch_id]);
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
        let revision_object = signed_revision_object_message(&signing_key, sender, &[&patch_id]);
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
    fn sync_pull_from_transcript_fails_before_storing_invalid_object_message() {
        let signing_key = signing_key();
        let sender = "node:alpha";
        let patch_object = signed_patch_object_message(&signing_key, sender, "rev:genesis-null");
        let revision_object = signed_revision_object_message(
            &signing_key,
            sender,
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
    fn sync_pull_from_transcript_fails_for_unrequested_object_message() {
        let signing_key = signing_key();
        let sender = "node:alpha";
        let patch_object = signed_patch_object_message(&signing_key, sender, "rev:genesis-null");
        let revision_object = signed_revision_object_message(
            &signing_key,
            sender,
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
        let revision_object = signed_revision_object_message(&signing_key, sender, &[&patch_id]);
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
