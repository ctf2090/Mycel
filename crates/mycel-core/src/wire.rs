use std::collections::{BTreeMap, BTreeSet};
use std::fmt;
use std::path::Path;
use std::str::FromStr;

use serde::{Deserialize, Serialize};
use serde_json::{Map, Value};

use crate::canonical::wire_envelope_signed_payload_bytes;
use crate::protocol::{
    ensure_supported_json_values, object_schema, parse_object_envelope, parse_patch_object,
    parse_revision_object, parse_snapshot_object, parse_view_object,
    recompute_declared_object_identity, reject_duplicate_strings, reject_unknown_fields,
    required_non_empty_string_array, required_prefixed_string_map, required_string_field,
    validate_canonical_object_id, validate_prefixed_string, StringFieldError,
    WIRE_OBJECT_HASH_ALGORITHM, WIRE_PROTOCOL_VERSION,
};
use crate::signature::verify_ed25519_signature;
use crate::store::{load_store_object_index, StoreRebuildError};
use crate::verify::verify_object_value;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum WireMessageType {
    Hello,
    Manifest,
    Heads,
    Want,
    Object,
    SnapshotOffer,
    ViewAnnounce,
    Bye,
    Error,
}

impl WireMessageType {
    pub fn as_str(self) -> &'static str {
        match self {
            Self::Hello => "HELLO",
            Self::Manifest => "MANIFEST",
            Self::Heads => "HEADS",
            Self::Want => "WANT",
            Self::Object => "OBJECT",
            Self::SnapshotOffer => "SNAPSHOT_OFFER",
            Self::ViewAnnounce => "VIEW_ANNOUNCE",
            Self::Bye => "BYE",
            Self::Error => "ERROR",
        }
    }
}

impl fmt::Display for WireMessageType {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(self.as_str())
    }
}

impl FromStr for WireMessageType {
    type Err = String;

    fn from_str(value: &str) -> Result<Self, Self::Err> {
        match value {
            "HELLO" => Ok(Self::Hello),
            "MANIFEST" => Ok(Self::Manifest),
            "HEADS" => Ok(Self::Heads),
            "WANT" => Ok(Self::Want),
            "OBJECT" => Ok(Self::Object),
            "SNAPSHOT_OFFER" => Ok(Self::SnapshotOffer),
            "VIEW_ANNOUNCE" => Ok(Self::ViewAnnounce),
            "BYE" => Ok(Self::Bye),
            "ERROR" => Ok(Self::Error),
            _ => Err(format!("unsupported wire message type '{value}'")),
        }
    }
}

#[derive(Debug)]
pub struct ParsedWireEnvelope<'a> {
    from: &'a str,
    message_type: WireMessageType,
    payload: &'a Map<String, Value>,
}

impl<'a> ParsedWireEnvelope<'a> {
    pub fn from(&self) -> &'a str {
        self.from
    }

    pub fn message_type(&self) -> WireMessageType {
        self.message_type
    }

    pub fn payload(&self) -> &'a Map<String, Value> {
        self.payload
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct WireObjectPayloadIdentity {
    pub object_type: String,
    pub object_id: String,
    pub hash: String,
}

pub fn parse_wire_envelope(value: &Value) -> Result<ParsedWireEnvelope<'_>, String> {
    ensure_supported_json_values(value)?;
    let object = value
        .as_object()
        .ok_or_else(|| "wire envelope top-level JSON value must be an object".to_string())?;
    reject_unknown_fields(
        object,
        "top-level",
        &[
            "type",
            "version",
            "msg_id",
            "timestamp",
            "from",
            "payload",
            "sig",
        ],
    )
    .map_err(|error| error.to_string())?;

    let message_type =
        WireMessageType::from_str(&required_wire_string(object, "type", "wire envelope")?)?;

    let version = required_wire_string(object, "version", "wire envelope")?;
    if version != WIRE_PROTOCOL_VERSION {
        return Err(format!(
            "wire envelope 'version' must equal '{WIRE_PROTOCOL_VERSION}'"
        ));
    }

    validate_prefixed_string(
        &required_wire_string(object, "msg_id", "wire envelope")?,
        "msg_id",
        "msg:",
    )
    .map_err(|error| error.to_string())?;
    validate_wire_timestamp(&required_wire_string(object, "timestamp", "wire envelope")?)?;
    validate_prefixed_string(
        &required_wire_string(object, "from", "wire envelope")?,
        "from",
        "node:",
    )
    .map_err(|error| error.to_string())?;
    let from = object
        .get("from")
        .and_then(Value::as_str)
        .ok_or_else(|| "wire envelope is missing string field 'from'".to_string())?;
    validate_prefixed_string(
        &required_wire_string(object, "sig", "wire envelope")?,
        "sig",
        "sig:",
    )
    .map_err(|error| error.to_string())?;

    let payload = match object.get("payload") {
        Some(Value::Object(payload)) => payload,
        Some(_) => return Err("top-level 'payload' must be an object".to_string()),
        None => return Err("missing object field 'payload'".to_string()),
    };

    Ok(ParsedWireEnvelope {
        from,
        message_type,
        payload,
    })
}

pub fn validate_wire_envelope(value: &Value) -> Result<ParsedWireEnvelope<'_>, String> {
    let envelope = parse_wire_envelope(value)?;
    validate_wire_payload(envelope.message_type(), envelope.payload())?;
    Ok(envelope)
}

pub fn verify_wire_envelope_signature<'a>(
    value: &'a Value,
    sender_public_key: &str,
) -> Result<ParsedWireEnvelope<'a>, String> {
    let envelope = validate_wire_envelope(value)?;
    verify_wire_envelope_signature_bytes(value, sender_public_key, "sender public key")?;
    Ok(envelope)
}

#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct WirePeerDirectory {
    sender_public_keys: BTreeMap<String, String>,
}

impl WirePeerDirectory {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn register_known_peer(
        &mut self,
        node_id: &str,
        public_key: &str,
    ) -> Result<Option<String>, String> {
        validate_prefixed_string(node_id, "node_id", "node:").map_err(|error| error.to_string())?;
        crate::signature::parse_ed25519_public_key(public_key, "public key")?;
        Ok(self
            .sender_public_keys
            .insert(node_id.to_owned(), public_key.to_owned()))
    }

    pub fn sender_public_key(&self, node_id: &str) -> Option<&str> {
        self.sender_public_keys.get(node_id).map(String::as_str)
    }
}

#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct WirePeerSessionState {
    hello_received: bool,
    advertised_capabilities: BTreeSet<String>,
    advertised_document_heads: BTreeMap<String, BTreeSet<String>>,
    accepted_sync_roots: BTreeSet<String>,
    reachable_object_ids: BTreeSet<String>,
    pending_object_ids: BTreeSet<String>,
    closed: bool,
}

impl WirePeerSessionState {
    fn advertises_revision(&self, revision_id: &str) -> bool {
        self.advertised_document_heads
            .values()
            .any(|revisions| revisions.contains(revision_id))
    }

    fn advertises_capability(&self, capability: &str) -> bool {
        self.advertised_capabilities.contains(capability)
    }

    pub fn hello_received(&self) -> bool {
        self.hello_received
    }

    pub fn has_head_context(&self) -> bool {
        !self.advertised_document_heads.is_empty()
    }

    pub fn pending_object_count(&self) -> usize {
        self.pending_object_ids.len()
    }

    pub fn is_closed(&self) -> bool {
        self.closed
    }
}

#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct WireSession {
    known_peers: WirePeerDirectory,
    known_verified_object_index: BTreeMap<String, Value>,
    peer_sessions: BTreeMap<String, WirePeerSessionState>,
}

impl WireSession {
    pub fn new(known_peers: WirePeerDirectory) -> Self {
        Self {
            known_peers,
            known_verified_object_index: BTreeMap::new(),
            peer_sessions: BTreeMap::new(),
        }
    }

    pub fn from_store_root(
        known_peers: WirePeerDirectory,
        store_root: &Path,
    ) -> Result<Self, StoreRebuildError> {
        let mut session = Self::new(known_peers);
        session.load_known_verified_object_index_from_store(store_root)?;
        Ok(session)
    }

    pub fn register_known_peer(
        &mut self,
        node_id: &str,
        public_key: &str,
    ) -> Result<Option<String>, String> {
        self.known_peers.register_known_peer(node_id, public_key)
    }

    pub fn known_peers(&self) -> &WirePeerDirectory {
        &self.known_peers
    }

    pub fn set_known_verified_object_index(&mut self, object_index: BTreeMap<String, Value>) {
        self.known_verified_object_index = object_index;
    }

    pub fn load_known_verified_object_index_from_store(
        &mut self,
        store_root: &Path,
    ) -> Result<(), StoreRebuildError> {
        self.known_verified_object_index =
            load_store_object_index(store_root)?.into_iter().collect();
        Ok(())
    }

    pub fn peer_session(&self, node_id: &str) -> Option<&WirePeerSessionState> {
        self.peer_sessions.get(node_id)
    }

    pub fn verify_incoming<'a>(
        &mut self,
        value: &'a Value,
    ) -> Result<ParsedWireEnvelope<'a>, String> {
        let envelope = validate_wire_envelope(value)?;
        validate_wire_sender_identity(&envelope)?;
        if matches!(envelope.message_type(), WireMessageType::Object) {
            validate_wire_object_payload_behavior(envelope.payload())?;
        }
        let sender_public_key = self
            .known_peers
            .sender_public_key(envelope.from())
            .ok_or_else(|| format!("unknown wire sender '{}'", envelope.from()))?;
        verify_wire_envelope_signature_bytes(value, sender_public_key, "known sender public key")?;
        let known_verified_object_index = &self.known_verified_object_index;
        let peer_session = self
            .peer_sessions
            .entry(envelope.from().to_owned())
            .or_default();
        validate_wire_inbound_sequence(&envelope, peer_session)?;
        advance_wire_inbound_sequence(&envelope, peer_session)?;
        expand_reachable_object_ids_from_known_index(peer_session, known_verified_object_index)?;
        Ok(envelope)
    }
}

