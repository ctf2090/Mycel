use super::*;

#[test]
fn head_render_json_replays_selected_head_from_store() {
    let doc_id = "doc:render";
    let revision_author = signing_key(81);
    let maintainer_a = signing_key(91);
    let maintainer_b = signing_key(92);
    let policy = json!({
        "accept_keys": [
            signer_id(&maintainer_a),
            signer_id(&maintainer_b)
        ],
        "merge_rule": "manual-reviewed",
        "preferred_branches": ["main"]
    });
    let genesis_hash = empty_document_state_hash(doc_id);
    let revision_a = signed_revision(&revision_author, doc_id, vec![], 1000, &genesis_hash);
    let patch = signed_patch(
        &revision_author,
        doc_id,
        revision_a["revision_id"]
            .as_str()
            .expect("revision id should exist"),
        1010,
        json!([
            {
                "op": "insert_block",
                "new_block": {
                    "block_id": "blk:render-001",
                    "block_type": "paragraph",
                    "content": "Hello render",
                    "attrs": {},
                    "children": [
                        {
                            "block_id": "blk:render-002",
                            "block_type": "paragraph",
                            "content": "Nested reply",
                            "attrs": {},
                            "children": []
                        }
                    ]
                }
            }
        ]),
    );
    let revision_b = signed_revision_with_patches(
        &revision_author,
        doc_id,
        vec![revision_a["revision_id"]
            .as_str()
            .expect("revision id should exist")
            .to_string()],
        vec![patch["patch_id"]
            .as_str()
            .expect("patch id should exist")
            .to_string()],
        1020,
        &document_state_hash(
            doc_id,
            vec![json!({
                "block_id": "blk:render-001",
                "block_type": "paragraph",
                "content": "Hello render",
                "attrs": {},
                "children": [
                    {
                        "block_id": "blk:render-002",
                        "block_type": "paragraph",
                        "content": "Nested reply",
                        "attrs": {},
                        "children": []
                    }
                ]
            })],
        ),
    );
    let view_a = signed_view(
        &maintainer_a,
        &policy,
        documents_value(doc_id, &revision_b["revision_id"]),
        1100,
    );
    let view_b = signed_view(
        &maintainer_b,
        &policy,
        documents_value(doc_id, &revision_b["revision_id"]),
        1110,
    );
    let store_dir =
        build_store_from_objects(&[revision_a, patch, revision_b.clone(), view_a, view_b]);
    let input = write_input_file(
        "head-render-store-backed",
        "input.json",
        json!({
            "profile": head_profile(hash_json(&policy), 1200),
            "revisions": [],
            "views": [],
            "critical_violations": []
        }),
    );

    let output = run_mycel(&[
        "head",
        "render",
        doc_id,
        "--input",
        &path_arg(&input.path),
        "--store-root",
        &path_arg(&store_dir.path().to_path_buf()),
        "--json",
    ]);

    assert_success(&output);
    let json = parse_json_stdout(&output);
    assert_eq!(json["status"], "ok");
    assert_eq!(json["selected_head"], revision_b["revision_id"]);
    assert_eq!(json["rendered_block_count"], 2);
    assert_eq!(json["rendered_text"], "Hello render\n  Nested reply");
    assert!(json["recomputed_state_hash"]
        .as_str()
        .is_some_and(|value| value.starts_with("hash:")));
    assert_eq!(
        json["rendered_blocks"]
            .as_array()
            .and_then(|blocks| blocks.first())
            .and_then(|block| block["content"].as_str()),
        Some("Hello render")
    );
}

