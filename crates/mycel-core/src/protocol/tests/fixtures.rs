use super::*;

pub(super) fn strict_id_case_value(kind: &str) -> Value {
    match kind {
        "document" => json!({
            "type": "document",
            "version": "mycel/0.1",
            "doc_id": "doc:test",
            "title": "Origin Text",
            "language": "zh-Hant",
            "content_model": "block-tree",
            "created_at": 1u64,
            "created_by": "pk:ed25519:test",
            "genesis_revision": "rev:test"
        }),
        "block" => json!({
            "block_id": "blk:001",
            "block_type": "paragraph",
            "content": "Hello",
            "attrs": {},
            "children": []
        }),
        "patch" => json!({
            "type": "patch",
            "version": "mycel/0.1",
            "patch_id": "patch:test",
            "doc_id": "doc:test",
            "base_revision": "rev:base",
            "author": "pk:ed25519:test",
            "timestamp": 1u64,
            "ops": []
        }),
        "revision" => json!({
            "type": "revision",
            "version": "mycel/0.1",
            "revision_id": "rev:test",
            "doc_id": "doc:test",
            "parents": ["rev:base"],
            "patches": ["patch:test"],
            "state_hash": "hash:test",
            "author": "pk:ed25519:test",
            "timestamp": 2u64
        }),
        "view" => json!({
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
        }),
        "snapshot" => json!({
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
        }),
        _ => panic!("unknown strict ID case: {kind}"),
    }
}

pub(super) fn parse_strict_id_case(kind: &str, value: &Value) -> String {
    let error = match kind {
        "document" => parse_document_object(value).unwrap_err(),
        "block" => parse_block_object(value).unwrap_err(),
        "patch" => parse_patch_object(value).unwrap_err(),
        "revision" => parse_revision_object(value).unwrap_err(),
        "view" => parse_view_object(value).unwrap_err(),
        "snapshot" => parse_snapshot_object(value).unwrap_err(),
        _ => panic!("unknown strict ID case: {kind}"),
    };

    error.to_string()
}

pub(super) fn protocol_spec_document_example() -> Value {
    json!({
        "type": "document",
        "version": "mycel/0.1",
        "doc_id": "doc:origin-text",
        "title": "Origin Text",
        "language": "zh-Hant",
        "content_model": "block-tree",
        "created_at": 1777777777u64,
        "created_by": "pk:authorA",
        "genesis_revision": "rev:0ab1"
    })
}

pub(super) fn protocol_spec_block_example() -> Value {
    json!({
        "type": "block",
        "block_id": "blk:001",
        "block_type": "paragraph",
        "content": "At first there was no final draft, only transmission.",
        "attrs": {},
        "children": []
    })
}

pub(super) fn protocol_spec_patch_example() -> Value {
    json!({
        "type": "patch",
        "version": "mycel/0.1",
        "patch_id": "patch:91ac",
        "doc_id": "doc:origin-text",
        "base_revision": "rev:0ab1",
        "author": "pk:authorA",
        "timestamp": 1777778888u64,
        "ops": [
            {
                "op": "replace_block",
                "block_id": "blk:001",
                "new_content": "At first there was no final draft, only transmission and rewriting."
            },
            {
                "op": "insert_block_after",
                "after_block_id": "blk:001",
                "new_block": {
                    "block_id": "blk:002",
                    "block_type": "paragraph",
                    "content": "Whatever is written can be rewritten.",
                    "attrs": {},
                    "children": []
                }
            }
        ],
        "signature": "sig:..."
    })
}

pub(super) fn protocol_spec_revision_example() -> Value {
    json!({
        "type": "revision",
        "version": "mycel/0.1",
        "revision_id": "rev:8fd2",
        "doc_id": "doc:origin-text",
        "parents": ["rev:0ab1"],
        "patches": ["patch:91ac"],
        "state_hash": "hash:state001",
        "author": "pk:authorA",
        "timestamp": 1777778890u64,
        "signature": "sig:..."
    })
}

