use super::*;

#[test]
fn wire_session_accepts_heads_before_manifest_and_unlocks_want() {
    let signing_key = signing_key();
    let sender_key = sender_public_key(&signing_key);
    let mut session = WireSession::default();
    session
        .register_known_peer("node:alpha", &sender_key)
        .expect("known peer should register");
    let patch_object = signed_patch_object_message(&signing_key, "node:alpha", "rev:genesis-null");
    let patch_id = patch_object["payload"]["object_id"]
        .as_str()
        .expect("signed patch OBJECT should include object_id")
        .to_owned();
    let revision_object =
        signed_revision_object_message(&signing_key, "node:alpha", &[], &[patch_id.as_str()]);
    let revision_id = revision_object["payload"]["object_id"]
        .as_str()
        .expect("signed revision OBJECT should include object_id")
        .to_owned();
    let hello = signed_hello_message(&signing_key, "node:alpha", "node:alpha");
    let heads = signed_heads_message(
        &signing_key,
        "node:alpha",
        json!({
            "doc:test": [revision_id.clone()]
        }),
        true,
    );
    let want = signed_want_message(&signing_key, "node:alpha", &[revision_id.as_str()]);

    session.verify_incoming(&hello).expect("HELLO should verify");
    session
        .verify_incoming(&heads)
        .expect("HEADS should verify before MANIFEST");
    session
        .verify_incoming(&want)
        .expect("WANT should verify after HEADS establishes sync roots");

    let state = session
        .peer_session("node:alpha")
        .expect("peer session should exist");
    assert!(state
        .advertised_document_heads
        .get("doc:test")
        .is_some_and(|revisions| revisions.contains(&revision_id)));
    assert!(state.pending_object_ids.contains(&revision_id));
}

#[test]
fn wire_session_records_manifest_heads() {
    let signing_key = signing_key();
    let sender_key = sender_public_key(&signing_key);
    let mut session = WireSession::default();
    session
        .register_known_peer("node:alpha", &sender_key)
        .expect("known peer should register");
    let hello = signed_hello_message(&signing_key, "node:alpha", "node:alpha");
    let manifest = signed_manifest_message(&signing_key, "node:alpha", "node:alpha");

    session.verify_incoming(&hello).expect("HELLO should verify");
    session
        .verify_incoming(&manifest)
        .expect("MANIFEST should verify");

    let state = session
        .peer_session("node:alpha")
        .expect("peer session should exist");
    assert_eq!(
        state
            .advertised_document_heads
            .get("doc:test")
            .map(|revisions| revisions.len()),
        Some(1)
    );
    assert!(state
        .advertised_document_heads
        .get("doc:test")
        .is_some_and(|revisions| revisions.contains("rev:test")));
}

#[test]
fn wire_session_merges_incremental_heads_updates() {
    let signing_key = signing_key();
    let sender_key = sender_public_key(&signing_key);
    let mut session = WireSession::default();
    session
        .register_known_peer("node:alpha", &sender_key)
        .expect("known peer should register");
    let hello = signed_hello_message(&signing_key, "node:alpha", "node:alpha");
    let manifest = signed_manifest_message(&signing_key, "node:alpha", "node:alpha");
    let heads = signed_heads_message(
        &signing_key,
        "node:alpha",
        json!({
            "doc:test": ["rev:next"],
            "doc:extra": ["rev:extra"]
        }),
        false,
    );

    session.verify_incoming(&hello).expect("HELLO should verify");
    session
        .verify_incoming(&manifest)
        .expect("MANIFEST should verify");
    session.verify_incoming(&heads).expect("HEADS should verify");

    let state = session
        .peer_session("node:alpha")
        .expect("peer session should exist");
    assert!(state
        .advertised_document_heads
        .get("doc:test")
        .is_some_and(|revisions| {
            revisions.contains("rev:test") && revisions.contains("rev:next")
        }));
    assert!(state
        .advertised_document_heads
        .get("doc:extra")
        .is_some_and(|revisions| revisions.contains("rev:extra")));
}

