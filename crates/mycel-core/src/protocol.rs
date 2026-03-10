//! Shared protocol-facing types for the first Rust workspace cut.
//!
//! This module intentionally starts with a narrow typed object model instead of
//! a full canonical object graph. The current goal is to keep object-kind,
//! signing, and derived-ID knowledge in one place so the verifier and later
//! protocol layers do not each re-encode the same rules.

use std::collections::BTreeMap;
use std::fmt;
use std::str::FromStr;

use serde::{Deserialize, Serialize};
use serde_json::{Map, Value};
use sha2::{Digest, Sha256};

pub const CORE_PROTOCOL_VERSION: &str = "mycel/0.1";
pub const WIRE_PROTOCOL_VERSION: &str = "mycel-wire/0.1";

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct ProtocolVersion {
    pub core: &'static str,
    pub wire: &'static str,
}

impl Default for ProtocolVersion {
    fn default() -> Self {
        Self {
            core: CORE_PROTOCOL_VERSION,
            wire: WIRE_PROTOCOL_VERSION,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
#[serde(rename_all = "lowercase")]
pub enum SignatureRule {
    Forbidden,
    Required,
}

impl SignatureRule {
    pub fn as_str(self) -> &'static str {
        match self {
            Self::Forbidden => "forbidden",
            Self::Required => "required",
        }
    }

    pub fn is_required(self) -> bool {
        matches!(self, Self::Required)
    }
}

impl fmt::Display for SignatureRule {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(self.as_str())
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum ObjectKind {
    Document,
    Block,
    Patch,
    Revision,
    View,
    Snapshot,
}

impl ObjectKind {
    pub fn as_str(self) -> &'static str {
        match self {
            Self::Document => "document",
            Self::Block => "block",
            Self::Patch => "patch",
            Self::Revision => "revision",
            Self::View => "view",
            Self::Snapshot => "snapshot",
        }
    }

    pub fn schema(self) -> ObjectSchema {
        match self {
            Self::Document => ObjectSchema {
                kind: self,
                signature_rule: SignatureRule::Forbidden,
                signer_field: None,
                logical_id_field: Some("doc_id"),
                derived_id_field: None,
                derived_id_prefix: None,
            },
            Self::Block => ObjectSchema {
                kind: self,
                signature_rule: SignatureRule::Forbidden,
                signer_field: None,
                logical_id_field: Some("block_id"),
                derived_id_field: None,
                derived_id_prefix: None,
            },
            Self::Patch => ObjectSchema {
                kind: self,
                signature_rule: SignatureRule::Required,
                signer_field: Some("author"),
                logical_id_field: None,
                derived_id_field: Some("patch_id"),
                derived_id_prefix: Some("patch"),
            },
            Self::Revision => ObjectSchema {
                kind: self,
                signature_rule: SignatureRule::Required,
                signer_field: Some("author"),
                logical_id_field: None,
                derived_id_field: Some("revision_id"),
                derived_id_prefix: Some("rev"),
            },
            Self::View => ObjectSchema {
                kind: self,
                signature_rule: SignatureRule::Required,
                signer_field: Some("maintainer"),
                logical_id_field: None,
                derived_id_field: Some("view_id"),
                derived_id_prefix: Some("view"),
            },
            Self::Snapshot => ObjectSchema {
                kind: self,
                signature_rule: SignatureRule::Required,
                signer_field: Some("created_by"),
                logical_id_field: None,
                derived_id_field: Some("snapshot_id"),
                derived_id_prefix: Some("snap"),
            },
        }
    }
}

impl fmt::Display for ObjectKind {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(self.as_str())
    }
}

impl FromStr for ObjectKind {
    type Err = ();

    fn from_str(value: &str) -> Result<Self, Self::Err> {
        match value {
            "document" => Ok(Self::Document),
            "block" => Ok(Self::Block),
            "patch" => Ok(Self::Patch),
            "revision" => Ok(Self::Revision),
            "view" => Ok(Self::View),
            "snapshot" => Ok(Self::Snapshot),
            _ => Err(()),
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
pub struct ObjectSchema {
    pub kind: ObjectKind,
    pub signature_rule: SignatureRule,
    pub signer_field: Option<&'static str>,
    pub logical_id_field: Option<&'static str>,
    pub derived_id_field: Option<&'static str>,
    pub derived_id_prefix: Option<&'static str>,
}

impl ObjectSchema {
    pub fn logical_id_field(self) -> Option<&'static str> {
        self.logical_id_field
    }

    pub fn derived_id(self) -> Option<(&'static str, &'static str)> {
        match (self.derived_id_field, self.derived_id_prefix) {
            (Some(field), Some(prefix)) => Some((field, prefix)),
            _ => None,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
pub struct DeclaredDerivedId<'a> {
    pub field: &'static str,
    pub prefix: &'static str,
    pub value: &'a str,
}

pub fn object_schema(object_type: &str) -> Option<ObjectSchema> {
    ObjectKind::from_str(object_type)
        .ok()
        .map(ObjectKind::schema)
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum StringFieldError {
    Missing,
    WrongType,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ParseObjectEnvelopeError {
    TopLevelNotObject,
    MissingType,
    TypeNotString,
    UnsupportedType(String),
}

impl fmt::Display for ParseObjectEnvelopeError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::TopLevelNotObject => f.write_str("top-level JSON value must be an object"),
            Self::MissingType => f.write_str("object is missing string field 'type'"),
            Self::TypeNotString => f.write_str("top-level 'type' should be a string"),
            Self::UnsupportedType(object_type) => {
                write!(f, "unsupported object type '{object_type}'")
            }
        }
    }
}

#[derive(Debug, Clone, Copy)]
pub struct ParsedObjectEnvelope<'a> {
    object: &'a Map<String, Value>,
    kind: ObjectKind,
    schema: ObjectSchema,
}

impl<'a> ParsedObjectEnvelope<'a> {
    pub fn object(&self) -> &'a Map<String, Value> {
        self.object
    }

    pub fn kind(&self) -> ObjectKind {
        self.kind
    }

    pub fn schema(&self) -> ObjectSchema {
        self.schema
    }

    pub fn object_type(&self) -> &'static str {
        self.kind.as_str()
    }

    pub fn has_signature(&self) -> bool {
        self.object.contains_key("signature")
    }

    pub fn top_level_keys(&self) -> Vec<String> {
        let mut keys: Vec<String> = self.object.keys().cloned().collect();
        keys.sort_unstable();
        keys
    }

