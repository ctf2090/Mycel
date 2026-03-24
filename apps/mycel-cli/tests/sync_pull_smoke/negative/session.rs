use super::*;

#[test]
fn sync_pull_json_rejects_messages_after_bye() {
    let signing_key = signing_key();
    let sender = "node:alpha";
    let transcript_dir = create_temp_dir("sync-pull-after-bye");
    let transcript_path = transcript_dir.path().join("after-bye-transcript.json");
    let store_root = create_temp_dir("sync-pull-after-bye-store");
    write_transcript(
        &transcript_path,
        &json!({
            "peer": {
                "node_id": sender,
                "public_key": sender_public_key(&signing_key)
            },
            "messages": [
                signed_hello_message(&signing_key, sender),
                signed_bye_message(&signing_key, sender),
                signed_want_message(&signing_key, sender, &["patch:test"])
            ]
        }),
    );

    let output = run_mycel(&[
        "sync",
        "pull",
        &path_arg(&transcript_path),
        "--into",
        &path_arg(store_root.path()),
        "--json",
    ]);

    assert!(
        !output.status.success(),
        "expected failure, stdout: {}, stderr: {}",
        stdout_text(&output),
        String::from_utf8_lossy(&output.stderr)
    );
    let json = assert_json_status(&output, "failed");
    assert_eq!(json["verified_message_count"], 2);
    assert_eq!(json["object_message_count"], 0);
    assert_eq!(json["written_object_count"], 0);
    assert!(
        json["errors"]
            .as_array()
            .is_some_and(|errors| errors.iter().any(|error| {
                error.as_str().is_some_and(|message| {
                    message.contains("wire session for 'node:alpha' is already closed")
                })
            })),
        "expected already-closed error, stdout: {}",
        stdout_text(&output)
    );
    assert!(!store_root
        .path()
        .join("indexes")
        .join("manifest.json")
        .exists());
}

#[test]
fn sync_pull_json_rejects_bye_before_hello() {
    let signing_key = signing_key();
    let sender = "node:alpha";
    let transcript_dir = create_temp_dir("sync-pull-bye-before-hello");
    let transcript_path = transcript_dir
        .path()
        .join("bye-before-hello-transcript.json");
    let store_root = create_temp_dir("sync-pull-bye-before-hello-store");
    write_transcript(
        &transcript_path,
        &json!({
            "peer": {
                "node_id": sender,
                "public_key": sender_public_key(&signing_key)
            },
            "messages": [
                signed_bye_message(&signing_key, sender),
                signed_hello_message(&signing_key, sender)
            ]
        }),
    );

    let output = run_mycel(&[
        "sync",
        "pull",
        &path_arg(&transcript_path),
        "--into",
        &path_arg(store_root.path()),
        "--json",
    ]);

    assert!(
        !output.status.success(),
        "expected failure, stdout: {}, stderr: {}",
        stdout_text(&output),
        String::from_utf8_lossy(&output.stderr)
    );
    let json = assert_json_status(&output, "failed");
    assert_eq!(json["verified_message_count"], 0);
    assert_eq!(json["object_message_count"], 0);
    assert_eq!(json["written_object_count"], 0);
    assert!(
        json["errors"]
            .as_array()
            .is_some_and(|errors| errors.iter().any(|error| {
                error.as_str().is_some_and(|message| {
                    message.contains("wire BYE requires prior HELLO from 'node:alpha'")
                })
            })),
        "expected prior-HELLO BYE error, stdout: {}",
        stdout_text(&output)
    );
    assert!(!store_root
        .path()
        .join("indexes")
        .join("manifest.json")
        .exists());
}