pub(super) fn protocol_spec_merge_revision_example() -> Value {
    json!({
        "type": "revision",
        "version": "mycel/0.1",
        "revision_id": "rev:c7d4",
        "doc_id": "doc:origin-text",
        "parents": ["rev:8fd2", "rev:b351"],
        "patches": ["patch:a12f"],
        "state_hash": "hash:merged-state",
        "author": "pk:curator1",
        "timestamp": 1777780000u64,
        "merge_strategy": "semantic-block-merge",
        "signature": "sig:..."
    })
}

pub(super) fn protocol_spec_view_example() -> Value {
    json!({
        "type": "view",
        "version": "mycel/0.1",
        "view_id": "view:9aa0",
        "maintainer": "pk:community-curator",
        "documents": {
            "doc:origin-text": "rev:c7d4",
            "doc:governance-rules": "rev:91de"
        },
        "policy": {
            "preferred_branches": ["community-mainline"],
            "accept_keys": ["pk:community-curator", "pk:reviewerB"],
            "merge_rule": "manual-reviewed"
        },
        "timestamp": 1777781000u64,
        "signature": "sig:..."
    })
}

pub(super) fn protocol_spec_snapshot_example() -> Value {
    json!({
        "type": "snapshot",
        "version": "mycel/0.1",
        "snapshot_id": "snap:44cc",
        "documents": {
            "doc:origin-text": "rev:c7d4"
        },
        "included_objects": [
            "rev:c7d4",
            "patch:91ac",
            "patch:a12f"
        ],
        "root_hash": "hash:snapshot-root",
        "created_by": "pk:mirrorA",
        "timestamp": 1777782000u64,
        "signature": "sig:..."
    })
}

pub(super) fn wire_protocol_hello_example() -> Value {
    json!({
        "type": "HELLO",
        "version": "mycel-wire/0.1",
        "msg_id": "msg:hello-001",
        "timestamp": "2026-03-08T20:00:00+08:00",
        "from": "node:alpha",
        "payload": {
            "node_id": "node:alpha",
            "agent": "mycel-node/0.1",
            "capabilities": ["patch-sync", "snapshot-sync", "view-sync"],
            "topics": ["text/core", "text/commentary"],
            "nonce": "n:01f4..."
        },
        "sig": "sig:..."
    })
}

pub(super) fn wire_protocol_manifest_example() -> Value {
    json!({
        "type": "MANIFEST",
        "version": "mycel-wire/0.1",
        "msg_id": "msg:manifest-001",
        "timestamp": "2026-03-08T20:00:10+08:00",
        "from": "node:alpha",
        "payload": {
            "node_id": "node:alpha",
            "capabilities": ["patch-sync", "snapshot-sync", "view-sync"],
            "topics": ["text/core", "text/commentary"],
            "heads": {
                "doc:origin-text": ["rev:c7d4", "rev:b351"]
            },
            "snapshots": ["snap:44cc"],
            "views": ["view:9aa0"]
        },
        "sig": "sig:..."
    })
}

pub(super) fn wire_protocol_heads_example() -> Value {
    json!({
        "type": "HEADS",
        "version": "mycel-wire/0.1",
        "msg_id": "msg:heads-001",
        "timestamp": "2026-03-08T20:00:30+08:00",
        "from": "node:alpha",
        "payload": {
            "documents": {
                "doc:origin-text": ["rev:c7d4", "rev:b351"],
                "doc:governance-rules": ["rev:91de"]
            },
            "replace": true
        },
        "sig": "sig:..."
    })
}

pub(super) fn wire_protocol_want_example() -> Value {
    json!({
        "type": "WANT",
        "version": "mycel-wire/0.1",
        "msg_id": "msg:want-001",
        "timestamp": "2026-03-08T20:01:00+08:00",
        "from": "node:beta",
        "payload": {
            "objects": ["rev:c7d4", "patch:a12f"],
            "max_items": 256u64
        },
        "sig": "sig:..."
    })
}

