use serde_json::{Map, Value};
use sha2::{Digest, Sha256};

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

pub fn canonical_bytes(value: &Value) -> Result<Vec<u8>, String> {
    canonical_json(value).map(String::into_bytes)
}

pub fn canonical_object_json_excluding_fields(
    value: &Value,
    omitted_fields: &[&str],
) -> Result<String, String> {
    let object = object_without_fields(value, omitted_fields)?;
    canonical_json(&Value::Object(object))
}

pub fn canonical_object_bytes_excluding_fields(
    value: &Value,
    omitted_fields: &[&str],
) -> Result<Vec<u8>, String> {
    canonical_object_json_excluding_fields(value, omitted_fields).map(String::into_bytes)
}

pub fn canonical_sha256_hex(value: &Value) -> Result<String, String> {
    let canonical = canonical_bytes(value)?;
    Ok(sha256_hex(&canonical))
}

pub fn canonical_object_sha256_hex_excluding_fields(
    value: &Value,
    omitted_fields: &[&str],
) -> Result<String, String> {
    let canonical = canonical_object_bytes_excluding_fields(value, omitted_fields)?;
    Ok(sha256_hex(&canonical))
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
    canonical_object_bytes_excluding_fields(value, &[signature_field])
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

fn sha256_hex(bytes: &[u8]) -> String {
    let mut hasher = Sha256::new();
    hasher.update(bytes);
    hex_encode(&hasher.finalize())
}

fn hex_encode(bytes: &[u8]) -> String {
    let mut output = String::with_capacity(bytes.len() * 2);
    for byte in bytes {
        output.push_str(&format!("{byte:02x}"));
    }
    output
}
