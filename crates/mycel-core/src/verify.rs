use std::collections::HashMap;
use std::fs;
use std::path::{Path, PathBuf};

use base64::Engine;
use ed25519_dalek::{Signature, Verifier, VerifyingKey};
use serde::Serialize;
use serde_json::Value;

use crate::protocol::{
    collect_unsupported_json_value_errors, parse_block_object, parse_document_object,
    parse_json_value_strict, parse_object_envelope, parse_patch_object, parse_revision_object,
    parse_snapshot_object, parse_view_object, recompute_object_id, signed_payload_bytes,
    ParseObjectEnvelopeError, StringFieldError,
};
use crate::replay::replay_revision_from_index;

#[derive(Debug, Clone, Serialize)]
pub struct ObjectVerificationSummary {
    pub path: PathBuf,
    pub status: String,
    pub object_type: Option<String>,
    pub signature_rule: Option<String>,
    pub signer_field: Option<String>,
    pub signer: Option<String>,
    pub signature_verification: Option<String>,
    pub declared_id: Option<String>,
    pub recomputed_id: Option<String>,
    pub declared_state_hash: Option<String>,
    pub recomputed_state_hash: Option<String>,
    pub state_hash_verification: Option<String>,
    pub notes: Vec<String>,
    pub errors: Vec<String>,
}

#[derive(Debug, Clone, Serialize)]
pub struct ObjectInspectionSummary {
    pub path: PathBuf,
    pub status: String,
    pub object_type: Option<String>,
    pub version: Option<String>,
    pub signature_rule: Option<String>,
    pub signer_field: Option<String>,
    pub signer: Option<String>,
    pub declared_id_field: Option<String>,
    pub declared_id: Option<String>,
    pub has_signature: bool,
    pub top_level_keys: Vec<String>,
    pub notes: Vec<String>,
    pub errors: Vec<String>,
}