#[test]
fn head_render_store_backed_reports_multi_hop_ancestry_context() {
    let doc_id = "doc:render-store-ancestry";
    let revision_author = signing_key(86);
    let maintainer = signing_key(97);
    let policy = json!({
        "accept_keys": [signer_id(&maintainer)],
        "merge_rule": "manual-reviewed",
        "preferred_branches": ["main"]
    });
    let genesis_hash = empty_document_state_hash(doc_id);
    let parent_revision = signed_revision_with_patches(
        &revision_author,
        doc_id,
        vec![],
        vec!["patch:missing-ancestor".to_string()],
        1000,
        &genesis_hash,
    );
    let revision_b = signed_revision(
        &revision_author,
        doc_id,
        vec![parent_revision["revision_id"]
            .as_str()
            .expect("parent revision id should exist")
            .to_string()],
        1010,
        &genesis_hash,
    );
    let view = signed_view(
        &maintainer,
        &policy,
        documents_value(doc_id, &revision_b["revision_id"]),
        1100,
    );
    let store_dir = create_temp_dir("head-render-store-ancestry-root");
    let objects_dir = store_dir.path().join("objects");
    let indexes_dir = store_dir.path().join("indexes");
    fs::create_dir_all(objects_dir.join("revision")).expect("revision store dir should exist");
    fs::create_dir_all(objects_dir.join("view")).expect("view store dir should exist");
    fs::create_dir_all(&indexes_dir).expect("index dir should exist");

    let write_store_object = |object_type: &str, object_id: &str, value: &Value| -> PathBuf {
        let (_, object_hash) = object_id
            .split_once(':')
            .expect("object id should contain a type prefix");
        let path = objects_dir
            .join(object_type)
            .join(format!("{object_hash}.json"));
        fs::write(
            &path,
            serde_json::to_string_pretty(value).expect("store object should serialize"),
        )
        .expect("store object should write");
        path
    };
    let parent_revision_id = parent_revision["revision_id"]
        .as_str()
        .expect("parent revision id should exist")
        .to_string();
    let revision_b_id = revision_b["revision_id"]
        .as_str()
        .expect("selected head id should exist")
        .to_string();
    let view_id = view["view_id"]
        .as_str()
        .expect("view id should exist")
        .to_string();
    write_store_object("revision", &parent_revision_id, &parent_revision);
    write_store_object("revision", &revision_b_id, &revision_b);
    write_store_object("view", &view_id, &view);
    fs::write(
        indexes_dir.join("manifest.json"),
        serde_json::to_string_pretty(&json!({
            "version": "mycel-store-index/0.1",
            "stored_object_count": 3,
            "object_ids_by_type": {
                "revision": [parent_revision_id.clone(), revision_b_id.clone()],
                "view": [view_id.clone()]
            },
            "doc_revisions": {
                doc_id: [parent_revision_id.clone(), revision_b_id.clone()]
            },
            "revision_parents": {
                revision_b_id.clone(): [parent_revision_id.clone()]
            },
            "author_patches": {},
            "view_governance": [
                {
                    "view_id": view_id,
                    "maintainer": signer_id(&maintainer),
                    "profile_id": hash_json(&policy),
                    "documents": {
                        doc_id: revision_b_id.clone()
                    }
                }
            ],
            "maintainer_views": {},
            "profile_views": {},
            "document_views": {},
            "profile_heads": {}
        }))
        .expect("manifest should serialize"),
    )
    .expect("manifest should write");
    let input = write_input_file(
        "head-render-store-ancestry",
        "input.json",
        json!({
            "profile": head_profile(hash_json(&policy), 1200),
            "revisions": [],
            "views": [],
            "critical_violations": []
        }),
    );

    let output = run_mycel(&[
        "head",
        "render",
        doc_id,
        "--input",
        &path_arg(&input.path),
        "--store-root",
        &path_arg(&store_dir.path().to_path_buf()),
        "--json",
    ]);

    assert_exit_code(&output, 1);
    let json = parse_json_stdout(&output);
    assert_eq!(json["status"], "failed");
    assert!(
        json["errors"]
            .as_array()
            .is_some_and(|errors| errors.iter().any(|entry| {
                entry.as_str().is_some_and(|message| {
                    message.contains("failed verification before render replay")
                        && message.contains(&format!(
                            "while verifying ancestry through parent revision '{parent_revision_id}'"
                        ))
                        && message.contains("missing patch 'patch:missing-ancestor' for replay")
                })
            })),
        "expected nested ancestry-context render error, stdout: {}",
        stdout_text(&output)
    );
}