    pub fn optional_string_field(&self, field: &str) -> Result<Option<&'a str>, StringFieldError> {
        optional_string_field(self.object, field)
    }

    pub fn required_string_field(&self, field: &str) -> Result<&'a str, StringFieldError> {
        required_string_field(self.object, field)
    }

    pub fn signer(&self) -> Result<Option<&'a str>, StringFieldError> {
        match self.schema.signer_field {
            Some(field) => required_string_field(self.object, field).map(Some),
            None => Ok(None),
        }
    }

    pub fn logical_id(&self) -> Result<Option<&'a str>, StringFieldError> {
        match self.schema.logical_id_field {
            Some(field) => required_string_field(self.object, field).map(Some),
            None => Ok(None),
        }
    }

    pub fn declared_id(&self) -> Result<Option<&'a str>, StringFieldError> {
        match self.declared_derived_id()? {
            Some(declared) => Ok(Some(declared.value)),
            None => Ok(None),
        }
    }

    pub fn declared_derived_id(&self) -> Result<Option<DeclaredDerivedId<'a>>, StringFieldError> {
        match self.schema.derived_id() {
            Some((field, prefix)) => required_string_field(self.object, field).map(|value| {
                Some(DeclaredDerivedId {
                    field,
                    prefix,
                    value,
                })
            }),
            None => Ok(None),
        }
    }
}

pub fn parse_object_envelope(
    value: &Value,
) -> Result<ParsedObjectEnvelope<'_>, ParseObjectEnvelopeError> {
    let object = value
        .as_object()
        .ok_or(ParseObjectEnvelopeError::TopLevelNotObject)?;
    let object_type = match object.get("type") {
        Some(Value::String(object_type)) => object_type.as_str(),
        Some(_) => return Err(ParseObjectEnvelopeError::TypeNotString),
        None => return Err(ParseObjectEnvelopeError::MissingType),
    };

    let kind = ObjectKind::from_str(object_type)
        .map_err(|_| ParseObjectEnvelopeError::UnsupportedType(object_type.to_string()))?;

    Ok(ParsedObjectEnvelope {
        object,
        kind,
        schema: kind.schema(),
    })
}

pub fn optional_string_field<'a>(
    object: &'a Map<String, Value>,
    field: &str,
) -> Result<Option<&'a str>, StringFieldError> {
    match object.get(field) {
        Some(Value::String(value)) => Ok(Some(value.as_str())),
        Some(_) => Err(StringFieldError::WrongType),
        None => Ok(None),
    }
}

pub fn required_string_field<'a>(
    object: &'a Map<String, Value>,
    field: &str,
) -> Result<&'a str, StringFieldError> {
    match optional_string_field(object, field)? {
        Some(value) => Ok(value),
        None => Err(StringFieldError::Missing),
    }
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct BlockObject {
    pub block_id: String,
    pub block_type: String,
    pub content: String,
    pub attrs: Map<String, Value>,
    pub children: Vec<BlockObject>,
}

#[derive(Debug, Clone, PartialEq)]
pub struct DocumentObject {
    pub doc_id: String,
    pub title: String,
    pub language: String,
    pub content_model: String,
    pub created_at: u64,
    pub created_by: String,
    pub genesis_revision: String,
}

#[derive(Debug, Clone, PartialEq)]
pub enum PatchOperation {
    InsertBlock {
        parent_block_id: Option<String>,
        index: Option<usize>,
        new_block: BlockObject,
    },
    InsertBlockAfter {
        after_block_id: String,
        new_block: BlockObject,
    },
    DeleteBlock {
        block_id: String,
    },
    ReplaceBlock {
        block_id: String,
        new_content: String,
    },
    MoveBlock {
        block_id: String,
        parent_block_id: Option<String>,
        after_block_id: Option<String>,
    },
    AnnotateBlock {
        block_id: String,
        annotation: BlockObject,
    },
    SetMetadata {
        entries: Map<String, Value>,
    },
}

#[derive(Debug, Clone, PartialEq)]
pub struct PatchObject {
    pub patch_id: String,
    pub doc_id: String,
    pub base_revision: String,
    pub author: String,
    pub timestamp: u64,
    pub ops: Vec<PatchOperation>,
}

#[derive(Debug, Clone, PartialEq)]
pub struct RevisionObject {
    pub revision_id: String,
    pub doc_id: String,
    pub parents: Vec<String>,
    pub patches: Vec<String>,
    pub state_hash: String,
    pub author: String,
    pub timestamp: u64,
}

#[derive(Debug, Clone, PartialEq)]
pub struct ViewObject {
    pub view_id: String,
    pub maintainer: String,
    pub documents: BTreeMap<String, String>,
    pub policy: Value,
    pub timestamp: u64,
}

#[derive(Debug, Clone, PartialEq)]
pub struct SnapshotObject {
    pub snapshot_id: String,
    pub documents: BTreeMap<String, String>,
    pub included_objects: Vec<String>,
    pub root_hash: String,
    pub created_by: String,
    pub timestamp: u64,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TypedObjectError {
    message: String,
}

impl TypedObjectError {
    fn new(message: impl Into<String>) -> Self {
        Self {
            message: message.into(),
        }
    }
}

impl fmt::Display for TypedObjectError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(&self.message)
    }
}

impl std::error::Error for TypedObjectError {}

pub fn parse_document_object(value: &Value) -> Result<DocumentObject, TypedObjectError> {
    let envelope = parse_object_envelope(value)
        .map_err(|error| TypedObjectError::new(format!("document parse error: {error}")))?;
    if envelope.kind() != ObjectKind::Document {
        return Err(TypedObjectError::new(format!(
            "expected document object, found '{}'",
            envelope.object_type()
        )));
    }

    let object = envelope.object();
    Ok(DocumentObject {
        doc_id: required_prefixed_string(object, "doc_id", "doc:")?,
        title: required_string(object, "title")?,
        language: required_string(object, "language")?,
        content_model: required_exact_string(object, "content_model", "block-tree")?,
        created_at: required_u64(object, "created_at")?,
        created_by: required_prefixed_string(object, "created_by", "pk:")?,
        genesis_revision: required_prefixed_string(object, "genesis_revision", "rev:")?,
    })
}

pub fn parse_block_object(value: &Value) -> Result<BlockObject, TypedObjectError> {
    let object = value
        .as_object()
        .ok_or_else(|| TypedObjectError::new("block must be a JSON object"))?;
    let block_type = required_string(object, "block_type")?;
    validate_block_type(&block_type)?;

    Ok(BlockObject {
        block_id: required_prefixed_string(object, "block_id", "blk:")?,
        block_type,
        content: required_string(object, "content")?,
        attrs: required_object(object, "attrs")?,
        children: required_array(object, "children")?
            .iter()
            .map(parse_block_object)
            .collect::<Result<Vec<_>, _>>()?,
    })
}

