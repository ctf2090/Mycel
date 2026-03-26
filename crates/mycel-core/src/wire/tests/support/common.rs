use super::*;

pub(crate) fn signing_key() -> SigningKey {
    SigningKey::from_bytes(&[9u8; 32])
}

pub(crate) fn temp_dir(prefix: &str) -> PathBuf {
    let unique = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .expect("system time should be after unix epoch")
        .as_nanos();
    let path = std::env::temp_dir().join(format!("mycel-wire-{prefix}-{unique}"));
    fs::create_dir_all(&path).expect("temp dir should be created");
    path
}

pub(crate) fn sender_public_key(signing_key: &SigningKey) -> String {
    format!(
        "pk:ed25519:{}",
        base64::engine::general_purpose::STANDARD.encode(signing_key.verifying_key().as_bytes())
    )
}

pub(crate) fn sign_wire_value(signing_key: &SigningKey, value: &Value) -> String {
    let payload =
        wire_envelope_signed_payload_bytes(value).expect("wire payload should canonicalize");
    let signature = signing_key.sign(&payload);
    format!(
        "sig:ed25519:{}",
        base64::engine::general_purpose::STANDARD.encode(signature.to_bytes())
    )
}

pub(crate) fn sign_object_value(
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

pub(crate) fn empty_state_hash(doc_id: &str) -> String {
    compute_state_hash(&DocumentState {
        doc_id: doc_id.to_string(),
        blocks: Vec::new(),
        metadata: serde_json::Map::new(),
    })
    .expect("empty state hash should compute")
}