#[test]
fn sync_pull_json_rejects_duplicate_hello() {
    let signing_key = signing_key();
    let sender = "node:alpha";
    let transcript_dir = create_temp_dir("sync-pull-duplicate-hello");
    let transcript_path = transcript_dir
        .path()
        .join("duplicate-hello-transcript.json");
    let store_root = create_temp_dir("sync-pull-duplicate-hello-store");
    write_transcript(
        &transcript_path,
        &json!({
            "peer": {
                "node_id": sender,
                "public_key": sender_public_key(&signing_key)
            },
            "messages": [
                signed_hello_message(&signing_key, sender),
                signed_hello_message(&signing_key, sender)
            ]
        }),
    );

    let output = run_mycel(&[
        "sync",
        "pull",
        &path_arg(&transcript_path),
        "--into",
        &path_arg(store_root.path()),
        "--json",
    ]);

    assert!(
        !output.status.success(),
        "expected failure, stdout: {}, stderr: {}",
        stdout_text(&output),
        String::from_utf8_lossy(&output.stderr)
    );
    let json = assert_json_status(&output, "failed");
    assert_eq!(json["verified_message_count"], 1);
    assert_eq!(json["object_message_count"], 0);
    assert_eq!(json["written_object_count"], 0);
    assert!(
        json["errors"]
            .as_array()
            .is_some_and(|errors| errors.iter().any(|error| {
                error
                    .as_str()
                    .is_some_and(|message| message.contains("wire session already received HELLO"))
            })),
        "expected duplicate-HELLO error, stdout: {}",
        stdout_text(&output)
    );
    assert!(!store_root
        .path()
        .join("indexes")
        .join("manifest.json")
        .exists());
}

#[test]
fn sync_pull_json_rejects_manifest_before_hello() {
    let signing_key = signing_key();
    let sender = "node:alpha";
    let transcript_dir = create_temp_dir("sync-pull-manifest-before-hello");
    let transcript_path = transcript_dir
        .path()
        .join("manifest-before-hello-transcript.json");
    let store_root = create_temp_dir("sync-pull-manifest-before-hello-store");
    write_transcript(
        &transcript_path,
        &json!({
            "peer": {
                "node_id": sender,
                "public_key": sender_public_key(&signing_key)
            },
            "messages": [
                signed_manifest_message(&signing_key, sender, "rev:test"),
                signed_hello_message(&signing_key, sender)
            ]
        }),
    );

    let output = run_mycel(&[
        "sync",
        "pull",
        &path_arg(&transcript_path),
        "--into",
        &path_arg(store_root.path()),
        "--json",
    ]);

    assert!(
        !output.status.success(),
        "expected failure, stdout: {}, stderr: {}",
        stdout_text(&output),
        String::from_utf8_lossy(&output.stderr)
    );
    let json = assert_json_status(&output, "failed");
    assert_eq!(json["verified_message_count"], 0);
    assert_eq!(json["object_message_count"], 0);
    assert_eq!(json["written_object_count"], 0);
    assert!(
        json["errors"]
            .as_array()
            .is_some_and(|errors| errors.iter().any(|error| {
                error.as_str().is_some_and(|message| {
                    message.contains("wire MANIFEST requires prior HELLO from 'node:alpha'")
                })
            })),
        "expected prior-HELLO MANIFEST error, stdout: {}",
        stdout_text(&output)
    );
    assert!(!store_root
        .path()
        .join("indexes")
        .join("manifest.json")
        .exists());
}

#[test]
fn sync_pull_json_rejects_heads_before_hello() {
    let signing_key = signing_key();
    let sender = "node:alpha";
    let transcript_dir = create_temp_dir("sync-pull-heads-before-hello");
    let transcript_path = transcript_dir
        .path()
        .join("heads-before-hello-transcript.json");
    let store_root = create_temp_dir("sync-pull-heads-before-hello-store");
    write_transcript(
        &transcript_path,
        &json!({
            "peer": {
                "node_id": sender,
                "public_key": sender_public_key(&signing_key)
            },
            "messages": [
                signed_heads_message(&signing_key, sender, "rev:test", false),
                signed_hello_message(&signing_key, sender)
            ]
        }),
    );

    let output = run_mycel(&[
        "sync",
        "pull",
        &path_arg(&transcript_path),
        "--into",
        &path_arg(store_root.path()),
        "--json",
    ]);

    assert!(
        !output.status.success(),
        "expected failure, stdout: {}, stderr: {}",
        stdout_text(&output),
        String::from_utf8_lossy(&output.stderr)
    );
    let json = assert_json_status(&output, "failed");
    assert_eq!(json["verified_message_count"], 0);
    assert_eq!(json["object_message_count"], 0);
    assert_eq!(json["written_object_count"], 0);
    assert!(
        json["errors"]
            .as_array()
            .is_some_and(|errors| errors.iter().any(|error| {
                error.as_str().is_some_and(|message| {
                    message.contains("wire HEADS requires prior HELLO from 'node:alpha'")
                })
            })),
        "expected prior-HELLO HEADS error, stdout: {}",
        stdout_text(&output)
    );
    assert!(!store_root
        .path()
        .join("indexes")
        .join("manifest.json")
        .exists());
}