pub(super) fn wire_protocol_object_example() -> Value {
    json!({
        "type": "OBJECT",
        "version": "mycel-wire/0.1",
        "msg_id": "msg:obj-001",
        "timestamp": "2026-03-08T20:01:02+08:00",
        "from": "node:alpha",
        "payload": {
            "object_id": "patch:a12f",
            "object_type": "patch",
            "encoding": "json",
            "hash_alg": "blake3",
            "hash": "hash:placeholder",
            "body": {
                "type": "patch",
                "patch_id": "patch:a12f",
                "doc_id": "doc:origin-text",
                "base_revision": "rev:0ab1",
                "author": "pk:authorA",
                "timestamp": 1777778888u64,
                "ops": [],
                "signature": "sig:..."
            }
        },
        "sig": "sig:..."
    })
}

pub(super) fn wire_protocol_snapshot_offer_example() -> Value {
    json!({
        "type": "SNAPSHOT_OFFER",
        "version": "mycel-wire/0.1",
        "msg_id": "msg:snap-001",
        "timestamp": "2026-03-08T20:02:00+08:00",
        "from": "node:alpha",
        "payload": {
            "snapshot_id": "snap:44cc",
            "root_hash": "hash:snapshot-root",
            "documents": ["doc:origin-text"],
            "object_count": 3912u64,
            "size_bytes": 1048576u64
        },
        "sig": "sig:..."
    })
}

pub(super) fn wire_protocol_view_announce_example() -> Value {
    json!({
        "type": "VIEW_ANNOUNCE",
        "version": "mycel-wire/0.1",
        "msg_id": "msg:view-001",
        "timestamp": "2026-03-08T20:02:05+08:00",
        "from": "node:alpha",
        "payload": {
            "view_id": "view:9aa0",
            "maintainer": "pk:community-curator",
            "documents": {
                "doc:origin-text": "rev:c7d4"
            }
        },
        "sig": "sig:..."
    })
}

pub(super) fn wire_protocol_bye_example() -> Value {
    json!({
        "type": "BYE",
        "version": "mycel-wire/0.1",
        "msg_id": "msg:bye-001",
        "timestamp": "2026-03-08T20:02:10+08:00",
        "from": "node:alpha",
        "payload": {
            "reason": "normal-close"
        },
        "sig": "sig:..."
    })
}

pub(super) fn wire_protocol_error_example() -> Value {
    json!({
        "type": "ERROR",
        "version": "mycel-wire/0.1",
        "msg_id": "msg:err-001",
        "timestamp": "2026-03-08T20:01:03+08:00",
        "from": "node:beta",
        "payload": {
            "in_reply_to": "msg:obj-001",
            "code": "INVALID_HASH",
            "detail": "Hash mismatch for object patch:a12f"
        },
        "sig": "sig:..."
    })
}

pub(super) fn concrete_wire_object_example() -> Value {
    let mut body = json!({
        "type": "patch",
        "version": "mycel/0.1",
        "patch_id": "patch:placeholder",
        "doc_id": "doc:test",
        "base_revision": "rev:genesis-null",
        "author": "pk:ed25519:test",
        "timestamp": 1u64,
        "ops": [],
        "signature": "sig:placeholder"
    });
    let object_id = recompute_object_id(&body, "patch_id", "patch")
        .expect("concrete wire object ID should recompute");
    body["patch_id"] = Value::String(object_id.clone());
    let object_hash = object_id
        .split_once(':')
        .map(|(_, hash)| hash)
        .expect("wire object ID should contain hash");

    json!({
        "type": "OBJECT",
        "version": "mycel-wire/0.1",
        "msg_id": "msg:obj-concrete-001",
        "timestamp": "2026-03-08T20:01:02+08:00",
        "from": "node:alpha",
        "payload": {
            "object_id": object_id,
            "object_type": "patch",
            "encoding": "json",
            "hash_alg": "sha256",
            "hash": format!("hash:{object_hash}"),
            "body": body
        },
        "sig": "sig:..."
    })
}

pub(super) fn wire_required_string(
    object: &Map<String, Value>,
    field: &str,
) -> Result<String, String> {
    required_string_field(object, field)
        .map(str::to_owned)
        .map_err(|error| match error {
            StringFieldError::Missing => {
                format!("wire envelope is missing string field '{field}'")
            }
            StringFieldError::WrongType => {
                format!("wire envelope field '{field}' must be a string")
            }
        })
}

