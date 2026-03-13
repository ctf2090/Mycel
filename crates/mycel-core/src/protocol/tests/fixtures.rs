use super::*;
use base64::Engine;
use ed25519_dalek::{Signer, SigningKey};

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
            "hash_alg": "sha256",
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
    let signing_key = SigningKey::from_bytes(&[7u8; 32]);
    let public_key = format!(
        "pk:ed25519:{}",
        base64::engine::general_purpose::STANDARD.encode(signing_key.verifying_key().as_bytes())
    );
    let mut body = json!({
        "type": "patch",
        "version": "mycel/0.1",
        "patch_id": "patch:placeholder",
        "doc_id": "doc:test",
        "base_revision": "rev:genesis-null",
        "author": public_key,
        "timestamp": 1u64,
        "ops": [],
        "signature": "sig:placeholder"
    });
    let object_id = recompute_object_id(&body, "patch_id", "patch")
        .expect("concrete wire object ID should recompute");
    body["patch_id"] = Value::String(object_id.clone());
    let payload = signed_payload_bytes(&body).expect("concrete wire body should canonicalize");
    let signature = signing_key.sign(&payload);
    body["signature"] = Value::String(format!(
        "sig:ed25519:{}",
        base64::engine::general_purpose::STANDARD.encode(signature.to_bytes())
    ));
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
