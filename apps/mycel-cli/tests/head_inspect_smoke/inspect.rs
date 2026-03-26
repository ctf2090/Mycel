use super::*;
use insta::assert_json_snapshot;

#[test]
fn head_inspect_json_selects_highest_supported_head() {
    let doc_id = "doc:sample";
    let path = repo_root()
        .join("fixtures/head-inspect/minimal-head-selection")
        .to_string_lossy()
        .into_owned();
    let output = run_mycel(&["head", "inspect", doc_id, "--input", &path, "--json"]);

    assert_success(&output);
    let json = parse_json_stdout(&output);
    assert_json_snapshot!(
        "head_inspect_json_selects_highest_supported_head",
        json,
        {
            ".input_path" => "[input_path]",
        }
    );
}

#[test]
fn head_inspect_json_resolves_repo_native_fixture_name() {
    let output = run_mycel(&[
        "head",
        "inspect",
        "doc:sample",
        "--input",
        "minimal-head-selection",
        "--json",
    ]);

    assert_success(&output);
    let json = parse_json_stdout(&output);
    assert_json_snapshot!(
        "head_inspect_json_resolves_repo_native_fixture_name",
        json,
        {
            ".input_path" => "[input_path]",
        }
    );
}

#[test]
fn head_inspect_json_applies_fixture_backed_viewer_score_channels() {
    let output = run_mycel(&[
        "head",
        "inspect",
        "doc:sample",
        "--input",
        "viewer-score-channels",
        "--json",
    ]);

    assert_success(&output);
    let json = parse_json_stdout(&output);
    assert_json_snapshot!(
        "head_inspect_json_applies_fixture_backed_viewer_score_channels",
        json,
        {
            ".input_path" => "[input_path]",
        }
    );
}

#[test]
fn head_inspect_viewer_score_fixture_is_deterministic_across_repeated_runs() {
    let first = run_mycel(&[
        "head",
        "inspect",
        "doc:sample",
        "--input",
        "viewer-score-channels",
        "--json",
    ]);
    let second = run_mycel(&[
        "head",
        "inspect",
        "doc:sample",
        "--input",
        "viewer-score-channels",
        "--json",
    ]);

    assert_success(&first);
    assert_success(&second);
    assert_eq!(parse_json_stdout(&first), parse_json_stdout(&second));
}

#[test]
fn head_inspect_requires_profile_id_for_multi_profile_bundle() {
    let doc_id = "doc:multi-profile";
    let revision_author = signing_key(62);
    let maintainer = signing_key(74);
    let policy = json!({
        "accept_keys": [signer_id(&maintainer)],
        "merge_rule": "manual-reviewed",
        "preferred_branches": ["main"]
    });
    let revision_a = signed_revision(
        &revision_author,
        doc_id,
        vec![],
        10,
        &empty_document_state_hash(doc_id),
    );
    let revision_b = signed_revision(
        &revision_author,
        doc_id,
        vec![revision_a["revision_id"]
            .as_str()
            .expect("revision id should exist")
            .to_string()],
        20,
        &empty_document_state_hash(doc_id),
    );
    let bundle = json!({
        "profiles": named_profiles(&[
            ("stable", head_profile(hash_json(&policy), 18)),
            ("preview", head_profile(hash_json(&policy), 30))
        ]),
        "revisions": [revision_a, revision_b.clone()],
        "views": [
            signed_view(
                &maintainer,
                &policy,
                documents_value(doc_id, &revision_b["revision_id"]),
                25
            )
        ],
        "critical_violations": []
    });
    let input = write_input_file("head-inspect-multi-profile", "input.json", bundle);
    let output = run_mycel(&["head", "inspect", doc_id, "--input", &path_arg(&input.path)]);

    assert_exit_code(&output, 1);
    assert_stdout_contains(&output, "Head inspection: failed");
    assert_stdout_contains(&output, "Document");
    assert_stdout_contains(&output, "- available profiles: preview, stable");
    assert_stdout_contains(
        &output,
        "- retry with one of: --profile-id preview | --profile-id stable",
    );
    assert_stdout_contains(&output, "Decision");
    assert_stderr_contains(
        &output,
        "head input declares multiple named profiles; pass --profile-id (preview, stable)",
    );
}