#[test]
fn sync_pull_json_rejects_want_before_hello() {
    let signing_key = signing_key();
    let sender = "node:alpha";
    let transcript_dir = create_temp_dir("sync-pull-want-before-hello");
    let transcript_path = transcript_dir
        .path()
        .join("want-before-hello-transcript.json");
    let store_root = create_temp_dir("sync-pull-want-before-hello-store");
    write_transcript(
        &transcript_path,
        &json!({
            "peer": {
                "node_id": sender,
                "public_key": sender_public_key(&signing_key)
            },
            "messages": [
                signed_want_message(&signing_key, sender, &["patch:test"]),
                signed_hello_message(&signing_key, sender)
            ]
        }),
    );

    let output = run_mycel(&[
        "sync",
        "pull",
        &path_arg(&transcript_path),
        "--into",
        &path_arg(store_root.path()),
        "--json",
    ]);

    assert!(
        !output.status.success(),
        "expected failure, stdout: {}, stderr: {}",
        stdout_text(&output),
        String::from_utf8_lossy(&output.stderr)
    );
    let json = assert_json_status(&output, "failed");
    assert_eq!(json["verified_message_count"], 0);
    assert_eq!(json["object_message_count"], 0);
    assert_eq!(json["written_object_count"], 0);
    assert!(
        json["errors"]
            .as_array()
            .is_some_and(|errors| errors.iter().any(|error| {
                error.as_str().is_some_and(|message| {
                    message.contains("wire WANT requires prior HELLO from 'node:alpha'")
                })
            })),
        "expected prior-HELLO WANT error, stdout: {}",
        stdout_text(&output)
    );
    assert!(!store_root
        .path()
        .join("indexes")
        .join("manifest.json")
        .exists());
}

#[test]
fn sync_pull_json_rejects_object_before_manifest() {
    let signing_key = signing_key();
    let sender = "node:alpha";
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
    let transcript_dir = create_temp_dir("sync-pull-object-before-manifest");
    let transcript_path = transcript_dir
        .path()
        .join("object-before-manifest-transcript.json");
    let store_root = create_temp_dir("sync-pull-object-before-manifest-store");
    write_transcript(
        &transcript_path,
        &json!({
            "peer": {
                "node_id": sender,
                "public_key": sender_public_key(&signing_key)
            },
            "messages": [
                signed_hello_message(&signing_key, sender),
                revision_object,
                signed_manifest_message(&signing_key, sender, &revision_id)
            ]
        }),
    );

    let output = run_mycel(&[
        "sync",
        "pull",
        &path_arg(&transcript_path),
        "--into",
        &path_arg(store_root.path()),
        "--json",
    ]);

    assert!(
        !output.status.success(),
        "expected failure, stdout: {}, stderr: {}",
        stdout_text(&output),
        String::from_utf8_lossy(&output.stderr)
    );
    let json = assert_json_status(&output, "failed");
    assert_eq!(json["verified_message_count"], 1);
    assert_eq!(json["object_message_count"], 0);
    assert_eq!(json["written_object_count"], 0);
    assert!(
        json["errors"]
            .as_array()
            .is_some_and(|errors| errors.iter().any(|error| {
                error.as_str().is_some_and(|message| {
                    message.contains(&format!(
                        "wire OBJECT '{revision_id}' was not requested from '{sender}'"
                    ))
                })
            })),
        "expected object-before-manifest error, stdout: {}",
        stdout_text(&output)
    );
    assert!(!store_root
        .path()
        .join("indexes")
        .join("manifest.json")
        .exists());
}