#[test]
fn wire_session_replaces_heads_when_replace_is_true() {
    let signing_key = signing_key();
    let sender_key = sender_public_key(&signing_key);
    let mut session = WireSession::default();
    session
        .register_known_peer("node:alpha", &sender_key)
        .expect("known peer should register");
    let hello = signed_hello_message(&signing_key, "node:alpha", "node:alpha");
    let manifest = signed_manifest_message(&signing_key, "node:alpha", "node:alpha");
    let heads = signed_heads_message(
        &signing_key,
        "node:alpha",
        json!({
            "doc:replacement": ["rev:replacement"]
        }),
        true,
    );

    session.verify_incoming(&hello).expect("HELLO should verify");
    session
        .verify_incoming(&manifest)
        .expect("MANIFEST should verify");
    session.verify_incoming(&heads).expect("HEADS should verify");

    let state = session
        .peer_session("node:alpha")
        .expect("peer session should exist");
    assert!(!state.advertised_document_heads.contains_key("doc:test"));
    assert!(state
        .advertised_document_heads
        .get("doc:replacement")
        .is_some_and(|revisions| revisions.contains("rev:replacement")));
}

#[test]
fn wire_session_rejects_stale_dependency_want_after_heads_replace() {
    let signing_key = signing_key();
    let sender_key = sender_public_key(&signing_key);
    let mut session = WireSession::default();
    session
        .register_known_peer("node:alpha", &sender_key)
        .expect("known peer should register");

    let patch_object = signed_patch_object_message(&signing_key, "node:alpha", "rev:genesis-null");
    let patch_id = patch_object["payload"]["object_id"]
        .as_str()
        .expect("signed patch OBJECT should include object_id")
        .to_owned();
    let revision_object =
        signed_revision_object_message(&signing_key, "node:alpha", &[], &[patch_id.as_str()]);
    let revision_id = revision_object["payload"]["object_id"]
        .as_str()
        .expect("signed revision OBJECT should include object_id")
        .to_owned();

    let hello = signed_hello_message(&signing_key, "node:alpha", "node:alpha");
    let initial_heads = signed_heads_message(
        &signing_key,
        "node:alpha",
        json!({
            "doc:test": [revision_id.clone()]
        }),
        true,
    );
    let request_revision = signed_want_message(&signing_key, "node:alpha", &[revision_id.as_str()]);
    let replacement_heads = signed_heads_message(
        &signing_key,
        "node:alpha",
        json!({
            "doc:replacement": ["rev:replacement"]
        }),
        true,
    );
    let request_stale_patch = signed_want_message(&signing_key, "node:alpha", &[patch_id.as_str()]);

    session.verify_incoming(&hello).expect("HELLO should verify");
    session
        .verify_incoming(&initial_heads)
        .expect("initial HEADS should verify");
    session
        .verify_incoming(&request_revision)
        .expect("root revision WANT should verify");
    session
        .verify_incoming(&revision_object)
        .expect("root revision OBJECT should verify");
    session
        .verify_incoming(&replacement_heads)
        .expect("replacement HEADS should verify");
    let error = session.verify_incoming(&request_stale_patch).unwrap_err();

    assert_eq!(
        error,
        format!(
            "wire WANT object '{}' is not reachable from accepted sync roots for 'node:alpha'",
            patch_id
        )
    );
}

#[test]
fn wire_session_rejects_stale_root_revision_want_after_heads_replace() {
    let signing_key = signing_key();
    let sender_key = sender_public_key(&signing_key);
    let mut session = WireSession::default();
    session
        .register_known_peer("node:alpha", &sender_key)
        .expect("known peer should register");

    let revision_id = "rev:stale-root";
    let hello = signed_hello_message(&signing_key, "node:alpha", "node:alpha");
    let initial_heads = signed_heads_message(
        &signing_key,
        "node:alpha",
        json!({
            "doc:test": [revision_id]
        }),
        true,
    );
    let replacement_heads = signed_heads_message(
        &signing_key,
        "node:alpha",
        json!({
            "doc:replacement": ["rev:replacement"]
        }),
        true,
    );
    let request_stale_revision = signed_want_message(&signing_key, "node:alpha", &[revision_id]);

    session.verify_incoming(&hello).expect("HELLO should verify");
    session
        .verify_incoming(&initial_heads)
        .expect("initial HEADS should verify");
    session
        .verify_incoming(&replacement_heads)
        .expect("replacement HEADS should verify");
    let error = session.verify_incoming(&request_stale_revision).unwrap_err();

    assert_eq!(
        error,
        "wire WANT revision 'rev:stale-root' is not reachable from accepted sync roots for 'node:alpha'"
    );
}

