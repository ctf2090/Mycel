use super::*;

#[test]
fn sync_peer_store_json_fetches_offered_snapshots_into_local_store() {
    let signing_key = signing_key();
    let sender = "node:alpha";
    let remote_store = create_temp_dir("sync-peer-store-snapshot-remote");
    let local_store = create_temp_dir("sync-peer-store-snapshot-local");
    let signing_key_path = remote_store.path().join("peer.key");

    let patch_object = signed_patch_object_message(&signing_key, sender, "rev:genesis-null");
    let patch_id = patch_object["payload"]["object_id"]
        .as_str()
        .expect("patch object id should exist")
        .to_string();
    let revision_object = signed_revision_object_message(&signing_key, sender, &[], &[&patch_id]);
    let revision_id = revision_object["payload"]["object_id"]
        .as_str()
        .expect("revision object id should exist")
        .to_string();
    let snapshot_object = signed_snapshot_object_message(&signing_key, sender, &revision_id);
    let snapshot_id = snapshot_object["payload"]["object_id"]
        .as_str()
        .expect("snapshot object id should exist")
        .to_string();

    for body in [
        &patch_object["payload"]["body"],
        &revision_object["payload"]["body"],
        &snapshot_object["payload"]["body"],
    ] {
        write_object_value_to_store(remote_store.path(), body)
            .expect("object should write to remote store");
    }
    write_signing_key(&signing_key_path, &signing_key);

    let output = run_mycel(&[
        "sync",
        "peer-store",
        "--from",
        &path_arg(remote_store.path()),
        "--into",
        &path_arg(local_store.path()),
        "--peer-node-id",
        sender,
        "--signing-key",
        &path_arg(&signing_key_path),
        "--json",
    ]);

    assert_success(&output);
    let json = assert_json_status(&output, "ok");
    assert_eq!(json["peer_node_id"], sender);
    assert_eq!(
        json["object_message_count"], 3,
        "expected revision, patch, and snapshot transfer"
    );
    assert_eq!(json["written_object_count"], 3);

    let manifest_path = local_store.path().join("indexes").join("manifest.json");
    let manifest: Value =
        serde_json::from_str(&fs::read_to_string(&manifest_path).expect("manifest should read"))
            .expect("manifest should parse");
    assert_eq!(manifest["stored_object_count"], 3);
    assert_eq!(manifest["object_ids_by_type"]["snapshot"][0], snapshot_id);
}

#[test]
fn sync_peer_store_json_runs_first_time_sync_into_local_store() {
    let signing_key = signing_key();
    let sender = "node:alpha";
    let remote_store = create_temp_dir("sync-peer-store-remote");
    let local_store = create_temp_dir("sync-peer-store-local");
    let signing_key_path = remote_store.path().join("peer.key");

    let patch_object = signed_patch_object_message(&signing_key, sender, "rev:genesis-null");
    let patch_id = patch_object["payload"]["object_id"]
        .as_str()
        .expect("patch object id should exist")
        .to_string();
    let revision_object = signed_revision_object_message(&signing_key, sender, &[], &[&patch_id]);

    write_object_value_to_store(remote_store.path(), &patch_object["payload"]["body"])
        .expect("patch should write to remote store");
    write_object_value_to_store(remote_store.path(), &revision_object["payload"]["body"])
        .expect("revision should write to remote store");
    write_signing_key(&signing_key_path, &signing_key);

    let output = run_mycel(&[
        "sync",
        "peer-store",
        "--from",
        &path_arg(remote_store.path()),
        "--into",
        &path_arg(local_store.path()),
        "--peer-node-id",
        sender,
        "--signing-key",
        &path_arg(&signing_key_path),
        "--json",
    ]);

    assert_success(&output);
    let json = assert_json_status(&output, "ok");
    assert_eq!(json["peer_node_id"], sender);
    assert_eq!(json["source_store"], path_arg(remote_store.path()));
    assert_eq!(json["message_count"], 7);
    assert_eq!(json["object_message_count"], 2);
    assert_eq!(json["written_object_count"], 2);
    assert_eq!(json["existing_object_count"], 0);

    let manifest_path = local_store.path().join("indexes").join("manifest.json");
    let manifest: Value =
        serde_json::from_str(&fs::read_to_string(&manifest_path).expect("manifest should read"))
            .expect("manifest should parse");
    assert_eq!(manifest["stored_object_count"], 2);
}

