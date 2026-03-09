use std::fs;
use std::path::{Path, PathBuf};

use serde::Serialize;
use serde_json::{Map, Value};
use sha2::{Digest, Sha256};

#[derive(Debug, Clone, Serialize)]
pub struct ObjectVerificationSummary {
    pub path: PathBuf,
    pub status: String,
    pub object_type: Option<String>,
    pub signature_rule: Option<String>,
    pub signer_field: Option<String>,
    pub signer: Option<String>,
    pub declared_id: Option<String>,
    pub recomputed_id: Option<String>,
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

    let object_type = match object.get("type").and_then(Value::as_str) {
        Some(object_type) => object_type,
        None => {
            summary.push_error("object is missing string field 'type'");
            return summary;
        }
    };

    summary.object_type = Some(object_type.to_string());

    let descriptor = match object_descriptor(object_type) {
        Some(descriptor) => descriptor,
        None => {
            summary.push_error(format!("unsupported object type '{object_type}'"));
            return summary;
        }
    };

    summary.signature_rule = Some(descriptor.signature_rule.to_string());

    if let Some(signer_field) = descriptor.signer_field {
        summary.signer_field = Some(signer_field.to_string());
    }

    if descriptor.signature_required {
        let Some(signature) = object.get("signature") else {
            summary.push_error(format!(
                "{object_type} object is missing required top-level 'signature'"
            ));
            return finalize_signed_summary(summary, object, descriptor);
        };

        if !signature.is_string() {
            summary.push_error("top-level 'signature' must be a string");
        }

        if let Some(signer_field) = descriptor.signer_field {
            match object.get(signer_field).and_then(Value::as_str) {
                Some(signer) => summary.signer = Some(signer.to_string()),
                None => summary.push_error(format!(
                    "{object_type} object is missing string signer field '{signer_field}'"
                )),
            }
        }
    } else if object.contains_key("signature") {
        summary.push_error(format!(
            "{object_type} object must not include top-level 'signature'"
        ));
    }

    if let Some((id_field, prefix)) = descriptor.derived_id {
        match object.get(id_field).and_then(Value::as_str) {
            Some(declared_id) => summary.declared_id = Some(declared_id.to_string()),
            None => summary.push_error(format!(
                "{object_type} object is missing string field '{id_field}'"
            )),
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

    finalize_signed_summary(summary, object, descriptor)
}

fn finalize_signed_summary(
    mut summary: ObjectVerificationSummary,
    object: &Map<String, Value>,
    descriptor: ObjectTypeDescriptor,
) -> ObjectVerificationSummary {
    if descriptor.signature_required && object.contains_key("signature") {
        summary.notes.push(
            "cryptographic signature verification is not implemented yet; only signature presence and signer-field checks ran"
                .to_string(),
        );
    }

    if summary.errors.is_empty() {
        summary.status = "ok".to_string();
    } else {
        summary.status = "failed".to_string();
    }

    summary
}

#[derive(Copy, Clone)]
struct ObjectTypeDescriptor {
    signature_required: bool,
    signature_rule: &'static str,
    signer_field: Option<&'static str>,
    derived_id: Option<(&'static str, &'static str)>,
}

fn object_descriptor(object_type: &str) -> Option<ObjectTypeDescriptor> {
    match object_type {
        "document" => Some(ObjectTypeDescriptor {
            signature_required: false,
            signature_rule: "forbidden",
            signer_field: None,
            derived_id: None,
        }),
        "block" => Some(ObjectTypeDescriptor {
            signature_required: false,
            signature_rule: "forbidden",
            signer_field: None,
            derived_id: None,
        }),
        "patch" => Some(ObjectTypeDescriptor {
            signature_required: true,
            signature_rule: "required",
            signer_field: Some("author"),
            derived_id: Some(("patch_id", "patch")),
        }),
        "revision" => Some(ObjectTypeDescriptor {
            signature_required: true,
            signature_rule: "required",
            signer_field: Some("author"),
            derived_id: Some(("revision_id", "rev")),
        }),
        "view" => Some(ObjectTypeDescriptor {
            signature_required: true,
            signature_rule: "required",
            signer_field: Some("maintainer"),
            derived_id: Some(("view_id", "view")),
        }),
        "snapshot" => Some(ObjectTypeDescriptor {
            signature_required: true,
            signature_rule: "required",
            signer_field: Some("created_by"),
            derived_id: Some(("snapshot_id", "snap")),
        }),
        _ => None,
    }
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

fn canonical_json(value: &Value) -> Result<String, String> {
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

fn hex_encode(bytes: &[u8]) -> String {
    let mut output = String::with_capacity(bytes.len() * 2);
    for byte in bytes {
        output.push_str(&format!("{byte:02x}"));
    }
    output
}

#[cfg(test)]
mod tests {
    use serde_json::json;

    use super::verify_object_path;

    fn write_test_file(name: &str, content: &str) -> std::path::PathBuf {
        let unique = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .expect("system time should be after unix epoch")
            .as_nanos();
        let path = std::env::temp_dir().join(format!("mycel-core-{name}-{unique}.json"));
        std::fs::write(&path, content).expect("test JSON should be written");
        path
    }

    #[test]
    fn patch_id_recomputes_from_canonical_json() {
        let path = write_test_file(
            "patch-valid",
            &serde_json::to_string_pretty(&json!({
                "type": "patch",
                "version": "mycel/0.1",
                "doc_id": "doc:test",
                "base_revision": "rev:genesis-null",
                "author": "pk:authorA",
                "timestamp": 1777778888u64,
                "ops": [],
                "patch_id": "patch:76d519509ad9f7b9c2bf4a7a4def39ff5f9c5e4fb4d798e9c8cfdfa2cb48bc43",
                "signature": "sig:test"
            }))
            .expect("test JSON should serialize"),
        );

        let summary = verify_object_path(&path);

        assert!(summary.is_ok(), "expected success, got {summary:?}");
        assert_eq!(
            summary.recomputed_id.as_deref(),
            Some("patch:76d519509ad9f7b9c2bf4a7a4def39ff5f9c5e4fb4d798e9c8cfdfa2cb48bc43")
        );

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
}