#[test]
fn head_render_store_backed_applies_editor_admission_from_profile() {
    let doc_id = "doc:render-store-editor-admission";
    let admitted_author = signing_key(87);
    let non_admitted_author = signing_key(88);
    let policy = json!({
        "merge_rule": "manual-reviewed",
        "preferred_branches": ["main"]
    });
    let admitted_patch = signed_patch(
        &admitted_author,
        doc_id,
        "rev:genesis-null",
        1000,
        json!([
            {
                "op": "insert_block",
                "new_block": {
                    "block_id": "blk:render-admitted-001",
                    "block_type": "paragraph",
                    "content": "Admitted render line",
                    "attrs": {},
                    "children": []
                }
            }
        ]),
    );
    let admitted_revision = signed_revision_with_patches(
        &admitted_author,
        doc_id,
        vec![],
        vec![admitted_patch["patch_id"]
            .as_str()
            .expect("patch id should exist")
            .to_string()],
        1010,
        &document_state_hash(
            doc_id,
            vec![json!({
                "block_id": "blk:render-admitted-001",
                "block_type": "paragraph",
                "content": "Admitted render line",
                "attrs": {},
                "children": []
            })],
        ),
    );
    let non_admitted_patch = signed_patch(
        &non_admitted_author,
        doc_id,
        "rev:genesis-null",
        1020,
        json!([
            {
                "op": "insert_block",
                "new_block": {
                    "block_id": "blk:render-non-admitted-001",
                    "block_type": "paragraph",
                    "content": "Non-admitted render line",
                    "attrs": {},
                    "children": []
                }
            }
        ]),
    );
    let non_admitted_revision = signed_revision_with_patches(
        &non_admitted_author,
        doc_id,
        vec![],
        vec![non_admitted_patch["patch_id"]
            .as_str()
            .expect("patch id should exist")
            .to_string()],
        1030,
        &document_state_hash(
            doc_id,
            vec![json!({
                "block_id": "blk:render-non-admitted-001",
                "block_type": "paragraph",
                "content": "Non-admitted render line",
                "attrs": {},
                "children": []
            })],
        ),
    );
    let store_dir = build_store_from_objects(&[
        admitted_patch,
        admitted_revision.clone(),
        non_admitted_patch,
        non_admitted_revision.clone(),
    ]);
    let mut profile = head_profile(hash_json(&policy), 1200);
    profile["editor_admission"] = json!({
        "mode": "admitted-only",
        "admitted_keys": [signer_id(&admitted_author)]
    });
    let input = write_input_file(
        "head-render-store-editor-admission",
        "input.json",
        json!({
            "profile": profile,
            "revisions": [],
            "views": [],
            "critical_violations": []
        }),
    );

    let output = run_mycel(&[
        "head",
        "render",
        doc_id,
        "--input",
        &path_arg(&input.path),
        "--store-root",
        &path_arg(&store_dir.path().to_path_buf()),
        "--json",
    ]);

    assert_success(&output);
    let json = parse_json_stdout(&output);
    assert_eq!(json["selected_head"], admitted_revision["revision_id"]);
    assert_eq!(json["rendered_text"], "Admitted render line");
    assert!(
        json["notes"]
            .as_array()
            .is_some_and(|notes| notes.iter().any(|entry| entry
                .as_str()
                .is_some_and(|message| message.contains("store-backed replay")))),
        "expected store-backed render note, stdout: {}",
        stdout_text(&output)
    );
}