#[test]
fn sync_stream_to_pull_via_pipe_replays_peer_store_into_local_store() {
    let signing_key = signing_key();
    let sender = "node:alpha";
    let remote_store = create_temp_dir("sync-stream-multi-process-remote");
    let local_store = create_temp_dir("sync-stream-multi-process-local");
    let signing_key_path = remote_store.path().join("peer.key");

    let patch_object = signed_patch_object_message(&signing_key, sender, "rev:genesis-null");
    let patch_id = patch_object["payload"]["object_id"]
        .as_str()
        .expect("patch object id should exist")
        .to_string();
    let revision_object = signed_revision_object_message(&signing_key, sender, &[], &[&patch_id]);
    let revision_id = revision_object["payload"]["object_id"]
        .as_str()
        .expect("revision object id should exist")
        .to_string();

    write_object_value_to_store(remote_store.path(), &patch_object["payload"]["body"])
        .expect("patch should write to remote store");
    write_object_value_to_store(remote_store.path(), &revision_object["payload"]["body"])
        .expect("revision should write to remote store");
    write_signing_key(&signing_key_path, &signing_key);

    let mut stream_child = Command::new(mycel_bin())
        .args([
            "sync",
            "stream",
            "--store",
            &path_arg(remote_store.path()),
            "--signing-key",
            &path_arg(&signing_key_path),
            "--node-id",
            sender,
        ])
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .expect("sync stream process should spawn");

    let stream_stdout = stream_child
        .stdout
        .take()
        .expect("sync stream stdout should be piped");

    let pull_output = Command::new(mycel_bin())
        .args([
            "sync",
            "pull",
            "-",
            "--into",
            &path_arg(local_store.path()),
            "--json",
        ])
        .stdin(stream_stdout)
        .output()
        .expect("sync pull process should run");

    let stream_output = stream_child
        .wait_with_output()
        .expect("sync stream process should finish");

    assert_success(&stream_output);
    assert_success(&pull_output);

    let json = assert_json_status(&pull_output, "ok");
    assert_eq!(json["peer_node_id"], sender);
    assert_eq!(json["message_count"], 7);
    assert_eq!(json["object_message_count"], 2);
    assert_eq!(json["written_object_count"], 2);

    let remote_manifest_path = remote_store.path().join("indexes").join("manifest.json");
    let remote_manifest: Value = serde_json::from_str(
        &fs::read_to_string(&remote_manifest_path).expect("remote manifest should read"),
    )
    .expect("remote manifest should parse");
    let local_manifest_path = local_store.path().join("indexes").join("manifest.json");
    let local_manifest: Value = serde_json::from_str(
        &fs::read_to_string(&local_manifest_path).expect("local manifest should read"),
    )
    .expect("local manifest should parse");

    assert_eq!(local_manifest["stored_object_count"], 2);
    assert_eq!(
        local_manifest["doc_revisions"]["doc:test"],
        remote_manifest["doc_revisions"]["doc:test"]
    );
    assert_eq!(local_manifest["doc_revisions"]["doc:test"][0], revision_id);
}

#[test]
fn sync_peer_store_json_fetches_announced_views_into_governance_indexes() {
    let signing_key = signing_key();
    let sender = "node:alpha";
    let remote_store = create_temp_dir("sync-peer-store-view-remote");
    let local_store = create_temp_dir("sync-peer-store-view-local");
    let signing_key_path = remote_store.path().join("peer.key");

    let patch_object = signed_patch_object_message(&signing_key, sender, "rev:genesis-null");
    let patch_id = patch_object["payload"]["object_id"]
        .as_str()
        .expect("patch object id should exist")
        .to_string();
    let revision_object = signed_revision_object_message(&signing_key, sender, &[], &[&patch_id]);
    let revision_id = revision_object["payload"]["object_id"]
        .as_str()
        .expect("revision object id should exist")
        .to_string();
    let view_object = signed_view_object_message(&signing_key, sender, &revision_id);
    let view_id = view_object["payload"]["object_id"]
        .as_str()
        .expect("view object id should exist")
        .to_string();

    for body in [
        &patch_object["payload"]["body"],
        &revision_object["payload"]["body"],
        &view_object["payload"]["body"],
    ] {
        write_object_value_to_store(remote_store.path(), body)
            .expect("object should write to remote store");
    }
    write_signing_key(&signing_key_path, &signing_key);

    let output = run_mycel(&[
        "sync",
        "peer-store",
        "--from",
        &path_arg(remote_store.path()),
        "--into",
        &path_arg(local_store.path()),
        "--peer-node-id",
        sender,
        "--signing-key",
        &path_arg(&signing_key_path),
        "--json",
    ]);

    assert_success(&output);
    let json = assert_json_status(&output, "ok");
    assert_eq!(json["peer_node_id"], sender);
    assert_eq!(json["object_message_count"], 3);
    assert_eq!(json["written_object_count"], 3);

    let manifest_path = local_store.path().join("indexes").join("manifest.json");
    let manifest: Value =
        serde_json::from_str(&fs::read_to_string(&manifest_path).expect("manifest should read"))
            .expect("manifest should parse");
    assert_eq!(manifest["stored_object_count"], 3);
    assert_eq!(
        manifest["view_governance"].as_array().map(Vec::len),
        Some(1)
    );
    assert_eq!(manifest["view_governance"][0]["view_id"], view_id);
    assert_eq!(manifest["document_views"]["doc:test"][0], view_id);
}

