use super::*;

#[test]
fn wire_session_rejects_stale_root_object_after_heads_replace() {
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

    session.verify_incoming(&hello).expect("HELLO should verify");
    session
        .verify_incoming(&initial_heads)
        .expect("initial HEADS should verify");
    session
        .verify_incoming(&request_revision)
        .expect("root revision WANT should verify");
    session
        .verify_incoming(&replacement_heads)
        .expect("replacement HEADS should verify");
    let error = session.verify_incoming(&revision_object).unwrap_err();

    assert_eq!(
        error,
        format!(
            "wire OBJECT '{}' was not requested from 'node:alpha'",
            revision_id
        )
    );
}

#[test]
fn wire_session_rejects_stale_dependency_object_after_heads_replace() {
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
    let request_patch = signed_want_message(&signing_key, "node:alpha", &[patch_id.as_str()]);
    let replacement_heads = signed_heads_message(
        &signing_key,
        "node:alpha",
        json!({
            "doc:replacement": ["rev:replacement"]
        }),
        true,
    );

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
        .verify_incoming(&request_patch)
        .expect("follow-on patch WANT should verify");
    session
        .verify_incoming(&replacement_heads)
        .expect("replacement HEADS should verify");
    let error = session.verify_incoming(&patch_object).unwrap_err();

    assert_eq!(
        error,
        format!(
            "wire OBJECT '{}' was not requested from 'node:alpha'",
            patch_id
        )
    );
}

#[test]
fn wire_session_rejects_follow_on_object_before_root_object_arrives() {
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
    let manifest = signed_manifest_message_with_heads(
        &signing_key,
        "node:alpha",
        "node:alpha",
        json!({
            "doc:test": [revision_id.clone()]
        }),
    );
    let want = signed_want_message(
        &signing_key,
        "node:alpha",
        &[revision_id.as_str(), patch_id.as_str()],
    );

    session.verify_incoming(&hello).expect("HELLO should verify");
    session
        .verify_incoming(&manifest)
        .expect("MANIFEST should verify");
    let error = session.verify_incoming(&want).unwrap_err();

    assert_eq!(
        error,
        format!(
            "wire WANT object '{}' is not reachable from accepted sync roots for 'node:alpha'",
            patch_id
        )
    );
}

#[test]
fn wire_session_accepts_follow_on_patch_after_reachable_revision_object() {
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
    let manifest = signed_manifest_message_with_heads(
        &signing_key,
        "node:alpha",
        "node:alpha",
        json!({
            "doc:test": [revision_id.clone()]
        }),
    );
    let root_want = signed_want_message(&signing_key, "node:alpha", &[revision_id.as_str()]);
    let follow_on_want = signed_want_message(&signing_key, "node:alpha", &[patch_id.as_str()]);

    session.verify_incoming(&hello).expect("HELLO should verify");
    session
        .verify_incoming(&manifest)
        .expect("MANIFEST should verify");
    session
        .verify_incoming(&root_want)
        .expect("root WANT should verify");
    let envelope = session
        .verify_incoming(&revision_object)
        .expect("reachable revision OBJECT should verify");

    assert_eq!(envelope.message_type(), WireMessageType::Object);
    assert!(session
        .peer_session("node:alpha")
        .is_some_and(|state| state.reachable_object_ids.contains(&patch_id)));

    session
        .verify_incoming(&follow_on_want)
        .expect("follow-on patch WANT should verify");
    let patch_envelope = session
        .verify_incoming(&patch_object)
        .expect("reachable patch OBJECT should verify");

    assert_eq!(patch_envelope.message_type(), WireMessageType::Object);
    assert_eq!(
        session
            .peer_session("node:alpha")
            .map(|state| state.pending_object_ids.len()),
        Some(0)
    );
    assert!(session
        .peer_session("node:alpha")
        .is_some_and(|state| state.accepted_sync_roots.contains(&revision_id)));
}

