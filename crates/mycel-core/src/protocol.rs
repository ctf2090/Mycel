//! Shared protocol-facing types for the first Rust workspace cut.
//!
//! This module intentionally starts with a narrow typed object model instead of
//! a full canonical object graph. The current goal is to keep object-kind,
//! signing, and derived-ID knowledge in one place so the verifier and later
//! protocol layers do not each re-encode the same rules.

use std::collections::BTreeMap;
use std::fmt;
use std::str::FromStr;

use serde::de::{self, DeserializeOwned, MapAccess, SeqAccess, Visitor};
use serde::{Deserialize, Deserializer, Serialize};
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
    pub merge_strategy: Option<String>,
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
    reject_unknown_fields(
        object,
        "top-level",
        &[
            "type",
            "version",
            "doc_id",
            "title",
            "language",
            "content_model",
            "created_at",
            "created_by",
            "genesis_revision",
        ],
    )?;
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
    parse_block_object_with_context(value, "top-level")
}

fn parse_block_object_with_context(
    value: &Value,
    context: &str,
) -> Result<BlockObject, TypedObjectError> {
    let object = value
        .as_object()
        .ok_or_else(|| TypedObjectError::new(format!("{context} must be a JSON object")))?;
    reject_unknown_fields(
        object,
        context,
        &[
            "type",
            "version",
            "block_id",
            "block_type",
            "content",
            "attrs",
            "children",
        ],
    )?;
    let block_type = required_string(object, "block_type")?;
    validate_block_type(&block_type)?;

    Ok(BlockObject {
        block_id: required_prefixed_string(object, "block_id", "blk:")?,
        block_type,
        content: required_string(object, "content")?,
        attrs: required_object(object, "attrs")?,
        children: required_array(object, "children")?
            .iter()
            .enumerate()
            .map(|(index, child)| {
                parse_block_object_with_context(child, &format!("{context} 'children[{index}]'"))
            })
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
    reject_unknown_fields(
        object,
        "top-level",
        &[
            "type",
            "version",
            "patch_id",
            "doc_id",
            "base_revision",
            "author",
            "timestamp",
            "ops",
            "signature",
        ],
    )?;
    Ok(PatchObject {
        patch_id: required_prefixed_string(object, "patch_id", "patch:")?,
        doc_id: required_prefixed_string(object, "doc_id", "doc:")?,
        base_revision: required_prefixed_string(object, "base_revision", "rev:")?,
        author: required_prefixed_string(object, "author", "pk:")?,
        timestamp: required_u64(object, "timestamp")?,
        ops: required_array(object, "ops")?
            .iter()
            .enumerate()
            .map(|(index, operation)| {
                parse_patch_operation(operation)
                    .map_err(|error| prepend_context(error, &format!("top-level 'ops[{index}]'")))
            })
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
    reject_unknown_fields(
        object,
        "top-level",
        &[
            "type",
            "version",
            "revision_id",
            "doc_id",
            "parents",
            "patches",
            "merge_strategy",
            "state_hash",
            "author",
            "timestamp",
            "signature",
        ],
    )?;
    let parents = required_prefixed_string_array(object, "parents", "rev:")?;
    let merge_strategy = optional_string(object, "merge_strategy")?;
    if parents.is_empty() {
        if merge_strategy.is_some() {
            return Err(TypedObjectError::new(
                "top-level 'merge_strategy' is not allowed when 'parents' is empty",
            ));
        }
    } else if parents.len() == 1 {
        if merge_strategy.is_some() {
            return Err(TypedObjectError::new(
                "top-level 'merge_strategy' requires multiple parents",
            ));
        }
    } else if merge_strategy.is_none() {
        return Err(TypedObjectError::new(
            "top-level 'merge_strategy' is required when 'parents' has multiple entries",
        ));
    }

    Ok(RevisionObject {
        revision_id: required_prefixed_string(object, "revision_id", "rev:")?,
        doc_id: required_prefixed_string(object, "doc_id", "doc:")?,
        parents,
        patches: required_prefixed_string_array(object, "patches", "patch:")?,
        merge_strategy,
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
    reject_unknown_fields(
        object,
        "top-level",
        &[
            "type",
            "version",
            "view_id",
            "maintainer",
            "documents",
            "policy",
            "timestamp",
            "signature",
        ],
    )?;
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
    reject_unknown_fields(
        object,
        "top-level",
        &[
            "type",
            "version",
            "snapshot_id",
            "documents",
            "included_objects",
            "root_hash",
            "created_by",
            "timestamp",
            "signature",
        ],
    )?;
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
    prefixed_canonical_object_hash_excluding_fields(value, prefix, &[derived_id_field, "signature"])
}

pub fn signed_payload_bytes(value: &Value) -> Result<Vec<u8>, String> {
    signature_payload_bytes_for_field(value, "signature")
}

pub fn wire_envelope_signed_payload_bytes(value: &Value) -> Result<Vec<u8>, String> {
    signature_payload_bytes_for_field(value, "sig")
}

pub fn signature_payload_bytes_for_field(
    value: &Value,
    signature_field: &str,
) -> Result<Vec<u8>, String> {
    let canonical = canonical_object_json_excluding_fields(value, &[signature_field])?;
    Ok(canonical.into_bytes())
}

pub fn parse_json_value_strict(input: &str) -> Result<Value, String> {
    let mut deserializer = serde_json::Deserializer::from_str(input);
    let value = StrictJsonValue::deserialize(&mut deserializer)
        .map(|value| value.0)
        .map_err(|error| error.to_string())?;
    deserializer.end().map_err(|error| error.to_string())?;
    Ok(value)
}

pub fn parse_json_strict<T>(input: &str) -> Result<T, String>
where
    T: DeserializeOwned,
{
    let value = parse_json_value_strict(input)?;
    ensure_supported_json_values(&value)?;
    serde_json::from_value(value).map_err(|error| error.to_string())
}

pub fn ensure_supported_json_values(value: &Value) -> Result<(), String> {
    let mut errors = Vec::new();
    collect_unsupported_json_value_errors(value, "$", &mut errors);
    match errors.into_iter().next() {
        Some(error) => Err(error),
        None => Ok(()),
    }
}

pub fn collect_unsupported_json_value_errors(value: &Value, path: &str, errors: &mut Vec<String>) {
    match value {
        Value::Null => errors.push(format!("{path}: null is not allowed")),
        Value::Bool(_) | Value::String(_) => {}
        Value::Number(number) => {
            if !(number.is_i64() || number.is_u64()) {
                errors.push(format!(
                    "{path}: floating-point numbers are not allowed in canonical objects"
                ));
            }
        }
        Value::Array(values) => {
            for (index, entry) in values.iter().enumerate() {
                let entry_path = format!("{path}[{index}]");
                collect_unsupported_json_value_errors(entry, &entry_path, errors);
            }
        }
        Value::Object(entries) => {
            for (key, entry) in entries {
                let entry_path = format!("{path}.{key}");
                collect_unsupported_json_value_errors(entry, &entry_path, errors);
            }
        }
    }
}

pub fn canonical_json(value: &Value) -> Result<String, String> {
    let mut output = String::new();
    write_canonical_json(value, &mut output)?;
    Ok(output)
}

pub fn canonical_object_json_excluding_fields(
    value: &Value,
    omitted_fields: &[&str],
) -> Result<String, String> {
    let object = object_without_fields(value, omitted_fields)?;
    canonical_json(&Value::Object(object))
}

pub fn canonical_sha256_hex(value: &Value) -> Result<String, String> {
    let canonical = canonical_json(value)?;
    let mut hasher = Sha256::new();
    hasher.update(canonical.as_bytes());
    let digest = hasher.finalize();
    Ok(hex_encode(&digest))
}

pub fn canonical_object_sha256_hex_excluding_fields(
    value: &Value,
    omitted_fields: &[&str],
) -> Result<String, String> {
    let canonical = canonical_object_json_excluding_fields(value, omitted_fields)?;
    let mut hasher = Sha256::new();
    hasher.update(canonical.as_bytes());
    let digest = hasher.finalize();
    Ok(hex_encode(&digest))
}

pub fn prefixed_canonical_hash(value: &Value, prefix: &str) -> Result<String, String> {
    let digest = canonical_sha256_hex(value)?;
    Ok(format!("{prefix}:{digest}"))
}

pub fn prefixed_canonical_object_hash_excluding_fields(
    value: &Value,
    prefix: &str,
    omitted_fields: &[&str],
) -> Result<String, String> {
    let digest = canonical_object_sha256_hex_excluding_fields(value, omitted_fields)?;
    Ok(format!("{prefix}:{digest}"))
}

#[derive(Debug)]
struct StrictJsonValue(Value);

impl<'de> Deserialize<'de> for StrictJsonValue {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        deserializer.deserialize_any(StrictJsonVisitor)
    }
}

struct StrictJsonVisitor;

impl<'de> Visitor<'de> for StrictJsonVisitor {
    type Value = StrictJsonValue;

    fn expecting(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str("valid JSON value without duplicate object keys")
    }

    fn visit_bool<E>(self, value: bool) -> Result<Self::Value, E> {
        Ok(StrictJsonValue(Value::Bool(value)))
    }

    fn visit_i64<E>(self, value: i64) -> Result<Self::Value, E> {
        Ok(StrictJsonValue(Value::Number(value.into())))
    }

    fn visit_u64<E>(self, value: u64) -> Result<Self::Value, E> {
        Ok(StrictJsonValue(Value::Number(value.into())))
    }

    fn visit_f64<E>(self, value: f64) -> Result<Self::Value, E>
    where
        E: de::Error,
    {
        let number = serde_json::Number::from_f64(value)
            .ok_or_else(|| de::Error::custom("invalid floating-point number"))?;
        Ok(StrictJsonValue(Value::Number(number)))
    }

    fn visit_str<E>(self, value: &str) -> Result<Self::Value, E>
    where
        E: de::Error,
    {
        self.visit_string(value.to_string())
    }

    fn visit_string<E>(self, value: String) -> Result<Self::Value, E> {
        Ok(StrictJsonValue(Value::String(value)))
    }

    fn visit_none<E>(self) -> Result<Self::Value, E> {
        Ok(StrictJsonValue(Value::Null))
    }

    fn visit_unit<E>(self) -> Result<Self::Value, E> {
        Ok(StrictJsonValue(Value::Null))
    }

    fn visit_seq<A>(self, mut seq: A) -> Result<Self::Value, A::Error>
    where
        A: SeqAccess<'de>,
    {
        let mut values = Vec::new();
        while let Some(value) = seq.next_element::<StrictJsonValue>()? {
            values.push(value.0);
        }
        Ok(StrictJsonValue(Value::Array(values)))
    }

    fn visit_map<A>(self, mut map: A) -> Result<Self::Value, A::Error>
    where
        A: MapAccess<'de>,
    {
        let mut entries = Map::new();
        while let Some(key) = map.next_key::<String>()? {
            if entries.contains_key(&key) {
                return Err(de::Error::custom(format!("duplicate object key '{key}'")));
            }
            let value = map.next_value::<StrictJsonValue>()?;
            entries.insert(key, value.0);
        }
        Ok(StrictJsonValue(Value::Object(entries)))
    }
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

fn object_without_fields(
    value: &Value,
    omitted_fields: &[&str],
) -> Result<Map<String, Value>, String> {
    let mut object = value
        .as_object()
        .cloned()
        .ok_or_else(|| "top-level JSON value must be an object".to_string())?;
    for field in omitted_fields {
        object.remove(*field);
    }
    Ok(object)
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
        "insert_block" => {
            reject_unknown_fields(
                object,
                "patch op",
                &["op", "parent_block_id", "index", "new_block"],
            )?;
            Ok(PatchOperation::InsertBlock {
                parent_block_id: optional_prefixed_string(object, "parent_block_id", "blk:")?,
                index: optional_usize(object, "index")?,
                new_block: parse_block_field(object, "new_block")?,
            })
        }
        "insert_block_after" => {
            reject_unknown_fields(object, "patch op", &["op", "after_block_id", "new_block"])?;
            Ok(PatchOperation::InsertBlockAfter {
                after_block_id: required_prefixed_string(object, "after_block_id", "blk:")?,
                new_block: parse_block_field(object, "new_block")?,
            })
        }
        "delete_block" => {
            reject_unknown_fields(object, "patch op", &["op", "block_id"])?;
            Ok(PatchOperation::DeleteBlock {
                block_id: required_prefixed_string(object, "block_id", "blk:")?,
            })
        }
        "replace_block" => {
            reject_unknown_fields(object, "patch op", &["op", "block_id", "new_content"])?;
            Ok(PatchOperation::ReplaceBlock {
                block_id: required_prefixed_string(object, "block_id", "blk:")?,
                new_content: required_string(object, "new_content")?,
            })
        }
        "move_block" => {
            reject_unknown_fields(
                object,
                "patch op",
                &["op", "block_id", "parent_block_id", "after_block_id"],
            )?;
            Ok(PatchOperation::MoveBlock {
                block_id: required_prefixed_string(object, "block_id", "blk:")?,
                parent_block_id: optional_prefixed_string(object, "parent_block_id", "blk:")?,
                after_block_id: optional_prefixed_string(object, "after_block_id", "blk:")?,
            })
        }
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
        "annotate_block" => {
            reject_unknown_fields(object, "patch op", &["op", "block_id", "annotation"])?;
            Ok(PatchOperation::AnnotateBlock {
                block_id: required_prefixed_string(object, "block_id", "blk:")?,
                annotation: parse_block_field(object, "annotation")?,
            })
        }
        "set_metadata" => {
            if object.contains_key("metadata") {
                reject_unknown_fields(object, "patch op", &["op", "metadata"])?;
            } else {
                reject_unknown_fields(object, "patch op", &["op", "key", "value"])?;
            }
            Ok(PatchOperation::SetMetadata {
                entries: parse_metadata_entries(object)?,
            })
        }
        _ => Err(TypedObjectError::new(format!(
            "unsupported patch op '{op}'"
        ))),
    }
}

fn reject_unknown_fields(
    object: &Map<String, Value>,
    scope: &str,
    allowed_fields: &[&str],
) -> Result<(), TypedObjectError> {
    let mut unknown_fields = object
        .keys()
        .filter(|field| !allowed_fields.contains(&field.as_str()))
        .cloned()
        .collect::<Vec<_>>();
    unknown_fields.sort_unstable();

    match unknown_fields.as_slice() {
        [] => Ok(()),
        [field] => Err(TypedObjectError::new(format!(
            "{scope} contains unexpected field '{field}'"
        ))),
        fields => Err(TypedObjectError::new(format!(
            "{scope} contains unexpected fields: {}",
            fields
                .iter()
                .map(|field| format!("'{field}'"))
                .collect::<Vec<_>>()
                .join(", ")
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
        .map_err(|error| prepend_context(error, &format!("top-level '{field}'")))
}

fn prepend_context(error: TypedObjectError, context: &str) -> TypedObjectError {
    TypedObjectError::new(format!("{context}: {error}"))
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
    reject_duplicate_strings(&values, field)?;
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
    reject_duplicate_strings(&values, field)?;
    Ok(values)
}

fn reject_duplicate_strings(values: &[String], field: &str) -> Result<(), TypedObjectError> {
    let mut first_seen = BTreeMap::new();
    for (index, value) in values.iter().enumerate() {
        if let Some(first_index) = first_seen.insert(value.as_str(), index) {
            return Err(TypedObjectError::new(format!(
                "top-level '{field}[{index}]' duplicates '{field}[{first_index}]'"
            )));
        }
    }
    Ok(())
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
    use std::collections::BTreeMap;

    use rstest::rstest;
    use serde_json::{json, Map, Value};

    use super::{
        canonical_json, canonical_object_json_excluding_fields, canonical_sha256_hex,
        ensure_supported_json_values, object_schema, parse_block_object, parse_document_object,
        parse_json_strict, parse_json_value_strict, parse_object_envelope, parse_patch_object,
        parse_revision_object, parse_snapshot_object, parse_view_object, prefixed_canonical_hash,
        prefixed_canonical_object_hash_excluding_fields, recompute_object_id,
        reject_duplicate_strings, reject_unknown_fields, required_non_empty_string_array,
        required_object, required_prefixed_string_map, required_string_field, signed_payload_bytes,
        validate_canonical_object_id, validate_prefixed_string, wire_envelope_signed_payload_bytes,
        ObjectKind, ParseObjectEnvelopeError, SignatureRule, StringFieldError,
        WIRE_PROTOCOL_VERSION,
    };

    #[path = "fixtures.rs"]
    mod fixtures;

    #[path = "json_and_ids.rs"]
    mod json_and_ids;
    #[path = "object_parsing.rs"]
    mod object_parsing;
    #[path = "schema_envelope.rs"]
    mod schema_envelope;
    #[path = "spec_examples.rs"]
    mod spec_examples;
}
