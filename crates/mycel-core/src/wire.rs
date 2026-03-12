use std::collections::{BTreeMap, BTreeSet};
use std::fmt;
use std::str::FromStr;

use serde::{Deserialize, Serialize};
use serde_json::{Map, Value};

use crate::canonical::wire_envelope_signed_payload_bytes;
use crate::protocol::{
    ensure_supported_json_values, object_schema, parse_object_envelope, recompute_object_id,
    reject_duplicate_strings, reject_unknown_fields, required_non_empty_string_array,
    required_prefixed_string_map, required_string_field, validate_canonical_object_id,
    validate_prefixed_string, StringFieldError, WIRE_PROTOCOL_VERSION,
};
use crate::signature::verify_ed25519_signature;

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
    pending_object_ids: BTreeSet<String>,
    closed: bool,
}

#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct WireSession {
    known_peers: WirePeerDirectory,
    peer_sessions: BTreeMap<String, WirePeerSessionState>,
}

impl WireSession {
    pub fn new(known_peers: WirePeerDirectory) -> Self {
        Self {
            known_peers,
            peer_sessions: BTreeMap::new(),
        }
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

    pub fn peer_session(&self, node_id: &str) -> Option<&WirePeerSessionState> {
        self.peer_sessions.get(node_id)
    }

    pub fn verify_incoming<'a>(
        &mut self,
        value: &'a Value,
    ) -> Result<ParsedWireEnvelope<'a>, String> {
        let envelope = validate_wire_envelope(value)?;
        validate_wire_sender_identity(&envelope)?;
        let sender_public_key = self
            .known_peers
            .sender_public_key(envelope.from())
            .ok_or_else(|| format!("unknown wire sender '{}'", envelope.from()))?;
        verify_wire_envelope_signature_bytes(value, sender_public_key, "known sender public key")?;
        let peer_session = self
            .peer_sessions
            .entry(envelope.from().to_owned())
            .or_default();
        validate_wire_inbound_sequence(&envelope, peer_session)?;
        advance_wire_inbound_sequence(&envelope, peer_session)?;
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

    Ok(())
}

fn advance_wire_inbound_sequence(
    envelope: &ParsedWireEnvelope<'_>,
    peer_session: &mut WirePeerSessionState,
) -> Result<(), String> {
    match envelope.message_type() {
        WireMessageType::Hello => {
            peer_session.hello_received = true;
        }
        WireMessageType::Want => {
            for object_id in validate_wire_string_array(envelope.payload(), "objects")? {
                peer_session.pending_object_ids.insert(object_id);
            }
        }
        WireMessageType::Object => {
            let object_id =
                required_wire_string(envelope.payload(), "object_id", "OBJECT payload")?;
            peer_session.pending_object_ids.remove(&object_id);
        }
        WireMessageType::Bye => {
            peer_session.closed = true;
        }
        WireMessageType::Manifest
        | WireMessageType::Heads
        | WireMessageType::SnapshotOffer
        | WireMessageType::ViewAnnounce
        | WireMessageType::Error => {}
    }

    Ok(())
}