pub(super) fn validate_wire_timestamp(timestamp: &str) -> Result<(), String> {
    let (date, time_with_offset) = timestamp
        .split_once('T')
        .ok_or_else(|| "wire envelope 'timestamp' must use RFC 3339 format".to_string())?;
    let date_parts = date.split('-').collect::<Vec<_>>();
    if date_parts.len() != 3
        || date_parts[0].len() != 4
        || date_parts[1].len() != 2
        || date_parts[2].len() != 2
        || !date_parts
            .iter()
            .all(|part| part.chars().all(|ch| ch.is_ascii_digit()))
    {
        return Err("wire envelope 'timestamp' must use RFC 3339 format".to_string());
    }

    let (time, offset) = if let Some(index) = time_with_offset.find(['+', '-']) {
        (&time_with_offset[..index], &time_with_offset[index..])
    } else if let Some(time) = time_with_offset.strip_suffix('Z') {
        (time, "Z")
    } else {
        return Err("wire envelope 'timestamp' must use RFC 3339 format".to_string());
    };

    let time_parts = time.split(':').collect::<Vec<_>>();
    if time_parts.len() != 3
        || !time_parts.iter().all(|part| part.len() == 2)
        || !time_parts
            .iter()
            .all(|part| part.chars().all(|ch| ch.is_ascii_digit()))
    {
        return Err("wire envelope 'timestamp' must use RFC 3339 format".to_string());
    }

    if offset != "Z" {
        let offset_parts = offset[1..].split(':').collect::<Vec<_>>();
        if offset.len() != 6
            || offset_parts.len() != 2
            || !offset_parts.iter().all(|part| part.len() == 2)
            || !offset_parts
                .iter()
                .all(|part| part.chars().all(|ch| ch.is_ascii_digit()))
        {
            return Err("wire envelope 'timestamp' must use RFC 3339 format".to_string());
        }
    }

    Ok(())
}

pub(super) fn validate_wire_string_array(
    payload: &Map<String, Value>,
    field: &str,
) -> Result<Vec<String>, String> {
    required_non_empty_string_array(payload, field).map_err(|error| error.to_string())
}

pub(super) fn validate_wire_head_map(
    payload: &Map<String, Value>,
    field: &str,
) -> Result<BTreeMap<String, Vec<String>>, String> {
    let entries = match payload.get(field) {
        Some(Value::Object(entries)) => entries,
        Some(_) => {
            return Err(format!("top-level '{field}' must be an object"));
        }
        None => {
            return Err(format!("missing object field '{field}'"));
        }
    };
    if entries.is_empty() {
        return Err(format!("top-level '{field}' must not be empty"));
    }

    let mut heads = BTreeMap::new();
    for (doc_id, revision_ids) in entries {
        validate_prefixed_string(doc_id, &format!("{field}.{doc_id} key"), "doc:")
            .map_err(|error| error.to_string())?;
        let revisions = match revision_ids {
            Value::Array(values) => {
                if values.is_empty() {
                    return Err(format!("top-level '{field}.{doc_id}' must not be empty"));
                }
                let revisions = values
                    .iter()
                    .enumerate()
                    .map(|(index, value)| match value {
                        Value::String(value) => Ok(value.clone()),
                        _ => Err(format!(
                            "top-level '{field}.{doc_id}[{index}]' must be a string"
                        )),
                    })
                    .collect::<Result<Vec<_>, _>>()?;
                for (index, revision_id) in revisions.iter().enumerate() {
                    validate_prefixed_string(
                        revision_id,
                        &format!("{field}.{doc_id}[{index}]"),
                        "rev:",
                    )
                    .map_err(|error| error.to_string())?;
                }
                reject_duplicate_strings(&revisions, &format!("{field}.{doc_id}"))
                    .map_err(|error| error.to_string())?;
                revisions
            }
            _ => {
                return Err(format!("top-level '{field}.{doc_id}' must be an array"));
            }
        };
        heads.insert(doc_id.clone(), revisions);
    }

    Ok(heads)
}