#[test]
fn head_render_store_backed_accepts_shared_dual_role_key_with_independent_admission() {
    let doc_id = "doc:render-store-shared-dual-role";
    let dual_role_author = signing_key(145);
    let other_author = signing_key(146);
    let policy = json!({
        "merge_rule": "manual-reviewed",
        "preferred_branches": ["main"]
    });
    let dual_patch = signed_patch(
        &dual_role_author,
        doc_id,
        "rev:genesis-null",
        1000,
        json!([
            {
                "op": "insert_block",
                "new_block": {
                    "block_id": "blk:render-dual-role-001",
                    "block_type": "paragraph",
                    "content": "Shared dual-role line",
                    "attrs": {},
                    "children": []
                }
            }
        ]),
    );
    let dual_revision = signed_revision_with_patches(
        &dual_role_author,
        doc_id,
        vec![],
        vec![dual_patch["patch_id"]
            .as_str()
            .expect("patch id should exist")
            .to_string()],
        1010,
        &document_state_hash(
            doc_id,
            vec![json!({
                "block_id": "blk:render-dual-role-001",
                "block_type": "paragraph",
                "content": "Shared dual-role line",
                "attrs": {},
                "children": []
            })],
        ),
    );
    let other_patch = signed_patch(
        &other_author,
        doc_id,
        "rev:genesis-null",
        1020,
        json!([
            {
                "op": "insert_block",
                "new_block": {
                    "block_id": "blk:render-other-role-001",
                    "block_type": "paragraph",
                    "content": "Other line",
                    "attrs": {},
                    "children": []
                }
            }
        ]),
    );
    let other_revision = signed_revision_with_patches(
        &other_author,
        doc_id,
        vec![],
        vec![other_patch["patch_id"]
            .as_str()
            .expect("patch id should exist")
            .to_string()],
        1030,
        &document_state_hash(
            doc_id,
            vec![json!({
                "block_id": "blk:render-other-role-001",
                "block_type": "paragraph",
                "content": "Other line",
                "attrs": {},
                "children": []
            })],
        ),
    );
    let dual_view = signed_view(
        &dual_role_author,
        &policy,
        documents_value(doc_id, &dual_revision["revision_id"]),
        1100,
    );
    let store_dir = build_store_from_objects(&[
        dual_patch,
        dual_revision.clone(),
        other_patch,
        other_revision,
        dual_view,
    ]);
    let mut profile = head_profile(hash_json(&policy), 1200);
    profile["editor_admission"] = json!({
        "mode": "admitted-only",
        "admitted_keys": [signer_id(&dual_role_author)]
    });
    profile["view_admission"] = json!({
        "mode": "admitted-only",
        "admitted_keys": [signer_id(&dual_role_author)]
    });
    let input = write_input_file(
        "head-render-store-shared-dual-role",
        "input.json",
        json!({
            "profile": profile,
            "revisions": [],
            "views": [],
            "critical_violations": []
        }),
    );

    let output = run_mycel(&[
        "head",
        "render",
        doc_id,
        "--input",
        &path_arg(&input.path),
        "--store-root",
        &path_arg(&store_dir.path().to_path_buf()),
        "--json",
    ]);

    assert_success(&output);
    let json = parse_json_stdout(&output);
    assert_eq!(json["selected_head"], dual_revision["revision_id"]);
    assert_eq!(json["rendered_text"], "Shared dual-role line");
}

#[test]
fn head_render_text_reports_rendered_text() {
    let doc_id = "doc:render-text";
    let revision_author = signing_key(82);
    let maintainer = signing_key(93);
    let policy = json!({
        "accept_keys": [signer_id(&maintainer)],
        "merge_rule": "manual-reviewed",
        "preferred_branches": ["main"]
    });
    let genesis_hash = empty_document_state_hash(doc_id);
    let revision_a = signed_revision(&revision_author, doc_id, vec![], 1000, &genesis_hash);
    let patch = signed_patch(
        &revision_author,
        doc_id,
        revision_a["revision_id"]
            .as_str()
            .expect("revision id should exist"),
        1010,
        json!([
            {
                "op": "insert_block",
                "new_block": {
                    "block_id": "blk:render-text-001",
                    "block_type": "paragraph",
                    "content": "Rendered line",
                    "attrs": {},
                    "children": []
                }
            }
        ]),
    );
    let revision_b = signed_revision_with_patches(
        &revision_author,
        doc_id,
        vec![revision_a["revision_id"]
            .as_str()
            .expect("revision id should exist")
            .to_string()],
        vec![patch["patch_id"]
            .as_str()
            .expect("patch id should exist")
            .to_string()],
        1020,
        &document_state_hash(
            doc_id,
            vec![json!({
                "block_id": "blk:render-text-001",
                "block_type": "paragraph",
                "content": "Rendered line",
                "attrs": {},
                "children": []
            })],
        ),
    );
    let view = signed_view(
        &maintainer,
        &policy,
        documents_value(doc_id, &revision_b["revision_id"]),
        1100,
    );
    let store_dir = build_store_from_objects(&[revision_a, patch, revision_b, view]);
    let input = write_input_file(
        "head-render-store-backed-text",
        "input.json",
        json!({
            "profile": head_profile(hash_json(&policy), 1200),
            "revisions": [],
            "views": [],
            "critical_violations": []
        }),
    );

    let output = run_mycel(&[
        "head",
        "render",
        doc_id,
        "--input",
        &path_arg(&input.path),
        "--store-root",
        &path_arg(&store_dir.path().to_path_buf()),
    ]);

    assert_success(&output);
    assert_stdout_contains(&output, "Head render: ok");
    assert_stdout_contains(&output, "Rendered Text");
    assert_stdout_contains(&output, "Rendered line");
}