#[test]
fn sync_peer_store_json_reports_noop_when_local_store_is_current() {
    let signing_key = signing_key();
    let sender = "node:alpha";
    let remote_store = create_temp_dir("sync-peer-store-noop-remote");
    let local_store = create_temp_dir("sync-peer-store-noop-local");
    let signing_key_path = remote_store.path().join("peer.key");

    let patch_object = signed_patch_object_message(&signing_key, sender, "rev:genesis-null");
    let patch_id = patch_object["payload"]["object_id"]
        .as_str()
        .expect("patch object id should exist")
        .to_string();
    let revision_object = signed_revision_object_message(&signing_key, sender, &[], &[&patch_id]);

    for store_root in [remote_store.path(), local_store.path()] {
        write_object_value_to_store(store_root, &patch_object["payload"]["body"])
            .expect("patch should write to store");
        write_object_value_to_store(store_root, &revision_object["payload"]["body"])
            .expect("revision should write to store");
    }
    write_signing_key(&signing_key_path, &signing_key);

    let output = run_mycel(&[
        "sync",
        "peer-store",
        "--from",
        &path_arg(remote_store.path()),
        "--into",
        &path_arg(local_store.path()),
        "--peer-node-id",
        sender,
        "--signing-key",
        &path_arg(&signing_key_path),
        "--json",
    ]);

    assert_success(&output);
    let json = assert_json_status(&output, "ok");
    assert_eq!(json["peer_node_id"], sender);
    assert_eq!(json["object_message_count"], 0);
    assert_eq!(json["written_object_count"], 0);
    assert!(
        json["notes"]
            .as_array()
            .is_some_and(|notes| notes.iter().any(|note| {
                note.as_str()
                    .is_some_and(|value| value.contains("no WANT messages"))
            })),
        "expected no-op note, stdout: {}",
        stdout_text(&output)
    );
}

#[test]
fn sync_peer_store_json_limits_sync_to_requested_document_subset() {
    let signing_key = signing_key();
    let sender = "node:alpha";
    let remote_store = create_temp_dir("sync-peer-store-partial-doc-remote");
    let local_store = create_temp_dir("sync-peer-store-partial-doc-local");
    let signing_key_path = remote_store.path().join("peer.key");

    let alpha_patch = signed_patch_object_message_for_doc(
        &signing_key,
        sender,
        "doc:partial-alpha",
        "rev:genesis-null",
    );
    let alpha_patch_id = alpha_patch["payload"]["object_id"]
        .as_str()
        .expect("alpha patch object id should exist")
        .to_string();
    let alpha_revision = signed_revision_object_message_for_doc(
        &signing_key,
        sender,
        "doc:partial-alpha",
        &[],
        &[&alpha_patch_id],
    );
    let alpha_revision_id = alpha_revision["payload"]["object_id"]
        .as_str()
        .expect("alpha revision object id should exist")
        .to_string();

    let beta_patch = signed_patch_object_message_for_doc(
        &signing_key,
        sender,
        "doc:partial-beta",
        "rev:genesis-null",
    );
    let beta_patch_id = beta_patch["payload"]["object_id"]
        .as_str()
        .expect("beta patch object id should exist")
        .to_string();
    let beta_revision = signed_revision_object_message_for_doc(
        &signing_key,
        sender,
        "doc:partial-beta",
        &[],
        &[&beta_patch_id],
    );

    for body in [
        &alpha_patch["payload"]["body"],
        &alpha_revision["payload"]["body"],
        &beta_patch["payload"]["body"],
        &beta_revision["payload"]["body"],
    ] {
        write_object_value_to_store(remote_store.path(), body)
            .expect("object should write to remote store");
    }
    write_signing_key(&signing_key_path, &signing_key);

    let output = run_mycel(&[
        "sync",
        "peer-store",
        "--from",
        &path_arg(remote_store.path()),
        "--into",
        &path_arg(local_store.path()),
        "--peer-node-id",
        sender,
        "--signing-key",
        &path_arg(&signing_key_path),
        "--doc-id",
        "doc:partial-alpha",
        "--json",
    ]);

    assert_success(&output);
    let json = assert_json_status(&output, "ok");
    assert_eq!(json["peer_node_id"], sender);
    assert_eq!(json["object_message_count"], 2);
    assert_eq!(json["written_object_count"], 2);

    let manifest_path = local_store.path().join("indexes").join("manifest.json");
    let manifest: Value =
        serde_json::from_str(&fs::read_to_string(&manifest_path).expect("manifest should read"))
            .expect("manifest should parse");
    assert_eq!(manifest["stored_object_count"], 2);
    assert_eq!(
        manifest["doc_revisions"]["doc:partial-alpha"]
            .as_array()
            .map(Vec::len),
        Some(1)
    );
    assert_eq!(
        manifest["doc_revisions"]["doc:partial-alpha"][0],
        alpha_revision_id
    );
    assert!(
        manifest["doc_revisions"].get("doc:partial-beta").is_none(),
        "expected excluded document to remain absent, manifest: {manifest}"
    );
}