pub(super) fn parse_wire_envelope_example(
    value: &Value,
) -> Result<(String, Map<String, Value>), String> {
    ensure_supported_json_values(value)?;
    let object = value
        .as_object()
        .ok_or_else(|| "wire envelope top-level JSON value must be an object".to_string())?;
    reject_unknown_fields(
        object,
        "top-level",
        &[
            "type",
            "version",
            "msg_id",
            "timestamp",
            "from",
            "payload",
            "sig",
        ],
    )
    .map_err(|error| error.to_string())?;

    let message_type = wire_required_string(object, "type")?;
    if !matches!(
        message_type.as_str(),
        "HELLO"
            | "MANIFEST"
            | "HEADS"
            | "WANT"
            | "OBJECT"
            | "SNAPSHOT_OFFER"
            | "VIEW_ANNOUNCE"
            | "BYE"
            | "ERROR"
    ) {
        return Err(format!("unsupported wire message type '{message_type}'"));
    }

    let version = wire_required_string(object, "version")?;
    if version != WIRE_PROTOCOL_VERSION {
        return Err(format!(
            "wire envelope 'version' must equal '{WIRE_PROTOCOL_VERSION}'"
        ));
    }

    validate_prefixed_string(&wire_required_string(object, "msg_id")?, "msg_id", "msg:")
        .map_err(|error| error.to_string())?;
    validate_wire_timestamp(&wire_required_string(object, "timestamp")?)?;
    validate_prefixed_string(&wire_required_string(object, "from")?, "from", "node:")
        .map_err(|error| error.to_string())?;
    validate_prefixed_string(&wire_required_string(object, "sig")?, "sig", "sig:")
        .map_err(|error| error.to_string())?;

    let payload = required_object(object, "payload").map_err(|error| error.to_string())?;

    Ok((message_type, payload))
}

pub(super) fn validate_wire_object_payload_behavior(
    payload: &Map<String, Value>,
) -> Result<(), String> {
    let object_id = wire_required_string(payload, "object_id")?;
    let object_type = wire_required_string(payload, "object_type")?;
    let hash = wire_required_string(payload, "hash")?;
    let body = payload
        .get("body")
        .ok_or_else(|| "missing object field 'body'".to_string())?;
    let body_envelope = parse_object_envelope(body).map_err(|error| error.to_string())?;
    if body_envelope.object_type() != object_type {
        return Err(format!(
            "OBJECT body type '{}' does not match object_type '{}'",
            body_envelope.object_type(),
            object_type
        ));
    }

    let expected_object_id = match object_type.as_str() {
        "patch" => recompute_object_id(body, "patch_id", "patch"),
        "revision" => recompute_object_id(body, "revision_id", "rev"),
        "view" => recompute_object_id(body, "view_id", "view"),
        "snapshot" => recompute_object_id(body, "snapshot_id", "snap"),
        other => return Err(format!("unsupported OBJECT object_type '{other}'")),
    }
    .map_err(|error| format!("failed to recompute OBJECT body ID: {error}"))?;

    let expected_hash = format!(
        "hash:{}",
        expected_object_id
            .split_once(':')
            .map(|(_, suffix)| suffix)
            .ok_or_else(|| "recomputed OBJECT ID is missing ':' separator".to_string())?
    );

    if object_id != expected_object_id {
        return Err(format!(
            "OBJECT payload object_id '{object_id}' does not match recomputed '{expected_object_id}'"
        ));
    }
    if hash != expected_hash {
        return Err(format!(
            "OBJECT payload hash '{hash}' does not match recomputed '{expected_hash}'"
        ));
    }

    Ok(())
}

