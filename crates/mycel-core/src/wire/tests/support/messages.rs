use super::*;

pub(crate) fn hello_envelope_with(timestamp: &str) -> Value {
    json!({
        "type": "HELLO",
        "version": "mycel-wire/0.1",
        "msg_id": "msg:hello-proptest-001",
        "timestamp": timestamp,
        "from": "node:alpha",
        "payload": {
            "node_id": "node:alpha",
            "capabilities": ["patch-sync"],
            "nonce": "n:test"
        },
        "sig": "sig:placeholder"
    })
}

pub(crate) fn signed_hello_message(
    signing_key: &SigningKey,
    sender: &str,
    payload_node_id: &str,
) -> Value {
    signed_hello_message_with_capabilities(
        signing_key,
        sender,
        payload_node_id,
        json!(["patch-sync"]),
    )
}

pub(crate) fn signed_hello_message_with_capabilities(
    signing_key: &SigningKey,
    sender: &str,
    payload_node_id: &str,
    capabilities: Value,
) -> Value {
    let mut value = json!({
        "type": "HELLO",
        "version": "mycel-wire/0.1",
        "msg_id": "msg:hello-signed-001",
        "timestamp": "2026-03-08T20:00:00+08:00",
        "from": sender,
        "payload": {
            "node_id": payload_node_id,
            "capabilities": capabilities,
            "nonce": "n:test"
        },
        "sig": "sig:placeholder"
    });
    value["sig"] = Value::String(sign_wire_value(signing_key, &value));
    value
}

pub(crate) fn signed_manifest_message(
    signing_key: &SigningKey,
    sender: &str,
    payload_node_id: &str,
) -> Value {
    signed_manifest_message_with_capabilities(
        signing_key,
        sender,
        payload_node_id,
        json!(["patch-sync"]),
    )
}

pub(crate) fn signed_manifest_message_with_capabilities(
    signing_key: &SigningKey,
    sender: &str,
    payload_node_id: &str,
    capabilities: Value,
) -> Value {
    let mut value = json!({
        "type": "MANIFEST",
        "version": "mycel-wire/0.1",
        "msg_id": "msg:manifest-signed-001",
        "timestamp": "2026-03-08T20:00:10+08:00",
        "from": sender,
        "payload": {
            "node_id": payload_node_id,
            "capabilities": capabilities,
            "heads": {
                "doc:test": ["rev:test"]
            }
        },
        "sig": "sig:placeholder"
    });
    value["sig"] = Value::String(sign_wire_value(signing_key, &value));
    value
}

pub(crate) fn signed_snapshot_offer_message(
    signing_key: &SigningKey,
    sender: &str,
    snapshot_id: &str,
) -> Value {
    let mut value = json!({
        "type": "SNAPSHOT_OFFER",
        "version": "mycel-wire/0.1",
        "msg_id": "msg:snapshot-offer-signed-001",
        "timestamp": "2026-03-08T20:00:40+08:00",
        "from": sender,
        "payload": {
            "snapshot_id": snapshot_id,
            "root_hash": "hash:snapshot-root",
            "documents": ["doc:test"]
        },
        "sig": "sig:placeholder"
    });
    value["sig"] = Value::String(sign_wire_value(signing_key, &value));
    value
}

pub(crate) fn signed_view_announce_message(
    signing_key: &SigningKey,
    sender: &str,
    view_id: &str,
) -> Value {
    let mut value = json!({
        "type": "VIEW_ANNOUNCE",
        "version": "mycel-wire/0.1",
        "msg_id": "msg:view-announce-signed-001",
        "timestamp": "2026-03-08T20:00:45+08:00",
        "from": sender,
        "payload": {
            "view_id": view_id,
            "maintainer": sender_public_key(signing_key),
            "documents": {
                "doc:test": "rev:test"
            }
        },
        "sig": "sig:placeholder"
    });
    value["sig"] = Value::String(sign_wire_value(signing_key, &value));
    value
}

pub(crate) fn signed_manifest_message_with_heads(
    signing_key: &SigningKey,
    sender: &str,
    payload_node_id: &str,
    heads: Value,
) -> Value {
    let mut value = json!({
        "type": "MANIFEST",
        "version": "mycel-wire/0.1",
        "msg_id": "msg:manifest-signed-001",
        "timestamp": "2026-03-08T20:00:10+08:00",
        "from": sender,
        "payload": {
            "node_id": payload_node_id,
            "capabilities": ["patch-sync"],
            "heads": heads
        },
        "sig": "sig:placeholder"
    });
    value["sig"] = Value::String(sign_wire_value(signing_key, &value));
    value
}

pub(crate) fn signed_want_message(
    signing_key: &SigningKey,
    sender: &str,
    object_ids: &[&str],
) -> Value {
    let mut value = json!({
        "type": "WANT",
        "version": "mycel-wire/0.1",
        "msg_id": "msg:want-signed-001",
        "timestamp": "2026-03-08T20:01:00+08:00",
        "from": sender,
        "payload": {
            "objects": object_ids
        },
        "sig": "sig:placeholder"
    });
    value["sig"] = Value::String(sign_wire_value(signing_key, &value));
    value
}

pub(crate) fn signed_heads_message(
    signing_key: &SigningKey,
    sender: &str,
    documents: Value,
    replace: bool,
) -> Value {
    let mut value = json!({
        "type": "HEADS",
        "version": "mycel-wire/0.1",
        "msg_id": "msg:heads-signed-001",
        "timestamp": "2026-03-08T20:00:30+08:00",
        "from": sender,
        "payload": {
            "documents": documents,
            "replace": replace
        },
        "sig": "sig:placeholder"
    });
    value["sig"] = Value::String(sign_wire_value(signing_key, &value));
    value
}

pub(crate) fn signed_error_message(
    signing_key: &SigningKey,
    sender: &str,
    in_reply_to: &str,
) -> Value {
    let mut value = json!({
        "type": "ERROR",
        "version": "mycel-wire/0.1",
        "msg_id": "msg:error-signed-001",
        "timestamp": "2026-03-08T20:02:00+08:00",
        "from": sender,
        "payload": {
            "in_reply_to": in_reply_to,
            "code": "ERR_UNKNOWN",
            "detail": "test error"
        },
        "sig": "sig:placeholder"
    });
    value["sig"] = Value::String(sign_wire_value(signing_key, &value));
    value
}

pub(crate) fn signed_bye_message(signing_key: &SigningKey, sender: &str) -> Value {
    let mut value = json!({
        "type": "BYE",
        "version": "mycel-wire/0.1",
        "msg_id": "msg:bye-signed-001",
        "timestamp": "2026-03-08T20:02:00+08:00",
        "from": sender,
        "payload": {
            "reason": "done"
        },
        "sig": "sig:placeholder"
    });
    value["sig"] = Value::String(sign_wire_value(signing_key, &value));
    value
}
