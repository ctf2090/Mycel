use std::collections::{HashMap, HashSet};
use std::fs;
use std::path::{Path, PathBuf};

use serde::Serialize;
use serde_json::Value;

use crate::protocol::{
    collect_unsupported_json_value_errors, parse_block_object, parse_document_object,
    parse_json_value_strict, parse_object_envelope, parse_patch_object, parse_revision_object,
    parse_snapshot_object, parse_view_object, recompute_object_id, signed_payload_bytes,
    ParseObjectEnvelopeError, StringFieldError,
};
use crate::replay::replay_revision_from_index;
use crate::signature::verify_ed25519_signature;

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

    if let Some((id_field, _prefix)) = schema.derived_id() {
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
    }

    if summary.errors.is_empty() {
        validate_typed_object_shape(object_type, &value, &mut summary);
    }

    if let Some((id_field, prefix)) = schema.derived_id() {
        if summary.errors.is_empty() {
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

fn append_inspection_shape_notes(
    object_type: &str,
    value: &Value,
    summary: &mut ObjectInspectionSummary,
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
        let message = error.to_string();
        if !summary.notes.iter().any(|note| note == &message) {
            summary.push_note(message);
        }
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

    append_inspection_shape_notes(object_type, &value, &mut summary);

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

    verify_revision_dependencies(value, object_index)
        .map_err(|error| format!("revision replay failed: {error}"))?;
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

fn verify_revision_dependencies(
    revision_value: &Value,
    object_index: &HashMap<String, Value>,
) -> Result<(), String> {
    let revision = parse_revision_object(revision_value)
        .map_err(|error| format!("failed to parse revision object: {error}"))?;
    let mut verified_revisions = HashSet::new();
    let mut visiting_revisions = HashSet::new();
    let mut verified_patches = HashSet::new();
    verify_revision_dependency_closure(
        &revision,
        object_index,
        &mut verified_revisions,
        &mut visiting_revisions,
        &mut verified_patches,
    )
}

fn verify_revision_dependency_closure(
    revision: &crate::protocol::RevisionObject,
    object_index: &HashMap<String, Value>,
    verified_revisions: &mut HashSet<String>,
    visiting_revisions: &mut HashSet<String>,
    verified_patches: &mut HashSet<String>,
) -> Result<(), String> {
    if verified_revisions.contains(&revision.revision_id) {
        return Ok(());
    }

    if !visiting_revisions.insert(revision.revision_id.clone()) {
        return Err(format!(
            "revision replay dependency cycle detected at '{}'",
            revision.revision_id
        ));
    }

    for parent_id in &revision.parents {
        let parent_value = object_index
            .get(parent_id)
            .ok_or_else(|| format!("missing parent revision '{parent_id}' for replay"))?;
        verify_replay_dependency_object("parent revision", parent_id, "revision", parent_value)?;
        let parent_revision = parse_revision_object(parent_value)
            .map_err(|error| format!("failed to parse parent revision '{parent_id}': {error}"))?;
        verify_revision_dependency_closure(
            &parent_revision,
            object_index,
            verified_revisions,
            visiting_revisions,
            verified_patches,
        )
        .map_err(|error| {
            format!("while verifying ancestry through parent revision '{parent_id}': {error}")
        })?;
    }

    for patch_id in &revision.patches {
        if !verified_patches.insert(patch_id.clone()) {
            continue;
        }
        let patch_value = object_index
            .get(patch_id)
            .ok_or_else(|| format!("missing patch '{patch_id}' for replay"))?;
        verify_replay_dependency_object("patch", patch_id, "patch", patch_value)?;
    }

    visiting_revisions.remove(&revision.revision_id);
    verified_revisions.insert(revision.revision_id.clone());
    Ok(())
}

fn verify_replay_dependency_object(
    label: &str,
    object_id: &str,
    expected_type: &str,
    value: &Value,
) -> Result<(), String> {
    let summary = verify_object_value(value);
    if !summary.is_ok() {
        return Err(format!(
            "failed to verify {label} '{object_id}': {}",
            summary.errors.join("; ")
        ));
    }

    match summary.object_type.as_deref() {
        Some(actual_type) if actual_type == expected_type => {}
        Some(actual_type) => {
            return Err(format!(
                "{label} '{object_id}' is a '{actual_type}' object instead of '{expected_type}'"
            ))
        }
        None => {
            return Err(format!(
                "{label} '{object_id}' does not declare an object type"
            ))
        }
    }

    match summary.declared_id.as_deref() {
        Some(declared_id) if declared_id == object_id => Ok(()),
        Some(declared_id) => Err(format!(
            "{label} '{object_id}' is declared as '{declared_id}' instead"
        )),
        None => Err(format!("{label} '{object_id}' is missing its declared ID")),
    }
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
        let declared_id = declared_id.to_string();
        if let Some(existing) = object_index.insert(declared_id.clone(), value) {
            let _ = existing;
            return Err(format!(
                "duplicate sibling object ID '{}' found while loading replay objects",
                declared_id
            ));
        }
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
    let payload = signed_payload_bytes(value)?;
    verify_ed25519_signature(
        &payload,
        signer,
        signature,
        "signer field",
        "signature field",
    )
}

#[cfg(test)]
mod tests {
    use std::collections::HashMap;

    use base64::Engine;
    use ed25519_dalek::{Signer, SigningKey};
    use rstest::rstest;
    use serde_json::{json, Map, Value};

    use super::{
        inspect_object_path, verify_object_path, verify_object_value_with_object_index,
        ObjectInspectionSummary, ObjectVerificationSummary,
    };
    use crate::protocol::{recompute_object_id, signed_payload_bytes, BlockObject};
    use crate::replay::{compute_state_hash, DocumentState};

    #[path = "fixtures.rs"]
    mod fixtures;

    #[path = "inspection.rs"]
    mod inspection;
    #[path = "replay.rs"]
    mod replay;
    #[path = "typed_validation.rs"]
    mod typed_validation;
}