#[test]
fn head_inspect_reports_unknown_profile_id_for_multi_profile_bundle() {
    let doc_id = "doc:inspect-unknown-profile";
    let revision_author = signing_key(63);
    let maintainer = signing_key(76);
    let policy = json!({
        "accept_keys": [signer_id(&maintainer)],
        "merge_rule": "manual-reviewed",
        "preferred_branches": ["main"]
    });
    let state_hash = empty_document_state_hash(doc_id);
    let revision = signed_revision(&revision_author, doc_id, vec![], 10, &state_hash);
    let bundle = json!({
        "profiles": named_profiles(&[
            ("stable", head_profile(hash_json(&policy), 18)),
            ("preview", head_profile(hash_json(&policy), 30))
        ]),
        "revisions": [revision],
        "views": [],
        "critical_violations": []
    });
    let input = write_input_file("head-inspect-unknown-profile", "input.json", bundle);
    let output = run_mycel(&[
        "head",
        "inspect",
        doc_id,
        "--input",
        &path_arg(&input.path),
        "--profile-id",
        "missing",
    ]);

    assert_exit_code(&output, 1);
    assert_stdout_contains(&output, "Head inspection: failed");
    assert_stdout_contains(&output, "Document");
    assert_stdout_contains(&output, "- available profiles: preview, stable");
    assert_stdout_contains(
        &output,
        "- retry with one of: --profile-id preview | --profile-id stable",
    );
    assert_stderr_contains(
        &output,
        "unknown --profile-id 'missing' for head input; available profiles: preview, stable",
    );
}

#[test]
fn head_inspect_json_can_source_objects_from_store_index() {
    let doc_id = "doc:sample";
    let revision_author = signing_key(61);
    let maintainer_a = signing_key(71);
    let maintainer_b = signing_key(72);
    let maintainer_c = signing_key(73);
    let policy = json!({
        "accept_keys": [
            signer_id(&maintainer_a),
            signer_id(&maintainer_b),
            signer_id(&maintainer_c)
        ],
        "merge_rule": "manual-reviewed",
        "preferred_branches": ["main"]
    });
    let state_hash = empty_document_state_hash(doc_id);
    let revision_a = signed_revision(&revision_author, doc_id, vec![], 1000, &state_hash);
    let revision_b = signed_revision(
        &revision_author,
        doc_id,
        vec![revision_a["revision_id"]
            .as_str()
            .expect("revision id should exist")
            .to_string()],
        1010,
        &state_hash,
    );
    let revision_c = signed_revision(
        &revision_author,
        doc_id,
        vec![revision_a["revision_id"]
            .as_str()
            .expect("revision id should exist")
            .to_string()],
        1020,
        &state_hash,
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
        documents_value(doc_id, &revision_c["revision_id"]),
        1110,
    );
    let view_c = signed_view(
        &maintainer_c,
        &policy,
        documents_value(doc_id, &revision_b["revision_id"]),
        1120,
    );
    let store_dir = build_store_from_objects(&[
        revision_a.clone(),
        revision_b.clone(),
        revision_c.clone(),
        view_a,
        view_b,
        view_c,
    ]);
    let input = write_input_file(
        "head-inspect-store-backed",
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
        "inspect",
        doc_id,
        "--input",
        &path_arg(&input.path),
        "--store-root",
        &path_arg(store_dir.path()),
        "--json",
    ]);

    assert_success(&output);
    let json = parse_json_stdout(&output);
    assert_json_snapshot!(
        "head_inspect_json_can_source_objects_from_store_index",
        json,
        {
            ".input_path" => "[input_path]",
            ".notes[0]" => "[store_selector_note]",
            ".store_root" => "[store_root]",
        }
    );
}