fn verify_wire_envelope_signature_bytes(
    value: &Value,
    sender_public_key: &str,
    sender_public_key_label: &str,
) -> Result<(), String> {
    let signature = value
        .as_object()
        .and_then(|object| object.get("sig"))
        .and_then(Value::as_str)
        .ok_or_else(|| "wire envelope is missing string field 'sig'".to_string())?;
    let payload = wire_envelope_signed_payload_bytes(value)?;
    verify_ed25519_signature(
        &payload,
        sender_public_key,
        signature,
        sender_public_key_label,
        "sig field",
    )
}

fn validate_wire_sender_identity(envelope: &ParsedWireEnvelope<'_>) -> Result<(), String> {
    match envelope.message_type() {
        WireMessageType::Hello | WireMessageType::Manifest => {
            let node_id = required_wire_string(envelope.payload(), "node_id", "wire payload")?;
            if node_id != envelope.from() {
                return Err(format!(
                    "wire {} payload 'node_id' must equal envelope 'from'",
                    envelope.message_type()
                ));
            }
        }
        _ => {}
    }
    Ok(())
}

fn validate_wire_inbound_sequence(
    envelope: &ParsedWireEnvelope<'_>,
    peer_session: &WirePeerSessionState,
) -> Result<(), String> {
    if peer_session.closed {
        return Err(format!(
            "wire session for '{}' is already closed",
            envelope.from()
        ));
    }

    match envelope.message_type() {
        WireMessageType::Hello => {
            if peer_session.hello_received {
                return Err(format!(
                    "wire session already received HELLO from '{}'",
                    envelope.from()
                ));
            }
        }
        WireMessageType::Manifest
        | WireMessageType::Heads
        | WireMessageType::Want
        | WireMessageType::Object
        | WireMessageType::SnapshotOffer
        | WireMessageType::ViewAnnounce
        | WireMessageType::Bye => {
            if !peer_session.hello_received {
                return Err(format!(
                    "wire {} requires prior HELLO from '{}'",
                    envelope.message_type(),
                    envelope.from()
                ));
            }
        }
        WireMessageType::Error => {}
    }

    if matches!(envelope.message_type(), WireMessageType::Want) {
        let requested_object_ids = validate_wire_string_array(envelope.payload(), "objects")?;
        if peer_session.advertised_document_heads.is_empty() {
            return Err(format!(
                "wire WANT requires prior MANIFEST or HEADS from '{}'",
                envelope.from()
            ));
        }

        for object_id in requested_object_ids {
            if object_id.starts_with("rev:") {
                if !peer_session.advertises_revision(&object_id)
                    && !peer_session.reachable_object_ids.contains(&object_id)
                {
                    return Err(format!(
                        "wire WANT revision '{}' is not reachable from accepted sync roots for '{}'",
                        object_id,
                        envelope.from()
                    ));
                }
            } else if !peer_session.reachable_object_ids.contains(&object_id) {
                return Err(format!(
                    "wire WANT object '{}' is not reachable from accepted sync roots for '{}'",
                    object_id,
                    envelope.from()
                ));
            }
        }
    }

    if matches!(envelope.message_type(), WireMessageType::Object) {
        let object_id = required_wire_string(envelope.payload(), "object_id", "OBJECT payload")?;
        if !peer_session.pending_object_ids.contains(&object_id) {
            return Err(format!(
                "wire OBJECT '{}' was not requested from '{}'",
                object_id,
                envelope.from()
            ));
        }
    }

    match envelope.message_type() {
        WireMessageType::SnapshotOffer => require_advertised_wire_capability(
            peer_session,
            "snapshot-sync",
            envelope.message_type(),
            envelope.from(),
        )?,
        WireMessageType::ViewAnnounce => require_advertised_wire_capability(
            peer_session,
            "view-sync",
            envelope.message_type(),
            envelope.from(),
        )?,
        _ => {}
    }

    Ok(())
}

fn advance_wire_inbound_sequence(
    envelope: &ParsedWireEnvelope<'_>,
    peer_session: &mut WirePeerSessionState,
) -> Result<(), String> {
    match envelope.message_type() {
        WireMessageType::Hello => {
            peer_session.hello_received = true;
            peer_session
                .advertised_capabilities
                .extend(validate_wire_string_array(
                    envelope.payload(),
                    "capabilities",
                )?);
        }
        WireMessageType::Manifest => {
            peer_session
                .advertised_capabilities
                .extend(validate_wire_string_array(
                    envelope.payload(),
                    "capabilities",
                )?);
            peer_session.advertised_document_heads =
                wire_head_map_to_sets(validate_wire_head_map(envelope.payload(), "heads")?);
        }
        WireMessageType::Heads => {
            let documents =
                wire_head_map_to_sets(validate_wire_head_map(envelope.payload(), "documents")?);
            let replace = required_wire_bool(envelope.payload(), "replace", "wire payload")?;
            if replace {
                peer_session.advertised_document_heads = documents;
            } else {
                for (doc_id, revisions) in documents {
                    peer_session
                        .advertised_document_heads
                        .entry(doc_id)
                        .or_default()
                        .extend(revisions);
                }
            }
        }
        WireMessageType::Want => {
            for object_id in validate_wire_string_array(envelope.payload(), "objects")? {
                if object_id.starts_with("rev:") && peer_session.advertises_revision(&object_id) {
                    peer_session.accepted_sync_roots.insert(object_id.clone());
                }
                peer_session.pending_object_ids.insert(object_id);
            }
        }
        WireMessageType::Object => {
            let object_id =
                required_wire_string(envelope.payload(), "object_id", "OBJECT payload")?;
            peer_session.pending_object_ids.remove(&object_id);
            extend_reachable_object_ids_from_object(envelope.payload(), peer_session)?;
        }
        WireMessageType::SnapshotOffer => {
            let snapshot_id =
                required_wire_string(envelope.payload(), "snapshot_id", "wire payload")?;
            peer_session.reachable_object_ids.insert(snapshot_id);
        }
        WireMessageType::ViewAnnounce => {
            let view_id = required_wire_string(envelope.payload(), "view_id", "wire payload")?;
            peer_session.reachable_object_ids.insert(view_id);
        }
        WireMessageType::Bye => {
            peer_session.closed = true;
        }
        WireMessageType::Error => {}
    }

    Ok(())
}

pub fn validate_wire_payload(
    message_type: WireMessageType,
    payload: &Map<String, Value>,
) -> Result<(), String> {
    match message_type {
        WireMessageType::Hello => validate_wire_hello_payload(payload)?,
        WireMessageType::Manifest => validate_wire_manifest_payload(payload)?,
        WireMessageType::Heads => validate_wire_heads_payload(payload)?,
        WireMessageType::Want => validate_wire_want_payload(payload)?,
        WireMessageType::Object => validate_wire_object_payload(payload)?,
        WireMessageType::SnapshotOffer => validate_wire_snapshot_offer_payload(payload)?,
        WireMessageType::ViewAnnounce => validate_wire_view_announce_payload(payload)?,
        WireMessageType::Bye => validate_wire_bye_payload(payload)?,
        WireMessageType::Error => validate_wire_error_payload(payload)?,
    }

    Ok(())
}

fn validate_wire_hello_payload(payload: &Map<String, Value>) -> Result<(), String> {
    reject_unknown_fields(
        payload,
        "top-level",
        &["node_id", "agent", "capabilities", "topics", "nonce"],
    )
    .map_err(|error| error.to_string())?;
    validate_wire_node_capabilities_payload(payload)?;
    optional_wire_string(payload, "agent", "wire payload")?;
    validate_prefixed_string(
        &required_wire_string(payload, "nonce", "wire payload")?,
        "nonce",
        "n:",
    )
    .map_err(|error| error.to_string())?;
    Ok(())
}

fn validate_wire_manifest_payload(payload: &Map<String, Value>) -> Result<(), String> {
    reject_unknown_fields(
        payload,
        "top-level",
        &[
            "node_id",
            "capabilities",
            "topics",
            "heads",
            "snapshots",
            "views",
        ],
    )
    .map_err(|error| error.to_string())?;
    validate_wire_node_capabilities_payload(payload)?;
    validate_wire_head_map(payload, "heads")?;
    validate_wire_optional_canonical_object_array(payload, "snapshots")?;
    validate_wire_optional_canonical_object_array(payload, "views")?;
    Ok(())
}

fn validate_wire_heads_payload(payload: &Map<String, Value>) -> Result<(), String> {
    reject_unknown_fields(payload, "top-level", &["documents", "replace"])
        .map_err(|error| error.to_string())?;
    validate_wire_head_map(payload, "documents")?;
    match payload.get("replace") {
        Some(Value::Bool(_)) => Ok(()),
        Some(_) => Err("top-level 'replace' must be a boolean".to_string()),
        None => Err("missing boolean field 'replace'".to_string()),
    }
}

fn validate_wire_want_payload(payload: &Map<String, Value>) -> Result<(), String> {
    reject_unknown_fields(payload, "top-level", &["objects", "max_items"])
        .map_err(|error| error.to_string())?;
    validate_wire_required_canonical_object_array(payload, "objects")?;
    optional_wire_u64(payload, "max_items", "wire payload")?;
    Ok(())
}