pub(super) fn validate_wire_payload_example(
    message_type: &str,
    payload: &Map<String, Value>,
) -> Result<(), String> {
    match message_type {
        "HELLO" => {
            validate_prefixed_string(
                &wire_required_string(payload, "node_id")?,
                "node_id",
                "node:",
            )
            .map_err(|error| error.to_string())?;
            validate_wire_string_array(payload, "capabilities")?;
            validate_prefixed_string(&wire_required_string(payload, "nonce")?, "nonce", "n:")
                .map_err(|error| error.to_string())?;
        }
        "MANIFEST" => {
            validate_prefixed_string(
                &wire_required_string(payload, "node_id")?,
                "node_id",
                "node:",
            )
            .map_err(|error| error.to_string())?;
            validate_wire_string_array(payload, "capabilities")?;
            validate_wire_head_map(payload, "heads")?;
            if payload.contains_key("snapshots") {
                for (index, object_id) in validate_wire_string_array(payload, "snapshots")?
                    .iter()
                    .enumerate()
                {
                    validate_canonical_object_id(object_id, &format!("snapshots[{index}]"))
                        .map_err(|error| error.to_string())?;
                }
            }
            if payload.contains_key("views") {
                for (index, object_id) in validate_wire_string_array(payload, "views")?
                    .iter()
                    .enumerate()
                {
                    validate_canonical_object_id(object_id, &format!("views[{index}]"))
                        .map_err(|error| error.to_string())?;
                }
            }
        }
        "HEADS" => {
            validate_wire_head_map(payload, "documents")?;
            match payload.get("replace") {
                Some(Value::Bool(_)) => {}
                Some(_) => return Err("top-level 'replace' must be a boolean".to_string()),
                None => return Err("missing boolean field 'replace'".to_string()),
            }
        }
        "WANT" => {
            for (index, object_id) in validate_wire_string_array(payload, "objects")?
                .iter()
                .enumerate()
            {
                validate_canonical_object_id(object_id, &format!("objects[{index}]"))
                    .map_err(|error| error.to_string())?;
            }
        }
        "OBJECT" => {
            validate_canonical_object_id(&wire_required_string(payload, "object_id")?, "object_id")
                .map_err(|error| error.to_string())?;
            object_schema(&wire_required_string(payload, "object_type")?).ok_or_else(|| {
                "OBJECT payload 'object_type' must be a supported object type".to_string()
            })?;
            let encoding = wire_required_string(payload, "encoding")?;
            if encoding != "json" {
                return Err("OBJECT payload 'encoding' must equal 'json'".to_string());
            }
            validate_prefixed_string(&wire_required_string(payload, "hash")?, "hash", "hash:")
                .map_err(|error| error.to_string())?;
            wire_required_string(payload, "hash_alg")?;
            if !matches!(payload.get("body"), Some(Value::Object(_))) {
                return Err("top-level 'body' must be an object".to_string());
            }
        }
        "SNAPSHOT_OFFER" => {
            validate_prefixed_string(
                &wire_required_string(payload, "snapshot_id")?,
                "snapshot_id",
                "snap:",
            )
            .map_err(|error| error.to_string())?;
            validate_prefixed_string(
                &wire_required_string(payload, "root_hash")?,
                "root_hash",
                "hash:",
            )
            .map_err(|error| error.to_string())?;
            for (index, doc_id) in validate_wire_string_array(payload, "documents")?
                .iter()
                .enumerate()
            {
                validate_prefixed_string(doc_id, &format!("documents[{index}]"), "doc:")
                    .map_err(|error| error.to_string())?;
            }
        }
        "VIEW_ANNOUNCE" => {
            validate_prefixed_string(
                &wire_required_string(payload, "view_id")?,
                "view_id",
                "view:",
            )
            .map_err(|error| error.to_string())?;
            validate_prefixed_string(
                &wire_required_string(payload, "maintainer")?,
                "maintainer",
                "pk:",
            )
            .map_err(|error| error.to_string())?;
            let documents = required_prefixed_string_map(payload, "documents", "doc:", "rev:")
                .map_err(|error| error.to_string())?;
            if documents.is_empty() {
                return Err("top-level 'documents' must not be empty".to_string());
            }
        }
        "BYE" => {
            wire_required_string(payload, "reason")?;
        }
        "ERROR" => {
            validate_prefixed_string(
                &wire_required_string(payload, "in_reply_to")?,
                "in_reply_to",
                "msg:",
            )
            .map_err(|error| error.to_string())?;
            wire_required_string(payload, "code")?;
        }
        _ => return Err(format!("unsupported wire message type '{message_type}'")),
    }

    Ok(())
}