#[test]
fn head_inspect_store_backed_falls_back_to_view_governance_when_profile_views_missing() {
    let doc_id = "doc:legacy-store-index";
    let revision_author = signing_key(161);
    let maintainer_a = signing_key(171);
    let maintainer_b = signing_key(172);
    let policy = json!({
        "accept_keys": [
            signer_id(&maintainer_a),
            signer_id(&maintainer_b)
        ],
        "merge_rule": "manual-reviewed",
        "preferred_branches": ["main"]
    });
    let state_hash = empty_document_state_hash(doc_id);
    let revision_a = signed_revision(&revision_author, doc_id, vec![], 1000, &state_hash);
    let revision_b = signed_revision(
        &revision_author,
        doc_id,
        vec![revision_a["revision_id"]
            .as_str()
            .expect("revision id should exist")
            .to_string()],
        1010,
        &state_hash,
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
        build_store_from_objects(&[revision_a.clone(), revision_b.clone(), view_a, view_b]);
    let manifest_path = store_dir.path().join("indexes").join("manifest.json");
    let mut manifest: serde_json::Value = serde_json::from_str(
        &fs::read_to_string(&manifest_path).expect("manifest should be readable"),
    )
    .expect("manifest should parse");
    manifest
        .as_object_mut()
        .expect("manifest should be object")
        .remove("profile_views");
    fs::write(
        &manifest_path,
        serde_json::to_string_pretty(&manifest).expect("manifest should serialize"),
    )
    .expect("manifest should write");

    let input = write_input_file(
        "head-inspect-legacy-store-index",
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
        "inspect",
        doc_id,
        "--input",
        &path_arg(&input.path),
        "--store-root",
        &path_arg(store_dir.path()),
        "--json",
    ]);

    assert_success(&output);
    let json = parse_json_stdout(&output);
    assert_eq!(json["selected_head"], revision_b["revision_id"]);
    assert_eq!(json["verified_revision_count"], 2);
    assert_eq!(json["verified_view_count"], 2);
}

#[test]
fn head_inspect_store_backed_applies_editor_admission_from_profile() {
    let doc_id = "doc:store-backed-editor-admission";
    let admitted_author = signing_key(64);
    let non_admitted_author = signing_key(65);
    let policy = json!({
        "merge_rule": "manual-reviewed",
        "preferred_branches": ["main"]
    });
    let state_hash = empty_document_state_hash(doc_id);
    let admitted_revision = signed_revision(&admitted_author, doc_id, vec![], 1000, &state_hash);
    let non_admitted_revision =
        signed_revision(&non_admitted_author, doc_id, vec![], 1010, &state_hash);
    let store_dir =
        build_store_from_objects(&[admitted_revision.clone(), non_admitted_revision.clone()]);
    let mut profile = head_profile(hash_json(&policy), 1200);
    profile["editor_admission"] = json!({
        "mode": "admitted-only",
        "admitted_keys": [signer_id(&admitted_author)]
    });
    let input = write_input_file(
        "head-inspect-store-backed-editor-admission",
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
        "inspect",
        doc_id,
        "--input",
        &path_arg(&input.path),
        "--store-root",
        &path_arg(store_dir.path()),
        "--json",
    ]);

    assert_success(&output);
    let json = parse_json_stdout(&output);
    assert_json_snapshot!(
        "head_inspect_store_backed_applies_editor_admission_from_profile",
        json,
        {
            ".input_path" => "[input_path]",
            ".notes[0]" => "[store_selector_note]",
            ".store_root" => "[store_root]",
        }
    );
}

#[test]
fn head_inspect_store_backed_accepts_shared_dual_role_key_with_independent_admission() {
    let doc_id = "doc:store-backed-shared-dual-role";
    let dual_role_author = signing_key(141);
    let other_author = signing_key(142);
    let policy = json!({
        "merge_rule": "manual-reviewed",
        "preferred_branches": ["main"]
    });
    let state_hash = empty_document_state_hash(doc_id);
    let dual_revision = signed_revision(&dual_role_author, doc_id, vec![], 1000, &state_hash);
    let other_revision = signed_revision(&other_author, doc_id, vec![], 1010, &state_hash);
    let dual_view = signed_view(
        &dual_role_author,
        &policy,
        documents_value(doc_id, &dual_revision["revision_id"]),
        1100,
    );
    let store_dir =
        build_store_from_objects(&[dual_revision.clone(), other_revision.clone(), dual_view]);
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
        "head-inspect-store-backed-shared-dual-role",
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
        "inspect",
        doc_id,
        "--input",
        &path_arg(&input.path),
        "--store-root",
        &path_arg(store_dir.path()),
        "--json",
    ]);

    assert_success(&output);
    let json = parse_json_stdout(&output);
    assert_json_snapshot!(
        "head_inspect_store_backed_accepts_shared_dual_role_key_with_independent_admission",
        json,
        {
            ".input_path" => "[input_path]",
            ".notes[0]" => "[store_selector_note]",
            ".store_root" => "[store_root]",
        }
    );
}