#[test]
fn sync_pull_json_rejects_unadvertised_revision_want_after_manifest() {
    let signing_key = signing_key();
    let sender = "node:alpha";
    let transcript_dir = create_temp_dir("sync-pull-unadvertised-revision-want");
    let transcript_path = transcript_dir
        .path()
        .join("unadvertised-revision-want-transcript.json");
    let store_root = create_temp_dir("sync-pull-unadvertised-revision-want-store");
    write_transcript(
        &transcript_path,
        &json!({
            "peer": {
                "node_id": sender,
                "public_key": sender_public_key(&signing_key)
            },
            "messages": [
                signed_hello_message(&signing_key, sender),
                signed_manifest_message(&signing_key, sender, "rev:advertised-root"),
                signed_want_message(&signing_key, sender, &["rev:missing"])
            ]
        }),
    );

    let output = run_mycel(&[
        "sync",
        "pull",
        &path_arg(&transcript_path),
        "--into",
        &path_arg(store_root.path()),
        "--json",
    ]);

    assert!(
        !output.status.success(),
        "expected failure, stdout: {}, stderr: {}",
        stdout_text(&output),
        String::from_utf8_lossy(&output.stderr)
    );
    let json = assert_json_status(&output, "failed");
    assert_eq!(json["verified_message_count"], 2);
    assert_eq!(json["object_message_count"], 0);
    assert_eq!(json["written_object_count"], 0);
    assert!(
        json["errors"]
            .as_array()
            .is_some_and(|errors| errors.iter().any(|error| {
                error.as_str().is_some_and(|message| {
                    message.contains("wire WANT revision 'rev:missing'")
                        && message.contains("is not reachable from accepted sync roots")
                })
            })),
        "expected unadvertised revision WANT error, stdout: {}",
        stdout_text(&output)
    );
    assert!(!store_root
        .path()
        .join("indexes")
        .join("manifest.json")
        .exists());
}

#[test]
fn sync_pull_json_rejects_unadvertised_object_want_after_manifest() {
    let signing_key = signing_key();
    let sender = "node:alpha";
    let transcript_dir = create_temp_dir("sync-pull-unadvertised-object-want");
    let transcript_path = transcript_dir
        .path()
        .join("unadvertised-object-want-transcript.json");
    let store_root = create_temp_dir("sync-pull-unadvertised-object-want-store");
    write_transcript(
        &transcript_path,
        &json!({
            "peer": {
                "node_id": sender,
                "public_key": sender_public_key(&signing_key)
            },
            "messages": [
                signed_hello_message(&signing_key, sender),
                signed_manifest_message(&signing_key, sender, "rev:advertised-root"),
                signed_want_message(&signing_key, sender, &["patch:missing"])
            ]
        }),
    );

    let output = run_mycel(&[
        "sync",
        "pull",
        &path_arg(&transcript_path),
        "--into",
        &path_arg(store_root.path()),
        "--json",
    ]);

    assert!(
        !output.status.success(),
        "expected failure, stdout: {}, stderr: {}",
        stdout_text(&output),
        String::from_utf8_lossy(&output.stderr)
    );
    let json = assert_json_status(&output, "failed");
    assert_eq!(json["verified_message_count"], 2);
    assert_eq!(json["object_message_count"], 0);
    assert_eq!(json["written_object_count"], 0);
    assert!(
        json["errors"]
            .as_array()
            .is_some_and(|errors| errors.iter().any(|error| {
                error.as_str().is_some_and(|message| {
                    message.contains("wire WANT object 'patch:missing'")
                        && message.contains("is not reachable from accepted sync roots")
                })
            })),
        "expected unadvertised object WANT error, stdout: {}",
        stdout_text(&output)
    );
    assert!(!store_root
        .path()
        .join("indexes")
        .join("manifest.json")
        .exists());
}