#[test]
fn sync_peer_store_json_converges_partial_and_empty_local_stores() {
    let signing_key = signing_key();
    let sender = "node:alpha";
    let remote_store = create_temp_dir("sync-peer-store-mixed-remote");
    let partial_local_store = create_temp_dir("sync-peer-store-mixed-partial-local");
    let empty_local_store = create_temp_dir("sync-peer-store-mixed-empty-local");
    let signing_key_path = remote_store.path().join("peer.key");

    let patch_object = signed_patch_object_message(&signing_key, sender, "rev:genesis-null");
    let patch_id = patch_object["payload"]["object_id"]
        .as_str()
        .expect("patch object id should exist")
        .to_string();
    let revision_object = signed_revision_object_message(&signing_key, sender, &[], &[&patch_id]);

    write_object_value_to_store(remote_store.path(), &patch_object["payload"]["body"])
        .expect("patch should write to remote store");
    write_object_value_to_store(remote_store.path(), &revision_object["payload"]["body"])
        .expect("revision should write to remote store");
    write_object_value_to_store(partial_local_store.path(), &patch_object["payload"]["body"])
        .expect("patch should write to partial local store");
    write_signing_key(&signing_key_path, &signing_key);

    let partial_output = run_mycel(&[
        "sync",
        "peer-store",
        "--from",
        &path_arg(remote_store.path()),
        "--into",
        &path_arg(partial_local_store.path()),
        "--peer-node-id",
        sender,
        "--signing-key",
        &path_arg(&signing_key_path),
        "--json",
    ]);
    let empty_output = run_mycel(&[
        "sync",
        "peer-store",
        "--from",
        &path_arg(remote_store.path()),
        "--into",
        &path_arg(empty_local_store.path()),
        "--peer-node-id",
        sender,
        "--signing-key",
        &path_arg(&signing_key_path),
        "--json",
    ]);

    assert_success(&partial_output);
    assert_success(&empty_output);

    let partial_json = assert_json_status(&partial_output, "ok");
    let empty_json = assert_json_status(&empty_output, "ok");
    assert_eq!(partial_json["peer_node_id"], sender);
    assert_eq!(empty_json["peer_node_id"], sender);
    assert_eq!(partial_json["written_object_count"], 1);
    assert_eq!(empty_json["written_object_count"], 2);

    let partial_manifest_path = partial_local_store
        .path()
        .join("indexes")
        .join("manifest.json");
    let partial_manifest: Value = serde_json::from_str(
        &fs::read_to_string(&partial_manifest_path).expect("manifest should read"),
    )
    .expect("manifest should parse");
    let empty_manifest_path = empty_local_store
        .path()
        .join("indexes")
        .join("manifest.json");
    let empty_manifest: Value = serde_json::from_str(
        &fs::read_to_string(&empty_manifest_path).expect("manifest should read"),
    )
    .expect("manifest should parse");

    assert_eq!(partial_manifest["stored_object_count"], 2);
    assert_eq!(empty_manifest["stored_object_count"], 2);
    assert_eq!(
        partial_manifest["doc_revisions"]["doc:test"],
        empty_manifest["doc_revisions"]["doc:test"]
    );
    assert_eq!(
        partial_manifest["object_ids_by_type"],
        empty_manifest["object_ids_by_type"]
    );
}