#[test]
fn wire_session_snapshot_offer_before_manifest_still_requires_head_context_for_want() {
    let signing_key = signing_key();
    let sender_key = sender_public_key(&signing_key);
    let mut session = WireSession::default();
    session
        .register_known_peer("node:alpha", &sender_key)
        .expect("known peer should register");
    let hello = signed_hello_message_with_capabilities(
        &signing_key,
        "node:alpha",
        "node:alpha",
        json!(["patch-sync", "snapshot-sync"]),
    );
    let snapshot_offer =
        signed_snapshot_offer_message(&signing_key, "node:alpha", "snap:test-offer");
    let want = signed_want_message(&signing_key, "node:alpha", &["snap:test-offer"]);

    session.verify_incoming(&hello).expect("HELLO should verify");
    session
        .verify_incoming(&snapshot_offer)
        .expect("SNAPSHOT_OFFER should verify before MANIFEST");
    let error = session.verify_incoming(&want).unwrap_err();

    assert_eq!(
        error,
        "wire WANT requires prior MANIFEST or HEADS from 'node:alpha'"
    );
    assert!(session
        .peer_session("node:alpha")
        .is_some_and(|state| state.reachable_object_ids.contains("snap:test-offer")));
}

#[test]
fn wire_session_view_announce_before_manifest_still_requires_head_context_for_want() {
    let signing_key = signing_key();
    let sender_key = sender_public_key(&signing_key);
    let mut session = WireSession::default();
    session
        .register_known_peer("node:alpha", &sender_key)
        .expect("known peer should register");
    let hello = signed_hello_message_with_capabilities(
        &signing_key,
        "node:alpha",
        "node:alpha",
        json!(["patch-sync", "view-sync"]),
    );
    let view_announce =
        signed_view_announce_message(&signing_key, "node:alpha", "view:test-announce");
    let want = signed_want_message(&signing_key, "node:alpha", &["view:test-announce"]);

    session.verify_incoming(&hello).expect("HELLO should verify");
    session
        .verify_incoming(&view_announce)
        .expect("VIEW_ANNOUNCE should verify before MANIFEST");
    let error = session.verify_incoming(&want).unwrap_err();

    assert_eq!(
        error,
        "wire WANT requires prior MANIFEST or HEADS from 'node:alpha'"
    );
    assert!(session
        .peer_session("node:alpha")
        .is_some_and(|state| state.reachable_object_ids.contains("view:test-announce")));
}

#[test]
fn wire_session_rejects_want_before_head_context() {
    let signing_key = signing_key();
    let sender_key = sender_public_key(&signing_key);
    let mut session = WireSession::default();
    session
        .register_known_peer("node:alpha", &sender_key)
        .expect("known peer should register");
    let hello = signed_hello_message(&signing_key, "node:alpha", "node:alpha");
    let want = signed_want_message(&signing_key, "node:alpha", &["patch:test"]);

    session.verify_incoming(&hello).expect("HELLO should verify");
    let error = session.verify_incoming(&want).unwrap_err();

    assert_eq!(
        error,
        "wire WANT requires prior MANIFEST or HEADS from 'node:alpha'"
    );
}

#[test]
fn wire_session_rejects_unadvertised_revision_want() {
    let signing_key = signing_key();
    let sender_key = sender_public_key(&signing_key);
    let mut session = WireSession::default();
    session
        .register_known_peer("node:alpha", &sender_key)
        .expect("known peer should register");
    let hello = signed_hello_message(&signing_key, "node:alpha", "node:alpha");
    let manifest = signed_manifest_message(&signing_key, "node:alpha", "node:alpha");
    let want = signed_want_message(&signing_key, "node:alpha", &["rev:missing"]);

    session.verify_incoming(&hello).expect("HELLO should verify");
    session
        .verify_incoming(&manifest)
        .expect("MANIFEST should verify");
    let error = session.verify_incoming(&want).unwrap_err();

    assert_eq!(
        error,
        "wire WANT revision 'rev:missing' is not reachable from accepted sync roots for 'node:alpha'"
    );
}

#[test]
fn wire_session_rejects_non_revision_want_without_sync_root() {
    let signing_key = signing_key();
    let sender_key = sender_public_key(&signing_key);
    let mut session = WireSession::default();
    session
        .register_known_peer("node:alpha", &sender_key)
        .expect("known peer should register");
    let hello = signed_hello_message(&signing_key, "node:alpha", "node:alpha");
    let manifest = signed_manifest_message(&signing_key, "node:alpha", "node:alpha");
    let want = signed_want_message(&signing_key, "node:alpha", &["patch:test"]);

    session.verify_incoming(&hello).expect("HELLO should verify");
    session
        .verify_incoming(&manifest)
        .expect("MANIFEST should verify");
    let error = session.verify_incoming(&want).unwrap_err();

    assert_eq!(
        error,
        "wire WANT object 'patch:test' is not reachable from accepted sync roots for 'node:alpha'"
    );
}