#[test]
fn wire_session_expands_reachability_from_known_object_index() {
    let signing_key = signing_key();
    let sender_key = sender_public_key(&signing_key);
    let mut session = WireSession::default();
    session
        .register_known_peer("node:alpha", &sender_key)
        .expect("known peer should register");
    let base_revision_object = signed_revision_object_message(&signing_key, "node:alpha", &[], &[]);
    let base_revision_id = base_revision_object["payload"]["object_id"]
        .as_str()
        .expect("signed base revision OBJECT should include object_id")
        .to_owned();
    let patch_object = signed_patch_object_message(&signing_key, "node:alpha", &base_revision_id);
    let patch_id = patch_object["payload"]["object_id"]
        .as_str()
        .expect("signed patch OBJECT should include object_id")
        .to_owned();
    let root_revision_object = signed_revision_object_message(
        &signing_key,
        "node:alpha",
        &[base_revision_id.as_str()],
        &[patch_id.as_str()],
    );
    let root_revision_id = root_revision_object["payload"]["object_id"]
        .as_str()
        .expect("signed root revision OBJECT should include object_id")
        .to_owned();
    session.set_known_verified_object_index(std::collections::BTreeMap::from([
        (
            root_revision_id.clone(),
            root_revision_object["payload"]["body"].clone(),
        ),
        (patch_id.clone(), patch_object["payload"]["body"].clone()),
        (
            base_revision_id.clone(),
            base_revision_object["payload"]["body"].clone(),
        ),
    ]));

    let hello = signed_hello_message(&signing_key, "node:alpha", "node:alpha");
    let manifest = signed_manifest_message_with_heads(
        &signing_key,
        "node:alpha",
        "node:alpha",
        json!({
            "doc:test": [root_revision_id.clone()]
        }),
    );
    let root_want = signed_want_message(&signing_key, "node:alpha", &[root_revision_id.as_str()]);
    let follow_on_want = signed_want_message(
        &signing_key,
        "node:alpha",
        &[patch_id.as_str(), base_revision_id.as_str()],
    );

    session.verify_incoming(&hello).expect("HELLO should verify");
    session
        .verify_incoming(&manifest)
        .expect("MANIFEST should verify");
    session
        .verify_incoming(&root_want)
        .expect("root WANT should verify");

    assert!(session.peer_session("node:alpha").is_some_and(|state| {
        state.reachable_object_ids.contains(&patch_id)
            && state.reachable_object_ids.contains(&base_revision_id)
    }));

    session
        .verify_incoming(&follow_on_want)
        .expect("known-index-expanded WANT should verify");
}

#[test]
fn wire_session_loads_known_verified_object_index_from_store() {
    let store_root = temp_dir("known-index");
    let signing_key = signing_key();
    let sender_key = sender_public_key(&signing_key);
    let base_revision_object = signed_revision_object_message(&signing_key, "node:alpha", &[], &[]);
    let base_revision_id = base_revision_object["payload"]["object_id"]
        .as_str()
        .expect("signed base revision OBJECT should include object_id")
        .to_owned();
    let patch_object = signed_patch_object_message(&signing_key, "node:alpha", &base_revision_id);
    let patch_id = patch_object["payload"]["object_id"]
        .as_str()
        .expect("signed patch OBJECT should include object_id")
        .to_owned();
    let root_revision_object = signed_revision_object_message(
        &signing_key,
        "node:alpha",
        &[base_revision_id.as_str()],
        &[patch_id.as_str()],
    );
    let root_revision_id = root_revision_object["payload"]["object_id"]
        .as_str()
        .expect("signed root revision OBJECT should include object_id")
        .to_owned();

    write_object_value_to_store(&store_root, &base_revision_object["payload"]["body"])
        .expect("base revision should write to store");
    write_object_value_to_store(&store_root, &patch_object["payload"]["body"])
        .expect("patch should write to store");
    write_object_value_to_store(&store_root, &root_revision_object["payload"]["body"])
        .expect("root revision should write to store");

    let mut known_peers = WirePeerDirectory::new();
    known_peers
        .register_known_peer("node:alpha", &sender_key)
        .expect("known peer should register");
    let mut session = WireSession::from_store_root(known_peers, &store_root)
        .expect("session should bootstrap from store root");

    let hello = signed_hello_message(&signing_key, "node:alpha", "node:alpha");
    let manifest = signed_manifest_message_with_heads(
        &signing_key,
        "node:alpha",
        "node:alpha",
        json!({
            "doc:test": [root_revision_id.clone()]
        }),
    );
    let root_want = signed_want_message(&signing_key, "node:alpha", &[root_revision_id.as_str()]);
    let follow_on_want = signed_want_message(
        &signing_key,
        "node:alpha",
        &[patch_id.as_str(), base_revision_id.as_str()],
    );

    session.verify_incoming(&hello).expect("HELLO should verify");
    session
        .verify_incoming(&manifest)
        .expect("MANIFEST should verify");
    session
        .verify_incoming(&root_want)
        .expect("root WANT should verify");
    session
        .verify_incoming(&follow_on_want)
        .expect("store-backed reachable WANT should verify");

    let _ = fs::remove_dir_all(store_root);
}

