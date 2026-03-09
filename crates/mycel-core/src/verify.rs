use std::fs;
use std::path::{Path, PathBuf};

use base64::Engine;
use ed25519_dalek::{Signature, Verifier, VerifyingKey};
use serde::Serialize;
use serde_json::Value;
use sha2::{Digest, Sha256};

use crate::protocol::{parse_object_envelope, ParseObjectEnvelopeError, StringFieldError};

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

    let value: Value = match serde_json::from_str(&content) {
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

    let value: Value = match serde_json::from_str(&content) {
        Ok(value) => value,
        Err(err) => {
            summary.push_error(format!("failed to parse JSON: {err}"));
            return summary;
        }
    };

    verify_object_value_with_summary(path, value, summary)
}

pub fn verify_object_value(value: &Value) -> ObjectVerificationSummary {
    verify_object_value_with_summary(
        Path::new("<inline-object>"),
        value.clone(),
        ObjectVerificationSummary::new(Path::new("<inline-object>")),
    )
}

fn verify_object_value_with_summary(
    path: &Path,
    value: Value,
    mut summary: ObjectVerificationSummary,
) -> ObjectVerificationSummary {
    summary.path = path.to_path_buf();

    collect_value_errors(&value, "$", &mut summary.errors);
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

    finalize_signed_summary(summary)
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

fn finalize_signed_summary(mut summary: ObjectVerificationSummary) -> ObjectVerificationSummary {
    if summary.errors.is_empty() {
        summary.status = "ok".to_string();
    } else {
        summary.status = "failed".to_string();
    }

    summary
}

fn collect_value_errors(value: &Value, path: &str, errors: &mut Vec<String>) {
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
                collect_value_errors(entry, &entry_path, errors);
            }
        }
        Value::Object(entries) => {
            for (key, entry) in entries {
                let entry_path = format!("{path}.{key}");
                collect_value_errors(entry, &entry_path, errors);
            }
        }
    }
}

fn recompute_object_id(
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

fn signed_payload_bytes(value: &Value) -> Result<Vec<u8>, String> {
    let mut object = value
        .as_object()
        .cloned()
        .ok_or_else(|| "top-level JSON value must be an object".to_string())?;
    object.remove("signature");
    let canonical = canonical_json(&Value::Object(object))?;
    Ok(canonical.into_bytes())
}

pub(crate) fn canonical_json(value: &Value) -> Result<String, String> {
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

pub(crate) fn hex_encode(bytes: &[u8]) -> String {
    let mut output = String::with_capacity(bytes.len() * 2);
    for byte in bytes {
        output.push_str(&format!("{byte:02x}"));
    }
    output
}

#[cfg(test)]
mod tests {
    use base64::Engine;
    use ed25519_dalek::{Signer, SigningKey};
    use serde_json::{json, Value};

    use super::{inspect_object_path, verify_object_path};

    fn write_test_file(name: &str, content: &str) -> std::path::PathBuf {
        let unique = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .expect("system time should be after unix epoch")
            .as_nanos();
        let path = std::env::temp_dir().join(format!("mycel-core-{name}-{unique}.json"));
        std::fs::write(&path, content).expect("test JSON should be written");
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
}