#[test]
fn head_render_json_replays_selected_head_from_bundle_objects() {
    let doc_id = "doc:render-bundle";
    let revision_author = signing_key(83);
    let maintainer = signing_key(94);
    let policy = json!({
        "accept_keys": [signer_id(&maintainer)],
        "merge_rule": "manual-reviewed",
        "preferred_branches": ["main"]
    });
    let genesis_hash = empty_document_state_hash(doc_id);
    let revision_a = signed_revision(&revision_author, doc_id, vec![], 1000, &genesis_hash);
    let patch = signed_patch(
        &revision_author,
        doc_id,
        revision_a["revision_id"]
            .as_str()
            .expect("revision id should exist"),
        1010,
        json!([
            {
                "op": "insert_block",
                "new_block": {
                    "block_id": "blk:render-bundle-001",
                    "block_type": "paragraph",
                    "content": "Bundle line",
                    "attrs": {},
                    "children": []
                }
            }
        ]),
    );
    let revision_b = signed_revision_with_patches(
        &revision_author,
        doc_id,
        vec![revision_a["revision_id"]
            .as_str()
            .expect("revision id should exist")
            .to_string()],
        vec![patch["patch_id"]
            .as_str()
            .expect("patch id should exist")
            .to_string()],
        1020,
        &document_state_hash(
            doc_id,
            vec![json!({
                "block_id": "blk:render-bundle-001",
                "block_type": "paragraph",
                "content": "Bundle line",
                "attrs": {},
                "children": []
            })],
        ),
    );
    let view = signed_view(
        &maintainer,
        &policy,
        documents_value(doc_id, &revision_b["revision_id"]),
        1100,
    );
    let input = write_input_file(
        "head-render-bundle-backed",
        "input.json",
        json!({
            "profile": head_profile(hash_json(&policy), 1200),
            "revisions": [revision_a, revision_b.clone()],
            "objects": [patch],
            "views": [view],
            "critical_violations": []
        }),
    );

    let output = run_mycel(&[
        "head",
        "render",
        doc_id,
        "--input",
        &path_arg(&input.path),
        "--json",
    ]);

    assert_success(&output);
    let json = parse_json_stdout(&output);
    assert_eq!(json["status"], "ok");
    assert_eq!(json["selected_head"], revision_b["revision_id"]);
    assert_eq!(json["rendered_text"], "Bundle line");
    assert_eq!(json["store_root"], Value::Null);
    assert!(
        json["notes"]
            .as_array()
            .is_some_and(|notes| notes.iter().any(|entry| entry
                .as_str()
                .is_some_and(|message| message.contains("bundle-backed replay objects")))),
        "expected bundle-backed render note, stdout: {}",
        stdout_text(&output)
    );
}