pub fn parse_patch_object(value: &Value) -> Result<PatchObject, TypedObjectError> {
    let envelope = parse_object_envelope(value)
        .map_err(|error| TypedObjectError::new(format!("patch parse error: {error}")))?;
    if envelope.kind() != ObjectKind::Patch {
        return Err(TypedObjectError::new(format!(
            "expected patch object, found '{}'",
            envelope.object_type()
        )));
    }

    let object = envelope.object();
    Ok(PatchObject {
        patch_id: required_prefixed_string(object, "patch_id", "patch:")?,
        doc_id: required_prefixed_string(object, "doc_id", "doc:")?,
        base_revision: required_prefixed_string(object, "base_revision", "rev:")?,
        author: required_prefixed_string(object, "author", "pk:")?,
        timestamp: required_u64(object, "timestamp")?,
        ops: required_array(object, "ops")?
            .iter()
            .map(parse_patch_operation)
            .collect::<Result<Vec<_>, _>>()?,
    })
}

pub fn parse_revision_object(value: &Value) -> Result<RevisionObject, TypedObjectError> {
    let envelope = parse_object_envelope(value)
        .map_err(|error| TypedObjectError::new(format!("revision parse error: {error}")))?;
    if envelope.kind() != ObjectKind::Revision {
        return Err(TypedObjectError::new(format!(
            "expected revision object, found '{}'",
            envelope.object_type()
        )));
    }

    let object = envelope.object();
    Ok(RevisionObject {
        revision_id: required_prefixed_string(object, "revision_id", "rev:")?,
        doc_id: required_prefixed_string(object, "doc_id", "doc:")?,
        parents: required_prefixed_string_array(object, "parents", "rev:")?,
        patches: required_prefixed_string_array(object, "patches", "patch:")?,
        state_hash: required_prefixed_string(object, "state_hash", "hash:")?,
        author: required_prefixed_string(object, "author", "pk:")?,
        timestamp: required_u64(object, "timestamp")?,
    })
}

pub fn parse_view_object(value: &Value) -> Result<ViewObject, TypedObjectError> {
    let envelope = parse_object_envelope(value)
        .map_err(|error| TypedObjectError::new(format!("view parse error: {error}")))?;
    if envelope.kind() != ObjectKind::View {
        return Err(TypedObjectError::new(format!(
            "expected view object, found '{}'",
            envelope.object_type()
        )));
    }

    let object = envelope.object();
    let documents = required_string_map(object, "documents")?;
    if documents.is_empty() {
        return Err(TypedObjectError::new(
            "top-level 'documents' must not be empty",
        ));
    }

    Ok(ViewObject {
        view_id: required_prefixed_string(object, "view_id", "view:")?,
        maintainer: required_prefixed_string(object, "maintainer", "pk:")?,
        documents: require_prefixed_string_map_entries(documents, "documents", "doc:", "rev:")?,
        policy: Value::Object(required_object(object, "policy")?),
        timestamp: required_u64(object, "timestamp")?,
    })
}

pub fn parse_snapshot_object(value: &Value) -> Result<SnapshotObject, TypedObjectError> {
    let envelope = parse_object_envelope(value)
        .map_err(|error| TypedObjectError::new(format!("snapshot parse error: {error}")))?;
    if envelope.kind() != ObjectKind::Snapshot {
        return Err(TypedObjectError::new(format!(
            "expected snapshot object, found '{}'",
            envelope.object_type()
        )));
    }

    let object = envelope.object();
    let documents = required_prefixed_string_map(object, "documents", "doc:", "rev:")?;
    if documents.is_empty() {
        return Err(TypedObjectError::new(
            "top-level 'documents' must not be empty",
        ));
    }
    let included_objects = required_canonical_object_id_array(object, "included_objects")?;
    for (doc_id, revision_id) in &documents {
        if !included_objects
            .iter()
            .any(|object_id| object_id == revision_id)
        {
            return Err(TypedObjectError::new(format!(
                "top-level 'included_objects' must include revision '{revision_id}' declared by 'documents.{doc_id}'"
            )));
        }
    }

    Ok(SnapshotObject {
        snapshot_id: required_prefixed_string(object, "snapshot_id", "snap:")?,
        documents,
        included_objects,
        root_hash: required_prefixed_string(object, "root_hash", "hash:")?,
        created_by: required_prefixed_string(object, "created_by", "pk:")?,
        timestamp: required_u64(object, "timestamp")?,
    })
}

pub fn recompute_object_id(
    value: &Value,
    derived_id_field: &str,
    prefix: &str,
) -> Result<String, String> {
    let mut object = value
        .as_object()
        .cloned()
        .ok_or_else(|| "top-level JSON value must be an object".to_string())?;
    object.remove(derived_id_field);
    object.remove("signature");

    let canonical = canonical_json(&Value::Object(object))?;
    let mut hasher = Sha256::new();
    hasher.update(canonical.as_bytes());
    let digest = hasher.finalize();
    Ok(format!("{prefix}:{}", hex_encode(&digest)))
}

pub fn signed_payload_bytes(value: &Value) -> Result<Vec<u8>, String> {
    let mut object = value
        .as_object()
        .cloned()
        .ok_or_else(|| "top-level JSON value must be an object".to_string())?;
    object.remove("signature");
    let canonical = canonical_json(&Value::Object(object))?;
    Ok(canonical.into_bytes())
}

pub fn canonical_json(value: &Value) -> Result<String, String> {
    let mut output = String::new();
    write_canonical_json(value, &mut output)?;
    Ok(output)
}