fn validate_wire_object_payload(payload: &Map<String, Value>) -> Result<(), String> {
    reject_unknown_fields(
        payload,
        "top-level",
        &[
            "object_id",
            "object_type",
            "encoding",
            "hash_alg",
            "hash",
            "body",
        ],
    )
    .map_err(|error| error.to_string())?;
    validate_canonical_object_id(
        &required_wire_string(payload, "object_id", "OBJECT payload")?,
        "object_id",
    )
    .map_err(|error| error.to_string())?;
    object_schema(&required_wire_string(
        payload,
        "object_type",
        "OBJECT payload",
    )?)
    .ok_or_else(|| "OBJECT payload 'object_type' must be a supported object type".to_string())?;
    let encoding = required_wire_string(payload, "encoding", "OBJECT payload")?;
    if encoding != "json" {
        return Err("OBJECT payload 'encoding' must equal 'json'".to_string());
    }
    validate_prefixed_string(
        &required_wire_string(payload, "hash", "OBJECT payload")?,
        "hash",
        "hash:",
    )
    .map_err(|error| error.to_string())?;
    let hash_alg = required_wire_string(payload, "hash_alg", "OBJECT payload")?;
    if hash_alg != WIRE_OBJECT_HASH_ALGORITHM {
        return Err(format!(
            "OBJECT payload 'hash_alg' must equal '{WIRE_OBJECT_HASH_ALGORITHM}'"
        ));
    }
    if !matches!(payload.get("body"), Some(Value::Object(_))) {
        return Err("top-level 'body' must be an object".to_string());
    }
    Ok(())
}

fn validate_wire_snapshot_offer_payload(payload: &Map<String, Value>) -> Result<(), String> {
    reject_unknown_fields(
        payload,
        "top-level",
        &[
            "snapshot_id",
            "root_hash",
            "documents",
            "object_count",
            "size_bytes",
        ],
    )
    .map_err(|error| error.to_string())?;
    validate_prefixed_string(
        &required_wire_string(payload, "snapshot_id", "wire payload")?,
        "snapshot_id",
        "snap:",
    )
    .map_err(|error| error.to_string())?;
    validate_prefixed_string(
        &required_wire_string(payload, "root_hash", "wire payload")?,
        "root_hash",
        "hash:",
    )
    .map_err(|error| error.to_string())?;
    for (index, doc_id) in validate_wire_string_array(payload, "documents")?
        .iter()
        .enumerate()
    {
        validate_prefixed_string(doc_id, &format!("documents[{index}]"), "doc:")
            .map_err(|error| error.to_string())?;
    }
    optional_wire_u64(payload, "object_count", "wire payload")?;
    optional_wire_u64(payload, "size_bytes", "wire payload")?;
    Ok(())
}

fn validate_wire_view_announce_payload(payload: &Map<String, Value>) -> Result<(), String> {
    reject_unknown_fields(
        payload,
        "top-level",
        &["view_id", "maintainer", "documents"],
    )
    .map_err(|error| error.to_string())?;
    validate_prefixed_string(
        &required_wire_string(payload, "view_id", "wire payload")?,
        "view_id",
        "view:",
    )
    .map_err(|error| error.to_string())?;
    validate_prefixed_string(
        &required_wire_string(payload, "maintainer", "wire payload")?,
        "maintainer",
        "pk:",
    )
    .map_err(|error| error.to_string())?;
    let documents = required_prefixed_string_map(payload, "documents", "doc:", "rev:")
        .map_err(|error| error.to_string())?;
    if documents.is_empty() {
        return Err("top-level 'documents' must not be empty".to_string());
    }
    Ok(())
}

fn validate_wire_bye_payload(payload: &Map<String, Value>) -> Result<(), String> {
    reject_unknown_fields(payload, "top-level", &["reason"]).map_err(|error| error.to_string())?;
    required_wire_string(payload, "reason", "wire payload")?;
    Ok(())
}

fn validate_wire_error_payload(payload: &Map<String, Value>) -> Result<(), String> {
    reject_unknown_fields(payload, "top-level", &["in_reply_to", "code", "detail"])
        .map_err(|error| error.to_string())?;
    validate_prefixed_string(
        &required_wire_string(payload, "in_reply_to", "wire payload")?,
        "in_reply_to",
        "msg:",
    )
    .map_err(|error| error.to_string())?;
    required_wire_string(payload, "code", "wire payload")?;
    optional_wire_string(payload, "detail", "wire payload")?;
    Ok(())
}

fn validate_wire_node_capabilities_payload(payload: &Map<String, Value>) -> Result<(), String> {
    validate_prefixed_string(
        &required_wire_string(payload, "node_id", "wire payload")?,
        "node_id",
        "node:",
    )
    .map_err(|error| error.to_string())?;
    validate_wire_string_array(payload, "capabilities")?;
    if payload.contains_key("topics") {
        validate_wire_string_array(payload, "topics")?;
    }
    Ok(())
}

fn validate_wire_optional_canonical_object_array(
    payload: &Map<String, Value>,
    field: &str,
) -> Result<(), String> {
    if payload.contains_key(field) {
        validate_wire_required_canonical_object_array(payload, field)?;
    }
    Ok(())
}

fn validate_wire_required_canonical_object_array(
    payload: &Map<String, Value>,
    field: &str,
) -> Result<(), String> {
    for (index, object_id) in validate_wire_string_array(payload, field)?
        .iter()
        .enumerate()
    {
        validate_canonical_object_id(object_id, &format!("{field}[{index}]"))
            .map_err(|error| error.to_string())?;
    }
    Ok(())
}

pub fn validate_wire_object_payload_behavior(payload: &Map<String, Value>) -> Result<(), String> {
    let object_id = required_wire_string(payload, "object_id", "OBJECT payload")?;
    let object_type = required_wire_string(payload, "object_type", "OBJECT payload")?;
    let hash = required_wire_string(payload, "hash", "OBJECT payload")?;
    let body = payload
        .get("body")
        .ok_or_else(|| "missing object field 'body'".to_string())?;
    let expected_identity = derive_wire_object_payload_identity(body)?;

    if expected_identity.object_type != object_type {
        return Err(format!(
            "OBJECT body type '{}' does not match object_type '{}'",
            expected_identity.object_type, object_type
        ));
    }

    validate_wire_object_body_with_shared_verifier(body)?;

    if object_id != expected_identity.object_id {
        return Err(format!(
            "OBJECT payload object_id '{object_id}' does not match recomputed '{}'",
            expected_identity.object_id
        ));
    }
    if hash != expected_identity.hash {
        return Err(format!(
            "OBJECT payload hash '{hash}' does not match recomputed '{}'",
            expected_identity.hash
        ));
    }

    Ok(())
}

fn validate_wire_object_body_with_shared_verifier(body: &Value) -> Result<(), String> {
    let summary = verify_object_value(body);
    if summary.is_ok() {
        return Ok(());
    }

    let first_error = summary
        .errors
        .into_iter()
        .next()
        .unwrap_or_else(|| "shared object verification failed".to_string());
    Err(format!(
        "OBJECT body failed shared verification: {first_error}"
    ))
}

pub(crate) fn derive_wire_object_payload_identity(
    body: &Value,
) -> Result<WireObjectPayloadIdentity, String> {
    let body_envelope = parse_object_envelope(body).map_err(|error| error.to_string())?;
    let expected_identity = recompute_declared_object_identity(body)
        .map_err(|error| format!("failed to recompute OBJECT body ID: {error}"))?;

    Ok(WireObjectPayloadIdentity {
        object_type: body_envelope.object_type().to_string(),
        object_id: expected_identity.object_id,
        hash: expected_identity.hash,
    })
}

fn required_wire_string(
    object: &Map<String, Value>,
    field: &str,
    scope: &str,
) -> Result<String, String> {
    required_string_field(object, field)
        .map(str::to_owned)
        .map_err(|error| match error {
            StringFieldError::Missing => format!("{scope} is missing string field '{field}'"),
            StringFieldError::WrongType => format!("{scope} field '{field}' must be a string"),
        })
}

fn required_wire_bool(
    object: &Map<String, Value>,
    field: &str,
    scope: &str,
) -> Result<bool, String> {
    match object.get(field) {
        Some(Value::Bool(value)) => Ok(*value),
        Some(_) => Err(format!("{scope} field '{field}' must be a boolean")),
        None => Err(format!("{scope} is missing boolean field '{field}'")),
    }
}

fn optional_wire_string(
    object: &Map<String, Value>,
    field: &str,
    scope: &str,
) -> Result<Option<String>, String> {
    match object.get(field) {
        Some(Value::String(value)) => Ok(Some(value.clone())),
        Some(_) => Err(format!("{scope} field '{field}' must be a string")),
        None => Ok(None),
    }
}

fn optional_wire_u64(
    object: &Map<String, Value>,
    field: &str,
    scope: &str,
) -> Result<Option<u64>, String> {
    match object.get(field) {
        Some(Value::Number(value)) => value
            .as_u64()
            .map(Some)
            .ok_or_else(|| format!("{scope} field '{field}' must be a non-negative integer")),
        Some(_) => Err(format!(
            "{scope} field '{field}' must be a non-negative integer"
        )),
        None => Ok(None),
    }
}

fn require_advertised_wire_capability(
    peer_session: &WirePeerSessionState,
    capability: &str,
    message_type: WireMessageType,
    sender: &str,
) -> Result<(), String> {
    if peer_session.advertises_capability(capability) {
        return Ok(());
    }

    Err(format!(
        "wire {message_type} requires advertised capability '{capability}' from '{sender}'"
    ))
}

fn wire_head_map_to_sets(
    heads: BTreeMap<String, Vec<String>>,
) -> BTreeMap<String, BTreeSet<String>> {
    heads
        .into_iter()
        .map(|(doc_id, revisions)| (doc_id, revisions.into_iter().collect()))
        .collect()
}

fn extend_reachable_object_ids_from_object(
    payload: &Map<String, Value>,
    peer_session: &mut WirePeerSessionState,
) -> Result<(), String> {
    let object_id = required_wire_string(payload, "object_id", "OBJECT payload")?;
    if !peer_session.accepted_sync_roots.contains(&object_id)
        && !peer_session.reachable_object_ids.contains(&object_id)
    {
        return Ok(());
    }

    let body = payload
        .get("body")
        .ok_or_else(|| "missing object field 'body'".to_string())?;
    peer_session
        .reachable_object_ids
        .extend(discover_reachable_object_ids_from_value(body)?);

    Ok(())
}