impl ObjectVerificationSummary {
    fn new(path: &Path) -> Self {
        Self {
            path: path.to_path_buf(),
            status: "ok".to_string(),
            object_type: None,
            signature_rule: None,
            signer_field: None,
            signer: None,
            signature_verification: None,
            declared_id: None,
            recomputed_id: None,
            declared_state_hash: None,
            recomputed_state_hash: None,
            state_hash_verification: None,
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
}

impl ObjectInspectionSummary {
    fn new(path: &Path) -> Self {
        Self {
            path: path.to_path_buf(),
            status: "ok".to_string(),
            object_type: None,
            version: None,
            signature_rule: None,
            signer_field: None,
            signer: None,
            declared_id_field: None,
            declared_id: None,
            has_signature: false,
            top_level_keys: Vec::new(),
            notes: Vec::new(),
            errors: Vec::new(),
        }
    }

    pub fn is_failed(&self) -> bool {
        !self.errors.is_empty()
    }

    fn push_note(&mut self, message: impl Into<String>) {
        self.notes.push(message.into());
        self.refresh_status();
    }

    fn push_error(&mut self, message: impl Into<String>) {
        self.errors.push(message.into());
        self.refresh_status();
    }

    fn refresh_status(&mut self) {
        self.status = if !self.errors.is_empty() {
            "failed".to_string()
        } else if !self.notes.is_empty() {
            "warning".to_string()
        } else {
            "ok".to_string()
        };
    }
}

pub fn inspect_object_path(path: &Path) -> ObjectInspectionSummary {
    let mut summary = ObjectInspectionSummary::new(path);

    let content = match fs::read_to_string(path) {
        Ok(content) => content,
        Err(err) => {
            summary.push_error(format!("failed to read object file: {err}"));
            return summary;
        }
    };

    let value: Value = match parse_json_value_strict(&content) {
        Ok(value) => value,
        Err(err) => {
            summary.push_error(format!("failed to parse JSON: {err}"));
            return summary;
        }
    };

    inspect_object_value_with_summary(path, value, summary)
}

pub fn verify_object_path(path: &Path) -> ObjectVerificationSummary {
    let mut summary = ObjectVerificationSummary::new(path);

    let content = match fs::read_to_string(path) {
        Ok(content) => content,
        Err(err) => {
            summary.push_error(format!("failed to read object file: {err}"));
            return summary;
        }
    };

    let value: Value = match parse_json_value_strict(&content) {
        Ok(value) => value,
        Err(err) => {
            summary.push_error(format!("failed to parse JSON: {err}"));
            return summary;
        }
    };

    verify_object_value_with_summary(path, value, summary, None)
}

pub fn verify_object_value(value: &Value) -> ObjectVerificationSummary {
    verify_object_value_with_object_index(value, None)
}

pub fn verify_object_value_with_object_index(
    value: &Value,
    object_index: Option<&HashMap<String, Value>>,
) -> ObjectVerificationSummary {
    verify_object_value_with_summary(
        Path::new("<inline-object>"),
        value.clone(),
        ObjectVerificationSummary::new(Path::new("<inline-object>")),
        object_index,
    )
}

fn verify_object_value_with_summary(
    path: &Path,
    value: Value,
    mut summary: ObjectVerificationSummary,
    object_index: Option<&HashMap<String, Value>>,
) -> ObjectVerificationSummary {
    summary.path = path.to_path_buf();

    collect_unsupported_json_value_errors(&value, "$", &mut summary.errors);
    if !summary.errors.is_empty() {
        summary.status = "failed".to_string();
        return summary;
    }

    let object = match value.as_object() {
        Some(object) => object,
        None => {
            summary.push_error("top-level JSON value must be an object");
            return summary;
        }
    };
    if let Some(object_type) = object.get("type").and_then(Value::as_str) {
        summary.object_type = Some(object_type.to_string());
    }

    let envelope = match parse_object_envelope(&value) {
        Ok(envelope) => envelope,
        Err(error) => {
            summary.push_error(error.to_string());
            return summary;
        }
    };
    let object_type = envelope.object_type();
    let schema = envelope.schema();

    summary.signature_rule = Some(schema.signature_rule.to_string());
    summary.signature_verification = Some(if schema.signature_rule.is_required() {
        "failed".to_string()
    } else {
        "not_applicable".to_string()
    });

    if let Some(signer_field) = schema.signer_field {
        summary.signer_field = Some(signer_field.to_string());
    }

    if let Some(logical_id_field) = schema.logical_id_field() {
        match envelope.logical_id() {
            Ok(Some(_)) => {}
            Err(StringFieldError::Missing) => summary.push_error(format!(
                "{object_type} object is missing string field '{logical_id_field}'"
            )),
            Err(StringFieldError::WrongType) => {
                summary.push_error(format!("top-level '{logical_id_field}' must be a string"))
            }
            Ok(None) => {}
        }
    }

    if object_type == "revision" {
        match envelope.required_string_field("state_hash") {
            Ok(state_hash) => summary.declared_state_hash = Some(state_hash.to_string()),
            Err(StringFieldError::Missing) => {
                summary.push_error("revision object is missing string field 'state_hash'")
            }
            Err(StringFieldError::WrongType) => {
                summary.push_error("top-level 'state_hash' must be a string")
            }
        }
    }

    if schema.signature_rule.is_required() {
        let Some(signature) = object.get("signature") else {
            summary.push_error(format!(
                "{object_type} object is missing required top-level 'signature'"
            ));
            return finalize_signed_summary(summary);
        };

        if !signature.is_string() {
            summary.push_error("top-level 'signature' must be a string");
        }

        let mut signer_value = None;
        if let Some(signer_field) = schema.signer_field {
            match envelope.signer() {
                Ok(Some(signer)) => {
                    summary.signer = Some(signer.to_string());
                    signer_value = Some(signer);
                }
                Err(StringFieldError::Missing) => summary.push_error(format!(
                    "{object_type} object is missing string signer field '{signer_field}'"
                )),
                Err(StringFieldError::WrongType) => {
                    summary.push_error(format!("top-level '{signer_field}' must be a string"))
                }
                Ok(None) => {}
            }
        }

        if summary.errors.is_empty() {
            let signer_value = signer_value.expect("signer should exist when errors are empty");
            match verify_object_signature(
                &value,
                signer_value,
                signature.as_str().unwrap_or_default(),
            ) {
                Ok(()) => summary.signature_verification = Some("verified".to_string()),
                Err(err) => summary.push_error(err),
            }
        }
    } else if object.contains_key("signature") {
        summary.push_error(format!(
            "{object_type} object must not include top-level 'signature'"
        ));
    }

    if let Some((id_field, prefix)) = schema.derived_id() {
        match envelope.declared_derived_id() {
            Ok(Some(declared)) => {
                summary.declared_id = Some(declared.value.to_string());
            }
            Err(StringFieldError::Missing) => summary.push_error(format!(
                "{object_type} object is missing string field '{id_field}'"
            )),
            Err(StringFieldError::WrongType) => {
                summary.push_error(format!("top-level '{id_field}' must be a string"))
            }
            Ok(None) => {}
        }

        match recompute_object_id(&value, id_field, prefix) {
            Ok(recomputed_id) => {
                summary.recomputed_id = Some(recomputed_id.clone());
                if summary.declared_id.as_deref() != Some(recomputed_id.as_str()) {
                    summary.push_error(format!(
                        "declared {id_field} does not match recomputed canonical object ID"
                    ));
                }
            }
            Err(err) => summary.push_error(err),
        }
    }

    if summary.errors.is_empty() {
        validate_typed_object_shape(object_type, &value, &mut summary);
    }

    if object_type == "revision"
        && summary.errors.is_empty()
        && (path != Path::new("<inline-object>") || object_index.is_some())
    {
        match verify_revision_replay(path, &value, object_index) {
            Ok(recomputed_state_hash) => {
                summary.recomputed_state_hash = Some(recomputed_state_hash.clone());
                summary.state_hash_verification = Some("verified".to_string());
            }
            Err(err) => {
                summary.state_hash_verification = Some("failed".to_string());
                summary.push_error(err);
            }
        }
    }

    finalize_signed_summary(summary)
}

fn validate_typed_object_shape(
    object_type: &str,
    value: &Value,
    summary: &mut ObjectVerificationSummary,
) {
    let result = match object_type {
        "document" => parse_document_object(value).map(|_| ()),
        "block" => parse_block_object(value).map(|_| ()),
        "patch" => parse_patch_object(value).map(|_| ()),
        "revision" => parse_revision_object(value).map(|_| ()),
        "view" => parse_view_object(value).map(|_| ()),
        "snapshot" => parse_snapshot_object(value).map(|_| ()),
        _ => return,
    };

    if let Err(error) = result {
        summary.push_error(error.to_string());
    }
}

fn inspect_object_value_with_summary(
    path: &Path,
    value: Value,
    mut summary: ObjectInspectionSummary,
) -> ObjectInspectionSummary {
    summary.path = path.to_path_buf();
    let object = match value.as_object() {
        Some(object) => object,
        None => {
            summary.push_error("top-level JSON value must be an object");
            return summary;
        }
    };
    summary.top_level_keys = object.keys().cloned().collect();
    summary.top_level_keys.sort_unstable();
    summary.has_signature = object.contains_key("signature");

    match object.get("version") {
        Some(Value::String(version)) => summary.version = Some(version.clone()),
        Some(_) => summary.push_note("top-level 'version' should be a string"),
        None => {}
    }

    if let Some(object_type) = object.get("type").and_then(Value::as_str) {
        summary.object_type = Some(object_type.to_string());
    }

    let envelope = match parse_object_envelope(&value) {
        Ok(envelope) => envelope,
        Err(ParseObjectEnvelopeError::TopLevelNotObject) => {
            summary.push_error("top-level JSON value must be an object");
            return summary;
        }
        Err(error) => {
            summary.push_note(error.to_string());
            return summary;
        }
    };
    let object = envelope.object();
    let object_type = envelope.object_type();
    let schema = envelope.schema();
    summary.object_type = Some(object_type.to_string());

    summary.signature_rule = Some(schema.signature_rule.to_string());
    if let Some(logical_id_field) = schema.logical_id_field() {
        match envelope.logical_id() {
            Ok(Some(_)) => {}
            Err(StringFieldError::WrongType) => {
                summary.push_note(format!("top-level '{logical_id_field}' should be a string"))
            }
            Err(StringFieldError::Missing) => summary.push_note(format!(
                "{object_type} object is missing string field '{logical_id_field}'"
            )),
            Ok(None) => {}
        }
    }

    if let Some(signer_field) = schema.signer_field {
        summary.signer_field = Some(signer_field.to_string());
        match envelope.signer() {
            Ok(Some(signer)) => summary.signer = Some(signer.to_string()),
            Err(StringFieldError::WrongType) => {
                summary.push_note(format!("top-level '{signer_field}' should be a string"))
            }
            Err(StringFieldError::Missing) => summary.push_note(format!(
                "{object_type} object is missing string signer field '{signer_field}'"
            )),
            Ok(None) => {}
        }
    }

    if let Some((id_field, _prefix)) = schema.derived_id() {
        summary.declared_id_field = Some(id_field.to_string());
        match envelope.declared_derived_id() {
            Ok(Some(declared)) => summary.declared_id = Some(declared.value.to_string()),
            Err(StringFieldError::WrongType) => {
                summary.push_note(format!("top-level '{id_field}' should be a string"))
            }
            Err(StringFieldError::Missing) => summary.push_note(format!(
                "{object_type} object is missing string field '{id_field}'"
            )),
            Ok(None) => {}
        }
    }

    if schema.signature_rule.is_required() {
        match object.get("signature") {
            Some(Value::String(_)) => {}
            Some(_) => summary.push_note("top-level 'signature' should be a string"),
            None => summary.push_note(format!(
                "{object_type} object is missing top-level 'signature'"
            )),
        }
    } else if object.contains_key("signature") {
        summary.push_note(format!(
            "{object_type} object includes top-level 'signature' even though signatures are forbidden"
        ));
    }

    summary
}

fn verify_revision_replay(
    path: &Path,
    value: &Value,
    object_index: Option<&HashMap<String, Value>>,
) -> Result<String, String> {
    let owned_index;
    let object_index = match object_index {
        Some(index) => index,
        None => {
            owned_index = load_neighbor_object_index(path)?;
            &owned_index
        }
    };

    let replay = replay_revision_from_index(value, object_index)
        .map_err(|error| format!("revision replay failed: {error}"))?;
    let declared_state_hash = value
        .as_object()
        .and_then(|object| object.get("state_hash"))
        .and_then(Value::as_str)
        .ok_or_else(|| "revision replay requires string field 'state_hash'".to_string())?;

    if replay.recomputed_state_hash != declared_state_hash {
        return Err(format!(
            "declared state_hash does not match replayed state hash: expected '{}' but recomputed '{}'",
            declared_state_hash, replay.recomputed_state_hash
        ));
    }

    Ok(replay.recomputed_state_hash)
}

fn load_neighbor_object_index(
    path: &Path,
) -> Result<std::collections::HashMap<String, Value>, String> {
    let parent = path
        .parent()
        .ok_or_else(|| format!("cannot inspect sibling objects for {}", path.display()))?;
    let entries = fs::read_dir(parent).map_err(|err| {
        format!(
            "failed to read object directory {}: {err}",
            parent.display()
        )
    })?;

    let mut object_index = std::collections::HashMap::new();
    for entry in entries {
        let entry = entry.map_err(|err| {
            format!(
                "failed to read object directory entry {}: {err}",
                parent.display()
            )
        })?;
        let entry_path = entry.path();
        if entry_path.extension().and_then(|value| value.to_str()) != Some("json") {
            continue;
        }

        let content = fs::read_to_string(&entry_path).map_err(|err| {
            format!(
                "failed to read sibling object {}: {err}",
                entry_path.display()
            )
        })?;
        let value: Value = parse_json_value_strict(&content).map_err(|err| {
            format!(
                "failed to parse sibling object {}: {err}",
                entry_path.display()
            )
        })?;
        let envelope = match parse_object_envelope(&value) {
            Ok(envelope) => envelope,
            Err(_) => continue,
        };
        let Some(declared_id) = envelope.declared_id().map_err(|_| {
            format!(
                "sibling object {} has invalid declared ID",
                entry_path.display()
            )
        })?
        else {
            continue;
        };
        object_index.insert(declared_id.to_string(), value);
    }

    Ok(object_index)
}

fn finalize_signed_summary(mut summary: ObjectVerificationSummary) -> ObjectVerificationSummary {
    if summary.errors.is_empty() {
        summary.status = "ok".to_string();
    } else {
        summary.status = "failed".to_string();
    }

    summary
}

fn verify_object_signature(value: &Value, signer: &str, signature: &str) -> Result<(), String> {
    let public_key = parse_public_key(signer)?;
    let signature = parse_signature(signature)?;
    let payload = signed_payload_bytes(value)?;

    public_key
        .verify(&payload, &signature)
        .map_err(|err| format!("Ed25519 signature verification failed: {err}"))
}

fn parse_public_key(value: &str) -> Result<VerifyingKey, String> {
    let encoded = value
        .strip_prefix("pk:ed25519:")
        .ok_or_else(|| "signer field must use format 'pk:ed25519:<base64>'".to_string())?;
    let decoded = base64::engine::general_purpose::STANDARD
        .decode(encoded)
        .map_err(|err| format!("failed to decode Ed25519 public key: {err}"))?;
    let bytes: [u8; 32] = decoded
        .try_into()
        .map_err(|_| "Ed25519 public key must decode to 32 bytes".to_string())?;
    VerifyingKey::from_bytes(&bytes)
        .map_err(|err| format!("invalid Ed25519 public key bytes: {err}"))
}

fn parse_signature(value: &str) -> Result<Signature, String> {
    let encoded = value
        .strip_prefix("sig:ed25519:")
        .ok_or_else(|| "signature field must use format 'sig:ed25519:<base64>'".to_string())?;
    let decoded = base64::engine::general_purpose::STANDARD
        .decode(encoded)
        .map_err(|err| format!("failed to decode Ed25519 signature: {err}"))?;
    Signature::from_slice(&decoded).map_err(|err| format!("invalid Ed25519 signature bytes: {err}"))
}

#[cfg(test)]
mod tests {
    use std::collections::HashMap;

    use base64::Engine;
    use ed25519_dalek::{Signer, SigningKey};
    use serde_json::{json, Map, Value};

    use super::{inspect_object_path, verify_object_path, verify_object_value_with_object_index};
    use crate::protocol::BlockObject;
    use crate::replay::{compute_state_hash, DocumentState};

    fn write_test_file(name: &str, content: &str) -> std::path::PathBuf {
        let unique = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .expect("system time should be after unix epoch")
            .as_nanos();
        let path = std::env::temp_dir().join(format!("mycel-core-{name}-{unique}.json"));
        std::fs::write(&path, content).expect("test JSON should be written");
        path
    }

    fn write_test_dir(name: &str) -> std::path::PathBuf {
        let unique = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .expect("system time should be after unix epoch")
            .as_nanos();
        let path = std::env::temp_dir().join(format!("mycel-core-{name}-{unique}"));
        std::fs::create_dir_all(&path).expect("test directory should be created");
        path
    }

    fn signer_material() -> (SigningKey, String) {
        let signing_key = SigningKey::from_bytes(&[7u8; 32]);
        let public_key = format!(
            "pk:ed25519:{}",
            base64::engine::general_purpose::STANDARD
                .encode(signing_key.verifying_key().as_bytes())
        );
        (signing_key, public_key)
    }

    fn sign_value(signing_key: &SigningKey, value: &Value) -> String {
        let payload = super::signed_payload_bytes(value).expect("payload should canonicalize");
        let signature = signing_key.sign(&payload);
        format!(
            "sig:ed25519:{}",
            base64::engine::general_purpose::STANDARD.encode(signature.to_bytes())
        )
    }

    fn state_hash_for_blocks(doc_id: &str, blocks: Vec<BlockObject>) -> String {
        compute_state_hash(&DocumentState {
            doc_id: doc_id.to_string(),
            blocks,
            metadata: Map::new(),
        })
        .expect("state hash should compute")
    }

    #[test]
    fn patch_id_recomputes_from_canonical_json() {
        let (signing_key, public_key) = signer_material();
        let mut value = json!({
            "type": "patch",
            "version": "mycel/0.1",
            "doc_id": "doc:test",
            "base_revision": "rev:genesis-null",
            "author": public_key,
            "timestamp": 1777778888u64,
            "ops": [],
        });
        let patch_id = super::recompute_object_id(&value, "patch_id", "patch")
            .expect("patch ID should recompute");
        value["patch_id"] = Value::String(patch_id.clone());
        value["signature"] = Value::String(sign_value(&signing_key, &value));
        let path = write_test_file(
            "patch-valid",
            &serde_json::to_string_pretty(&value).expect("test JSON should serialize"),
        );

        let summary = verify_object_path(&path);

        assert!(summary.is_ok(), "expected success, got {summary:?}");
        assert_eq!(summary.signature_verification.as_deref(), Some("verified"));
        assert_eq!(summary.recomputed_id.as_deref(), Some(patch_id.as_str()));

        let _ = std::fs::remove_file(path);
    }

    #[test]
    fn patch_signature_is_reproducible_across_object_key_order() {
        let (signing_key, public_key) = signer_material();
        let left = json!({
            "type": "patch",
            "version": "mycel/0.1",
            "patch_id": "patch:test",
            "doc_id": "doc:test",
            "base_revision": "rev:genesis-null",
            "author": public_key,
            "timestamp": 1777778888u64,
            "ops": []
        });
        let right = json!({
            "ops": [],
            "timestamp": 1777778888u64,
            "author": public_key,
            "base_revision": "rev:genesis-null",
            "doc_id": "doc:test",
            "patch_id": "patch:test",
            "version": "mycel/0.1",
            "type": "patch"
        });

        let left_signature = sign_value(&signing_key, &left);
        let right_signature = sign_value(&signing_key, &right);

        assert_eq!(left_signature, right_signature);
    }

    #[test]
    fn null_values_are_rejected() {
        let path = write_test_file(
            "document-null",
            &serde_json::to_string_pretty(&json!({
                "type": "document",
                "version": "mycel/0.1",
                "doc_id": "doc:test",
                "title": null
            }))
            .expect("test JSON should serialize"),
        );

        let summary = verify_object_path(&path);

        assert!(!summary.is_ok(), "expected failure, got {summary:?}");
        assert!(
            summary
                .errors
                .iter()
                .any(|message| message.contains("$.title: null is not allowed")),
            "expected null validation error, got {summary:?}"
        );

        let _ = std::fs::remove_file(path);
    }

    #[test]
    fn invalid_signature_is_rejected() {
        let (_signing_key, public_key) = signer_material();
        let mut value = json!({
            "type": "patch",
            "version": "mycel/0.1",
            "doc_id": "doc:test",
            "base_revision": "rev:genesis-null",
            "author": public_key,
            "timestamp": 1777778888u64,
            "ops": []
        });
        let patch_id = super::recompute_object_id(&value, "patch_id", "patch")
            .expect("patch ID should recompute");
        value["patch_id"] = Value::String(patch_id);
        value["signature"] = Value::String(
            "sig:ed25519:AAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAA=="
                .to_string(),
        );
        let path = write_test_file(
            "patch-invalid-signature",
            &serde_json::to_string_pretty(&value).expect("test JSON should serialize"),
        );

        let summary = verify_object_path(&path);

        assert!(!summary.is_ok(), "expected failure, got {summary:?}");
        assert_eq!(summary.signature_verification.as_deref(), Some("failed"));
        assert!(
            summary
                .errors
                .iter()
                .any(|message| message.contains("Ed25519 signature verification failed")),
            "expected signature failure, got {summary:?}"
        );

        let _ = std::fs::remove_file(path);
    }

    #[test]
    fn document_missing_logical_id_is_rejected() {
        let path = write_test_file(
            "document-missing-doc-id",
            &serde_json::to_string_pretty(&json!({
                "type": "document",
                "version": "mycel/0.1",
                "title": "Plain document"
            }))
            .expect("test JSON should serialize"),
        );

        let summary = verify_object_path(&path);

        assert!(!summary.is_ok(), "expected failure, got {summary:?}");
        assert!(
            summary.errors.iter().any(|message| {
                message.contains("document object is missing string field 'doc_id'")
            }),
            "expected missing logical ID error, got {summary:?}"
        );

        let _ = std::fs::remove_file(path);
    }

    #[test]
    fn document_wrong_logical_id_prefix_is_rejected() {
        let path = write_test_file(
            "document-wrong-doc-id-prefix",
            &serde_json::to_string_pretty(&json!({
                "type": "document",
                "version": "mycel/0.1",
                "doc_id": "revision:test",
                "title": "Plain document",
                "language": "zh-Hant",
                "content_model": "block-tree",
                "created_at": 1u64,
                "created_by": "pk:ed25519:test",
                "genesis_revision": "rev:test"
            }))
            .expect("test JSON should serialize"),
        );

        let summary = verify_object_path(&path);

        assert!(!summary.is_ok(), "expected failure, got {summary:?}");
        assert!(
            summary
                .errors
                .iter()
                .any(|message| message.contains("top-level 'doc_id' must use 'doc:' prefix")),
            "expected logical ID prefix error, got {summary:?}"
        );

        let _ = std::fs::remove_file(path);
    }

    #[test]
    fn document_missing_baseline_fields_is_rejected_by_typed_validation() {
        let path = write_test_file(
            "document-missing-title",
            &serde_json::to_string_pretty(&json!({
                "type": "document",
                "version": "mycel/0.1",
                "doc_id": "doc:test",
                "language": "zh-Hant",
                "content_model": "block-tree",
                "created_at": 1u64,
                "created_by": "pk:ed25519:test",
                "genesis_revision": "rev:test"
            }))
            .expect("test JSON should serialize"),
        );

        let summary = verify_object_path(&path);

        assert!(!summary.is_ok(), "expected failure, got {summary:?}");
        assert!(
            summary
                .errors
                .iter()
                .any(|message| message.contains("missing string field 'title'")),
            "expected typed document parse error, got {summary:?}"
        );

        let _ = std::fs::remove_file(path);
    }

    #[test]
    fn document_wrong_content_model_is_rejected() {
        let path = write_test_file(
            "document-wrong-content-model",
            &serde_json::to_string_pretty(&json!({
                "type": "document",
                "version": "mycel/0.1",
                "doc_id": "doc:test",
                "title": "Plain document",
                "language": "zh-Hant",
                "content_model": "markdown",
                "created_at": 1u64,
                "created_by": "pk:ed25519:test",
                "genesis_revision": "rev:test"
            }))
            .expect("test JSON should serialize"),
        );

        let summary = verify_object_path(&path);

        assert!(!summary.is_ok(), "expected failure, got {summary:?}");
        assert!(
            summary.errors.iter().any(|message| {
                message.contains("top-level 'content_model' must equal 'block-tree'")
            }),
            "expected content_model validation error, got {summary:?}"
        );

        let _ = std::fs::remove_file(path);
    }

    #[test]
    fn document_wrong_created_by_prefix_is_rejected() {
        let path = write_test_file(
            "document-wrong-created-by-prefix",
            &serde_json::to_string_pretty(&json!({
                "type": "document",
                "version": "mycel/0.1",
                "doc_id": "doc:test",
                "title": "Plain document",
                "language": "zh-Hant",
                "content_model": "block-tree",
                "created_at": 1u64,
                "created_by": "sig:bad",
                "genesis_revision": "rev:test"
            }))
            .expect("test JSON should serialize"),
        );

        let summary = verify_object_path(&path);

        assert!(!summary.is_ok(), "expected failure, got {summary:?}");
        assert!(
            summary.errors.iter().any(|message| {
                message.contains("top-level 'created_by' must use 'pk:' prefix")
            }),
            "expected created_by prefix error, got {summary:?}"
        );

        let _ = std::fs::remove_file(path);
    }

    #[test]
    fn document_wrong_genesis_revision_prefix_is_rejected() {
        let path = write_test_file(
            "document-wrong-genesis-revision-prefix",
            &serde_json::to_string_pretty(&json!({
                "type": "document",
                "version": "mycel/0.1",
                "doc_id": "doc:test",
                "title": "Plain document",
                "language": "zh-Hant",
                "content_model": "block-tree",
                "created_at": 1u64,
                "created_by": "pk:ed25519:test",
                "genesis_revision": "hash:test"
            }))
            .expect("test JSON should serialize"),
        );

        let summary = verify_object_path(&path);

        assert!(!summary.is_ok(), "expected failure, got {summary:?}");
        assert!(
            summary.errors.iter().any(|message| {
                message.contains("top-level 'genesis_revision' must use 'rev:' prefix")
            }),
            "expected genesis_revision prefix error, got {summary:?}"
        );

        let _ = std::fs::remove_file(path);
    }

    #[test]
    fn revision_duplicate_parent_ids_are_rejected_by_typed_validation() {
        let (signing_key, public_key) = signer_material();
        let mut revision = json!({
            "type": "revision",
            "version": "mycel/0.1",
            "doc_id": "doc:test",
            "parents": ["rev:base", "rev:base"],
            "patches": [],
            "state_hash": "hash:test",
            "author": public_key,
            "timestamp": 11u64
        });
        let revision_id = super::recompute_object_id(&revision, "revision_id", "rev")
            .expect("revision ID should recompute");
        revision["revision_id"] = Value::String(revision_id);
        revision["signature"] = Value::String(sign_value(&signing_key, &revision));
        let path = write_test_file(
            "revision-duplicate-parents",
            &serde_json::to_string_pretty(&revision).expect("test JSON should serialize"),
        );

        let summary = verify_object_path(&path);

        assert!(!summary.is_ok(), "expected failure, got {summary:?}");
        assert!(
            summary
                .errors
                .iter()
                .any(|message| message.contains("top-level 'parents[1]' duplicates 'parents[0]'")),
            "expected duplicate parent error, got {summary:?}"
        );

        let _ = std::fs::remove_file(path);
    }

    #[test]
    fn revision_wrong_parent_prefix_is_rejected() {
        let (signing_key, public_key) = signer_material();
        let mut revision = json!({
            "type": "revision",
            "version": "mycel/0.1",
            "doc_id": "doc:test",
            "parents": ["hash:base"],
            "patches": [],
            "state_hash": "hash:test",
            "author": public_key,
            "timestamp": 11u64
        });
        let revision_id = super::recompute_object_id(&revision, "revision_id", "rev")
            .expect("revision ID should recompute");
        revision["revision_id"] = Value::String(revision_id);
        revision["signature"] = Value::String(sign_value(&signing_key, &revision));
        let path = write_test_file(
            "revision-wrong-parent-prefix",
            &serde_json::to_string_pretty(&revision).expect("test JSON should serialize"),
        );

        let summary = verify_object_path(&path);

        assert!(!summary.is_ok(), "expected failure, got {summary:?}");
        assert!(
            summary
                .errors
                .iter()
                .any(|message| message.contains("top-level 'parents[0]' must use 'rev:' prefix")),
            "expected parent prefix error, got {summary:?}"
        );

        let _ = std::fs::remove_file(path);
    }

    #[test]
    fn patch_wrong_base_revision_prefix_is_rejected() {
        let (signing_key, public_key) = signer_material();
        let mut patch = json!({
            "type": "patch",
            "version": "mycel/0.1",
            "doc_id": "doc:test",
            "base_revision": "hash:base",
            "author": public_key,
            "timestamp": 11u64,
            "ops": []
        });
        let patch_id = super::recompute_object_id(&patch, "patch_id", "patch")
            .expect("patch ID should recompute");
        patch["patch_id"] = Value::String(patch_id);
        patch["signature"] = Value::String(sign_value(&signing_key, &patch));
        let path = write_test_file(
            "patch-wrong-base-revision-prefix",
            &serde_json::to_string_pretty(&patch).expect("test JSON should serialize"),
        );

        let summary = verify_object_path(&path);

        assert!(!summary.is_ok(), "expected failure, got {summary:?}");
        assert!(
            summary.errors.iter().any(|message| {
                message.contains("top-level 'base_revision' must use 'rev:' prefix")
            }),
            "expected base_revision prefix error, got {summary:?}"
        );

        let _ = std::fs::remove_file(path);
    }

    #[test]
    fn patch_wrong_author_prefix_is_rejected() {
        let (signing_key, public_key) = signer_material();
        let mut patch = json!({
            "type": "patch",
            "version": "mycel/0.1",
            "doc_id": "doc:test",
            "base_revision": "rev:genesis-null",
            "author": public_key,
            "timestamp": 11u64,
            "ops": []
        });
        let patch_id = super::recompute_object_id(&patch, "patch_id", "patch")
            .expect("patch ID should recompute");
        patch["patch_id"] = Value::String(patch_id);
        patch["signature"] = Value::String(sign_value(&signing_key, &patch));
        patch["author"] = Value::String("author:test".to_string());
        let path = write_test_file(
            "patch-wrong-author-prefix",
            &serde_json::to_string_pretty(&patch).expect("test JSON should serialize"),
        );

        let summary = verify_object_path(&path);

        assert!(!summary.is_ok(), "expected failure, got {summary:?}");
        assert!(
            summary.errors.iter().any(|message| {
                message.contains("signer field must use format 'pk:ed25519:<base64>'")
            }),
            "expected signer format error, got {summary:?}"
        );

        let _ = std::fs::remove_file(path);
    }

    #[test]
    fn patch_unknown_top_level_field_is_rejected() {
        let (signing_key, public_key) = signer_material();
        let mut patch = json!({
            "type": "patch",
            "version": "mycel/0.1",
            "doc_id": "doc:test",
            "base_revision": "rev:genesis-null",
            "author": public_key,
            "timestamp": 11u64,
            "ops": [],
            "unexpected": true
        });
        let patch_id = super::recompute_object_id(&patch, "patch_id", "patch")
            .expect("patch ID should recompute");
        patch["patch_id"] = Value::String(patch_id);
        patch["signature"] = Value::String(sign_value(&signing_key, &patch));
        let path = write_test_file(
            "patch-unknown-top-level-field",
            &serde_json::to_string_pretty(&patch).expect("test JSON should serialize"),
        );

        let summary = verify_object_path(&path);

        assert!(!summary.is_ok(), "expected failure, got {summary:?}");
        assert!(
            summary
                .errors
                .iter()
                .any(|message| message.contains("top-level contains unexpected field 'unexpected'")),
            "expected unknown top-level field error, got {summary:?}"
        );

        let _ = std::fs::remove_file(path);
    }

    #[test]
    fn patch_move_without_destination_is_rejected() {
        let (signing_key, public_key) = signer_material();
        let mut patch = json!({
            "type": "patch",
            "version": "mycel/0.1",
            "doc_id": "doc:test",
            "base_revision": "rev:genesis-null",
            "author": public_key,
            "timestamp": 11u64,
            "ops": [
                {
                    "op": "move_block",
                    "block_id": "blk:001"
                }
            ]
        });
        let patch_id = super::recompute_object_id(&patch, "patch_id", "patch")
            .expect("patch ID should recompute");
        patch["patch_id"] = Value::String(patch_id);
        patch["signature"] = Value::String(sign_value(&signing_key, &patch));
        let path = write_test_file(
            "patch-move-without-destination",
            &serde_json::to_string_pretty(&patch).expect("test JSON should serialize"),
        );

        let summary = verify_object_path(&path);

        assert!(!summary.is_ok(), "expected failure, got {summary:?}");
        assert!(
            summary.errors.iter().any(|message| {
                message.contains(
                    "top-level 'ops[0]': move_block requires at least one destination reference",
                )
            }),
            "expected move_block destination error, got {summary:?}"
        );

        let _ = std::fs::remove_file(path);
    }

    #[test]
    fn patch_mixed_set_metadata_forms_are_rejected() {
        let (signing_key, public_key) = signer_material();
        let mut patch = json!({
            "type": "patch",
            "version": "mycel/0.1",
            "doc_id": "doc:test",
            "base_revision": "rev:genesis-null",
            "author": public_key,
            "timestamp": 11u64,
            "ops": [
                {
                    "op": "set_metadata",
                    "metadata": {
                        "title": "Hello"
                    },
                    "key": "extra"
                }
            ]
        });
        let patch_id = super::recompute_object_id(&patch, "patch_id", "patch")
            .expect("patch ID should recompute");
        patch["patch_id"] = Value::String(patch_id);
        patch["signature"] = Value::String(sign_value(&signing_key, &patch));
        let path = write_test_file(
            "patch-mixed-set-metadata-forms",
            &serde_json::to_string_pretty(&patch).expect("test JSON should serialize"),
        );

        let summary = verify_object_path(&path);

        assert!(!summary.is_ok(), "expected failure, got {summary:?}");
        assert!(
            summary.errors.iter().any(|message| {
                message.contains("top-level 'ops[0]': patch op contains unexpected field 'key'")
            }),
            "expected mixed set_metadata forms error, got {summary:?}"
        );

        let _ = std::fs::remove_file(path);
    }

    #[test]
    fn revision_duplicate_patch_ids_are_rejected_by_typed_validation() {
        let (signing_key, public_key) = signer_material();
        let mut revision = json!({
            "type": "revision",
            "version": "mycel/0.1",
            "doc_id": "doc:test",
            "parents": ["rev:base"],
            "patches": ["patch:test", "patch:test"],
            "state_hash": "hash:test",
            "author": public_key,
            "timestamp": 11u64
        });
        let revision_id = super::recompute_object_id(&revision, "revision_id", "rev")
            .expect("revision ID should recompute");
        revision["revision_id"] = Value::String(revision_id);
        revision["signature"] = Value::String(sign_value(&signing_key, &revision));
        let path = write_test_file(
            "revision-duplicate-patches",
            &serde_json::to_string_pretty(&revision).expect("test JSON should serialize"),
        );

        let summary = verify_object_path(&path);

        assert!(!summary.is_ok(), "expected failure, got {summary:?}");
        assert!(
            summary
                .errors
                .iter()
                .any(|message| message.contains("top-level 'patches[1]' duplicates 'patches[0]'")),
            "expected duplicate patch error, got {summary:?}"
        );

        let _ = std::fs::remove_file(path);
    }

    #[test]
    fn revision_merge_strategy_requires_multiple_parents() {
        let (signing_key, public_key) = signer_material();
        let mut revision = json!({
            "type": "revision",
            "version": "mycel/0.1",
            "doc_id": "doc:test",
            "parents": ["rev:base"],
            "patches": [],
            "merge_strategy": "semantic-block-merge",
            "state_hash": "hash:test",
            "author": public_key,
            "timestamp": 11u64
        });
        let revision_id = super::recompute_object_id(&revision, "revision_id", "rev")
            .expect("revision ID should recompute");
        revision["revision_id"] = Value::String(revision_id);
        revision["signature"] = Value::String(sign_value(&signing_key, &revision));
        let path = write_test_file(
            "revision-merge-strategy-single-parent",
            &serde_json::to_string_pretty(&revision).expect("test JSON should serialize"),
        );

        let summary = verify_object_path(&path);

        assert!(!summary.is_ok(), "expected failure, got {summary:?}");
        assert!(
            summary.errors.iter().any(|message| {
                message.contains("top-level 'merge_strategy' requires multiple parents")
            }),
            "expected merge_strategy parent-count error, got {summary:?}"
        );

        let _ = std::fs::remove_file(path);
    }

    #[test]
    fn genesis_revision_rejects_merge_strategy() {
        let (signing_key, public_key) = signer_material();
        let mut revision = json!({
            "type": "revision",
            "version": "mycel/0.1",
            "doc_id": "doc:test",
            "parents": [],
            "patches": [],
            "merge_strategy": "semantic-block-merge",
            "state_hash": "hash:test",
            "author": public_key,
            "timestamp": 11u64
        });
        let revision_id = super::recompute_object_id(&revision, "revision_id", "rev")
            .expect("revision ID should recompute");
        revision["revision_id"] = Value::String(revision_id);
        revision["signature"] = Value::String(sign_value(&signing_key, &revision));
        let path = write_test_file(
            "revision-merge-strategy-genesis",
            &serde_json::to_string_pretty(&revision).expect("test JSON should serialize"),
        );

        let summary = verify_object_path(&path);

        assert!(!summary.is_ok(), "expected failure, got {summary:?}");
        assert!(
            summary.errors.iter().any(|message| {
                message
                    .contains("top-level 'merge_strategy' is not allowed when 'parents' is empty")
            }),
            "expected genesis merge_strategy error, got {summary:?}"
        );

        let _ = std::fs::remove_file(path);
    }

    #[test]
    fn multi_parent_revision_requires_merge_strategy() {
        let (signing_key, public_key) = signer_material();
        let mut revision = json!({
            "type": "revision",
            "version": "mycel/0.1",
            "doc_id": "doc:test",
            "parents": ["rev:base", "rev:side"],
            "patches": [],
            "state_hash": "hash:test",
            "author": public_key,
            "timestamp": 11u64
        });
        let revision_id = super::recompute_object_id(&revision, "revision_id", "rev")
            .expect("revision ID should recompute");
        revision["revision_id"] = Value::String(revision_id);
        revision["signature"] = Value::String(sign_value(&signing_key, &revision));
        let path = write_test_file(
            "revision-missing-merge-strategy",
            &serde_json::to_string_pretty(&revision).expect("test JSON should serialize"),
        );

        let summary = verify_object_path(&path);

        assert!(!summary.is_ok(), "expected failure, got {summary:?}");
        assert!(
            summary.errors.iter().any(|message| {
                message.contains(
                    "top-level 'merge_strategy' is required when 'parents' has multiple entries",
                )
            }),
            "expected missing merge_strategy error, got {summary:?}"
        );

        let _ = std::fs::remove_file(path);
    }

    #[test]
    fn revision_wrong_state_hash_prefix_is_rejected() {
        let (signing_key, public_key) = signer_material();
        let mut revision = json!({
            "type": "revision",
            "version": "mycel/0.1",
            "doc_id": "doc:test",
            "parents": ["rev:base"],
            "patches": [],
            "state_hash": "rev:test",
            "author": public_key,
            "timestamp": 11u64
        });
        let revision_id = super::recompute_object_id(&revision, "revision_id", "rev")
            .expect("revision ID should recompute");
        revision["revision_id"] = Value::String(revision_id);
        revision["signature"] = Value::String(sign_value(&signing_key, &revision));
        let path = write_test_file(
            "revision-wrong-state-hash-prefix",
            &serde_json::to_string_pretty(&revision).expect("test JSON should serialize"),
        );

        let summary = verify_object_path(&path);

        assert!(!summary.is_ok(), "expected failure, got {summary:?}");
        assert!(
            summary
                .errors
                .iter()
                .any(|message| message.contains("top-level 'state_hash' must use 'hash:' prefix")),
            "expected state_hash prefix error, got {summary:?}"
        );

        let _ = std::fs::remove_file(path);
    }

    #[test]
    fn revision_wrong_author_prefix_is_rejected() {
        let (signing_key, public_key) = signer_material();
        let mut revision = json!({
            "type": "revision",
            "version": "mycel/0.1",
            "doc_id": "doc:test",
            "parents": ["rev:base"],
            "patches": [],
            "state_hash": "hash:test",
            "author": public_key,
            "timestamp": 11u64
        });
        let revision_id = super::recompute_object_id(&revision, "revision_id", "rev")
            .expect("revision ID should recompute");
        revision["revision_id"] = Value::String(revision_id);
        revision["signature"] = Value::String(sign_value(&signing_key, &revision));
        revision["author"] = Value::String("author:test".to_string());
        let path = write_test_file(
            "revision-wrong-author-prefix",
            &serde_json::to_string_pretty(&revision).expect("test JSON should serialize"),
        );

        let summary = verify_object_path(&path);

        assert!(!summary.is_ok(), "expected failure, got {summary:?}");
        assert!(
            summary.errors.iter().any(|message| {
                message.contains("signer field must use format 'pk:ed25519:<base64>'")
            }),
            "expected signer format error, got {summary:?}"
        );

        let _ = std::fs::remove_file(path);
    }

    #[test]
    fn inspect_warns_when_document_logical_id_has_wrong_type() {
        let path = write_test_file(
            "document-wrong-doc-id-type",
            &serde_json::to_string_pretty(&json!({
                "type": "document",
                "version": "mycel/0.1",
                "doc_id": 7,
                "title": "Plain document"
            }))
            .expect("test JSON should serialize"),
        );

        let summary = inspect_object_path(&path);

        assert_eq!(summary.status, "warning");
        assert!(
            summary
                .notes
                .iter()
                .any(|message| message.contains("top-level 'doc_id' should be a string")),
            "expected logical ID warning, got {summary:?}"
        );

        let _ = std::fs::remove_file(path);
    }

    #[test]
    fn inspect_warns_when_block_logical_id_has_wrong_type() {
        let path = write_test_file(
            "block-wrong-block-id-type",
            &serde_json::to_string_pretty(&json!({
                "type": "block",
                "version": "mycel/0.1",
                "block_id": 7,
                "text": "Hello"
            }))
            .expect("test JSON should serialize"),
        );

        let summary = inspect_object_path(&path);

        assert_eq!(summary.status, "warning");
        assert!(
            summary
                .notes
                .iter()
                .any(|message| message.contains("top-level 'block_id' should be a string")),
            "expected logical ID warning, got {summary:?}"
        );

        let _ = std::fs::remove_file(path);
    }

    #[test]
    fn block_missing_logical_id_is_rejected() {
        let path = write_test_file(
            "block-missing-block-id",
            &serde_json::to_string_pretty(&json!({
                "type": "block",
                "version": "mycel/0.1",
                "block_type": "paragraph",
                "content": "Hello",
                "attrs": {},
                "children": []
            }))
            .expect("test JSON should serialize"),
        );

        let summary = verify_object_path(&path);

        assert!(!summary.is_ok(), "expected failure, got {summary:?}");
        assert!(
            summary
                .errors
                .iter()
                .any(|message| message.contains("block object is missing string field 'block_id'")),
            "expected missing logical ID error, got {summary:?}"
        );

        let _ = std::fs::remove_file(path);
    }

    #[test]
    fn block_wrong_logical_id_prefix_is_rejected() {
        let path = write_test_file(
            "block-wrong-block-id-prefix",
            &serde_json::to_string_pretty(&json!({
                "type": "block",
                "version": "mycel/0.1",
                "block_id": "paragraph-1",
                "block_type": "paragraph",
                "content": "Hello",
                "attrs": {},
                "children": []
            }))
            .expect("test JSON should serialize"),
        );

        let summary = verify_object_path(&path);

        assert!(!summary.is_ok(), "expected failure, got {summary:?}");
        assert!(
            summary
                .errors
                .iter()
                .any(|message| message.contains("top-level 'block_id' must use 'blk:' prefix")),
            "expected block_id prefix error, got {summary:?}"
        );

        let _ = std::fs::remove_file(path);
    }

    #[test]
    fn block_unknown_top_level_field_is_rejected() {
        let path = write_test_file(
            "block-unknown-top-level-field",
            &serde_json::to_string_pretty(&json!({
                "type": "block",
                "version": "mycel/0.1",
                "block_id": "blk:001",
                "block_type": "paragraph",
                "content": "Hello",
                "attrs": {},
                "children": [],
                "unexpected": true
            }))
            .expect("test JSON should serialize"),
        );

        let summary = verify_object_path(&path);

        assert!(!summary.is_ok(), "expected failure, got {summary:?}");
        assert!(
            summary
                .errors
                .iter()
                .any(|message| message.contains("top-level contains unexpected field 'unexpected'")),
            "expected unknown-field error, got {summary:?}"
        );

        let _ = std::fs::remove_file(path);
    }

    #[test]
    fn block_unknown_nested_child_field_is_rejected() {
        let path = write_test_file(
            "block-unknown-nested-child-field",
            &serde_json::to_string_pretty(&json!({
                "type": "block",
                "version": "mycel/0.1",
                "block_id": "blk:001",
                "block_type": "paragraph",
                "content": "Hello",
                "attrs": {},
                "children": [
                    {
                        "block_id": "blk:002",
                        "block_type": "paragraph",
                        "content": "Child",
                        "attrs": {},
                        "children": [],
                        "unexpected": true
                    }
                ]
            }))
            .expect("test JSON should serialize"),
        );

        let summary = verify_object_path(&path);

        assert!(!summary.is_ok(), "expected failure, got {summary:?}");
        assert!(
            summary.errors.iter().any(|message| {
                message.contains("top-level 'children[0]' contains unexpected field 'unexpected'")
            }),
            "expected nested child unknown-field error, got {summary:?}"
        );

        let _ = std::fs::remove_file(path);
    }

    #[test]
    fn snapshot_missing_documents_is_rejected_by_typed_validation() {
        let (signing_key, public_key) = signer_material();
        let mut snapshot = json!({
            "type": "snapshot",
            "version": "mycel/0.1",
            "included_objects": ["rev:test"],
            "root_hash": "hash:test",
            "created_by": public_key,
            "timestamp": 9u64
        });
        let snapshot_id = super::recompute_object_id(&snapshot, "snapshot_id", "snap")
            .expect("snapshot ID should recompute");
        snapshot["snapshot_id"] = Value::String(snapshot_id);
        snapshot["signature"] = Value::String(sign_value(&signing_key, &snapshot));
        let path = write_test_file(
            "snapshot-missing-documents",
            &serde_json::to_string_pretty(&snapshot).expect("test JSON should serialize"),
        );

        let summary = verify_object_path(&path);

        assert!(!summary.is_ok(), "expected failure, got {summary:?}");
        assert!(
            summary
                .errors
                .iter()
                .any(|message| message.contains("missing object field 'documents'")),
            "expected typed snapshot parse error, got {summary:?}"
        );

        let _ = std::fs::remove_file(path);
    }

    #[test]
    fn snapshot_empty_documents_is_rejected_by_typed_validation() {
        let (signing_key, public_key) = signer_material();
        let mut snapshot = json!({
            "type": "snapshot",
            "version": "mycel/0.1",
            "documents": {},
            "included_objects": ["rev:test"],
            "root_hash": "hash:test",
            "created_by": public_key,
            "timestamp": 9u64
        });
        let snapshot_id = super::recompute_object_id(&snapshot, "snapshot_id", "snap")
            .expect("snapshot ID should recompute");
        snapshot["snapshot_id"] = Value::String(snapshot_id);
        snapshot["signature"] = Value::String(sign_value(&signing_key, &snapshot));
        let path = write_test_file(
            "snapshot-empty-documents",
            &serde_json::to_string_pretty(&snapshot).expect("test JSON should serialize"),
        );

        let summary = verify_object_path(&path);

        assert!(!summary.is_ok(), "expected failure, got {summary:?}");
        assert!(
            summary
                .errors
                .iter()
                .any(|message| message.contains("top-level 'documents' must not be empty")),
            "expected empty documents error, got {summary:?}"
        );

        let _ = std::fs::remove_file(path);
    }

    #[test]
    fn snapshot_non_string_snapshot_id_is_rejected() {
        let (signing_key, public_key) = signer_material();
        let mut snapshot = json!({
            "type": "snapshot",
            "version": "mycel/0.1",
            "documents": {
                "doc:test": "rev:test"
            },
            "included_objects": ["rev:test", "patch:test"],
            "root_hash": "hash:test",
            "created_by": public_key,
            "timestamp": 9u64,
            "snapshot_id": 7
        });
        snapshot["signature"] = Value::String(sign_value(&signing_key, &snapshot));
        let path = write_test_file(
            "snapshot-wrong-id-type",
            &serde_json::to_string_pretty(&snapshot).expect("test JSON should serialize"),
        );

        let summary = verify_object_path(&path);

        assert!(!summary.is_ok(), "expected failure, got {summary:?}");
        assert!(
            summary
                .errors
                .iter()
                .any(|message| message.contains("top-level 'snapshot_id' must be a string")),
            "expected snapshot ID type error, got {summary:?}"
        );

        let _ = std::fs::remove_file(path);
    }

    #[test]
    fn snapshot_duplicate_included_objects_are_rejected() {
        let (signing_key, public_key) = signer_material();
        let mut snapshot = json!({
            "type": "snapshot",
            "version": "mycel/0.1",
            "documents": {
                "doc:test": "rev:test"
            },
            "included_objects": ["rev:test", "rev:test"],
            "root_hash": "hash:test",
            "created_by": public_key,
            "timestamp": 9u64
        });
        let snapshot_id = super::recompute_object_id(&snapshot, "snapshot_id", "snap")
            .expect("snapshot ID should recompute");
        snapshot["snapshot_id"] = Value::String(snapshot_id);
        snapshot["signature"] = Value::String(sign_value(&signing_key, &snapshot));
        let path = write_test_file(
            "snapshot-duplicate-included-objects",
            &serde_json::to_string_pretty(&snapshot).expect("test JSON should serialize"),
        );

        let summary = verify_object_path(&path);

        assert!(!summary.is_ok(), "expected failure, got {summary:?}");
        assert!(
            summary.errors.iter().any(|message| {
                message.contains("top-level 'included_objects[1]' duplicates 'included_objects[0]'")
            }),
            "expected duplicate included_objects error, got {summary:?}"
        );

        let _ = std::fs::remove_file(path);
    }

    #[test]
    fn snapshot_empty_included_object_entry_is_rejected() {
        let (signing_key, public_key) = signer_material();
        let mut snapshot = json!({
            "type": "snapshot",
            "version": "mycel/0.1",
            "documents": {
                "doc:test": "rev:test"
            },
            "included_objects": ["rev:test", ""],
            "root_hash": "hash:test",
            "created_by": public_key,
            "timestamp": 9u64
        });
        let snapshot_id = super::recompute_object_id(&snapshot, "snapshot_id", "snap")
            .expect("snapshot ID should recompute");
        snapshot["snapshot_id"] = Value::String(snapshot_id);
        snapshot["signature"] = Value::String(sign_value(&signing_key, &snapshot));
        let path = write_test_file(
            "snapshot-empty-included-object-entry",
            &serde_json::to_string_pretty(&snapshot).expect("test JSON should serialize"),
        );

        let summary = verify_object_path(&path);

        assert!(!summary.is_ok(), "expected failure, got {summary:?}");
        assert!(
            summary.errors.iter().any(|message| {
                message.contains("top-level 'included_objects[1]' must not be an empty string")
            }),
            "expected empty included_objects entry error, got {summary:?}"
        );

        let _ = std::fs::remove_file(path);
    }

    #[test]
    fn snapshot_non_canonical_included_object_id_is_rejected() {
        let (signing_key, public_key) = signer_material();
        let mut snapshot = json!({
            "type": "snapshot",
            "version": "mycel/0.1",
            "documents": {
                "doc:test": "rev:test"
            },
            "included_objects": ["doc:test"],
            "root_hash": "hash:test",
            "created_by": public_key,
            "timestamp": 9u64
        });
        let snapshot_id = super::recompute_object_id(&snapshot, "snapshot_id", "snap")
            .expect("snapshot ID should recompute");
        snapshot["snapshot_id"] = Value::String(snapshot_id);
        snapshot["signature"] = Value::String(sign_value(&signing_key, &snapshot));
        let path = write_test_file(
            "snapshot-non-canonical-included-object-id",
            &serde_json::to_string_pretty(&snapshot).expect("test JSON should serialize"),
        );

        let summary = verify_object_path(&path);

        assert!(!summary.is_ok(), "expected failure, got {summary:?}");
        assert!(
            summary.errors.iter().any(|message| {
                message.contains(
                    "top-level 'included_objects[0]' must use a canonical object ID prefix",
                )
            }),
            "expected canonical included_objects ID error, got {summary:?}"
        );

        let _ = std::fs::remove_file(path);
    }

    #[test]
    fn snapshot_missing_declared_revision_in_included_objects_is_rejected() {
        let (signing_key, public_key) = signer_material();
        let mut snapshot = json!({
            "type": "snapshot",
            "version": "mycel/0.1",
            "documents": {
                "doc:test": "rev:test"
            },
            "included_objects": ["patch:test"],
            "root_hash": "hash:test",
            "created_by": public_key,
            "timestamp": 9u64
        });
        let snapshot_id = super::recompute_object_id(&snapshot, "snapshot_id", "snap")
            .expect("snapshot ID should recompute");
        snapshot["snapshot_id"] = Value::String(snapshot_id);
        snapshot["signature"] = Value::String(sign_value(&signing_key, &snapshot));
        let path = write_test_file(
            "snapshot-missing-declared-revision",
            &serde_json::to_string_pretty(&snapshot).expect("test JSON should serialize"),
        );

        let summary = verify_object_path(&path);

        assert!(!summary.is_ok(), "expected failure, got {summary:?}");
        assert!(summary.errors.iter().any(|message| {
            message.contains(
                "top-level 'included_objects' must include revision 'rev:test' declared by 'documents.doc:test'",
            )
        }), "expected missing declared revision error, got {summary:?}");

        let _ = std::fs::remove_file(path);
    }

    #[test]
    fn snapshot_wrong_root_hash_prefix_is_rejected() {
        let (signing_key, public_key) = signer_material();
        let mut snapshot = json!({
            "type": "snapshot",
            "version": "mycel/0.1",
            "documents": {
                "doc:test": "rev:test"
            },
            "included_objects": ["rev:test", "patch:test"],
            "root_hash": "rev:test",
            "created_by": public_key,
            "timestamp": 9u64
        });
        let snapshot_id = super::recompute_object_id(&snapshot, "snapshot_id", "snap")
            .expect("snapshot ID should recompute");
        snapshot["snapshot_id"] = Value::String(snapshot_id);
        snapshot["signature"] = Value::String(sign_value(&signing_key, &snapshot));
        let path = write_test_file(
            "snapshot-wrong-root-hash-prefix",
            &serde_json::to_string_pretty(&snapshot).expect("test JSON should serialize"),
        );

        let summary = verify_object_path(&path);

        assert!(!summary.is_ok(), "expected failure, got {summary:?}");
        assert!(
            summary
                .errors
                .iter()
                .any(|message| message.contains("top-level 'root_hash' must use 'hash:' prefix")),
            "expected root_hash prefix error, got {summary:?}"
        );

        let _ = std::fs::remove_file(path);
    }

    #[test]
    fn snapshot_wrong_document_value_prefix_is_rejected() {
        let (signing_key, public_key) = signer_material();
        let mut snapshot = json!({
            "type": "snapshot",
            "version": "mycel/0.1",
            "documents": {
                "doc:test": "patch:test"
            },
            "included_objects": ["patch:test"],
            "root_hash": "hash:test",
            "created_by": public_key,
            "timestamp": 9u64
        });
        let snapshot_id = super::recompute_object_id(&snapshot, "snapshot_id", "snap")
            .expect("snapshot ID should recompute");
        snapshot["snapshot_id"] = Value::String(snapshot_id);
        snapshot["signature"] = Value::String(sign_value(&signing_key, &snapshot));
        let path = write_test_file(
            "snapshot-wrong-document-value-prefix",
            &serde_json::to_string_pretty(&snapshot).expect("test JSON should serialize"),
        );

        let summary = verify_object_path(&path);

        assert!(!summary.is_ok(), "expected failure, got {summary:?}");
        assert!(
            summary.errors.iter().any(|message| {
                message.contains("top-level 'documents.doc:test' must use 'rev:' prefix")
            }),
            "expected document revision-prefix error, got {summary:?}"
        );

        let _ = std::fs::remove_file(path);
    }

    #[test]
    fn snapshot_wrong_document_key_prefix_is_rejected() {
        let (signing_key, public_key) = signer_material();
        let mut snapshot = json!({
            "type": "snapshot",
            "version": "mycel/0.1",
            "documents": {
                "patch:test": "rev:test"
            },
            "included_objects": ["rev:test", "patch:test"],
            "root_hash": "hash:test",
            "created_by": public_key,
            "timestamp": 9u64
        });
        let snapshot_id = super::recompute_object_id(&snapshot, "snapshot_id", "snap")
            .expect("snapshot ID should recompute");
        snapshot["snapshot_id"] = Value::String(snapshot_id);
        snapshot["signature"] = Value::String(sign_value(&signing_key, &snapshot));
        let path = write_test_file(
            "snapshot-wrong-document-key-prefix",
            &serde_json::to_string_pretty(&snapshot).expect("test JSON should serialize"),
        );

        let summary = verify_object_path(&path);

        assert!(!summary.is_ok(), "expected failure, got {summary:?}");
        assert!(
            summary.errors.iter().any(|message| {
                message.contains("top-level 'documents.patch:test key' must use 'doc:' prefix")
            }),
            "expected document key-prefix error, got {summary:?}"
        );

        let _ = std::fs::remove_file(path);
    }

    #[test]
    fn snapshot_wrong_created_by_prefix_is_rejected() {
        let (signing_key, public_key) = signer_material();
        let mut snapshot = json!({
            "type": "snapshot",
            "version": "mycel/0.1",
            "documents": {
                "doc:test": "rev:test"
            },
            "included_objects": ["rev:test", "patch:test"],
            "root_hash": "hash:test",
            "created_by": public_key.replacen("pk:", "sig:", 1),
            "timestamp": 9u64
        });
        let snapshot_id = super::recompute_object_id(&snapshot, "snapshot_id", "snap")
            .expect("snapshot ID should recompute");
        snapshot["snapshot_id"] = Value::String(snapshot_id);
        snapshot["signature"] = Value::String(sign_value(&signing_key, &snapshot));
        let path = write_test_file(
            "snapshot-wrong-created-by-prefix",
            &serde_json::to_string_pretty(&snapshot).expect("test JSON should serialize"),
        );

        let summary = verify_object_path(&path);

        assert!(!summary.is_ok(), "expected failure, got {summary:?}");
        assert!(
            summary
                .errors
                .iter()
                .any(|message| message
                    .contains("signer field must use format 'pk:ed25519:<base64>'")),
            "expected created_by signer-format error, got {summary:?}"
        );

        let _ = std::fs::remove_file(path);
    }

    #[test]
    fn snapshot_unknown_top_level_field_is_rejected() {
        let (signing_key, public_key) = signer_material();
        let mut snapshot = json!({
            "type": "snapshot",
            "version": "mycel/0.1",
            "documents": {
                "doc:test": "rev:test"
            },
            "included_objects": ["rev:test", "patch:test"],
            "root_hash": "hash:test",
            "created_by": public_key,
            "timestamp": 9u64,
            "unexpected": true
        });
        let snapshot_id = super::recompute_object_id(&snapshot, "snapshot_id", "snap")
            .expect("snapshot ID should recompute");
        snapshot["snapshot_id"] = Value::String(snapshot_id);
        snapshot["signature"] = Value::String(sign_value(&signing_key, &snapshot));
        let path = write_test_file(
            "snapshot-unknown-top-level-field",
            &serde_json::to_string_pretty(&snapshot).expect("test JSON should serialize"),
        );

        let summary = verify_object_path(&path);

        assert!(!summary.is_ok(), "expected failure, got {summary:?}");
        assert!(
            summary
                .errors
                .iter()
                .any(|message| message.contains("top-level contains unexpected field 'unexpected'")),
            "expected unknown-field error, got {summary:?}"
        );

        let _ = std::fs::remove_file(path);
    }

    #[test]
    fn snapshot_mismatched_derived_id_is_rejected() {
        let (signing_key, public_key) = signer_material();
        let mut snapshot = json!({
            "type": "snapshot",
            "version": "mycel/0.1",
            "documents": {
                "doc:test": "rev:test"
            },
            "included_objects": ["rev:test", "patch:test"],
            "root_hash": "hash:test",
            "created_by": public_key,
            "timestamp": 9u64
        });
        let snapshot_id = super::recompute_object_id(&snapshot, "snapshot_id", "snap")
            .expect("snapshot ID should recompute");
        snapshot["snapshot_id"] = Value::String("snap:wrong".to_string());
        snapshot["signature"] = Value::String(sign_value(&signing_key, &snapshot));
        let path = write_test_file(
            "snapshot-mismatched-id",
            &serde_json::to_string_pretty(&snapshot).expect("test JSON should serialize"),
        );

        let summary = verify_object_path(&path);

        assert!(!summary.is_ok(), "expected failure, got {summary:?}");
        assert_eq!(summary.recomputed_id.as_deref(), Some(snapshot_id.as_str()));
        assert!(
            summary
                .errors
                .iter()
                .any(|message| message.contains("declared snapshot_id does not match")),
            "expected snapshot derived ID mismatch error, got {summary:?}"
        );

        let _ = std::fs::remove_file(path);
    }

    #[test]
    fn valid_snapshot_verifies_signature_and_typed_shape() {
        let (signing_key, public_key) = signer_material();
        let mut snapshot = json!({
            "type": "snapshot",
            "version": "mycel/0.1",
            "documents": {
                "doc:test": "rev:test"
            },
            "included_objects": ["rev:test", "patch:test"],
            "root_hash": "hash:test",
            "created_by": public_key,
            "timestamp": 9u64
        });
        let snapshot_id = super::recompute_object_id(&snapshot, "snapshot_id", "snap")
            .expect("snapshot ID should recompute");
        snapshot["snapshot_id"] = Value::String(snapshot_id.clone());
        snapshot["signature"] = Value::String(sign_value(&signing_key, &snapshot));
        let path = write_test_file(
            "snapshot-valid",
            &serde_json::to_string_pretty(&snapshot).expect("test JSON should serialize"),
        );

        let summary = verify_object_path(&path);

        assert!(summary.is_ok(), "expected success, got {summary:?}");
        assert_eq!(summary.signature_verification.as_deref(), Some("verified"));
        assert_eq!(summary.recomputed_id.as_deref(), Some(snapshot_id.as_str()));

        let _ = std::fs::remove_file(path);
    }

    #[test]
    fn view_wrong_maintainer_prefix_is_rejected_by_typed_validation() {
        let (signing_key, public_key) = signer_material();
        let mut view = json!({
            "type": "view",
            "version": "mycel/0.1",
            "maintainer": public_key.replacen("pk:", "sig:", 1),
            "documents": {
                "doc:test": "rev:test"
            },
            "policy": {
                "merge_rule": "manual-reviewed"
            },
            "timestamp": 12u64
        });
        let view_id =
            super::recompute_object_id(&view, "view_id", "view").expect("view ID should recompute");
        view["view_id"] = Value::String(view_id);
        view["signature"] = Value::String(sign_value(&signing_key, &view));
        let path = write_test_file(
            "view-wrong-maintainer-prefix",
            &serde_json::to_string_pretty(&view).expect("test JSON should serialize"),
        );

        let summary = verify_object_path(&path);

        assert!(!summary.is_ok(), "expected failure, got {summary:?}");
        assert!(
            summary
                .errors
                .iter()
                .any(|message| message
                    .contains("signer field must use format 'pk:ed25519:<base64>'")),
            "expected maintainer signer-format error, got {summary:?}"
        );

        let _ = std::fs::remove_file(path);
    }

    #[test]
    fn view_wrong_document_value_prefix_is_rejected_by_typed_validation() {
        let (signing_key, public_key) = signer_material();
        let mut view = json!({
            "type": "view",
            "version": "mycel/0.1",
            "maintainer": public_key,
            "documents": {
                "doc:test": "patch:test"
            },
            "policy": {
                "merge_rule": "manual-reviewed"
            },
            "timestamp": 12u64
        });
        let view_id =
            super::recompute_object_id(&view, "view_id", "view").expect("view ID should recompute");
        view["view_id"] = Value::String(view_id);
        view["signature"] = Value::String(sign_value(&signing_key, &view));
        let path = write_test_file(
            "view-wrong-document-value-prefix",
            &serde_json::to_string_pretty(&view).expect("test JSON should serialize"),
        );

        let summary = verify_object_path(&path);

        assert!(!summary.is_ok(), "expected failure, got {summary:?}");
        assert!(
            summary.errors.iter().any(|message| {
                message.contains("top-level 'documents.doc:test' must use 'rev:' prefix")
            }),
            "expected document revision-prefix error, got {summary:?}"
        );

        let _ = std::fs::remove_file(path);
    }

    #[test]
    fn view_wrong_document_key_prefix_is_rejected_by_typed_validation() {
        let (signing_key, public_key) = signer_material();
        let mut view = json!({
            "type": "view",
            "version": "mycel/0.1",
            "maintainer": public_key,
            "documents": {
                "patch:test": "rev:test"
            },
            "policy": {
                "merge_rule": "manual-reviewed"
            },
            "timestamp": 12u64
        });
        let view_id =
            super::recompute_object_id(&view, "view_id", "view").expect("view ID should recompute");
        view["view_id"] = Value::String(view_id);
        view["signature"] = Value::String(sign_value(&signing_key, &view));
        let path = write_test_file(
            "view-wrong-document-key-prefix",
            &serde_json::to_string_pretty(&view).expect("test JSON should serialize"),
        );

        let summary = verify_object_path(&path);

        assert!(!summary.is_ok(), "expected failure, got {summary:?}");
        assert!(
            summary.errors.iter().any(|message| {
                message.contains("top-level 'documents.patch:test key' must use 'doc:' prefix")
            }),
            "expected document key-prefix error, got {summary:?}"
        );

        let _ = std::fs::remove_file(path);
    }

    #[test]
    fn view_non_object_policy_is_rejected_by_typed_validation() {
        let (signing_key, public_key) = signer_material();
        let mut view = json!({
            "type": "view",
            "version": "mycel/0.1",
            "maintainer": public_key,
            "documents": {
                "doc:test": "rev:test"
            },
            "policy": "manual-reviewed",
            "timestamp": 12u64
        });
        let view_id =
            super::recompute_object_id(&view, "view_id", "view").expect("view ID should recompute");
        view["view_id"] = Value::String(view_id);
        view["signature"] = Value::String(sign_value(&signing_key, &view));
        let path = write_test_file(
            "view-policy-non-object",
            &serde_json::to_string_pretty(&view).expect("test JSON should serialize"),
        );

        let summary = verify_object_path(&path);

        assert!(!summary.is_ok(), "expected failure, got {summary:?}");
        assert!(
            summary
                .errors
                .iter()
                .any(|message| message.contains("top-level 'policy' must be an object")),
            "expected non-object policy error, got {summary:?}"
        );

        let _ = std::fs::remove_file(path);
    }

    #[test]
    fn view_with_empty_documents_is_rejected_by_typed_validation() {
        let (signing_key, public_key) = signer_material();
        let mut view = json!({
            "type": "view",
            "version": "mycel/0.1",
            "maintainer": public_key,
            "documents": {},
            "policy": {
                "merge_rule": "manual-reviewed"
            },
            "timestamp": 12u64
        });
        let view_id =
            super::recompute_object_id(&view, "view_id", "view").expect("view ID should recompute");
        view["view_id"] = Value::String(view_id);
        view["signature"] = Value::String(sign_value(&signing_key, &view));
        let path = write_test_file(
            "view-empty-documents",
            &serde_json::to_string_pretty(&view).expect("test JSON should serialize"),
        );

        let summary = verify_object_path(&path);

        assert!(!summary.is_ok(), "expected failure, got {summary:?}");
        assert!(
            summary
                .errors
                .iter()
                .any(|message| message.contains("top-level 'documents' must not be empty")),
            "expected typed view parse error, got {summary:?}"
        );

        let _ = std::fs::remove_file(path);
    }

    #[test]
    fn view_unknown_top_level_field_is_rejected_by_typed_validation() {
        let (signing_key, public_key) = signer_material();
        let mut view = json!({
            "type": "view",
            "version": "mycel/0.1",
            "maintainer": public_key,
            "documents": {
                "doc:test": "rev:test"
            },
            "policy": {
                "merge_rule": "manual-reviewed"
            },
            "timestamp": 12u64,
            "unexpected": true
        });
        let view_id =
            super::recompute_object_id(&view, "view_id", "view").expect("view ID should recompute");
        view["view_id"] = Value::String(view_id);
        view["signature"] = Value::String(sign_value(&signing_key, &view));
        let path = write_test_file(
            "view-unknown-top-level-field",
            &serde_json::to_string_pretty(&view).expect("test JSON should serialize"),
        );

        let summary = verify_object_path(&path);

        assert!(!summary.is_ok(), "expected failure, got {summary:?}");
        assert!(
            summary
                .errors
                .iter()
                .any(|message| message.contains("top-level contains unexpected field 'unexpected'")),
            "expected unknown-field error, got {summary:?}"
        );

        let _ = std::fs::remove_file(path);
    }

    #[test]
    fn revision_replay_verifies_state_hash_from_neighbor_patch() {
        let (signing_key, public_key) = signer_material();
        let dir = write_test_dir("revision-replay-valid");
        let patch_path = dir.join("patch.json");
        let revision_path = dir.join("revision.json");

        let mut patch = json!({
            "type": "patch",
            "version": "mycel/0.1",
            "doc_id": "doc:test",
            "base_revision": "rev:genesis-null",
            "author": public_key,
            "timestamp": 10u64,
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
        let patch_id = super::recompute_object_id(&patch, "patch_id", "patch")
            .expect("patch ID should recompute");
        patch["patch_id"] = Value::String(patch_id.clone());
        patch["signature"] = Value::String(sign_value(&signing_key, &patch));
        std::fs::write(
            &patch_path,
            serde_json::to_string_pretty(&patch).expect("patch should serialize"),
        )
        .expect("patch should write");

        let mut revision = json!({
            "type": "revision",
            "version": "mycel/0.1",
            "doc_id": "doc:test",
            "parents": [],
            "patches": [patch_id],
            "state_hash": state_hash_for_blocks(
                "doc:test",
                vec![BlockObject {
                    block_id: "blk:001".to_string(),
                    block_type: "paragraph".to_string(),
                    content: "Hello".to_string(),
                    attrs: Map::new(),
                    children: Vec::new()
                }]
            ),
            "author": public_key,
            "timestamp": 11u64
        });
        let revision_id = super::recompute_object_id(&revision, "revision_id", "rev")
            .expect("revision ID should recompute");
        revision["revision_id"] = Value::String(revision_id);
        revision["signature"] = Value::String(sign_value(&signing_key, &revision));
        std::fs::write(
            &revision_path,
            serde_json::to_string_pretty(&revision).expect("revision should serialize"),
        )
        .expect("revision should write");

        let summary = verify_object_path(&revision_path);

        assert!(summary.is_ok(), "expected success, got {summary:?}");
        assert_eq!(summary.state_hash_verification.as_deref(), Some("verified"));
        assert_eq!(
            summary.declared_state_hash.as_deref(),
            summary.recomputed_state_hash.as_deref()
        );

        let _ = std::fs::remove_file(patch_path);
        let _ = std::fs::remove_file(revision_path);
        let _ = std::fs::remove_dir(dir);
    }

    #[test]
    fn merge_revision_replay_verifies_state_hash_from_primary_parent_and_patch() {
        let (signing_key, public_key) = signer_material();
        let dir = write_test_dir("revision-replay-merge-valid");
        let base_patch_path = dir.join("patch-base.json");
        let base_revision_path = dir.join("revision-base.json");
        let side_patch_path = dir.join("patch-side.json");
        let side_revision_path = dir.join("revision-side.json");
        let merge_patch_path = dir.join("patch-merge.json");
        let merge_revision_path = dir.join("revision-merge.json");

        let mut base_patch = json!({
            "type": "patch",
            "version": "mycel/0.1",
            "doc_id": "doc:test",
            "base_revision": "rev:genesis-null",
            "author": public_key,
            "timestamp": 10u64,
            "ops": [
                {
                    "op": "insert_block",
                    "new_block": {
                        "block_id": "blk:001",
                        "block_type": "paragraph",
                        "content": "Base",
                        "attrs": {},
                        "children": []
                    }
                }
            ]
        });
        let base_patch_id = super::recompute_object_id(&base_patch, "patch_id", "patch")
            .expect("base patch ID should recompute");
        base_patch["patch_id"] = Value::String(base_patch_id.clone());
        base_patch["signature"] = Value::String(sign_value(&signing_key, &base_patch));
        std::fs::write(
            &base_patch_path,
            serde_json::to_string_pretty(&base_patch).expect("base patch should serialize"),
        )
        .expect("base patch should write");

        let mut base_revision = json!({
            "type": "revision",
            "version": "mycel/0.1",
            "doc_id": "doc:test",
            "parents": [],
            "patches": [base_patch_id.clone()],
            "state_hash": state_hash_for_blocks(
                "doc:test",
                vec![BlockObject {
                    block_id: "blk:001".to_string(),
                    block_type: "paragraph".to_string(),
                    content: "Base".to_string(),
                    attrs: Map::new(),
                    children: Vec::new()
                }]
            ),
            "author": public_key,
            "timestamp": 11u64
        });
        let base_revision_id = super::recompute_object_id(&base_revision, "revision_id", "rev")
            .expect("base revision ID should recompute");
        base_revision["revision_id"] = Value::String(base_revision_id.clone());
        base_revision["signature"] = Value::String(sign_value(&signing_key, &base_revision));
        std::fs::write(
            &base_revision_path,
            serde_json::to_string_pretty(&base_revision).expect("base revision should serialize"),
        )
        .expect("base revision should write");

        let mut side_patch = json!({
            "type": "patch",
            "version": "mycel/0.1",
            "doc_id": "doc:test",
            "base_revision": "rev:genesis-null",
            "author": public_key,
            "timestamp": 12u64,
            "ops": [
                {
                    "op": "insert_block",
                    "new_block": {
                        "block_id": "blk:002",
                        "block_type": "paragraph",
                        "content": "Side",
                        "attrs": {},
                        "children": []
                    }
                }
            ]
        });
        let side_patch_id = super::recompute_object_id(&side_patch, "patch_id", "patch")
            .expect("side patch ID should recompute");
        side_patch["patch_id"] = Value::String(side_patch_id.clone());
        side_patch["signature"] = Value::String(sign_value(&signing_key, &side_patch));
        std::fs::write(
            &side_patch_path,
            serde_json::to_string_pretty(&side_patch).expect("side patch should serialize"),
        )
        .expect("side patch should write");

        let mut side_revision = json!({
            "type": "revision",
            "version": "mycel/0.1",
            "doc_id": "doc:test",
            "parents": [],
            "patches": [side_patch_id.clone()],
            "state_hash": state_hash_for_blocks(
                "doc:test",
                vec![BlockObject {
                    block_id: "blk:002".to_string(),
                    block_type: "paragraph".to_string(),
                    content: "Side".to_string(),
                    attrs: Map::new(),
                    children: Vec::new()
                }]
            ),
            "author": public_key,
            "timestamp": 13u64
        });
        let side_revision_id = super::recompute_object_id(&side_revision, "revision_id", "rev")
            .expect("side revision ID should recompute");
        side_revision["revision_id"] = Value::String(side_revision_id.clone());
        side_revision["signature"] = Value::String(sign_value(&signing_key, &side_revision));
        std::fs::write(
            &side_revision_path,
            serde_json::to_string_pretty(&side_revision).expect("side revision should serialize"),
        )
        .expect("side revision should write");

        let mut merge_patch = json!({
            "type": "patch",
            "version": "mycel/0.1",
            "doc_id": "doc:test",
            "base_revision": base_revision_id.clone(),
            "author": public_key,
            "timestamp": 14u64,
            "ops": [
                {
                    "op": "replace_block",
                    "block_id": "blk:001",
                    "new_content": "Merged"
                }
            ]
        });
        let merge_patch_id = super::recompute_object_id(&merge_patch, "patch_id", "patch")
            .expect("merge patch ID should recompute");
        merge_patch["patch_id"] = Value::String(merge_patch_id.clone());
        merge_patch["signature"] = Value::String(sign_value(&signing_key, &merge_patch));
        std::fs::write(
            &merge_patch_path,
            serde_json::to_string_pretty(&merge_patch).expect("merge patch should serialize"),
        )
        .expect("merge patch should write");

        let mut merge_revision = json!({
            "type": "revision",
            "version": "mycel/0.1",
            "doc_id": "doc:test",
            "parents": [base_revision_id.clone(), side_revision_id.clone()],
            "patches": [merge_patch_id],
            "merge_strategy": "semantic-block-merge",
            "state_hash": state_hash_for_blocks(
                "doc:test",
                vec![BlockObject {
                    block_id: "blk:001".to_string(),
                    block_type: "paragraph".to_string(),
                    content: "Merged".to_string(),
                    attrs: Map::new(),
                    children: Vec::new()
                }]
            ),
            "author": public_key,
            "timestamp": 15u64
        });
        let merge_revision_id = super::recompute_object_id(&merge_revision, "revision_id", "rev")
            .expect("merge revision ID should recompute");
        merge_revision["revision_id"] = Value::String(merge_revision_id);
        merge_revision["signature"] = Value::String(sign_value(&signing_key, &merge_revision));
        std::fs::write(
            &merge_revision_path,
            serde_json::to_string_pretty(&merge_revision).expect("merge revision should serialize"),
        )
        .expect("merge revision should write");

        let summary = verify_object_path(&merge_revision_path);

        assert!(summary.is_ok(), "expected success, got {summary:?}");
        assert_eq!(summary.state_hash_verification.as_deref(), Some("verified"));
        assert_eq!(
            summary.declared_state_hash.as_deref(),
            summary.recomputed_state_hash.as_deref()
        );

        let _ = std::fs::remove_file(base_patch_path);
        let _ = std::fs::remove_file(base_revision_path);
        let _ = std::fs::remove_file(side_patch_path);
        let _ = std::fs::remove_file(side_revision_path);
        let _ = std::fs::remove_file(merge_patch_path);
        let _ = std::fs::remove_file(merge_revision_path);
        let _ = std::fs::remove_dir(dir);
    }

    #[test]
    fn merge_revision_does_not_implicitly_include_secondary_parent_content() {
        let (signing_key, public_key) = signer_material();
        let mut base_revision = json!({
            "type": "revision",
            "version": "mycel/0.1",
            "doc_id": "doc:test",
            "parents": [],
            "patches": [],
            "state_hash": state_hash_for_blocks(
                "doc:test",
                vec![BlockObject {
                    block_id: "blk:001".to_string(),
                    block_type: "paragraph".to_string(),
                    content: "Base".to_string(),
                    attrs: Map::new(),
                    children: Vec::new()
                }]
            ),
            "author": public_key,
            "timestamp": 9u64
        });
        let base_revision_id = super::recompute_object_id(&base_revision, "revision_id", "rev")
            .expect("base revision ID should recompute");
        base_revision["revision_id"] = Value::String(base_revision_id.clone());
        base_revision["signature"] = Value::String(sign_value(&signing_key, &base_revision));

        let mut side_revision = json!({
            "type": "revision",
            "version": "mycel/0.1",
            "doc_id": "doc:test",
            "parents": [],
            "patches": [],
            "state_hash": state_hash_for_blocks(
                "doc:test",
                vec![BlockObject {
                    block_id: "blk:002".to_string(),
                    block_type: "paragraph".to_string(),
                    content: "Side".to_string(),
                    attrs: Map::new(),
                    children: Vec::new()
                }]
            ),
            "author": public_key,
            "timestamp": 10u64
        });
        let side_revision_id = super::recompute_object_id(&side_revision, "revision_id", "rev")
            .expect("side revision ID should recompute");
        side_revision["revision_id"] = Value::String(side_revision_id.clone());
        side_revision["signature"] = Value::String(sign_value(&signing_key, &side_revision));

        let mut merge_revision = json!({
            "type": "revision",
            "version": "mycel/0.1",
            "doc_id": "doc:test",
            "parents": [base_revision_id.clone(), side_revision_id.clone()],
            "patches": [],
            "merge_strategy": "semantic-block-merge",
            "state_hash": state_hash_for_blocks(
                "doc:test",
                vec![
                    BlockObject {
                        block_id: "blk:001".to_string(),
                        block_type: "paragraph".to_string(),
                        content: "Base".to_string(),
                        attrs: Map::new(),
                        children: Vec::new()
                    },
                    BlockObject {
                        block_id: "blk:002".to_string(),
                        block_type: "paragraph".to_string(),
                        content: "Side".to_string(),
                        attrs: Map::new(),
                        children: Vec::new()
                    }
                ]
            ),
            "author": public_key,
            "timestamp": 11u64
        });
        let merge_revision_id = super::recompute_object_id(&merge_revision, "revision_id", "rev")
            .expect("merge revision ID should recompute");
        merge_revision["revision_id"] = Value::String(merge_revision_id);
        merge_revision["signature"] = Value::String(sign_value(&signing_key, &merge_revision));

        let object_index = HashMap::from([
            (base_revision_id, base_revision),
            (side_revision_id, side_revision),
        ]);
        let summary = verify_object_value_with_object_index(&merge_revision, Some(&object_index));

        assert!(!summary.is_ok(), "expected failure, got {summary:?}");
        assert_eq!(summary.state_hash_verification.as_deref(), Some("failed"));
        assert!(
            summary.errors.iter().any(|message| message
                .contains("declared state_hash does not match replayed state hash")),
            "expected ancestry-only state mismatch error, got {summary:?}"
        );
    }

    #[test]
    fn revision_replay_rejects_state_hash_mismatch() {
        let (signing_key, public_key) = signer_material();
        let dir = write_test_dir("revision-replay-mismatch");
        let patch_path = dir.join("patch.json");
        let revision_path = dir.join("revision.json");

        let mut patch = json!({
            "type": "patch",
            "version": "mycel/0.1",
            "doc_id": "doc:test",
            "base_revision": "rev:genesis-null",
            "author": public_key,
            "timestamp": 10u64,
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
        let patch_id = super::recompute_object_id(&patch, "patch_id", "patch")
            .expect("patch ID should recompute");
        patch["patch_id"] = Value::String(patch_id.clone());
        patch["signature"] = Value::String(sign_value(&signing_key, &patch));
        std::fs::write(
            &patch_path,
            serde_json::to_string_pretty(&patch).expect("patch should serialize"),
        )
        .expect("patch should write");

        let mut revision = json!({
            "type": "revision",
            "version": "mycel/0.1",
            "doc_id": "doc:test",
            "parents": [],
            "patches": [patch_id],
            "state_hash": "hash:wrong",
            "author": public_key,
            "timestamp": 11u64
        });
        let revision_id = super::recompute_object_id(&revision, "revision_id", "rev")
            .expect("revision ID should recompute");
        revision["revision_id"] = Value::String(revision_id);
        revision["signature"] = Value::String(sign_value(&signing_key, &revision));
        std::fs::write(
            &revision_path,
            serde_json::to_string_pretty(&revision).expect("revision should serialize"),
        )
        .expect("revision should write");

        let summary = verify_object_path(&revision_path);

        assert!(!summary.is_ok(), "expected failure, got {summary:?}");
        assert_eq!(summary.state_hash_verification.as_deref(), Some("failed"));
        assert!(
            summary.errors.iter().any(|message| message
                .contains("declared state_hash does not match replayed state hash")),
            "expected state-hash mismatch error, got {summary:?}"
        );

        let _ = std::fs::remove_file(patch_path);
        let _ = std::fs::remove_file(revision_path);
        let _ = std::fs::remove_dir(dir);
    }

    #[test]
    fn revision_replay_rejects_genesis_patch_with_non_genesis_base_revision() {
        let (signing_key, public_key) = signer_material();
        let mut patch = json!({
            "type": "patch",
            "version": "mycel/0.1",
            "doc_id": "doc:test",
            "base_revision": "rev:wrong-base",
            "author": public_key,
            "timestamp": 10u64,
            "ops": []
        });
        let patch_id = super::recompute_object_id(&patch, "patch_id", "patch")
            .expect("patch ID should recompute");
        patch["patch_id"] = Value::String(patch_id.clone());
        patch["signature"] = Value::String(sign_value(&signing_key, &patch));

        let mut revision = json!({
            "type": "revision",
            "version": "mycel/0.1",
            "doc_id": "doc:test",
            "parents": [],
            "patches": [patch_id.clone()],
            "state_hash": "hash:test",
            "author": public_key,
            "timestamp": 11u64
        });
        let revision_id = super::recompute_object_id(&revision, "revision_id", "rev")
            .expect("revision ID should recompute");
        revision["revision_id"] = Value::String(revision_id);
        revision["signature"] = Value::String(sign_value(&signing_key, &revision));

        let object_index = HashMap::from([(patch_id, patch)]);
        let summary = verify_object_value_with_object_index(&revision, Some(&object_index));

        assert!(!summary.is_ok(), "expected failure, got {summary:?}");
        assert_eq!(summary.state_hash_verification.as_deref(), Some("failed"));
        assert!(
            summary.errors.iter().any(|message| {
                message.contains("base_revision 'rev:wrong-base'")
                    && message.contains("expected 'rev:genesis-null'")
            }),
            "expected genesis base_revision mismatch error, got {summary:?}"
        );
    }

    #[test]
    fn revision_replay_rejects_non_genesis_patch_with_wrong_parent_base_revision() {
        let (signing_key, public_key) = signer_material();
        let base_hash = compute_state_hash(&DocumentState {
            doc_id: "doc:test".to_string(),
            blocks: Vec::new(),
            metadata: Map::new(),
        })
        .expect("empty state hash should compute");
        let mut base_revision = json!({
            "type": "revision",
            "version": "mycel/0.1",
            "doc_id": "doc:test",
            "parents": [],
            "patches": [],
            "state_hash": base_hash,
            "author": public_key,
            "timestamp": 9u64
        });
        let base_revision_id = super::recompute_object_id(&base_revision, "revision_id", "rev")
            .expect("base revision ID should recompute");
        base_revision["revision_id"] = Value::String(base_revision_id.clone());
        base_revision["signature"] = Value::String(sign_value(&signing_key, &base_revision));

        let mut patch = json!({
            "type": "patch",
            "version": "mycel/0.1",
            "doc_id": "doc:test",
            "base_revision": "rev:wrong-base",
            "author": public_key,
            "timestamp": 10u64,
            "ops": []
        });
        let patch_id = super::recompute_object_id(&patch, "patch_id", "patch")
            .expect("patch ID should recompute");
        patch["patch_id"] = Value::String(patch_id.clone());
        patch["signature"] = Value::String(sign_value(&signing_key, &patch));

        let mut revision = json!({
            "type": "revision",
            "version": "mycel/0.1",
            "doc_id": "doc:test",
            "parents": [base_revision_id.clone()],
            "patches": [patch_id.clone()],
            "state_hash": compute_state_hash(&DocumentState {
                doc_id: "doc:test".to_string(),
                blocks: Vec::new(),
                metadata: Map::new(),
            })
            .expect("empty state hash should compute"),
            "author": public_key,
            "timestamp": 11u64
        });
        let revision_id = super::recompute_object_id(&revision, "revision_id", "rev")
            .expect("revision ID should recompute");
        revision["revision_id"] = Value::String(revision_id);
        revision["signature"] = Value::String(sign_value(&signing_key, &revision));

        let object_index = HashMap::from([(base_revision_id, base_revision), (patch_id, patch)]);
        let summary = verify_object_value_with_object_index(&revision, Some(&object_index));

        assert!(!summary.is_ok(), "expected failure, got {summary:?}");
        assert_eq!(summary.state_hash_verification.as_deref(), Some("failed"));
        assert!(
            summary.errors.iter().any(|message| {
                message.contains("base_revision 'rev:wrong-base'")
                    && message.contains("does not match expected 'rev:")
            }),
            "expected non-genesis base_revision mismatch error, got {summary:?}"
        );
    }

    #[test]
    fn revision_replay_rejects_patch_from_other_document() {
        let (signing_key, public_key) = signer_material();
        let mut patch = json!({
            "type": "patch",
            "version": "mycel/0.1",
            "doc_id": "doc:other",
            "base_revision": "rev:genesis-null",
            "author": public_key,
            "timestamp": 10u64,
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
        let patch_id = super::recompute_object_id(&patch, "patch_id", "patch")
            .expect("patch ID should recompute");
        patch["patch_id"] = Value::String(patch_id.clone());
        patch["signature"] = Value::String(sign_value(&signing_key, &patch));

        let mut revision = json!({
            "type": "revision",
            "version": "mycel/0.1",
            "doc_id": "doc:test",
            "parents": [],
            "patches": [patch_id.clone()],
            "state_hash": "hash:wrong",
            "author": public_key,
            "timestamp": 11u64
        });
        let revision_id = super::recompute_object_id(&revision, "revision_id", "rev")
            .expect("revision ID should recompute");
        revision["revision_id"] = Value::String(revision_id);
        revision["signature"] = Value::String(sign_value(&signing_key, &revision));

        let object_index = HashMap::from([(patch_id, patch)]);
        let summary = verify_object_value_with_object_index(&revision, Some(&object_index));

        assert!(!summary.is_ok(), "expected failure, got {summary:?}");
        assert_eq!(summary.state_hash_verification.as_deref(), Some("failed"));
        assert!(
            summary.errors.iter().any(|message| {
                message.contains("patch '")
                    && message.contains("belongs to 'doc:other' instead of 'doc:test'")
            }),
            "expected cross-document replay error, got {summary:?}"
        );
    }
}