fn write_canonical_json(value: &Value, output: &mut String) -> Result<(), String> {
    match value {
        Value::Null => Err("null is not allowed in canonical objects".to_string()),
        Value::Bool(boolean) => {
            output.push_str(if *boolean { "true" } else { "false" });
            Ok(())
        }
        Value::Number(number) => {
            if !(number.is_i64() || number.is_u64()) {
                return Err(
                    "floating-point numbers are not allowed in canonical objects".to_string(),
                );
            }
            output.push_str(&number.to_string());
            Ok(())
        }
        Value::String(string) => {
            let encoded = serde_json::to_string(string)
                .map_err(|err| format!("failed to encode JSON string: {err}"))?;
            output.push_str(&encoded);
            Ok(())
        }
        Value::Array(values) => {
            output.push('[');
            for (index, entry) in values.iter().enumerate() {
                if index > 0 {
                    output.push(',');
                }
                write_canonical_json(entry, output)?;
            }
            output.push(']');
            Ok(())
        }
        Value::Object(entries) => {
            output.push('{');
            let mut keys: Vec<&String> = entries.keys().collect();
            keys.sort_unstable();

            for (index, key) in keys.iter().enumerate() {
                if index > 0 {
                    output.push(',');
                }

                let encoded_key = serde_json::to_string(key)
                    .map_err(|err| format!("failed to encode JSON object key: {err}"))?;
                output.push_str(&encoded_key);
                output.push(':');
                write_canonical_json(&entries[*key], output)?;
            }
            output.push('}');
            Ok(())
        }
    }
}

pub fn hex_encode(bytes: &[u8]) -> String {
    let mut output = String::with_capacity(bytes.len() * 2);
    for byte in bytes {
        output.push_str(&format!("{byte:02x}"));
    }
    output
}

fn parse_patch_operation(value: &Value) -> Result<PatchOperation, TypedObjectError> {
    let object = value
        .as_object()
        .ok_or_else(|| TypedObjectError::new("patch op must be a JSON object"))?;
    let op = required_string(object, "op")?;

    match op.as_str() {
        "insert_block" => Ok(PatchOperation::InsertBlock {
            parent_block_id: optional_prefixed_string(object, "parent_block_id", "blk:")?,
            index: optional_usize(object, "index")?,
            new_block: parse_block_field(object, "new_block")?,
        }),
        "insert_block_after" => Ok(PatchOperation::InsertBlockAfter {
            after_block_id: required_prefixed_string(object, "after_block_id", "blk:")?,
            new_block: parse_block_field(object, "new_block")?,
        }),
        "delete_block" => Ok(PatchOperation::DeleteBlock {
            block_id: required_prefixed_string(object, "block_id", "blk:")?,
        }),
        "replace_block" => Ok(PatchOperation::ReplaceBlock {
            block_id: required_prefixed_string(object, "block_id", "blk:")?,
            new_content: required_string(object, "new_content")?,
        }),
        "move_block" => Ok(PatchOperation::MoveBlock {
            block_id: required_prefixed_string(object, "block_id", "blk:")?,
            parent_block_id: optional_prefixed_string(object, "parent_block_id", "blk:")?,
            after_block_id: optional_prefixed_string(object, "after_block_id", "blk:")?,
        })
        .and_then(|operation| match operation {
            PatchOperation::MoveBlock {
                parent_block_id: None,
                after_block_id: None,
                ..
            } => Err(TypedObjectError::new(
                "move_block requires at least one destination reference",
            )),
            _ => Ok(operation),
        }),
        "annotate_block" => Ok(PatchOperation::AnnotateBlock {
            block_id: required_prefixed_string(object, "block_id", "blk:")?,
            annotation: parse_block_field(object, "annotation")?,
        }),
        "set_metadata" => Ok(PatchOperation::SetMetadata {
            entries: parse_metadata_entries(object)?,
        }),
        _ => Err(TypedObjectError::new(format!(
            "unsupported patch op '{op}'"
        ))),
    }
}

fn parse_block_field(
    object: &Map<String, Value>,
    field: &str,
) -> Result<BlockObject, TypedObjectError> {
    let value = object
        .get(field)
        .ok_or_else(|| TypedObjectError::new(format!("missing object field '{field}'")))?;
    parse_block_object(value)
}

fn parse_metadata_entries(
    object: &Map<String, Value>,
) -> Result<Map<String, Value>, TypedObjectError> {
    if let Some(metadata) = object.get("metadata") {
        let entries = metadata
            .as_object()
            .ok_or_else(|| TypedObjectError::new("top-level 'metadata' must be an object"))?;
        if entries.is_empty() {
            return Err(TypedObjectError::new(
                "top-level 'metadata' must not be empty",
            ));
        }
        if entries.keys().any(String::is_empty) {
            return Err(TypedObjectError::new(
                "top-level 'metadata' keys must not be empty strings",
            ));
        }
        return Ok(entries.clone());
    }

    let key = required_string(object, "key")?;
    if key.is_empty() {
        return Err(TypedObjectError::new(
            "top-level 'key' must not be an empty string",
        ));
    }
    let value = object
        .get("value")
        .ok_or_else(|| TypedObjectError::new("missing object field 'value'"))?;
    let mut entries = Map::new();
    entries.insert(key, value.clone());
    Ok(entries)
}

fn required_string(object: &Map<String, Value>, field: &str) -> Result<String, TypedObjectError> {
    match object.get(field) {
        Some(Value::String(value)) => Ok(value.clone()),
        Some(_) => Err(TypedObjectError::new(format!(
            "top-level '{field}' must be a string"
        ))),
        None => Err(TypedObjectError::new(format!(
            "missing string field '{field}'"
        ))),
    }
}

fn required_prefixed_string(
    object: &Map<String, Value>,
    field: &str,
    prefix: &str,
) -> Result<String, TypedObjectError> {
    let value = required_string(object, field)?;
    validate_prefixed_string(&value, field, prefix)?;
    Ok(value)
}

fn required_exact_string(
    object: &Map<String, Value>,
    field: &str,
    expected: &str,
) -> Result<String, TypedObjectError> {
    let value = required_string(object, field)?;
    if value != expected {
        return Err(TypedObjectError::new(format!(
            "top-level '{field}' must equal '{expected}'"
        )));
    }
    Ok(value)
}

fn optional_string(
    object: &Map<String, Value>,
    field: &str,
) -> Result<Option<String>, TypedObjectError> {
    match object.get(field) {
        Some(Value::String(value)) => Ok(Some(value.clone())),
        Some(_) => Err(TypedObjectError::new(format!(
            "top-level '{field}' must be a string"
        ))),
        None => Ok(None),
    }
}

fn optional_prefixed_string(
    object: &Map<String, Value>,
    field: &str,
    prefix: &str,
) -> Result<Option<String>, TypedObjectError> {
    let value = optional_string(object, field)?;
    if let Some(value) = &value {
        validate_prefixed_string(value, field, prefix)?;
    }
    Ok(value)
}

fn required_u64(object: &Map<String, Value>, field: &str) -> Result<u64, TypedObjectError> {
    match object.get(field) {
        Some(Value::Number(value)) => value.as_u64().ok_or_else(|| {
            TypedObjectError::new(format!(
                "top-level '{field}' must be a non-negative integer"
            ))
        }),
        Some(_) => Err(TypedObjectError::new(format!(
            "top-level '{field}' must be a non-negative integer"
        ))),
        None => Err(TypedObjectError::new(format!(
            "missing integer field '{field}'"
        ))),
    }
}