fn expand_reachable_object_ids_from_known_index(
    peer_session: &mut WirePeerSessionState,
    object_index: &BTreeMap<String, Value>,
) -> Result<(), String> {
    let mut frontier = peer_session
        .accepted_sync_roots
        .iter()
        .chain(peer_session.reachable_object_ids.iter())
        .cloned()
        .collect::<Vec<_>>();
    let mut visited = BTreeSet::new();

    while let Some(object_id) = frontier.pop() {
        if !visited.insert(object_id.clone()) {
            continue;
        }
        let Some(value) = object_index.get(&object_id) else {
            continue;
        };
        for discovered_id in discover_reachable_object_ids_from_value(value)? {
            if peer_session
                .reachable_object_ids
                .insert(discovered_id.clone())
            {
                frontier.push(discovered_id);
            }
        }
    }

    Ok(())
}

pub(crate) fn discover_reachable_object_ids_from_value(
    value: &Value,
) -> Result<BTreeSet<String>, String> {
    let object_type = parse_object_envelope(value)
        .map_err(|error| format!("failed to parse reachable object envelope: {error}"))?
        .object_type()
        .to_string();
    let mut reachable = BTreeSet::new();

    match object_type.as_str() {
        "patch" => {
            let patch = parse_patch_object(value)
                .map_err(|error| format!("failed to parse reachable patch object: {error}"))?;
            reachable.insert(patch.base_revision);
        }
        "revision" => {
            let revision = parse_revision_object(value)
                .map_err(|error| format!("failed to parse reachable revision object: {error}"))?;
            reachable.extend(revision.parents);
            reachable.extend(revision.patches);
        }
        "view" => {
            let view = parse_view_object(value)
                .map_err(|error| format!("failed to parse reachable view object: {error}"))?;
            reachable.extend(view.documents.into_values());
        }
        "snapshot" => {
            let snapshot = parse_snapshot_object(value)
                .map_err(|error| format!("failed to parse reachable snapshot object: {error}"))?;
            reachable.extend(snapshot.documents.into_values());
            reachable.extend(snapshot.included_objects);
        }
        _ => {}
    }

    Ok(reachable)
}

fn validate_wire_timestamp(timestamp: &str) -> Result<(), String> {
    let (date, time_with_offset) = timestamp
        .split_once('T')
        .ok_or_else(|| "wire envelope 'timestamp' must use RFC 3339 format".to_string())?;
    let date_parts = date.split('-').collect::<Vec<_>>();
    if date_parts.len() != 3
        || date_parts[0].len() != 4
        || date_parts[1].len() != 2
        || date_parts[2].len() != 2
        || !date_parts
            .iter()
            .all(|part| part.chars().all(|ch| ch.is_ascii_digit()))
    {
        return Err("wire envelope 'timestamp' must use RFC 3339 format".to_string());
    }

    let (time, offset) = if let Some(index) = time_with_offset.find(['+', '-']) {
        (&time_with_offset[..index], &time_with_offset[index..])
    } else if let Some(time) = time_with_offset.strip_suffix('Z') {
        (time, "Z")
    } else {
        return Err("wire envelope 'timestamp' must use RFC 3339 format".to_string());
    };

    let time_parts = time.split(':').collect::<Vec<_>>();
    if time_parts.len() != 3
        || !time_parts.iter().all(|part| part.len() == 2)
        || !time_parts
            .iter()
            .all(|part| part.chars().all(|ch| ch.is_ascii_digit()))
    {
        return Err("wire envelope 'timestamp' must use RFC 3339 format".to_string());
    }

    if offset != "Z" {
        let offset_parts = offset[1..].split(':').collect::<Vec<_>>();
        if offset.len() != 6
            || offset_parts.len() != 2
            || !offset_parts.iter().all(|part| part.len() == 2)
            || !offset_parts
                .iter()
                .all(|part| part.chars().all(|ch| ch.is_ascii_digit()))
        {
            return Err("wire envelope 'timestamp' must use RFC 3339 format".to_string());
        }
    }

    Ok(())
}

fn validate_wire_string_array(
    payload: &Map<String, Value>,
    field: &str,
) -> Result<Vec<String>, String> {
    required_non_empty_string_array(payload, field).map_err(|error| error.to_string())
}

fn validate_wire_head_map(
    payload: &Map<String, Value>,
    field: &str,
) -> Result<BTreeMap<String, Vec<String>>, String> {
    let entries = match payload.get(field) {
        Some(Value::Object(entries)) => entries,
        Some(_) => return Err(format!("top-level '{field}' must be an object")),
        None => return Err(format!("missing object field '{field}'")),
    };
    if entries.is_empty() {
        return Err(format!("top-level '{field}' must not be empty"));
    }

    let mut heads = BTreeMap::new();
    for (doc_id, revision_ids) in entries {
        validate_prefixed_string(doc_id, &format!("{field}.{doc_id} key"), "doc:")
            .map_err(|error| error.to_string())?;
        let revisions = match revision_ids {
            Value::Array(values) => {
                if values.is_empty() {
                    return Err(format!("top-level '{field}.{doc_id}' must not be empty"));
                }
                let revisions = values
                    .iter()
                    .enumerate()
                    .map(|(index, value)| match value {
                        Value::String(value) => Ok(value.clone()),
                        _ => Err(format!(
                            "top-level '{field}.{doc_id}[{index}]' must be a string"
                        )),
                    })
                    .collect::<Result<Vec<_>, _>>()?;
                for (index, revision_id) in revisions.iter().enumerate() {
                    validate_prefixed_string(
                        revision_id,
                        &format!("{field}.{doc_id}[{index}]"),
                        "rev:",
                    )
                    .map_err(|error| error.to_string())?;
                }
                reject_duplicate_strings(&revisions, &format!("{field}.{doc_id}"))
                    .map_err(|error| error.to_string())?;
                revisions
            }
            _ => return Err(format!("top-level '{field}.{doc_id}' must be an array")),
        };
        heads.insert(doc_id.clone(), revisions);
    }

    Ok(heads)
}

#[cfg(test)]
mod tests {
    use std::fs;
    use std::path::PathBuf;

    use base64::Engine;
    use ed25519_dalek::{Signer, SigningKey};
    use serde_json::{json, Value};

    use crate::canonical::{signed_payload_bytes, wire_envelope_signed_payload_bytes};
    use crate::protocol::{recompute_declared_object_identity, recompute_object_id};
    use crate::replay::{compute_state_hash, DocumentState};
    use crate::store::write_object_value_to_store;

    use super::{
        derive_wire_object_payload_identity, parse_wire_envelope, validate_wire_envelope,
        validate_wire_object_payload_behavior, validate_wire_payload,
        verify_wire_envelope_signature, WireMessageType, WirePeerDirectory, WireSession,
    };

    fn signing_key() -> SigningKey {
        SigningKey::from_bytes(&[9u8; 32])
    }