#[test]
fn sync_pull_json_rejects_unrequested_root_object_after_manifest() {
    let signing_key = signing_key();
    let sender = "node:alpha";
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
    let transcript_dir = create_temp_dir("sync-pull-unrequested-root-object");
    let transcript_path = transcript_dir
        .path()
        .join("unrequested-root-object-transcript.json");
    let store_root = create_temp_dir("sync-pull-unrequested-root-object-store");
    write_transcript(
        &transcript_path,
        &json!({
            "peer": {
                "node_id": sender,
                "public_key": sender_public_key(&signing_key)
            },
            "messages": [
                signed_hello_message(&signing_key, sender),
                signed_manifest_message(&signing_key, sender, &revision_id),
                revision_object
            ]
        }),
    );

    let output = run_mycel(&[
        "sync",
        "pull",
        &path_arg(&transcript_path),
        "--into",
        &path_arg(store_root.path()),
        "--json",
    ]);

    assert!(
        !output.status.success(),
        "expected failure, stdout: {}, stderr: {}",
        stdout_text(&output),
        String::from_utf8_lossy(&output.stderr)
    );
    let json = assert_json_status(&output, "failed");
    assert_eq!(json["verified_message_count"], 2);
    assert_eq!(json["object_message_count"], 0);
    assert_eq!(json["written_object_count"], 0);
    assert!(
        json["errors"]
            .as_array()
            .is_some_and(|errors| errors.iter().any(|error| {
                error.as_str().is_some_and(|message| {
                    message.contains(&format!(
                        "wire OBJECT '{revision_id}' was not requested from '{sender}'"
                    ))
                })
            })),
        "expected unrequested root OBJECT error, stdout: {}",
        stdout_text(&output)
    );
    assert!(!store_root
        .path()
        .join("indexes")
        .join("manifest.json")
        .exists());
}

#[test]
fn sync_pull_json_rejects_unrequested_dependency_object_after_root_object() {
    let signing_key = signing_key();
    let sender = "node:alpha";
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
    let transcript_dir = create_temp_dir("sync-pull-unrequested-dependency-object");
    let transcript_path = transcript_dir
        .path()
        .join("unrequested-dependency-object-transcript.json");
    let store_root = create_temp_dir("sync-pull-unrequested-dependency-object-store");
    write_transcript(
        &transcript_path,
        &json!({
            "peer": {
                "node_id": sender,
                "public_key": sender_public_key(&signing_key)
            },
            "messages": [
                signed_hello_message(&signing_key, sender),
                signed_manifest_message(&signing_key, sender, &revision_id),
                signed_want_message(&signing_key, sender, &[&revision_id]),
                revision_object,
                patch_object
            ]
        }),
    );

    let output = run_mycel(&[
        "sync",
        "pull",
        &path_arg(&transcript_path),
        "--into",
        &path_arg(store_root.path()),
        "--json",
    ]);

    assert!(
        !output.status.success(),
        "expected failure, stdout: {}, stderr: {}",
        stdout_text(&output),
        String::from_utf8_lossy(&output.stderr)
    );
    let json = assert_json_status(&output, "failed");
    assert_eq!(json["verified_message_count"], 4);
    assert_eq!(json["object_message_count"], 1);
    assert_eq!(json["written_object_count"], 0);
    assert!(
        json["errors"]
            .as_array()
            .is_some_and(|errors| errors.iter().any(|error| {
                error.as_str().is_some_and(|message| {
                    message.contains(&format!(
                        "wire OBJECT '{patch_id}' was not requested from '{sender}'"
                    ))
                })
            })),
        "expected unrequested dependency OBJECT error, stdout: {}",
        stdout_text(&output)
    );
    assert!(!store_root
        .path()
        .join("indexes")
        .join("manifest.json")
        .exists());
}