fn optional_usize(
    object: &Map<String, Value>,
    field: &str,
) -> Result<Option<usize>, TypedObjectError> {
    match object.get(field) {
        Some(Value::Number(value)) => {
            let index = value.as_u64().ok_or_else(|| {
                TypedObjectError::new(format!(
                    "top-level '{field}' must be a non-negative integer"
                ))
            })?;
            usize::try_from(index)
                .map(Some)
                .map_err(|_| TypedObjectError::new(format!("top-level '{field}' is too large")))
        }
        Some(_) => Err(TypedObjectError::new(format!(
            "top-level '{field}' must be a non-negative integer"
        ))),
        None => Ok(None),
    }
}

fn required_array<'a>(
    object: &'a Map<String, Value>,
    field: &str,
) -> Result<&'a Vec<Value>, TypedObjectError> {
    match object.get(field) {
        Some(Value::Array(values)) => Ok(values),
        Some(_) => Err(TypedObjectError::new(format!(
            "top-level '{field}' must be an array"
        ))),
        None => Err(TypedObjectError::new(format!(
            "missing array field '{field}'"
        ))),
    }
}

fn required_object(
    object: &Map<String, Value>,
    field: &str,
) -> Result<Map<String, Value>, TypedObjectError> {
    match object.get(field) {
        Some(Value::Object(value)) => Ok(value.clone()),
        Some(_) => Err(TypedObjectError::new(format!(
            "top-level '{field}' must be an object"
        ))),
        None => Err(TypedObjectError::new(format!(
            "missing object field '{field}'"
        ))),
    }
}

fn required_string_array(
    object: &Map<String, Value>,
    field: &str,
) -> Result<Vec<String>, TypedObjectError> {
    required_array(object, field)?
        .iter()
        .enumerate()
        .map(|(index, value)| match value {
            Value::String(value) => Ok(value.clone()),
            _ => Err(TypedObjectError::new(format!(
                "top-level '{field}[{index}]' must be a string"
            ))),
        })
        .collect()
}

fn required_non_empty_string_array(
    object: &Map<String, Value>,
    field: &str,
) -> Result<Vec<String>, TypedObjectError> {
    let values = required_string_array(object, field)?;
    for (index, value) in values.iter().enumerate() {
        if value.is_empty() {
            return Err(TypedObjectError::new(format!(
                "top-level '{field}[{index}]' must not be an empty string"
            )));
        }
    }
    Ok(values)
}

fn required_prefixed_string_array(
    object: &Map<String, Value>,
    field: &str,
    prefix: &str,
) -> Result<Vec<String>, TypedObjectError> {
    let values = required_string_array(object, field)?;
    for (index, value) in values.iter().enumerate() {
        validate_prefixed_string_with_path(value, &format!("{field}[{index}]"), prefix)?;
    }
    Ok(values)
}

fn required_canonical_object_id_array(
    object: &Map<String, Value>,
    field: &str,
) -> Result<Vec<String>, TypedObjectError> {
    let values = required_non_empty_string_array(object, field)?;
    for (index, value) in values.iter().enumerate() {
        validate_canonical_object_id(value, &format!("{field}[{index}]"))?;
    }
    Ok(values)
}

fn required_string_map(
    object: &Map<String, Value>,
    field: &str,
) -> Result<BTreeMap<String, String>, TypedObjectError> {
    match object.get(field) {
        Some(Value::Object(entries)) => entries
            .iter()
            .map(|(key, value)| match value {
                Value::String(value) => Ok((key.clone(), value.clone())),
                _ => Err(TypedObjectError::new(format!(
                    "top-level '{field}.{key}' must be a string"
                ))),
            })
            .collect(),
        Some(_) => Err(TypedObjectError::new(format!(
            "top-level '{field}' must be an object"
        ))),
        None => Err(TypedObjectError::new(format!(
            "missing object field '{field}'"
        ))),
    }
}

fn required_prefixed_string_map(
    object: &Map<String, Value>,
    field: &str,
    key_prefix: &str,
    value_prefix: &str,
) -> Result<BTreeMap<String, String>, TypedObjectError> {
    let entries = required_string_map(object, field)?;
    require_prefixed_string_map_entries(entries, field, key_prefix, value_prefix)
}

fn require_prefixed_string_map_entries(
    entries: BTreeMap<String, String>,
    field: &str,
    key_prefix: &str,
    value_prefix: &str,
) -> Result<BTreeMap<String, String>, TypedObjectError> {
    for (key, value) in &entries {
        validate_prefixed_string_with_path(key, &format!("{field}.{key} key"), key_prefix)?;
        validate_prefixed_string_with_path(value, &format!("{field}.{key}"), value_prefix)?;
    }
    Ok(entries)
}

fn validate_prefixed_string(
    value: &str,
    field: &str,
    prefix: &str,
) -> Result<(), TypedObjectError> {
    validate_prefixed_string_with_path(value, field, prefix)
}

fn validate_prefixed_string_with_path(
    value: &str,
    path: &str,
    prefix: &str,
) -> Result<(), TypedObjectError> {
    if !value.starts_with(prefix) || value.len() == prefix.len() {
        return Err(TypedObjectError::new(format!(
            "top-level '{path}' must use '{prefix}' prefix"
        )));
    }
    Ok(())
}

fn validate_canonical_object_id(value: &str, path: &str) -> Result<(), TypedObjectError> {
    if ["patch:", "rev:", "view:", "snap:"]
        .iter()
        .any(|prefix| value.starts_with(prefix) && value.len() > prefix.len())
    {
        return Ok(());
    }

    Err(TypedObjectError::new(format!(
        "top-level '{path}' must use a canonical object ID prefix"
    )))
}

fn validate_block_type(value: &str) -> Result<(), TypedObjectError> {
    match value {
        "title" | "heading" | "paragraph" | "quote" | "verse" | "list" | "annotation"
        | "metadata" => Ok(()),
        _ => Err(TypedObjectError::new(format!(
            "top-level 'block_type' must be one of: title, heading, paragraph, quote, verse, list, annotation, metadata"
        ))),
    }
}

#[cfg(test)]
mod tests {
    use serde_json::json;

    use super::{
        canonical_json, object_schema, parse_block_object, parse_document_object,
        parse_object_envelope, parse_patch_object, parse_revision_object, parse_snapshot_object,
        parse_view_object, recompute_object_id, ObjectKind, ParseObjectEnvelopeError,
        SignatureRule, StringFieldError,
    };