#[test]
fn head_render_json_uses_requested_named_profile_from_bundle() {
    let doc_id = "doc:render-named-profile";
    let revision_author = signing_key(85);
    let maintainer = signing_key(96);
    let policy = json!({
        "accept_keys": [signer_id(&maintainer)],
        "merge_rule": "manual-reviewed",
        "preferred_branches": ["main"]
    });
    let genesis_hash = empty_document_state_hash(doc_id);
    let revision_a = signed_revision(&revision_author, doc_id, vec![], 1000, &genesis_hash);
    let patch = signed_patch(
        &revision_author,
        doc_id,
        revision_a["revision_id"]
            .as_str()
            .expect("revision id should exist"),
        1010,
        json!([
            {
                "op": "insert_block",
                "new_block": {
                    "block_id": "blk:render-profile-001",
                    "block_type": "paragraph",
                    "content": "Preview line",
                    "attrs": {},
                    "children": []
                }
            }
        ]),
    );
    let revision_b = signed_revision_with_patches(
        &revision_author,
        doc_id,
        vec![revision_a["revision_id"]
            .as_str()
            .expect("revision id should exist")
            .to_string()],
        vec![patch["patch_id"]
            .as_str()
            .expect("patch id should exist")
            .to_string()],
        1020,
        &document_state_hash(
            doc_id,
            vec![json!({
                "block_id": "blk:render-profile-001",
                "block_type": "paragraph",
                "content": "Preview line",
                "attrs": {},
                "children": []
            })],
        ),
    );
    let bundle = json!({
        "profiles": named_profiles(&[
            ("stable", head_profile(hash_json(&policy), 1005)),
            ("preview", head_profile(hash_json(&policy), 1200))
        ]),
        "revisions": [revision_a.clone(), revision_b.clone()],
        "objects": [patch],
        "views": [
            signed_view(
                &maintainer,
                &policy,
                documents_value(doc_id, &revision_a["revision_id"]),
                1002
            ),
            signed_view(
                &maintainer,
                &policy,
                documents_value(doc_id, &revision_b["revision_id"]),
                1100
            )
        ],
        "critical_violations": []
    });
    let input = write_input_file("head-render-named-profile", "input.json", bundle);
    let output = run_mycel(&[
        "head",
        "render",
        doc_id,
        "--input",
        &path_arg(&input.path),
        "--profile-id",
        "preview",
        "--json",
    ]);

    assert_success(&output);
    let json = parse_json_stdout(&output);
    assert_eq!(json["available_profile_ids"], json!(["preview", "stable"]));
    assert_eq!(json["profile_id"], "preview");
    assert_eq!(json["selected_head"], revision_b["revision_id"]);
    assert_eq!(json["rendered_text"], "Preview line");
}

#[test]
fn head_render_requires_profile_id_for_multi_profile_bundle() {
    let doc_id = "doc:render-multi-profile";
    let revision_author = signing_key(86);
    let maintainer = signing_key(97);
    let policy = json!({
        "accept_keys": [signer_id(&maintainer)],
        "merge_rule": "manual-reviewed",
        "preferred_branches": ["main"]
    });
    let genesis_hash = empty_document_state_hash(doc_id);
    let revision_a = signed_revision(&revision_author, doc_id, vec![], 1000, &genesis_hash);
    let revision_b = signed_revision(&revision_author, doc_id, vec![], 1020, &genesis_hash);
    let bundle = json!({
        "profiles": named_profiles(&[
            ("stable", head_profile(hash_json(&policy), 1005)),
            ("preview", head_profile(hash_json(&policy), 1200))
        ]),
        "revisions": [revision_a, revision_b],
        "objects": [],
        "views": [],
        "critical_violations": []
    });
    let input = write_input_file("head-render-multi-profile", "input.json", bundle);
    let output = run_mycel(&["head", "render", doc_id, "--input", &path_arg(&input.path)]);

    assert_exit_code(&output, 1);
    assert_stdout_contains(&output, "Head render: failed");
    assert_stdout_contains(&output, "Document");
    assert_stderr_contains(
        &output,
        "head input declares multiple named profiles; pass --profile-id (preview, stable)",
    );
}

#[test]
fn head_render_reports_unknown_profile_id_for_multi_profile_bundle() {
    let doc_id = "doc:render-unknown-profile";
    let revision_author = signing_key(87);
    let maintainer = signing_key(98);
    let policy = json!({
        "accept_keys": [signer_id(&maintainer)],
        "merge_rule": "manual-reviewed",
        "preferred_branches": ["main"]
    });
    let genesis_hash = empty_document_state_hash(doc_id);
    let revision = signed_revision(&revision_author, doc_id, vec![], 1000, &genesis_hash);
    let bundle = json!({
        "profiles": named_profiles(&[
            ("stable", head_profile(hash_json(&policy), 1005)),
            ("preview", head_profile(hash_json(&policy), 1200))
        ]),
        "revisions": [revision],
        "objects": [],
        "views": [],
        "critical_violations": []
    });
    let input = write_input_file("head-render-unknown-profile", "input.json", bundle);
    let output = run_mycel(&[
        "head",
        "render",
        doc_id,
        "--input",
        &path_arg(&input.path),
        "--profile-id",
        "missing",
    ]);

    assert_exit_code(&output, 1);
    assert_stdout_contains(&output, "Head render: failed");
    assert_stdout_contains(&output, "Document");
    assert_stderr_contains(
        &output,
        "unknown --profile-id 'missing' for head input; available profiles: preview, stable",
    );
}