#[test]
fn wire_session_rejects_unrequested_object() {
    let signing_key = signing_key();
    let sender_key = sender_public_key(&signing_key);
    let mut session = WireSession::default();
    session
        .register_known_peer("node:alpha", &sender_key)
        .expect("known peer should register");
    let hello = signed_hello_message(&signing_key, "node:alpha", "node:alpha");
    let manifest = signed_manifest_message(&signing_key, "node:alpha", "node:alpha");
    let object = signed_object_message(&signing_key, "node:alpha");
    let object_id = object["payload"]["object_id"]
        .as_str()
        .expect("signed OBJECT payload should include object_id")
        .to_owned();

    session.verify_incoming(&hello).expect("HELLO should verify");
    session
        .verify_incoming(&manifest)
        .expect("MANIFEST should verify");
    let error = session.verify_incoming(&object).unwrap_err();

    assert_eq!(
        error,
        format!("wire OBJECT '{object_id}' was not requested from 'node:alpha'")
    );
}

#[test]
fn wire_session_rejects_unadvertised_root_object_after_root_want() {
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
    let requested_root =
        signed_revision_object_message(&signing_key, "node:alpha", &[], &[patch_id.as_str()]);
    let requested_root_id = requested_root["payload"]["object_id"]
        .as_str()
        .expect("signed requested root revision OBJECT should include object_id")
        .to_owned();
    let unadvertised_root = signed_revision_object_message(
        &signing_key,
        "node:alpha",
        &["rev:unexpected-parent"],
        &[patch_id.as_str()],
    );
    let unadvertised_root_id = unadvertised_root["payload"]["object_id"]
        .as_str()
        .expect("signed unadvertised root revision OBJECT should include object_id")
        .to_owned();

    let hello = signed_hello_message(&signing_key, "node:alpha", "node:alpha");
    let manifest = signed_manifest_message_with_heads(
        &signing_key,
        "node:alpha",
        "node:alpha",
        json!({
            "doc:test": [requested_root_id.clone()]
        }),
    );
    let root_want = signed_want_message(&signing_key, "node:alpha", &[requested_root_id.as_str()]);

    session.verify_incoming(&hello).expect("HELLO should verify");
    session
        .verify_incoming(&manifest)
        .expect("MANIFEST should verify");
    session
        .verify_incoming(&root_want)
        .expect("root WANT should verify");
    let error = session.verify_incoming(&unadvertised_root).unwrap_err();

    assert_eq!(
        error,
        format!(
            "wire OBJECT '{}' was not requested from 'node:alpha'",
            unadvertised_root_id
        )
    );
}

#[test]
fn wire_session_rejects_unrequested_object_before_manifest() {
    let signing_key = signing_key();
    let sender_key = sender_public_key(&signing_key);
    let mut session = WireSession::default();
    session
        .register_known_peer("node:alpha", &sender_key)
        .expect("known peer should register");
    let hello = signed_hello_message(&signing_key, "node:alpha", "node:alpha");
    let object = signed_object_message(&signing_key, "node:alpha");
    let object_id = object["payload"]["object_id"]
        .as_str()
        .expect("signed OBJECT payload should include object_id")
        .to_owned();

    session.verify_incoming(&hello).expect("HELLO should verify");
    let error = session.verify_incoming(&object).unwrap_err();

    assert_eq!(
        error,
        format!("wire OBJECT '{object_id}' was not requested from 'node:alpha'")
    );
}