#[test]
fn head_inspect_json_selects_requested_named_profile() {
    let doc_id = "doc:selected-profile";
    let revision_author = signing_key(63);
    let maintainer = signing_key(75);
    let policy = json!({
        "accept_keys": [signer_id(&maintainer)],
        "merge_rule": "manual-reviewed",
        "preferred_branches": ["main"]
    });
    let state_hash = empty_document_state_hash(doc_id);
    let revision_a = signed_revision(&revision_author, doc_id, vec![], 10, &state_hash);
    let revision_b = signed_revision(
        &revision_author,
        doc_id,
        vec![revision_a["revision_id"]
            .as_str()
            .expect("revision id should exist")
            .to_string()],
        20,
        &state_hash,
    );
    let bundle = json!({
        "profiles": named_profiles(&[
            ("stable", head_profile(hash_json(&policy), 18)),
            ("preview", head_profile(hash_json(&policy), 30))
        ]),
        "revisions": [revision_a.clone(), revision_b.clone()],
        "views": [
            signed_view(
                &maintainer,
                &policy,
                documents_value(doc_id, &revision_a["revision_id"]),
                15
            ),
            signed_view(
                &maintainer,
                &policy,
                documents_value(doc_id, &revision_b["revision_id"]),
                25
            )
        ],
        "critical_violations": []
    });
    let input = write_input_file("head-inspect-selected-profile", "input.json", bundle);
    let output = run_mycel(&[
        "head",
        "inspect",
        doc_id,
        "--input",
        &path_arg(&input.path),
        "--profile-id",
        "preview",
        "--json",
    ]);

    assert_success(&output);
    let json = parse_json_stdout(&output);
    assert_json_snapshot!(
        "head_inspect_json_selects_requested_named_profile",
        json,
        {
            ".input_path" => "[input_path]",
        }
    );
}

#[test]
fn head_inspect_named_profile_applies_requested_editor_admission_mode() {
    let doc_id = "doc:selected-editor-profile";
    let admitted_author = signing_key(66);
    let non_admitted_author = signing_key(67);
    let policy = json!({
        "merge_rule": "manual-reviewed",
        "preferred_branches": ["main"]
    });
    let state_hash = empty_document_state_hash(doc_id);
    let admitted_revision = signed_revision(&admitted_author, doc_id, vec![], 10, &state_hash);
    let non_admitted_revision =
        signed_revision(&non_admitted_author, doc_id, vec![], 20, &state_hash);
    let stable = head_profile(hash_json(&policy), 30);
    let mut preview = head_profile(hash_json(&policy), 30);
    preview["editor_admission"] = json!({
        "mode": "admitted-only",
        "admitted_keys": [signer_id(&admitted_author)]
    });
    let bundle = json!({
        "profiles": named_profiles(&[
            ("stable", stable),
            ("preview", preview)
        ]),
        "revisions": [admitted_revision.clone(), non_admitted_revision.clone()],
        "views": [],
        "critical_violations": []
    });
    let input = write_input_file("head-inspect-selected-editor-profile", "input.json", bundle);
    let output = run_mycel(&[
        "head",
        "inspect",
        doc_id,
        "--input",
        &path_arg(&input.path),
        "--profile-id",
        "preview",
        "--json",
    ]);

    assert_success(&output);
    let json = parse_json_stdout(&output);
    assert_json_snapshot!(
        "head_inspect_named_profile_applies_requested_editor_admission_mode",
        json,
        {
            ".input_path" => "[input_path]",
        }
    );
}