    #[test]
    fn object_kind_round_trips_from_strings() {
        let kind = "revision"
            .parse::<ObjectKind>()
            .expect("revision should parse");
        assert_eq!(kind, ObjectKind::Revision);
        assert_eq!(kind.to_string(), "revision");
    }

    #[test]
    fn patch_schema_requires_signature_and_derived_id() {
        let schema = object_schema("patch").expect("patch schema should exist");
        assert_eq!(schema.kind, ObjectKind::Patch);
        assert_eq!(schema.signature_rule, SignatureRule::Required);
        assert_eq!(schema.signer_field, Some("author"));
        assert_eq!(schema.logical_id_field(), None);
        assert_eq!(schema.derived_id(), Some(("patch_id", "patch")));
    }

    #[test]
    fn document_schema_uses_doc_id_as_logical_id() {
        let schema = object_schema("document").expect("document schema should exist");
        assert_eq!(schema.kind, ObjectKind::Document);
        assert_eq!(schema.signature_rule, SignatureRule::Forbidden);
        assert_eq!(schema.logical_id_field(), Some("doc_id"));
        assert_eq!(schema.derived_id(), None);
    }

    #[test]
    fn unknown_object_kind_has_no_schema() {
        assert!(object_schema("unknown-object").is_none());
    }

    #[test]
    fn parse_object_envelope_exposes_schema_and_fields() {
        let value = json!({
            "type": "view",
            "version": "mycel/0.1",
            "view_id": "view:test",
            "maintainer": "pk:ed25519:test"
        });

        let envelope = parse_object_envelope(&value).expect("view envelope should parse");
        assert_eq!(envelope.kind(), ObjectKind::View);
        assert_eq!(envelope.object_type(), "view");
        assert_eq!(envelope.schema().signer_field, Some("maintainer"));
        assert_eq!(
            envelope.optional_string_field("version"),
            Ok(Some("mycel/0.1"))
        );
        assert_eq!(envelope.signer(), Ok(Some("pk:ed25519:test")));
        assert_eq!(envelope.logical_id(), Ok(None));
        assert_eq!(envelope.declared_id(), Ok(Some("view:test")));
    }

    #[test]
    fn parse_document_envelope_exposes_logical_id() {
        let value = json!({
            "type": "document",
            "version": "mycel/0.1",
            "doc_id": "doc:test"
        });

        let envelope = parse_object_envelope(&value).expect("document envelope should parse");
        assert_eq!(envelope.kind(), ObjectKind::Document);
        assert_eq!(envelope.logical_id(), Ok(Some("doc:test")));
        assert_eq!(envelope.declared_id(), Ok(None));
    }

    #[test]
    fn parse_patch_envelope_exposes_typed_derived_id() {
        let value = json!({
            "type": "patch",
            "version": "mycel/0.1",
            "patch_id": "patch:test",
            "author": "pk:ed25519:test"
        });

        let envelope = parse_object_envelope(&value).expect("patch envelope should parse");
        let declared = envelope
            .declared_derived_id()
            .expect("patch derived ID should parse")
            .expect("patch should expose a derived ID");
        assert_eq!(declared.field, "patch_id");
        assert_eq!(declared.prefix, "patch");
        assert_eq!(declared.value, "patch:test");
        assert_eq!(envelope.declared_id(), Ok(Some("patch:test")));
    }

    #[test]
    fn parse_revision_envelope_exposes_typed_derived_id() {
        let value = json!({
            "type": "revision",
            "version": "mycel/0.1",
            "revision_id": "rev:test",
            "author": "pk:ed25519:test"
        });

        let envelope = parse_object_envelope(&value).expect("revision envelope should parse");
        let declared = envelope
            .declared_derived_id()
            .expect("revision derived ID should parse")
            .expect("revision should expose a derived ID");
        assert_eq!(declared.field, "revision_id");
        assert_eq!(declared.prefix, "rev");
        assert_eq!(declared.value, "rev:test");
    }

    #[test]
    fn parse_view_envelope_exposes_typed_derived_id() {
        let value = json!({
            "type": "view",
            "version": "mycel/0.1",
            "view_id": "view:test",
            "maintainer": "pk:ed25519:test"
        });

        let envelope = parse_object_envelope(&value).expect("view envelope should parse");
        let declared = envelope
            .declared_derived_id()
            .expect("view derived ID should parse")
            .expect("view should expose a derived ID");
        assert_eq!(declared.field, "view_id");
        assert_eq!(declared.prefix, "view");
        assert_eq!(declared.value, "view:test");
    }

    #[test]
    fn parse_snapshot_envelope_exposes_typed_derived_id() {
        let value = json!({
            "type": "snapshot",
            "version": "mycel/0.1",
            "snapshot_id": "snap:test",
            "created_by": "pk:ed25519:test"
        });

        let envelope = parse_object_envelope(&value).expect("snapshot envelope should parse");
        let declared = envelope
            .declared_derived_id()
            .expect("snapshot derived ID should parse")
            .expect("snapshot should expose a derived ID");
        assert_eq!(declared.field, "snapshot_id");
        assert_eq!(declared.prefix, "snap");
        assert_eq!(declared.value, "snap:test");
    }

    #[test]
    fn parse_block_envelope_reports_wrong_logical_id_type() {
        let value = json!({
            "type": "block",
            "version": "mycel/0.1",
            "block_id": 7
        });

        let envelope = parse_object_envelope(&value).expect("block envelope should parse");
        assert_eq!(envelope.kind(), ObjectKind::Block);
        assert_eq!(envelope.logical_id(), Err(StringFieldError::WrongType));
    }

    #[test]
    fn parse_patch_envelope_reports_wrong_derived_id_type() {
        let value = json!({
            "type": "patch",
            "version": "mycel/0.1",
            "patch_id": 7,
            "author": "pk:ed25519:test"
        });

        let envelope = parse_object_envelope(&value).expect("patch envelope should parse");
        assert_eq!(
            envelope.declared_derived_id(),
            Err(StringFieldError::WrongType)
        );
    }

    #[test]
    fn parse_object_envelope_rejects_non_object_values() {
        let value = json!(["not-an-object"]);
        assert_eq!(
            parse_object_envelope(&value).unwrap_err(),
            ParseObjectEnvelopeError::TopLevelNotObject
        );
    }