#[test]
fn head_render_named_profile_applies_requested_editor_admission_mode() {
    let doc_id = "doc:render-editor-profile";
    let admitted_author = signing_key(89);
    let non_admitted_author = signing_key(90);
    let policy = json!({
        "merge_rule": "manual-reviewed",
        "preferred_branches": ["main"]
    });
    let admitted_patch = signed_patch(
        &admitted_author,
        doc_id,
        "rev:genesis-null",
        1000,
        json!([
            {
                "op": "insert_block",
                "new_block": {
                    "block_id": "blk:render-profile-admitted-001",
                    "block_type": "paragraph",
                    "content": "Named admitted line",
                    "attrs": {},
                    "children": []
                }
            }
        ]),
    );
    let admitted_revision = signed_revision_with_patches(
        &admitted_author,
        doc_id,
        vec![],
        vec![admitted_patch["patch_id"]
            .as_str()
            .expect("patch id should exist")
            .to_string()],
        1010,
        &document_state_hash(
            doc_id,
            vec![json!({
                "block_id": "blk:render-profile-admitted-001",
                "block_type": "paragraph",
                "content": "Named admitted line",
                "attrs": {},
                "children": []
            })],
        ),
    );
    let non_admitted_patch = signed_patch(
        &non_admitted_author,
        doc_id,
        "rev:genesis-null",
        1020,
        json!([
            {
                "op": "insert_block",
                "new_block": {
                    "block_id": "blk:render-profile-non-admitted-001",
                    "block_type": "paragraph",
                    "content": "Named non-admitted line",
                    "attrs": {},
                    "children": []
                }
            }
        ]),
    );
    let non_admitted_revision = signed_revision_with_patches(
        &non_admitted_author,
        doc_id,
        vec![],
        vec![non_admitted_patch["patch_id"]
            .as_str()
            .expect("patch id should exist")
            .to_string()],
        1030,
        &document_state_hash(
            doc_id,
            vec![json!({
                "block_id": "blk:render-profile-non-admitted-001",
                "block_type": "paragraph",
                "content": "Named non-admitted line",
                "attrs": {},
                "children": []
            })],
        ),
    );
    let stable = head_profile(hash_json(&policy), 1200);
    let mut preview = head_profile(hash_json(&policy), 1200);
    preview["editor_admission"] = json!({
        "mode": "admitted-only",
        "admitted_keys": [signer_id(&admitted_author)]
    });
    let bundle = json!({
        "profiles": named_profiles(&[
            ("stable", stable),
            ("preview", preview)
        ]),
        "revisions": [admitted_revision.clone(), non_admitted_revision],
        "objects": [admitted_patch, non_admitted_patch],
        "views": [],
        "critical_violations": []
    });
    let input = write_input_file("head-render-editor-profile", "input.json", bundle);
    let output = run_mycel(&[
        "head",
        "render",
        doc_id,
        "--input",
        &path_arg(&input.path),
        "--profile-id",
        "preview",
        "--json",
    ]);

    assert_success(&output);
    let json = parse_json_stdout(&output);
    assert_eq!(json["profile_id"], "preview");
    assert_eq!(json["selected_head"], admitted_revision["revision_id"]);
    assert_eq!(json["rendered_text"], "Named admitted line");
}

