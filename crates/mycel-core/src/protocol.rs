//! Shared protocol-facing types for the first Rust workspace cut.
//!
//! This module intentionally starts with a narrow typed object model instead of
//! a full canonical object graph. The current goal is to keep object-kind,
//! signing, and derived-ID knowledge in one place so the verifier and later
//! protocol layers do not each re-encode the same rules.

use std::fmt;
use std::str::FromStr;

use serde::{Deserialize, Serialize};
use serde_json::{Map, Value};

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
        match self.schema.derived_id_field {
            Some(field) => required_string_field(self.object, field).map(Some),
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

#[cfg(test)]
mod tests {
    use serde_json::json;

    use super::{
        object_schema, parse_object_envelope, ObjectKind, ParseObjectEnvelopeError, SignatureRule,
        StringFieldError,
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
}