    #[test]
    fn required_string_field_reports_missing_and_wrong_type() {
        let value = json!({
            "type": "document",
            "version": 1
        });

        let envelope = parse_object_envelope(&value).expect("document should parse");
        assert_eq!(
            envelope.optional_string_field("version"),
            Err(StringFieldError::WrongType)
        );
        assert_eq!(
            envelope.required_string_field("missing"),
            Err(StringFieldError::Missing)
        );
    }

    #[test]
    fn parse_patch_object_reads_ops_and_new_block() {
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
                        "content": "Hello",
                        "attrs": {},
                        "children": []
                    }
                }
            ]
        }))
        .expect("patch should parse");

        assert_eq!(patch.patch_id, "patch:test");
        assert_eq!(patch.ops.len(), 1);
    }

    #[test]
    fn parse_patch_object_rejects_move_without_destination() {
        let error = parse_patch_object(&json!({
            "type": "patch",
            "version": "mycel/0.1",
            "patch_id": "patch:test",
            "doc_id": "doc:test",
            "base_revision": "rev:base",
            "author": "pk:ed25519:test",
            "timestamp": 1u64,
            "ops": [
                {
                    "op": "move_block",
                    "block_id": "blk:001"
                }
            ]
        }))
        .unwrap_err();

        assert_eq!(
            error.to_string(),
            "move_block requires at least one destination reference"
        );
    }

    #[test]
    fn parse_patch_object_rejects_empty_metadata_entries() {
        let error = parse_patch_object(&json!({
            "type": "patch",
            "version": "mycel/0.1",
            "patch_id": "patch:test",
            "doc_id": "doc:test",
            "base_revision": "rev:base",
            "author": "pk:ed25519:test",
            "timestamp": 1u64,
            "ops": [
                {
                    "op": "set_metadata",
                    "metadata": {}
                }
            ]
        }))
        .unwrap_err();

        assert_eq!(error.to_string(), "top-level 'metadata' must not be empty");
    }

    #[test]
    fn parse_patch_object_rejects_wrong_block_reference_prefix() {
        let error = parse_patch_object(&json!({
            "type": "patch",
            "version": "mycel/0.1",
            "patch_id": "patch:test",
            "doc_id": "doc:test",
            "base_revision": "rev:base",
            "author": "pk:ed25519:test",
            "timestamp": 1u64,
            "ops": [
                {
                    "op": "replace_block",
                    "block_id": "paragraph-1",
                    "new_content": "Hello"
                }
            ]
        }))
        .unwrap_err();

        assert_eq!(
            error.to_string(),
            "top-level 'block_id' must use 'blk:' prefix"
        );
    }

    #[test]
    fn parse_document_object_reads_identity_and_baseline_fields() {
        let document = parse_document_object(&json!({
            "type": "document",
            "version": "mycel/0.1",
            "doc_id": "doc:test",
            "title": "Origin Text",
            "language": "zh-Hant",
            "content_model": "block-tree",
            "created_at": 1u64,
            "created_by": "pk:ed25519:test",
            "genesis_revision": "rev:test"
        }))
        .expect("document should parse");

        assert_eq!(document.doc_id, "doc:test");
        assert_eq!(document.content_model, "block-tree");
        assert_eq!(document.genesis_revision, "rev:test");
    }

    #[test]
    fn parse_document_object_rejects_wrong_doc_id_prefix() {
        let error = parse_document_object(&json!({
            "type": "document",
            "version": "mycel/0.1",
            "doc_id": "revision:test",
            "title": "Origin Text",
            "language": "zh-Hant",
            "content_model": "block-tree",
            "created_at": 1u64,
            "created_by": "pk:ed25519:test",
            "genesis_revision": "rev:test"
        }))
        .unwrap_err();

        assert_eq!(
            error.to_string(),
            "top-level 'doc_id' must use 'doc:' prefix"
        );
    }

    #[test]
    fn parse_document_object_rejects_wrong_content_model() {
        let error = parse_document_object(&json!({
            "type": "document",
            "version": "mycel/0.1",
            "doc_id": "doc:test",
            "title": "Origin Text",
            "language": "zh-Hant",
            "content_model": "markdown",
            "created_at": 1u64,
            "created_by": "pk:ed25519:test",
            "genesis_revision": "rev:test"
        }))
        .unwrap_err();

        assert_eq!(
            error.to_string(),
            "top-level 'content_model' must equal 'block-tree'"
        );
    }

    #[test]
    fn parse_revision_object_reads_parent_and_patch_ids() {
        let revision = parse_revision_object(&json!({
            "type": "revision",
            "version": "mycel/0.1",
            "revision_id": "rev:test",
            "doc_id": "doc:test",
            "parents": ["rev:base"],
            "patches": ["patch:test"],
            "state_hash": "hash:test",
            "author": "pk:ed25519:test",
            "timestamp": 2u64
        }))
        .expect("revision should parse");

        assert_eq!(revision.parents, vec!["rev:base"]);
        assert_eq!(revision.patches, vec!["patch:test"]);
    }

    #[test]
    fn parse_revision_object_rejects_wrong_parent_prefix() {
        let error = parse_revision_object(&json!({
            "type": "revision",
            "version": "mycel/0.1",
            "revision_id": "rev:test",
            "doc_id": "doc:test",
            "parents": ["patch:base"],
            "patches": ["patch:test"],
            "state_hash": "hash:test",
            "author": "pk:ed25519:test",
            "timestamp": 2u64
        }))
        .unwrap_err();

        assert_eq!(
            error.to_string(),
            "top-level 'parents[0]' must use 'rev:' prefix"
        );
    }

    #[test]
    fn parse_block_object_requires_attrs_and_children() {
        let error = parse_block_object(&json!({
            "block_id": "blk:001",
            "block_type": "paragraph",
            "content": "Hello"
        }))
        .unwrap_err();

        assert_eq!(error.to_string(), "missing object field 'attrs'");
    }

    #[test]
    fn parse_block_object_rejects_wrong_block_prefix() {
        let error = parse_block_object(&json!({
            "block_id": "paragraph-1",
            "block_type": "paragraph",
            "content": "Hello",
            "attrs": {},
            "children": []
        }))
        .unwrap_err();

        assert_eq!(
            error.to_string(),
            "top-level 'block_id' must use 'blk:' prefix"
        );
    }

    #[test]
    fn parse_block_object_rejects_unknown_block_type() {
        let error = parse_block_object(&json!({
            "block_id": "blk:001",
            "block_type": "table",
            "content": "Hello",
            "attrs": {},
            "children": []
        }))
        .unwrap_err();

        assert_eq!(
            error.to_string(),
            "top-level 'block_type' must be one of: title, heading, paragraph, quote, verse, list, annotation, metadata"
        );
    }

    #[test]
    fn parse_view_object_reads_documents_and_policy() {
        let view = parse_view_object(&json!({
            "type": "view",
            "version": "mycel/0.1",
            "view_id": "view:test",
            "maintainer": "pk:ed25519:test",
            "documents": {
                "doc:test": "rev:test"
            },
            "policy": {
                "merge_rule": "manual-reviewed"
            },
            "timestamp": 7u64
        }))
        .expect("view should parse");

        assert_eq!(view.view_id, "view:test");
        assert_eq!(
            view.documents.get("doc:test").map(String::as_str),
            Some("rev:test")
        );
        assert!(view.policy.is_object());
    }

    #[test]
    fn parse_view_object_rejects_empty_documents() {
        let error = parse_view_object(&json!({
            "type": "view",
            "version": "mycel/0.1",
            "view_id": "view:test",
            "maintainer": "pk:ed25519:test",
            "documents": {},
            "policy": {
                "merge_rule": "manual-reviewed"
            },
            "timestamp": 7u64
        }))
        .unwrap_err();

        assert_eq!(error.to_string(), "top-level 'documents' must not be empty");
    }

    #[test]
    fn parse_view_object_rejects_non_object_policy() {
        let error = parse_view_object(&json!({
            "type": "view",
            "version": "mycel/0.1",
            "view_id": "view:test",
            "maintainer": "pk:ed25519:test",
            "documents": {
                "doc:test": "rev:test"
            },
            "policy": "manual-reviewed",
            "timestamp": 7u64
        }))
        .unwrap_err();

        assert_eq!(error.to_string(), "top-level 'policy' must be an object");
    }

    #[test]
    fn parse_view_object_rejects_wrong_document_value_prefix() {
        let error = parse_view_object(&json!({
            "type": "view",
            "version": "mycel/0.1",
            "view_id": "view:test",
            "maintainer": "pk:ed25519:test",
            "documents": {
                "doc:test": "patch:test"
            },
            "policy": {
                "merge_rule": "manual-reviewed"
            },
            "timestamp": 7u64
        }))
        .unwrap_err();

        assert_eq!(
            error.to_string(),
            "top-level 'documents.doc:test' must use 'rev:' prefix"
        );
    }

    #[test]
    fn parse_snapshot_object_reads_documents_and_included_objects() {
        let snapshot = parse_snapshot_object(&json!({
            "type": "snapshot",
            "version": "mycel/0.1",
            "snapshot_id": "snap:test",
            "documents": {
                "doc:test": "rev:test"
            },
            "included_objects": ["rev:test", "patch:test"],
            "root_hash": "hash:test",
            "created_by": "pk:ed25519:test",
            "timestamp": 9u64
        }))
        .expect("snapshot should parse");

        assert_eq!(snapshot.snapshot_id, "snap:test");
        assert_eq!(snapshot.included_objects, vec!["rev:test", "patch:test"]);
        assert_eq!(
            snapshot.documents.get("doc:test").map(String::as_str),
            Some("rev:test")
        );
    }

    #[test]
    fn parse_snapshot_object_rejects_empty_documents() {
        let error = parse_snapshot_object(&json!({
            "type": "snapshot",
            "version": "mycel/0.1",
            "snapshot_id": "snap:test",
            "documents": {},
            "included_objects": ["rev:test", "patch:test"],
            "root_hash": "hash:test",
            "created_by": "pk:ed25519:test",
            "timestamp": 9u64
        }))
        .unwrap_err();

        assert_eq!(error.to_string(), "top-level 'documents' must not be empty");
    }

    #[test]
    fn parse_snapshot_object_rejects_empty_included_object_entry() {
        let error = parse_snapshot_object(&json!({
            "type": "snapshot",
            "version": "mycel/0.1",
            "snapshot_id": "snap:test",
            "documents": {
                "doc:test": "rev:test"
            },
            "included_objects": ["rev:test", ""],
            "root_hash": "hash:test",
            "created_by": "pk:ed25519:test",
            "timestamp": 9u64
        }))
        .unwrap_err();

        assert_eq!(
            error.to_string(),
            "top-level 'included_objects[1]' must not be an empty string"
        );
    }

    #[test]
    fn parse_snapshot_object_rejects_non_canonical_included_object_id() {
        let error = parse_snapshot_object(&json!({
            "type": "snapshot",
            "version": "mycel/0.1",
            "snapshot_id": "snap:test",
            "documents": {
                "doc:test": "rev:test"
            },
            "included_objects": ["doc:test"],
            "root_hash": "hash:test",
            "created_by": "pk:ed25519:test",
            "timestamp": 9u64
        }))
        .unwrap_err();

        assert_eq!(
            error.to_string(),
            "top-level 'included_objects[0]' must use a canonical object ID prefix"
        );
    }

    #[test]
    fn parse_snapshot_object_requires_declared_document_revision_in_included_objects() {
        let error = parse_snapshot_object(&json!({
            "type": "snapshot",
            "version": "mycel/0.1",
            "snapshot_id": "snap:test",
            "documents": {
                "doc:test": "rev:test"
            },
            "included_objects": ["patch:test"],
            "root_hash": "hash:test",
            "created_by": "pk:ed25519:test",
            "timestamp": 9u64
        }))
        .unwrap_err();

        assert_eq!(
            error.to_string(),
            "top-level 'included_objects' must include revision 'rev:test' declared by 'documents.doc:test'"
        );
    }

    #[test]
    fn canonical_json_is_sorted_and_compact() {
        let canonical = canonical_json(&json!({
            "z": 2,
            "a": [true, {"b": "x", "a": 1}]
        }))
        .expect("canonical JSON should render");

        assert_eq!(canonical, "{\"a\":[true,{\"a\":1,\"b\":\"x\"}],\"z\":2}");
    }

    #[test]
    fn recompute_object_id_omits_signature_and_derived_id_field() {
        let value = json!({
            "type": "patch",
            "version": "mycel/0.1",
            "patch_id": "patch:declared",
            "doc_id": "doc:test",
            "base_revision": "rev:genesis-null",
            "author": "pk:ed25519:test",
            "timestamp": 1u64,
            "ops": [],
            "signature": "sig:ed25519:test"
        });

        let recomputed =
            recompute_object_id(&value, "patch_id", "patch").expect("patch ID should recompute");
        assert!(recomputed.starts_with("patch:"));
        assert_ne!(recomputed, "patch:declared");
    }
}