#[test]
fn head_render_bundle_reports_missing_replay_objects() {
    let doc_id = "doc:render-missing-objects";
    let revision_author = signing_key(84);
    let maintainer = signing_key(95);
    let policy = json!({
        "accept_keys": [signer_id(&maintainer)],
        "merge_rule": "manual-reviewed",
        "preferred_branches": ["main"]
    });
    let genesis_hash = empty_document_state_hash(doc_id);
    let revision_a = signed_revision(&revision_author, doc_id, vec![], 1000, &genesis_hash);
    let patch = signed_patch(
        &revision_author,
        doc_id,
        revision_a["revision_id"]
            .as_str()
            .expect("revision id should exist"),
        1010,
        json!([
            {
                "op": "insert_block",
                "new_block": {
                    "block_id": "blk:render-missing-001",
                    "block_type": "paragraph",
                    "content": "Missing patch payload",
                    "attrs": {},
                    "children": []
                }
            }
        ]),
    );
    let revision_b = signed_revision_with_patches(
        &revision_author,
        doc_id,
        vec![revision_a["revision_id"]
            .as_str()
            .expect("revision id should exist")
            .to_string()],
        vec![patch["patch_id"]
            .as_str()
            .expect("patch id should exist")
            .to_string()],
        1020,
        &document_state_hash(
            doc_id,
            vec![json!({
                "block_id": "blk:render-missing-001",
                "block_type": "paragraph",
                "content": "Missing patch payload",
                "attrs": {},
                "children": []
            })],
        ),
    );
    let view = signed_view(
        &maintainer,
        &policy,
        documents_value(doc_id, &revision_b["revision_id"]),
        1100,
    );
    let input = write_input_file(
        "head-render-bundle-missing-object",
        "input.json",
        json!({
            "profile": head_profile(hash_json(&policy), 1200),
            "revisions": [revision_a, revision_b],
            "views": [view],
            "critical_violations": []
        }),
    );

    let output = run_mycel(&[
        "head",
        "render",
        doc_id,
        "--input",
        &path_arg(&input.path),
        "--json",
    ]);

    assert_exit_code(&output, 1);
    let json = parse_json_stdout(&output);
    assert_eq!(json["status"], "failed");
    assert!(
        json["errors"]
            .as_array()
            .is_some_and(|errors| errors.iter().any(|entry| entry
                .as_str()
                .is_some_and(|message| message.contains("missing patch")))),
        "expected missing patch error, stdout: {}",
        stdout_text(&output)
    );
}

#[test]
fn head_render_bundle_reports_multi_hop_ancestry_context() {
    let doc_id = "doc:render-bundle-ancestry";
    let revision_author = signing_key(85);
    let maintainer = signing_key(96);
    let policy = json!({
        "accept_keys": [signer_id(&maintainer)],
        "merge_rule": "manual-reviewed",
        "preferred_branches": ["main"]
    });
    let genesis_hash = empty_document_state_hash(doc_id);
    let parent_revision = signed_revision_with_patches(
        &revision_author,
        doc_id,
        vec![],
        vec!["patch:missing-ancestor".to_string()],
        1000,
        &genesis_hash,
    );
    let revision_b = signed_revision(
        &revision_author,
        doc_id,
        vec![parent_revision["revision_id"]
            .as_str()
            .expect("parent revision id should exist")
            .to_string()],
        1010,
        &genesis_hash,
    );
    let view = signed_view(
        &maintainer,
        &policy,
        documents_value(doc_id, &revision_b["revision_id"]),
        1100,
    );
    let input = write_input_file(
        "head-render-bundle-ancestry",
        "input.json",
        json!({
            "profile": head_profile(hash_json(&policy), 1200),
            "revisions": [parent_revision.clone(), revision_b],
            "views": [view],
            "critical_violations": []
        }),
    );

    let output = run_mycel(&[
        "head",
        "render",
        doc_id,
        "--input",
        &path_arg(&input.path),
        "--json",
    ]);

    assert_exit_code(&output, 1);
    let json = parse_json_stdout(&output);
    assert_eq!(json["status"], "failed");
    let parent_revision_id = parent_revision["revision_id"]
        .as_str()
        .expect("parent revision id should exist");
    assert!(
        json["errors"]
            .as_array()
            .is_some_and(|errors| errors.iter().any(|entry| {
                entry.as_str().is_some_and(|message| {
                    message.contains("failed verification before render replay")
                        && message.contains(&format!(
                            "while verifying ancestry through parent revision '{parent_revision_id}'"
                        ))
                        && message.contains("missing patch 'patch:missing-ancestor' for replay")
                })
            })),
        "expected nested ancestry-context render error, stdout: {}",
        stdout_text(&output)
    );
}