    fn temp_dir(prefix: &str) -> PathBuf {
        let unique = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .expect("system time should be after unix epoch")
            .as_nanos();
        let path = std::env::temp_dir().join(format!("mycel-wire-{prefix}-{unique}"));
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

    fn signed_hello_message(
        signing_key: &SigningKey,
        sender: &str,
        payload_node_id: &str,
    ) -> Value {
        signed_hello_message_with_capabilities(
            signing_key,
            sender,
            payload_node_id,
            json!(["patch-sync"]),
        )
    }

    fn signed_hello_message_with_capabilities(
        signing_key: &SigningKey,
        sender: &str,
        payload_node_id: &str,
        capabilities: Value,
    ) -> Value {
        let mut value = json!({
            "type": "HELLO",
            "version": "mycel-wire/0.1",
            "msg_id": "msg:hello-signed-001",
            "timestamp": "2026-03-08T20:00:00+08:00",
            "from": sender,
            "payload": {
                "node_id": payload_node_id,
                "capabilities": capabilities,
                "nonce": "n:test"
            },
            "sig": "sig:placeholder"
        });
        value["sig"] = Value::String(sign_wire_value(signing_key, &value));
        value
    }

    fn signed_manifest_message(
        signing_key: &SigningKey,
        sender: &str,
        payload_node_id: &str,
    ) -> Value {
        signed_manifest_message_with_capabilities(
            signing_key,
            sender,
            payload_node_id,
            json!(["patch-sync"]),
        )
    }

    fn signed_manifest_message_with_capabilities(
        signing_key: &SigningKey,
        sender: &str,
        payload_node_id: &str,
        capabilities: Value,
    ) -> Value {
        let mut value = json!({
            "type": "MANIFEST",
            "version": "mycel-wire/0.1",
            "msg_id": "msg:manifest-signed-001",
            "timestamp": "2026-03-08T20:00:10+08:00",
            "from": sender,
            "payload": {
                "node_id": payload_node_id,
                "capabilities": capabilities,
                "heads": {
                    "doc:test": ["rev:test"]
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
            "msg_id": "msg:snapshot-offer-signed-001",
            "timestamp": "2026-03-08T20:00:40+08:00",
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
            "msg_id": "msg:view-announce-signed-001",
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

    fn signed_manifest_message_with_heads(
        signing_key: &SigningKey,
        sender: &str,
        payload_node_id: &str,
        heads: Value,
    ) -> Value {
        let mut value = json!({
            "type": "MANIFEST",
            "version": "mycel-wire/0.1",
            "msg_id": "msg:manifest-signed-001",
            "timestamp": "2026-03-08T20:00:10+08:00",
            "from": sender,
            "payload": {
                "node_id": payload_node_id,
                "capabilities": ["patch-sync"],
                "heads": heads
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
            "msg_id": "msg:want-signed-001",
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

    fn signed_heads_message(
        signing_key: &SigningKey,
        sender: &str,
        documents: Value,
        replace: bool,
    ) -> Value {
        let mut value = json!({
            "type": "HEADS",
            "version": "mycel-wire/0.1",
            "msg_id": "msg:heads-signed-001",
            "timestamp": "2026-03-08T20:00:30+08:00",
            "from": sender,
            "payload": {
                "documents": documents,
                "replace": replace
            },
            "sig": "sig:placeholder"
        });
        value["sig"] = Value::String(sign_wire_value(signing_key, &value));
        value
    }

    fn signed_object_message(signing_key: &SigningKey, sender: &str) -> Value {
        signed_patch_object_message(signing_key, sender, "rev:genesis-null")
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
            "msg_id": "msg:object-signed-001",
            "timestamp": "2026-03-08T20:01:02+08:00",
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
                "timestamp": 1u64,
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
            "msg_id": "msg:revision-object-signed-001",
            "timestamp": "2026-03-08T20:01:02+08:00",
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

    fn signed_error_message(signing_key: &SigningKey, sender: &str, in_reply_to: &str) -> Value {
        let mut value = json!({
            "type": "ERROR",
            "version": "mycel-wire/0.1",
            "msg_id": "msg:error-signed-001",
            "timestamp": "2026-03-08T20:02:00+08:00",
            "from": sender,
            "payload": {
                "in_reply_to": in_reply_to,
                "code": "ERR_UNKNOWN",
                "detail": "test error"
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
            "msg_id": "msg:bye-signed-001",
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
    fn parse_wire_envelope_accepts_minimal_hello() {
        let value = json!({
            "type": "HELLO",
            "version": "mycel-wire/0.1",
            "msg_id": "msg:hello-001",
            "timestamp": "2026-03-08T20:00:00+08:00",
            "from": "node:alpha",
            "payload": {
                "node_id": "node:alpha",
                "capabilities": ["patch-sync"],
                "nonce": "n:test"
            },
            "sig": "sig:placeholder"
        });

        let envelope = parse_wire_envelope(&value).expect("wire envelope should parse");

        assert_eq!(envelope.message_type(), WireMessageType::Hello);
        assert_eq!(envelope.from(), "node:alpha");
        assert_eq!(
            envelope.payload().get("node_id").and_then(Value::as_str),
            Some("node:alpha")
        );
    }

    #[test]
    fn parse_wire_envelope_rejects_wrong_version() {
        let value = json!({
            "type": "HELLO",
            "version": "mycel-wire/9.9",
            "msg_id": "msg:hello-001",
            "timestamp": "2026-03-08T20:00:00+08:00",
            "from": "node:alpha",
            "payload": {
                "node_id": "node:alpha",
                "capabilities": ["patch-sync"],
                "nonce": "n:test"
            },
            "sig": "sig:placeholder"
        });

        let error = parse_wire_envelope(&value).unwrap_err();

        assert_eq!(error, "wire envelope 'version' must equal 'mycel-wire/0.1'");
    }

    #[test]
    fn validate_wire_payload_rejects_object_body_type_mismatch() {
        let payload = json!({
            "object_id": "patch:test",
            "object_type": "patch",
            "encoding": "json",
            "hash_alg": "sha256",
            "hash": "hash:test",
            "body": {
                "type": "revision",
                "version": "mycel/0.1",
                "revision_id": "rev:test",
                "doc_id": "doc:test",
                "parents": [],
                "patches": [],
                "state_hash": "hash:test",
                "author": "pk:ed25519:test",
                "timestamp": 1u64
            }
        });

        validate_wire_payload(
            WireMessageType::Object,
            payload.as_object().expect("payload should be object"),
        )
        .expect("OBJECT payload shape should validate before behavior checks");
        let error = validate_wire_object_payload_behavior(
            payload.as_object().expect("payload should be object"),
        )
        .unwrap_err();

        assert!(error.contains("OBJECT body type 'revision' does not match object_type 'patch'"));
    }

    #[test]
    fn validate_wire_payload_rejects_non_sha256_object_hash_algorithm() {
        let payload = json!({
            "object_id": "patch:test",
            "object_type": "patch",
            "encoding": "json",
            "hash_alg": "blake3",
            "hash": "hash:test",
            "body": {
                "type": "patch",
                "version": "mycel/0.1",
                "patch_id": "patch:test",
                "doc_id": "doc:test",
                "base_revision": "rev:genesis-null",
                "author": "pk:ed25519:test",
                "timestamp": 1u64,
                "ops": []
            }
        });

        let error = validate_wire_payload(
            WireMessageType::Object,
            payload.as_object().expect("payload should be object"),
        )
        .unwrap_err();

        assert_eq!(error, "OBJECT payload 'hash_alg' must equal 'sha256'");
    }

    #[test]
    fn validate_wire_payload_rejects_unknown_hello_payload_field() {
        let payload = json!({
            "node_id": "node:alpha",
            "capabilities": ["patch-sync"],
            "nonce": "n:test",
            "unexpected": true
        });

        let error = validate_wire_payload(
            WireMessageType::Hello,
            payload.as_object().expect("payload should be object"),
        )
        .unwrap_err();

        assert_eq!(error, "top-level contains unexpected field 'unexpected'");
    }

    #[test]
    fn validate_wire_payload_rejects_non_array_hello_topics() {
        let payload = json!({
            "node_id": "node:alpha",
            "capabilities": ["patch-sync"],
            "topics": "text/core",
            "nonce": "n:test"
        });

        let error = validate_wire_payload(
            WireMessageType::Hello,
            payload.as_object().expect("payload should be object"),
        )
        .unwrap_err();

        assert_eq!(error, "top-level 'topics' must be an array");
    }

    #[test]
    fn validate_wire_payload_rejects_negative_snapshot_offer_size_bytes() {
        let payload = json!({
            "snapshot_id": "snap:test",
            "root_hash": "hash:test",
            "documents": ["doc:test"],
            "size_bytes": -1
        });

        let error = validate_wire_payload(
            WireMessageType::SnapshotOffer,
            payload.as_object().expect("payload should be object"),
        )
        .unwrap_err();

        assert_eq!(
            error,
            "wire payload field 'size_bytes' must be a non-negative integer"
        );
    }

    #[test]
    fn validate_wire_payload_rejects_unknown_snapshot_offer_payload_field() {
        let payload = json!({
            "snapshot_id": "snap:test",
            "root_hash": "hash:test",
            "documents": ["doc:test"],
            "unknown_count": 7u64
        });

        let error = validate_wire_payload(
            WireMessageType::SnapshotOffer,
            payload.as_object().expect("payload should be object"),
        )
        .unwrap_err();

        assert_eq!(error, "top-level contains unexpected field 'unknown_count'");
    }

    #[test]
    fn validate_wire_payload_rejects_non_string_error_detail() {
        let payload = json!({
            "in_reply_to": "msg:test",
            "code": "INVALID_HASH",
            "detail": 7
        });

        let error = validate_wire_payload(
            WireMessageType::Error,
            payload.as_object().expect("payload should be object"),
        )
        .unwrap_err();

        assert_eq!(error, "wire payload field 'detail' must be a string");
    }

    #[test]
    fn derive_wire_object_payload_identity_matches_signed_patch_body() {
        let signing_key = signing_key();
        let body = sign_object_value(
            &signing_key,
            json!({
                "type": "patch",
                "version": "mycel/0.1",
                "patch_id": "patch:placeholder",
                "doc_id": "doc:test",
                "base_revision": "rev:genesis-null",
                "author": "pk:ed25519:placeholder",
                "timestamp": 1u64,
                "ops": [],
                "signature": "sig:placeholder"
            }),
            "author",
            "patch_id",
            "patch",
        );

        let identity = derive_wire_object_payload_identity(&body)
            .expect("wire object payload identity should derive");

        assert_eq!(identity.object_type, "patch");
        assert_eq!(
            identity.object_id,
            body["patch_id"]
                .as_str()
                .expect("signed patch body should include patch_id")
        );
        assert_eq!(
            identity.hash,
            format!(
                "hash:{}",
                identity
                    .object_id
                    .split_once(':')
                    .map(|(_, digest)| digest)
                    .expect("object ID should include digest")
            )
        );
    }

    #[test]
    fn validate_wire_object_payload_behavior_accepts_payload_built_from_shared_identity_helper() {
        let signing_key = signing_key();
        let body = sign_object_value(
            &signing_key,
            json!({
                "type": "revision",
                "version": "mycel/0.1",
                "revision_id": "rev:placeholder",
                "doc_id": "doc:test",
                "parents": [],
                "patches": [],
                "state_hash": empty_state_hash("doc:test"),
                "author": "pk:ed25519:placeholder",
                "timestamp": 1u64,
                "signature": "sig:placeholder"
            }),
            "author",
            "revision_id",
            "rev",
        );
        let identity = derive_wire_object_payload_identity(&body)
            .expect("wire object payload identity should derive");
        let payload = json!({
            "object_id": identity.object_id,
            "object_type": identity.object_type,
            "encoding": "json",
            "hash_alg": "sha256",
            "hash": identity.hash,
            "body": body
        });

        validate_wire_payload(
            WireMessageType::Object,
            payload.as_object().expect("payload should be object"),
        )
        .expect("OBJECT payload shape should validate");
        validate_wire_object_payload_behavior(
            payload.as_object().expect("payload should be object"),
        )
        .expect("OBJECT payload should match shared identity helper output");
    }

    #[test]
    fn validate_wire_object_payload_behavior_rejects_missing_required_body_signature() {
        let body = json!({
            "type": "patch",
            "version": "mycel/0.1",
            "patch_id": "patch:8d13c0b560f101a83ed57f4ab84f5a39a214ba53cc4bfe4f4f6de643eb447c0a",
            "doc_id": "doc:test",
            "base_revision": "rev:genesis-null",
            "author": "pk:ed25519:test",
            "timestamp": 1u64,
            "ops": []
        });
        let payload = json!({
            "object_id": body["patch_id"],
            "object_type": "patch",
            "encoding": "json",
            "hash_alg": "sha256",
            "hash": "hash:8d13c0b560f101a83ed57f4ab84f5a39a214ba53cc4bfe4f4f6de643eb447c0a",
            "body": body
        });

        let error = validate_wire_object_payload_behavior(
            payload.as_object().expect("payload should be object"),
        )
        .unwrap_err();

        assert!(
            error.contains("OBJECT body failed shared verification"),
            "expected shared verification prefix, got {error}"
        );
        assert!(
            error.contains("patch object is missing required top-level 'signature'"),
            "expected missing signature error, got {error}"
        );
    }

    #[test]
    fn validate_wire_object_payload_behavior_rejects_shared_semantic_edge_failure() {
        let signing_key = signing_key();
        let body = sign_object_value(
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
                "timestamp": 12u64,
                "signature": "sig:placeholder"
            }),
            "maintainer",
            "view_id",
            "view",
        );
        let identity = derive_wire_object_payload_identity(&body)
            .expect("wire object payload identity should derive");
        let payload = json!({
            "object_id": identity.object_id,
            "object_type": identity.object_type,
            "encoding": "json",
            "hash_alg": "sha256",
            "hash": identity.hash,
            "body": body
        });

        let error = validate_wire_object_payload_behavior(
            payload.as_object().expect("payload should be object"),
        )
        .unwrap_err();

        assert!(
            error.contains("OBJECT body failed shared verification"),
            "expected shared verification prefix, got {error}"
        );
        assert!(
            error.contains("top-level 'policy.accept_keys[0]' must not be an empty string"),
            "expected view semantic-edge error, got {error}"
        );
    }

    #[test]
    fn validate_wire_envelope_accepts_concrete_object_payload() {
        let signing_key = signing_key();
        let body = sign_object_value(
            &signing_key,
            json!({
                "type": "patch",
                "version": "mycel/0.1",
                "patch_id": "patch:placeholder",
                "doc_id": "doc:test",
                "base_revision": "rev:genesis-null",
                "author": "pk:ed25519:placeholder",
                "timestamp": 1u64,
                "ops": [],
                "signature": "sig:placeholder"
            }),
            "author",
            "patch_id",
            "patch",
        );
        let identity = recompute_declared_object_identity(&body)
            .expect("concrete wire object identity should recompute");

        let value = json!({
            "type": "OBJECT",
            "version": "mycel-wire/0.1",
            "msg_id": "msg:obj-concrete-001",
            "timestamp": "2026-03-08T20:01:02+08:00",
            "from": "node:alpha",
            "payload": {
                "object_id": identity.object_id,
                "object_type": "patch",
                "encoding": "json",
                "hash_alg": "sha256",
                "hash": identity.hash,
                "body": body
            },
            "sig": "sig:..."
        });

        let envelope = validate_wire_envelope(&value).expect("wire envelope should validate");
        validate_wire_object_payload_behavior(envelope.payload())
            .expect("concrete OBJECT payload should match recomputed ID and hash");
    }

    #[test]
    fn verify_wire_envelope_signature_accepts_valid_signed_hello() {
        let signing_key = signing_key();
        let sender_key = sender_public_key(&signing_key);
        let value = signed_hello_message(&signing_key, "node:alpha", "node:alpha");

        let envelope = verify_wire_envelope_signature(&value, &sender_key)
            .expect("wire signature should verify");

        assert_eq!(envelope.message_type(), WireMessageType::Hello);
    }

    #[test]
    fn verify_wire_envelope_signature_rejects_invalid_signature() {
        let signing_key = signing_key();
        let sender_key = sender_public_key(&signing_key);
        let value = json!({
            "type": "HELLO",
            "version": "mycel-wire/0.1",
            "msg_id": "msg:hello-signed-001",
            "timestamp": "2026-03-08T20:00:00+08:00",
            "from": "node:alpha",
            "payload": {
                "node_id": "node:alpha",
                "capabilities": ["patch-sync"],
                "nonce": "n:test"
            },
            "sig": "sig:ed25519:AAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAA=="
        });

        let error = verify_wire_envelope_signature(&value, &sender_key).unwrap_err();

        assert!(error.contains("Ed25519 signature verification failed"));
    }

    #[test]
    fn verify_wire_envelope_signature_rejects_malformed_sender_public_key() {
        let signing_key = signing_key();
        let value = signed_hello_message(&signing_key, "node:alpha", "node:alpha");

        let error = verify_wire_envelope_signature(&value, "node:alpha").unwrap_err();

        assert_eq!(
            error,
            "sender public key must use format 'pk:ed25519:<base64>'"
        );
    }

    #[test]
    fn wire_session_verifies_incoming_hello_from_registered_peer() {
        let signing_key = signing_key();
        let sender_key = sender_public_key(&signing_key);
        let mut session = WireSession::default();
        session
            .register_known_peer("node:alpha", &sender_key)
            .expect("known peer should register");
        let value = signed_hello_message(&signing_key, "node:alpha", "node:alpha");

        let envelope = session
            .verify_incoming(&value)
            .expect("registered sender should verify");

        assert_eq!(envelope.from(), "node:alpha");
        assert_eq!(envelope.message_type(), WireMessageType::Hello);
    }

    #[test]
    fn wire_session_rejects_unknown_sender() {
        let signing_key = signing_key();
        let value = signed_hello_message(&signing_key, "node:alpha", "node:alpha");

        let error = WireSession::default().verify_incoming(&value).unwrap_err();

        assert_eq!(error, "unknown wire sender 'node:alpha'");
    }

    #[test]
    fn wire_session_rejects_hello_node_id_mismatch() {
        let signing_key = signing_key();
        let sender_key = sender_public_key(&signing_key);
        let mut session = WireSession::new(WirePeerDirectory::new());
        session
            .register_known_peer("node:alpha", &sender_key)
            .expect("known peer should register");
        let value = signed_hello_message(&signing_key, "node:alpha", "node:beta");

        let error = session.verify_incoming(&value).unwrap_err();

        assert_eq!(
            error,
            "wire HELLO payload 'node_id' must equal envelope 'from'"
        );
    }

    #[test]
    fn wire_session_rejects_manifest_before_hello() {
        let signing_key = signing_key();
        let sender_key = sender_public_key(&signing_key);
        let mut session = WireSession::default();
        session
            .register_known_peer("node:alpha", &sender_key)
            .expect("known peer should register");
        let value = signed_manifest_message(&signing_key, "node:alpha", "node:alpha");

        let error = session.verify_incoming(&value).unwrap_err();

        assert_eq!(
            error,
            "wire MANIFEST requires prior HELLO from 'node:alpha'"
        );
    }

    #[test]
    fn wire_session_records_manifest_heads() {
        let signing_key = signing_key();
        let sender_key = sender_public_key(&signing_key);
        let mut session = WireSession::default();
        session
            .register_known_peer("node:alpha", &sender_key)
            .expect("known peer should register");
        let hello = signed_hello_message(&signing_key, "node:alpha", "node:alpha");
        let manifest = signed_manifest_message(&signing_key, "node:alpha", "node:alpha");

        session
            .verify_incoming(&hello)
            .expect("HELLO should verify");
        session
            .verify_incoming(&manifest)
            .expect("MANIFEST should verify");

        let state = session
            .peer_session("node:alpha")
            .expect("peer session should exist");
        assert_eq!(
            state
                .advertised_document_heads
                .get("doc:test")
                .map(|revisions| revisions.len()),
            Some(1)
        );
        assert!(state
            .advertised_document_heads
            .get("doc:test")
            .is_some_and(|revisions| revisions.contains("rev:test")));
    }

    #[test]
    fn wire_session_merges_incremental_heads_updates() {
        let signing_key = signing_key();
        let sender_key = sender_public_key(&signing_key);
        let mut session = WireSession::default();
        session
            .register_known_peer("node:alpha", &sender_key)
            .expect("known peer should register");
        let hello = signed_hello_message(&signing_key, "node:alpha", "node:alpha");
        let manifest = signed_manifest_message(&signing_key, "node:alpha", "node:alpha");
        let heads = signed_heads_message(
            &signing_key,
            "node:alpha",
            json!({
                "doc:test": ["rev:next"],
                "doc:extra": ["rev:extra"]
            }),
            false,
        );

        session
            .verify_incoming(&hello)
            .expect("HELLO should verify");
        session
            .verify_incoming(&manifest)
            .expect("MANIFEST should verify");
        session
            .verify_incoming(&heads)
            .expect("HEADS should verify");

        let state = session
            .peer_session("node:alpha")
            .expect("peer session should exist");
        assert!(state
            .advertised_document_heads
            .get("doc:test")
            .is_some_and(|revisions| {
                revisions.contains("rev:test") && revisions.contains("rev:next")
            }));
        assert!(state
            .advertised_document_heads
            .get("doc:extra")
            .is_some_and(|revisions| revisions.contains("rev:extra")));
    }

    #[test]
    fn wire_session_replaces_heads_when_replace_is_true() {
        let signing_key = signing_key();
        let sender_key = sender_public_key(&signing_key);
        let mut session = WireSession::default();
        session
            .register_known_peer("node:alpha", &sender_key)
            .expect("known peer should register");
        let hello = signed_hello_message(&signing_key, "node:alpha", "node:alpha");
        let manifest = signed_manifest_message(&signing_key, "node:alpha", "node:alpha");
        let heads = signed_heads_message(
            &signing_key,
            "node:alpha",
            json!({
                "doc:replacement": ["rev:replacement"]
            }),
            true,
        );

        session
            .verify_incoming(&hello)
            .expect("HELLO should verify");
        session
            .verify_incoming(&manifest)
            .expect("MANIFEST should verify");
        session
            .verify_incoming(&heads)
            .expect("HEADS should verify");

        let state = session
            .peer_session("node:alpha")
            .expect("peer session should exist");
        assert!(!state.advertised_document_heads.contains_key("doc:test"));
        assert!(state
            .advertised_document_heads
            .get("doc:replacement")
            .is_some_and(|revisions| revisions.contains("rev:replacement")));
    }

    #[test]
    fn wire_session_rejects_snapshot_offer_without_snapshot_capability() {
        let signing_key = signing_key();
        let sender_key = sender_public_key(&signing_key);
        let mut session = WireSession::default();
        session
            .register_known_peer("node:alpha", &sender_key)
            .expect("known peer should register");
        let hello = signed_hello_message(&signing_key, "node:alpha", "node:alpha");
        let snapshot_offer =
            signed_snapshot_offer_message(&signing_key, "node:alpha", "snap:test-offer");

        session
            .verify_incoming(&hello)
            .expect("HELLO should verify");
        let error = session.verify_incoming(&snapshot_offer).unwrap_err();

        assert_eq!(
            error,
            "wire SNAPSHOT_OFFER requires advertised capability 'snapshot-sync' from 'node:alpha'"
        );
    }

    #[test]
    fn wire_session_accepts_snapshot_offer_with_snapshot_capability_and_unlocks_want() {
        let signing_key = signing_key();
        let sender_key = sender_public_key(&signing_key);
        let mut session = WireSession::default();
        session
            .register_known_peer("node:alpha", &sender_key)
            .expect("known peer should register");
        let hello = signed_hello_message_with_capabilities(
            &signing_key,
            "node:alpha",
            "node:alpha",
            json!(["patch-sync", "snapshot-sync"]),
        );
        let manifest = signed_manifest_message_with_capabilities(
            &signing_key,
            "node:alpha",
            "node:alpha",
            json!(["patch-sync", "snapshot-sync"]),
        );
        let snapshot_offer =
            signed_snapshot_offer_message(&signing_key, "node:alpha", "snap:test-offer");
        let want = signed_want_message(&signing_key, "node:alpha", &["snap:test-offer"]);

        session
            .verify_incoming(&hello)
            .expect("HELLO should verify");
        session
            .verify_incoming(&manifest)
            .expect("MANIFEST should verify");
        session
            .verify_incoming(&snapshot_offer)
            .expect("SNAPSHOT_OFFER should verify");
        session
            .verify_incoming(&want)
            .expect("snapshot WANT should verify after offer");

        let state = session
            .peer_session("node:alpha")
            .expect("peer session should exist");
        assert!(state.reachable_object_ids.contains("snap:test-offer"));
        assert!(state.pending_object_ids.contains("snap:test-offer"));
    }

    #[test]
    fn wire_session_rejects_view_announce_without_view_capability() {
        let signing_key = signing_key();
        let sender_key = sender_public_key(&signing_key);
        let mut session = WireSession::default();
        session
            .register_known_peer("node:alpha", &sender_key)
            .expect("known peer should register");
        let hello = signed_hello_message(&signing_key, "node:alpha", "node:alpha");
        let view_announce =
            signed_view_announce_message(&signing_key, "node:alpha", "view:test-announce");

        session
            .verify_incoming(&hello)
            .expect("HELLO should verify");
        let error = session.verify_incoming(&view_announce).unwrap_err();

        assert_eq!(
            error,
            "wire VIEW_ANNOUNCE requires advertised capability 'view-sync' from 'node:alpha'"
        );
    }

    #[test]
    fn wire_session_accepts_view_announce_with_view_capability_and_unlocks_want() {
        let signing_key = signing_key();
        let sender_key = sender_public_key(&signing_key);
        let mut session = WireSession::default();
        session
            .register_known_peer("node:alpha", &sender_key)
            .expect("known peer should register");
        let hello = signed_hello_message_with_capabilities(
            &signing_key,
            "node:alpha",
            "node:alpha",
            json!(["patch-sync", "view-sync"]),
        );
        let manifest = signed_manifest_message_with_capabilities(
            &signing_key,
            "node:alpha",
            "node:alpha",
            json!(["patch-sync", "view-sync"]),
        );
        let view_announce =
            signed_view_announce_message(&signing_key, "node:alpha", "view:test-announce");
        let want = signed_want_message(&signing_key, "node:alpha", &["view:test-announce"]);

        session
            .verify_incoming(&hello)
            .expect("HELLO should verify");
        session
            .verify_incoming(&manifest)
            .expect("MANIFEST should verify");
        session
            .verify_incoming(&view_announce)
            .expect("VIEW_ANNOUNCE should verify");
        session
            .verify_incoming(&want)
            .expect("view WANT should verify after announcement");

        let state = session
            .peer_session("node:alpha")
            .expect("peer session should exist");
        assert!(state.reachable_object_ids.contains("view:test-announce"));
        assert!(state.pending_object_ids.contains("view:test-announce"));
    }

    #[test]
    fn wire_session_rejects_want_before_head_context() {
        let signing_key = signing_key();
        let sender_key = sender_public_key(&signing_key);
        let mut session = WireSession::default();
        session
            .register_known_peer("node:alpha", &sender_key)
            .expect("known peer should register");
        let hello = signed_hello_message(&signing_key, "node:alpha", "node:alpha");
        let want = signed_want_message(&signing_key, "node:alpha", &["patch:test"]);

        session
            .verify_incoming(&hello)
            .expect("HELLO should verify");
        let error = session.verify_incoming(&want).unwrap_err();

        assert_eq!(
            error,
            "wire WANT requires prior MANIFEST or HEADS from 'node:alpha'"
        );
    }

    #[test]
    fn wire_session_rejects_unadvertised_revision_want() {
        let signing_key = signing_key();
        let sender_key = sender_public_key(&signing_key);
        let mut session = WireSession::default();
        session
            .register_known_peer("node:alpha", &sender_key)
            .expect("known peer should register");
        let hello = signed_hello_message(&signing_key, "node:alpha", "node:alpha");
        let manifest = signed_manifest_message(&signing_key, "node:alpha", "node:alpha");
        let want = signed_want_message(&signing_key, "node:alpha", &["rev:missing"]);

        session
            .verify_incoming(&hello)
            .expect("HELLO should verify");
        session
            .verify_incoming(&manifest)
            .expect("MANIFEST should verify");
        let error = session.verify_incoming(&want).unwrap_err();

        assert_eq!(
            error,
            "wire WANT revision 'rev:missing' is not reachable from accepted sync roots for 'node:alpha'"
        );
    }

    #[test]
    fn wire_session_rejects_non_revision_want_without_sync_root() {
        let signing_key = signing_key();
        let sender_key = sender_public_key(&signing_key);
        let mut session = WireSession::default();
        session
            .register_known_peer("node:alpha", &sender_key)
            .expect("known peer should register");
        let hello = signed_hello_message(&signing_key, "node:alpha", "node:alpha");
        let manifest = signed_manifest_message(&signing_key, "node:alpha", "node:alpha");
        let want = signed_want_message(&signing_key, "node:alpha", &["patch:test"]);

        session
            .verify_incoming(&hello)
            .expect("HELLO should verify");
        session
            .verify_incoming(&manifest)
            .expect("MANIFEST should verify");
        let error = session.verify_incoming(&want).unwrap_err();

        assert_eq!(
            error,
            "wire WANT object 'patch:test' is not reachable from accepted sync roots for 'node:alpha'"
        );
    }

    #[test]
    fn wire_session_rejects_follow_on_object_before_root_object_arrives() {
        let signing_key = signing_key();
        let sender_key = sender_public_key(&signing_key);
        let mut session = WireSession::default();
        session
            .register_known_peer("node:alpha", &sender_key)
            .expect("known peer should register");
        let patch_object =
            signed_patch_object_message(&signing_key, "node:alpha", "rev:genesis-null");
        let patch_id = patch_object["payload"]["object_id"]
            .as_str()
            .expect("signed patch OBJECT should include object_id")
            .to_owned();
        let revision_object =
            signed_revision_object_message(&signing_key, "node:alpha", &[], &[patch_id.as_str()]);
        let revision_id = revision_object["payload"]["object_id"]
            .as_str()
            .expect("signed revision OBJECT should include object_id")
            .to_owned();
        let hello = signed_hello_message(&signing_key, "node:alpha", "node:alpha");
        let manifest = signed_manifest_message_with_heads(
            &signing_key,
            "node:alpha",
            "node:alpha",
            json!({
                "doc:test": [revision_id.clone()]
            }),
        );
        let want = signed_want_message(
            &signing_key,
            "node:alpha",
            &[revision_id.as_str(), patch_id.as_str()],
        );

        session
            .verify_incoming(&hello)
            .expect("HELLO should verify");
        session
            .verify_incoming(&manifest)
            .expect("MANIFEST should verify");
        let error = session.verify_incoming(&want).unwrap_err();

        assert_eq!(
            error,
            format!(
                "wire WANT object '{}' is not reachable from accepted sync roots for 'node:alpha'",
                patch_id
            )
        );
    }

    #[test]
    fn wire_session_accepts_follow_on_patch_after_reachable_revision_object() {
        let signing_key = signing_key();
        let sender_key = sender_public_key(&signing_key);
        let mut session = WireSession::default();
        session
            .register_known_peer("node:alpha", &sender_key)
            .expect("known peer should register");
        let patch_object =
            signed_patch_object_message(&signing_key, "node:alpha", "rev:genesis-null");
        let patch_id = patch_object["payload"]["object_id"]
            .as_str()
            .expect("signed patch OBJECT should include object_id")
            .to_owned();
        let revision_object =
            signed_revision_object_message(&signing_key, "node:alpha", &[], &[patch_id.as_str()]);
        let revision_id = revision_object["payload"]["object_id"]
            .as_str()
            .expect("signed revision OBJECT should include object_id")
            .to_owned();
        let hello = signed_hello_message(&signing_key, "node:alpha", "node:alpha");
        let manifest = signed_manifest_message_with_heads(
            &signing_key,
            "node:alpha",
            "node:alpha",
            json!({
                "doc:test": [revision_id.clone()]
            }),
        );
        let root_want = signed_want_message(&signing_key, "node:alpha", &[revision_id.as_str()]);
        let follow_on_want = signed_want_message(&signing_key, "node:alpha", &[patch_id.as_str()]);

        session
            .verify_incoming(&hello)
            .expect("HELLO should verify");
        session
            .verify_incoming(&manifest)
            .expect("MANIFEST should verify");
        session
            .verify_incoming(&root_want)
            .expect("root WANT should verify");
        let envelope = session
            .verify_incoming(&revision_object)
            .expect("reachable revision OBJECT should verify");

        assert_eq!(envelope.message_type(), WireMessageType::Object);
        assert!(session
            .peer_session("node:alpha")
            .is_some_and(|state| state.reachable_object_ids.contains(&patch_id)));

        session
            .verify_incoming(&follow_on_want)
            .expect("follow-on patch WANT should verify");
        let patch_envelope = session
            .verify_incoming(&patch_object)
            .expect("reachable patch OBJECT should verify");

        assert_eq!(patch_envelope.message_type(), WireMessageType::Object);
        assert_eq!(
            session
                .peer_session("node:alpha")
                .map(|state| state.pending_object_ids.len()),
            Some(0)
        );
        assert!(session
            .peer_session("node:alpha")
            .is_some_and(|state| state.accepted_sync_roots.contains(&revision_id)));
    }

    #[test]
    fn wire_session_expands_reachability_from_known_object_index() {
        let signing_key = signing_key();
        let sender_key = sender_public_key(&signing_key);
        let mut session = WireSession::default();
        session
            .register_known_peer("node:alpha", &sender_key)
            .expect("known peer should register");
        let base_revision_object =
            signed_revision_object_message(&signing_key, "node:alpha", &[], &[]);
        let base_revision_id = base_revision_object["payload"]["object_id"]
            .as_str()
            .expect("signed base revision OBJECT should include object_id")
            .to_owned();
        let patch_object =
            signed_patch_object_message(&signing_key, "node:alpha", &base_revision_id);
        let patch_id = patch_object["payload"]["object_id"]
            .as_str()
            .expect("signed patch OBJECT should include object_id")
            .to_owned();
        let root_revision_object = signed_revision_object_message(
            &signing_key,
            "node:alpha",
            &[base_revision_id.as_str()],
            &[patch_id.as_str()],
        );
        let root_revision_id = root_revision_object["payload"]["object_id"]
            .as_str()
            .expect("signed root revision OBJECT should include object_id")
            .to_owned();
        session.set_known_verified_object_index(std::collections::BTreeMap::from([
            (
                root_revision_id.clone(),
                root_revision_object["payload"]["body"].clone(),
            ),
            (patch_id.clone(), patch_object["payload"]["body"].clone()),
            (
                base_revision_id.clone(),
                base_revision_object["payload"]["body"].clone(),
            ),
        ]));

        let hello = signed_hello_message(&signing_key, "node:alpha", "node:alpha");
        let manifest = signed_manifest_message_with_heads(
            &signing_key,
            "node:alpha",
            "node:alpha",
            json!({
                "doc:test": [root_revision_id.clone()]
            }),
        );
        let root_want =
            signed_want_message(&signing_key, "node:alpha", &[root_revision_id.as_str()]);
        let follow_on_want = signed_want_message(
            &signing_key,
            "node:alpha",
            &[patch_id.as_str(), base_revision_id.as_str()],
        );

        session
            .verify_incoming(&hello)
            .expect("HELLO should verify");
        session
            .verify_incoming(&manifest)
            .expect("MANIFEST should verify");
        session
            .verify_incoming(&root_want)
            .expect("root WANT should verify");

        assert!(session.peer_session("node:alpha").is_some_and(|state| {
            state.reachable_object_ids.contains(&patch_id)
                && state.reachable_object_ids.contains(&base_revision_id)
        }));

        session
            .verify_incoming(&follow_on_want)
            .expect("known-index-expanded WANT should verify");
    }

    #[test]
    fn wire_session_loads_known_verified_object_index_from_store() {
        let store_root = temp_dir("known-index");
        let signing_key = signing_key();
        let sender_key = sender_public_key(&signing_key);
        let base_revision_object =
            signed_revision_object_message(&signing_key, "node:alpha", &[], &[]);
        let base_revision_id = base_revision_object["payload"]["object_id"]
            .as_str()
            .expect("signed base revision OBJECT should include object_id")
            .to_owned();
        let patch_object =
            signed_patch_object_message(&signing_key, "node:alpha", &base_revision_id);
        let patch_id = patch_object["payload"]["object_id"]
            .as_str()
            .expect("signed patch OBJECT should include object_id")
            .to_owned();
        let root_revision_object = signed_revision_object_message(
            &signing_key,
            "node:alpha",
            &[base_revision_id.as_str()],
            &[patch_id.as_str()],
        );
        let root_revision_id = root_revision_object["payload"]["object_id"]
            .as_str()
            .expect("signed root revision OBJECT should include object_id")
            .to_owned();

        write_object_value_to_store(&store_root, &base_revision_object["payload"]["body"])
            .expect("base revision should write to store");
        write_object_value_to_store(&store_root, &patch_object["payload"]["body"])
            .expect("patch should write to store");
        write_object_value_to_store(&store_root, &root_revision_object["payload"]["body"])
            .expect("root revision should write to store");

        let mut known_peers = WirePeerDirectory::new();
        known_peers
            .register_known_peer("node:alpha", &sender_key)
            .expect("known peer should register");
        let mut session = WireSession::from_store_root(known_peers, &store_root)
            .expect("session should bootstrap from store root");

        let hello = signed_hello_message(&signing_key, "node:alpha", "node:alpha");
        let manifest = signed_manifest_message_with_heads(
            &signing_key,
            "node:alpha",
            "node:alpha",
            json!({
                "doc:test": [root_revision_id.clone()]
            }),
        );
        let root_want =
            signed_want_message(&signing_key, "node:alpha", &[root_revision_id.as_str()]);
        let follow_on_want = signed_want_message(
            &signing_key,
            "node:alpha",
            &[patch_id.as_str(), base_revision_id.as_str()],
        );

        session
            .verify_incoming(&hello)
            .expect("HELLO should verify");
        session
            .verify_incoming(&manifest)
            .expect("MANIFEST should verify");
        session
            .verify_incoming(&root_want)
            .expect("root WANT should verify");
        session
            .verify_incoming(&follow_on_want)
            .expect("store-backed reachable WANT should verify");

        let _ = fs::remove_dir_all(store_root);
    }

    #[test]
    fn wire_session_rejects_unrequested_object() {
        let signing_key = signing_key();
        let sender_key = sender_public_key(&signing_key);
        let mut session = WireSession::default();
        session
            .register_known_peer("node:alpha", &sender_key)
            .expect("known peer should register");
        let hello = signed_hello_message(&signing_key, "node:alpha", "node:alpha");
        let manifest = signed_manifest_message(&signing_key, "node:alpha", "node:alpha");
        let object = signed_object_message(&signing_key, "node:alpha");
        let object_id = object["payload"]["object_id"]
            .as_str()
            .expect("signed OBJECT payload should include object_id")
            .to_owned();

        session
            .verify_incoming(&hello)
            .expect("HELLO should verify");
        session
            .verify_incoming(&manifest)
            .expect("MANIFEST should verify");
        let error = session.verify_incoming(&object).unwrap_err();

        assert_eq!(
            error,
            format!("wire OBJECT '{object_id}' was not requested from 'node:alpha'")
        );
    }

    #[test]
    fn wire_session_rejects_messages_after_bye() {
        let signing_key = signing_key();
        let sender_key = sender_public_key(&signing_key);
        let mut session = WireSession::default();
        session
            .register_known_peer("node:alpha", &sender_key)
            .expect("known peer should register");
        let hello = signed_hello_message(&signing_key, "node:alpha", "node:alpha");
        let bye = signed_bye_message(&signing_key, "node:alpha");
        let want = signed_want_message(&signing_key, "node:alpha", &["patch:test"]);

        session
            .verify_incoming(&hello)
            .expect("HELLO should verify");
        session.verify_incoming(&bye).expect("BYE should verify");
        let error = session.verify_incoming(&want).unwrap_err();

        assert_eq!(error, "wire session for 'node:alpha' is already closed");
    }

    #[test]
    fn wire_session_rejects_duplicate_hello() {
        let signing_key = signing_key();
        let sender_key = sender_public_key(&signing_key);
        let mut session = WireSession::default();
        session
            .register_known_peer("node:alpha", &sender_key)
            .expect("known peer should register");
        let hello = signed_hello_message(&signing_key, "node:alpha", "node:alpha");

        session
            .verify_incoming(&hello)
            .expect("first HELLO should verify");
        let error = session.verify_incoming(&hello).unwrap_err();

        assert_eq!(
            error,
            "wire session already received HELLO from 'node:alpha'"
        );
    }

    #[test]
    fn wire_session_accepts_error_before_hello() {
        let signing_key = signing_key();
        let sender_key = sender_public_key(&signing_key);
        let mut session = WireSession::default();
        session
            .register_known_peer("node:alpha", &sender_key)
            .expect("known peer should register");
        let error_msg = signed_error_message(&signing_key, "node:alpha", "msg:some-prior-msg");

        // ERROR must be accepted even before HELLO has been received,
        // because it carries no sequencing restriction.
        session
            .verify_incoming(&error_msg)
            .expect("ERROR should be accepted before HELLO");

        let state = session
            .peer_session("node:alpha")
            .expect("peer session should exist");
        assert!(
            !state.hello_received(),
            "hello_received must remain false after an ERROR-only exchange"
        );
    }
}
