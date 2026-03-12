use base64::Engine;
use ed25519_dalek::{Signature, Verifier, VerifyingKey};

pub fn parse_ed25519_public_key(value: &str, label: &str) -> Result<VerifyingKey, String> {
    let encoded = value
        .strip_prefix("pk:ed25519:")
        .ok_or_else(|| format!("{label} must use format 'pk:ed25519:<base64>'"))?;
    let decoded = base64::engine::general_purpose::STANDARD
        .decode(encoded)
        .map_err(|err| format!("failed to decode Ed25519 public key: {err}"))?;
    let bytes: [u8; 32] = decoded
        .try_into()
        .map_err(|_| "Ed25519 public key must decode to 32 bytes".to_string())?;
    VerifyingKey::from_bytes(&bytes)
        .map_err(|err| format!("invalid Ed25519 public key bytes: {err}"))
}

pub fn parse_ed25519_signature(value: &str, label: &str) -> Result<Signature, String> {
    let encoded = value
        .strip_prefix("sig:ed25519:")
        .ok_or_else(|| format!("{label} must use format 'sig:ed25519:<base64>'"))?;
    let decoded = base64::engine::general_purpose::STANDARD
        .decode(encoded)
        .map_err(|err| format!("failed to decode Ed25519 signature: {err}"))?;
    Signature::from_slice(&decoded).map_err(|err| format!("invalid Ed25519 signature bytes: {err}"))
}

pub fn verify_ed25519_signature(
    payload: &[u8],
    public_key_value: &str,
    signature_value: &str,
    public_key_label: &str,
    signature_label: &str,
) -> Result<(), String> {
    let public_key = parse_ed25519_public_key(public_key_value, public_key_label)?;
    let signature = parse_ed25519_signature(signature_value, signature_label)?;

    public_key
        .verify(payload, &signature)
        .map_err(|err| format!("Ed25519 signature verification failed: {err}"))
}