#[test]
fn sync_peer_store_json_converges_two_empty_readers_on_same_store_state() {
    let signing_key = signing_key();
    let sender = "node:alpha";
    let remote_store = create_temp_dir("sync-peer-store-three-peer-remote");
    let reader_a_store = create_temp_dir("sync-peer-store-three-peer-reader-a");
    let reader_b_store = create_temp_dir("sync-peer-store-three-peer-reader-b");
    let signing_key_path = remote_store.path().join("peer.key");

    let patch_object = signed_patch_object_message(&signing_key, sender, "rev:genesis-null");
    let patch_id = patch_object["payload"]["object_id"]
        .as_str()
        .expect("patch object id should exist")
        .to_string();
    let revision_object = signed_revision_object_message(&signing_key, sender, &[], &[&patch_id]);
    let revision_id = revision_object["payload"]["object_id"]
        .as_str()
        .expect("revision object id should exist")
        .to_string();

    write_object_value_to_store(remote_store.path(), &patch_object["payload"]["body"])
        .expect("patch should write to remote store");
    write_object_value_to_store(remote_store.path(), &revision_object["payload"]["body"])
        .expect("revision should write to remote store");
    write_signing_key(&signing_key_path, &signing_key);

    let reader_a_output = run_mycel(&[
        "sync",
        "peer-store",
        "--from",
        &path_arg(remote_store.path()),
        "--into",
        &path_arg(reader_a_store.path()),
        "--peer-node-id",
        sender,
        "--signing-key",
        &path_arg(&signing_key_path),
        "--json",
    ]);
    let reader_b_output = run_mycel(&[
        "sync",
        "peer-store",
        "--from",
        &path_arg(remote_store.path()),
        "--into",
        &path_arg(reader_b_store.path()),
        "--peer-node-id",
        sender,
        "--signing-key",
        &path_arg(&signing_key_path),
        "--json",
    ]);

    assert_success(&reader_a_output);
    assert_success(&reader_b_output);

    let reader_a_json = assert_json_status(&reader_a_output, "ok");
    let reader_b_json = assert_json_status(&reader_b_output, "ok");
    assert_eq!(reader_a_json["peer_node_id"], sender);
    assert_eq!(reader_b_json["peer_node_id"], sender);
    assert_eq!(reader_a_json["written_object_count"], 2);
    assert_eq!(reader_b_json["written_object_count"], 2);

    let remote_manifest_path = remote_store.path().join("indexes").join("manifest.json");
    let remote_manifest: Value = serde_json::from_str(
        &fs::read_to_string(&remote_manifest_path).expect("remote manifest should read"),
    )
    .expect("remote manifest should parse");
    let reader_a_manifest_path = reader_a_store.path().join("indexes").join("manifest.json");
    let reader_a_manifest: Value = serde_json::from_str(
        &fs::read_to_string(&reader_a_manifest_path).expect("reader A manifest should read"),
    )
    .expect("reader A manifest should parse");
    let reader_b_manifest_path = reader_b_store.path().join("indexes").join("manifest.json");
    let reader_b_manifest: Value = serde_json::from_str(
        &fs::read_to_string(&reader_b_manifest_path).expect("reader B manifest should read"),
    )
    .expect("reader B manifest should parse");

    assert_eq!(reader_a_manifest["stored_object_count"], 2);
    assert_eq!(reader_b_manifest["stored_object_count"], 2);
    assert_eq!(
        reader_a_manifest["doc_revisions"],
        reader_b_manifest["doc_revisions"]
    );
    assert_eq!(
        reader_a_manifest["object_ids_by_type"],
        reader_b_manifest["object_ids_by_type"]
    );
    assert_eq!(
        reader_a_manifest["doc_revisions"],
        remote_manifest["doc_revisions"]
    );
    assert_eq!(
        reader_a_manifest["doc_revisions"]["doc:test"][0],
        revision_id
    );
}