#[test]
fn head_inspect_named_profile_separates_editor_and_view_admission() {
    let doc_id = "doc:named-dual-role-separation";
    let editor_only_author = signing_key(143);
    let view_only_author = signing_key(144);
    let policy = json!({
        "merge_rule": "manual-reviewed",
        "preferred_branches": ["main"]
    });
    let editor_revision = signed_revision(
        &editor_only_author,
        doc_id,
        vec![],
        10,
        "hash:named-dual-role-editor",
    );
    let view_revision = signed_revision(
        &view_only_author,
        doc_id,
        vec![],
        20,
        "hash:named-dual-role-view",
    );
    let stable = head_profile(hash_json(&policy), 30);
    let mut preview = head_profile(hash_json(&policy), 30);
    preview["editor_admission"] = json!({
        "mode": "mixed",
        "admitted_keys": [signer_id(&editor_only_author)]
    });
    preview["view_admission"] = json!({
        "mode": "admitted-only",
        "admitted_keys": [signer_id(&view_only_author)]
    });
    let bundle = json!({
        "profiles": named_profiles(&[
            ("stable", stable),
            ("preview", preview)
        ]),
        "revisions": [editor_revision.clone(), view_revision.clone()],
        "views": [
            signed_view(
                &editor_only_author,
                &policy,
                documents_value(doc_id, &editor_revision["revision_id"]),
                25
            ),
            signed_view(
                &view_only_author,
                &policy,
                documents_value(doc_id, &view_revision["revision_id"]),
                26
            )
        ],
        "critical_violations": []
    });
    let input = write_input_file(
        "head-inspect-named-dual-role-separation",
        "input.json",
        bundle,
    );
    let output = run_mycel(&[
        "head",
        "inspect",
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
    assert_eq!(json["selected_head"], view_revision["revision_id"]);
    assert!(
        json["editor_candidates"]
            .as_array()
            .is_some_and(|entries| entries.iter().any(|entry| {
                entry["revision_id"] == editor_revision["revision_id"]
                    && entry["editor_admitted"] == Value::Bool(true)
                    && entry["formal_candidate"] == Value::Bool(true)
            })),
        "expected editor-only author to remain a formal editor candidate, stdout: {}",
        stdout_text(&output)
    );
    assert!(
        json["effective_weights"]
            .as_array()
            .is_some_and(|weights| weights.iter().any(|entry| {
                entry["maintainer"] == Value::String(signer_id(&editor_only_author))
                    && entry["view_admitted"] == Value::Bool(false)
                    && entry["admitted"] == Value::Bool(false)
                    && entry["effective_weight"] == 0_u64
            })),
        "expected editor-only key to lose selector weight, stdout: {}",
        stdout_text(&output)
    );
    assert!(
        json["effective_weights"]
            .as_array()
            .is_some_and(|weights| weights.iter().any(|entry| {
                entry["maintainer"] == Value::String(signer_id(&view_only_author))
                    && entry["view_admitted"] == Value::Bool(true)
                    && entry["admitted"] == Value::Bool(true)
                    && entry["effective_weight"] == 1_u64
            })),
        "expected view-only key to retain selector weight, stdout: {}",
        stdout_text(&output)
    );
    assert!(
        json["maintainer_support"]
            .as_array()
            .is_some_and(|support| support.iter().any(|entry| {
                entry["maintainer"] == Value::String(signer_id(&editor_only_author))
                    && entry["revision_id"] == editor_revision["revision_id"]
                    && entry["effective_weight"] == 0_u64
            })),
        "expected editor-only support to remain zero-weight, stdout: {}",
        stdout_text(&output)
    );
    assert!(
        json["decision_trace"]
            .as_array()
            .is_some_and(|trace| trace.iter().any(|entry| {
                entry["step"].as_str() == Some("view_admission")
                    && entry["detail"].as_str().is_some_and(|detail| {
                        detail.contains("mode=admitted-only")
                            && detail.contains("maintainers=2")
                            && detail.contains("view_admitted=1")
                    })
            })),
        "expected named-profile view_admission trace entry, stdout: {}",
        stdout_text(&output)
    );
}

#[test]
fn head_inspect_store_root_reports_missing_manifest() {
    let input = write_input_file(
        "head-inspect-store-missing-manifest",
        "input.json",
        json!({
            "profile": head_profile("hash:missing".to_string(), 1200),
            "revisions": [],
            "views": [],
            "critical_violations": []
        }),
    );
    let store_dir = create_temp_dir("head-inspect-missing-store-root");
    let output = run_mycel(&[
        "head",
        "inspect",
        "doc:sample",
        "--input",
        &path_arg(&input.path),
        "--store-root",
        &path_arg(store_dir.path()),
        "--json",
    ]);

    assert_exit_code(&output, 1);
    let json = parse_json_stdout(&output);
    assert_eq!(json["status"], "failed");
    assert!(
        json["errors"]
            .as_array()
            .is_some_and(|errors| errors.iter().any(|entry| {
                entry
                    .as_str()
                    .is_some_and(|message| message.contains("failed to read store index manifest"))
            })),
        "expected missing manifest error, stdout: {}",
        stdout_text(&output)
    );
}

#[test]
fn head_inspect_text_fails_when_no_eligible_head_exists() {
    let author_key = signing_key(11);
    let revision = signed_revision(&author_key, "doc:other", vec![], 1000, "hash:state-a");
    let policy = json!({
        "accept_keys": [signer_id(&signing_key(12))],
        "merge_rule": "manual-reviewed",
        "preferred_branches": ["main"]
    });
    let bundle = json!({
        "profile": head_profile(hash_json(&policy), 1200),
        "revisions": [revision],
        "views": [],
        "critical_violations": []
    });
    let input = write_input_file("head-inspect-missing-doc", "input.json", bundle);
    let path = path_arg(&input.path);
    let output = run_mycel(&["head", "inspect", "doc:missing", "--input", &path]);

    assert_exit_code(&output, 1);
    assert_stdout_contains(&output, "Head inspection: failed");
    assert_stdout_contains(&output, "status: selection failed");
    assert_stderr_starts_with(&output, "error: ");
    assert_stderr_contains(&output, "NO_ELIGIBLE_HEAD");
}

#[test]
fn head_inspect_directory_resolves_input_json() {
    let author_key = signing_key(11);
    let revision = signed_revision(&author_key, "doc:sample", vec![], 1000, "hash:state-a");
    let policy = json!({
        "accept_keys": [signer_id(&signing_key(12))],
        "merge_rule": "manual-reviewed",
        "preferred_branches": ["main"]
    });
    let bundle = json!({
        "profile": head_profile(hash_json(&policy), 1200),
        "revisions": [revision],
        "views": [],
        "critical_violations": []
    });
    let dir = create_temp_dir("head-inspect-directory");
    let path = dir.path().join("input.json");
    fs::write(
        &path,
        serde_json::to_string_pretty(&bundle).expect("bundle JSON should serialize"),
    )
    .expect("bundle JSON should be written");
    let output = run_mycel(&[
        "head",
        "inspect",
        "doc:sample",
        "--input",
        &path_arg(dir.path()),
        "--json",
    ]);

    assert_success(&output);
    let json = parse_json_stdout(&output);
    assert_eq!(json["input_path"], path.to_string_lossy().into_owned());
}

#[test]
fn head_inspect_text_reports_decision_trace() {
    let output = run_mycel(&[
        "head",
        "inspect",
        "doc:sample",
        "--input",
        "minimal-head-selection",
    ]);

    assert_success(&output);
    assert_stdout_contains(&output, "Decision");
    assert_stdout_contains(&output, "- selected head:");
    assert_stdout_contains(&output, "- selector score: 2");
    assert_stdout_contains(
        &output,
        "- trace: selected=rev:b98e3dca59291ebab04e88eadafaf30d52fcc78dd18df41568e5689c2be300ad tie_break_reason=higher_selector_score",
    );
    assert!(
        !stdout_text(&output).contains("pk:ed25519:"),
        "expected high-level decision trace only, stdout: {}",
        stdout_text(&output)
    );
}