pub fn validate_wire_payload(
    message_type: WireMessageType,
    payload: &Map<String, Value>,
) -> Result<(), String> {
    match message_type {
        WireMessageType::Hello => {
            validate_prefixed_string(
                &required_wire_string(payload, "node_id", "wire payload")?,
                "node_id",
                "node:",
            )
            .map_err(|error| error.to_string())?;
            validate_wire_string_array(payload, "capabilities")?;
            validate_prefixed_string(
                &required_wire_string(payload, "nonce", "wire payload")?,
                "nonce",
                "n:",
            )
            .map_err(|error| error.to_string())?;
        }
        WireMessageType::Manifest => {
            validate_prefixed_string(
                &required_wire_string(payload, "node_id", "wire payload")?,
                "node_id",
                "node:",
            )
            .map_err(|error| error.to_string())?;
            validate_wire_string_array(payload, "capabilities")?;
            validate_wire_head_map(payload, "heads")?;
            if payload.contains_key("snapshots") {
                for (index, object_id) in validate_wire_string_array(payload, "snapshots")?
                    .iter()
                    .enumerate()
                {
                    validate_canonical_object_id(object_id, &format!("snapshots[{index}]"))
                        .map_err(|error| error.to_string())?;
                }
            }
            if payload.contains_key("views") {
                for (index, object_id) in validate_wire_string_array(payload, "views")?
                    .iter()
                    .enumerate()
                {
                    validate_canonical_object_id(object_id, &format!("views[{index}]"))
                        .map_err(|error| error.to_string())?;
                }
            }
        }
        WireMessageType::Heads => {
            validate_wire_head_map(payload, "documents")?;
            match payload.get("replace") {
                Some(Value::Bool(_)) => {}
                Some(_) => return Err("top-level 'replace' must be a boolean".to_string()),
                None => return Err("missing boolean field 'replace'".to_string()),
            }
        }
        WireMessageType::Want => {
            for (index, object_id) in validate_wire_string_array(payload, "objects")?
                .iter()
                .enumerate()
            {
                validate_canonical_object_id(object_id, &format!("objects[{index}]"))
                    .map_err(|error| error.to_string())?;
            }
        }
        WireMessageType::Object => {
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
            .ok_or_else(|| {
                "OBJECT payload 'object_type' must be a supported object type".to_string()
            })?;
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
            required_wire_string(payload, "hash_alg", "OBJECT payload")?;
            if !matches!(payload.get("body"), Some(Value::Object(_))) {
                return Err("top-level 'body' must be an object".to_string());
            }
        }
        WireMessageType::SnapshotOffer => {
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
        }
        WireMessageType::ViewAnnounce => {
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
        }
        WireMessageType::Bye => {
            required_wire_string(payload, "reason", "wire payload")?;
        }
        WireMessageType::Error => {
            validate_prefixed_string(
                &required_wire_string(payload, "in_reply_to", "wire payload")?,
                "in_reply_to",
                "msg:",
            )
            .map_err(|error| error.to_string())?;
            required_wire_string(payload, "code", "wire payload")?;
        }
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
    let body_envelope = parse_object_envelope(body).map_err(|error| error.to_string())?;
    if body_envelope.object_type() != object_type {
        return Err(format!(
            "OBJECT body type '{}' does not match object_type '{}'",
            body_envelope.object_type(),
            object_type
        ));
    }

    let expected_object_id = match object_type.as_str() {
        "patch" => recompute_object_id(body, "patch_id", "patch"),
        "revision" => recompute_object_id(body, "revision_id", "rev"),
        "view" => recompute_object_id(body, "view_id", "view"),
        "snapshot" => recompute_object_id(body, "snapshot_id", "snap"),
        other => return Err(format!("unsupported OBJECT object_type '{other}'")),
    }
    .map_err(|error| format!("failed to recompute OBJECT body ID: {error}"))?;

    let expected_hash = format!(
        "hash:{}",
        expected_object_id
            .split_once(':')
            .map(|(_, suffix)| suffix)
            .ok_or_else(|| "recomputed OBJECT ID is missing ':' separator".to_string())?
    );

    if object_id != expected_object_id {
        return Err(format!(
            "OBJECT payload object_id '{object_id}' does not match recomputed '{expected_object_id}'"
        ));
    }
    if hash != expected_hash {
        return Err(format!(
            "OBJECT payload hash '{hash}' does not match recomputed '{expected_hash}'"
        ));
    }

    Ok(())
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
    use base64::Engine;
    use ed25519_dalek::{Signer, SigningKey};
    use serde_json::{json, Value};

    use crate::canonical::wire_envelope_signed_payload_bytes;
    use crate::protocol::recompute_object_id;

    use super::{
        parse_wire_envelope, validate_wire_envelope, validate_wire_object_payload_behavior,
        validate_wire_payload, verify_wire_envelope_signature, WireMessageType, WirePeerDirectory,
        WireSession,
    };

    fn signing_key() -> SigningKey {
        SigningKey::from_bytes(&[9u8; 32])
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

    fn signed_hello_message(
        signing_key: &SigningKey,
        sender: &str,
        payload_node_id: &str,
    ) -> Value {
        let mut value = json!({
            "type": "HELLO",
            "version": "mycel-wire/0.1",
            "msg_id": "msg:hello-signed-001",
            "timestamp": "2026-03-08T20:00:00+08:00",
            "from": sender,
            "payload": {
                "node_id": payload_node_id,
                "capabilities": ["patch-sync"],
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
        let mut value = json!({
            "type": "MANIFEST",
            "version": "mycel-wire/0.1",
            "msg_id": "msg:manifest-signed-001",
            "timestamp": "2026-03-08T20:00:10+08:00",
            "from": sender,
            "payload": {
                "node_id": payload_node_id,
                "capabilities": ["patch-sync"],
                "heads": {
                    "doc:test": ["rev:test"]
                }
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

    fn signed_object_message(signing_key: &SigningKey, sender: &str) -> Value {
        let mut body = json!({
            "type": "patch",
            "version": "mycel/0.1",
            "patch_id": "patch:placeholder",
            "doc_id": "doc:test",
            "base_revision": "rev:genesis-null",
            "author": "pk:ed25519:test",
            "timestamp": 1u64,
            "ops": [],
            "signature": "sig:placeholder"
        });
        let object_id = recompute_object_id(&body, "patch_id", "patch")
            .expect("concrete wire object ID should recompute");
        body["patch_id"] = Value::String(object_id.clone());
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
            "hash_alg": "blake3",
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
    fn validate_wire_envelope_accepts_concrete_object_payload() {
        let mut body = json!({
            "type": "patch",
            "version": "mycel/0.1",
            "patch_id": "patch:placeholder",
            "doc_id": "doc:test",
            "base_revision": "rev:genesis-null",
            "author": "pk:ed25519:test",
            "timestamp": 1u64,
            "ops": [],
            "signature": "sig:placeholder"
        });
        let object_id = recompute_object_id(&body, "patch_id", "patch")
            .expect("concrete wire object ID should recompute");
        body["patch_id"] = Value::String(object_id.clone());
        let object_hash = object_id
            .split_once(':')
            .map(|(_, hash)| hash.to_string())
            .expect("wire object ID should contain hash");

        let value = json!({
            "type": "OBJECT",
            "version": "mycel-wire/0.1",
            "msg_id": "msg:obj-concrete-001",
            "timestamp": "2026-03-08T20:01:02+08:00",
            "from": "node:alpha",
            "payload": {
                "object_id": object_id,
                "object_type": "patch",
                "encoding": "json",
                "hash_alg": "sha256",
                "hash": format!("hash:{object_hash}"),
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
    fn wire_session_accepts_requested_object_after_want() {
        let signing_key = signing_key();
        let sender_key = sender_public_key(&signing_key);
        let mut session = WireSession::default();
        session
            .register_known_peer("node:alpha", &sender_key)
            .expect("known peer should register");
        let hello = signed_hello_message(&signing_key, "node:alpha", "node:alpha");
        let object = signed_object_message(&signing_key, "node:alpha");
        let object_id = object["payload"]["object_id"]
            .as_str()
            .expect("signed OBJECT payload should include object_id")
            .to_owned();
        let want = signed_want_message(&signing_key, "node:alpha", &[object_id.as_str()]);

        session
            .verify_incoming(&hello)
            .expect("HELLO should verify");
        session.verify_incoming(&want).expect("WANT should verify");
        let envelope = session
            .verify_incoming(&object)
            .expect("requested OBJECT should verify");

        assert_eq!(envelope.message_type(), WireMessageType::Object);
        assert_eq!(
            session
                .peer_session("node:alpha")
                .map(|state| state.pending_object_ids.len()),
            Some(0)
        );
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
        let object = signed_object_message(&signing_key, "node:alpha");
        let object_id = object["payload"]["object_id"]
            .as_str()
            .expect("signed OBJECT payload should include object_id")
            .to_owned();

        session
            .verify_incoming(&hello)
            .expect("HELLO should verify");
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
}