#[test]
fn sync_peer_store_json_converges_four_readers_on_same_multi_doc_state() {
    let signing_key = signing_key();
    let sender = "node:alpha";
    let remote_store = create_temp_dir("sync-peer-store-four-reader-remote");
    let reader_a_store = create_temp_dir("sync-peer-store-four-reader-a");
    let reader_b_store = create_temp_dir("sync-peer-store-four-reader-b");
    let reader_c_store = create_temp_dir("sync-peer-store-four-reader-c");
    let reader_d_store = create_temp_dir("sync-peer-store-four-reader-d");
    let signing_key_path = remote_store.path().join("peer.key");

    let alpha_patch =
        signed_patch_object_message_for_doc(&signing_key, sender, "doc:alpha", "rev:genesis-null");
    let alpha_patch_id = alpha_patch["payload"]["object_id"]
        .as_str()
        .expect("alpha patch object id should exist")
        .to_string();
    let alpha_revision = signed_revision_object_message_for_doc(
        &signing_key,
        sender,
        "doc:alpha",
        &[],
        &[&alpha_patch_id],
    );
    let alpha_revision_id = alpha_revision["payload"]["object_id"]
        .as_str()
        .expect("alpha revision object id should exist")
        .to_string();

    let beta_patch =
        signed_patch_object_message_for_doc(&signing_key, sender, "doc:beta", "rev:genesis-null");
    let beta_patch_id = beta_patch["payload"]["object_id"]
        .as_str()
        .expect("beta patch object id should exist")
        .to_string();
    let beta_revision = signed_revision_object_message_for_doc(
        &signing_key,
        sender,
        "doc:beta",
        &[],
        &[&beta_patch_id],
    );
    let beta_revision_id = beta_revision["payload"]["object_id"]
        .as_str()
        .expect("beta revision object id should exist")
        .to_string();

    for body in [
        &alpha_patch["payload"]["body"],
        &alpha_revision["payload"]["body"],
        &beta_patch["payload"]["body"],
        &beta_revision["payload"]["body"],
    ] {
        write_object_value_to_store(remote_store.path(), body)
            .expect("object should write to remote store");
    }
    write_signing_key(&signing_key_path, &signing_key);

    let reader_paths = [
        reader_a_store.path(),
        reader_b_store.path(),
        reader_c_store.path(),
        reader_d_store.path(),
    ];
    let outputs = reader_paths
        .iter()
        .map(|store_root| {
            run_mycel(&[
                "sync",
                "peer-store",
                "--from",
                &path_arg(remote_store.path()),
                "--into",
                &path_arg(store_root),
                "--peer-node-id",
                sender,
                "--signing-key",
                &path_arg(&signing_key_path),
                "--json",
            ])
        })
        .collect::<Vec<_>>();

    for output in &outputs {
        assert_success(output);
        let json = assert_json_status(output, "ok");
        assert_eq!(json["peer_node_id"], sender);
        assert_eq!(json["written_object_count"], 4);
    }

    let remote_manifest_path = remote_store.path().join("indexes").join("manifest.json");
    let remote_manifest: Value = serde_json::from_str(
        &fs::read_to_string(&remote_manifest_path).expect("remote manifest should read"),
    )
    .expect("remote manifest should parse");

    let reader_manifests = reader_paths
        .iter()
        .map(|store_root| {
            let manifest_path = store_root.join("indexes").join("manifest.json");
            serde_json::from_str::<Value>(
                &fs::read_to_string(&manifest_path).expect("reader manifest should read"),
            )
            .expect("reader manifest should parse")
        })
        .collect::<Vec<_>>();

    for manifest in &reader_manifests {
        assert_eq!(manifest["stored_object_count"], 4);
        assert_eq!(manifest["doc_revisions"], remote_manifest["doc_revisions"]);
        assert_eq!(
            manifest["object_ids_by_type"],
            remote_manifest["object_ids_by_type"]
        );
        assert!(
            manifest["doc_revisions"]["doc:alpha"]
                .as_array()
                .is_some_and(|values| values
                    .iter()
                    .any(|value| value == &json!(alpha_revision_id))),
            "expected doc:alpha revision in manifest: {manifest}"
        );
        assert!(
            manifest["doc_revisions"]["doc:beta"]
                .as_array()
                .is_some_and(|values| values.iter().any(|value| value == &json!(beta_revision_id))),
            "expected doc:beta revision in manifest: {manifest}"
        );
    }

    for window in reader_manifests.windows(2) {
        assert_eq!(window[0]["doc_revisions"], window[1]["doc_revisions"]);
        assert_eq!(
            window[0]["object_ids_by_type"],
            window[1]["object_ids_by_type"]
        );
    }
}
