use super::*;

#[test]
fn head_profile_list_json_reports_named_profiles() {
    let policy = json!({
        "merge_rule": "manual-reviewed",
        "preferred_branches": ["main"]
    });
    let stable = head_profile(hash_json(&policy), 18);
    let mut preview = head_profile(hash_json(&policy), 30);
    preview["editor_admission"] = json!({
        "mode": "mixed",
        "admitted_keys": ["key:editor-preview"]
    });
    preview["view_admission"] = json!({
        "mode": "admitted-only",
        "admitted_keys": ["key:view-preview"]
    });
    preview["viewer_score"] = bounded_viewer_score_profile();
    let input = write_input_file(
        "head-profile-list",
        "input.json",
        json!({
            "profiles": named_profiles(&[
                ("stable", stable),
                ("preview", preview)
            ]),
            "revisions": [],
            "views": [],
            "critical_violations": []
        }),
    );

    let output = run_mycel(&[
        "head",
        "profile",
        "list",
        "--input",
        &path_arg(&input.path),
        "--json",
    ]);

    assert_success(&output);
    let json = parse_json_stdout(&output);
    assert_eq!(json["status"], "ok");
    assert_eq!(json["profile_count"], 2);
    assert_eq!(json["available_profile_ids"], json!(["preview", "stable"]));
    let profiles = json["profiles"]
        .as_array()
        .expect("profiles should be an array");
    assert_eq!(profiles.len(), 2);
    assert_eq!(profiles[0]["profile_id"], "preview");
    assert_eq!(profiles[0]["source"], "named");
    assert_eq!(profiles[0]["editor_admission"]["mode"], "mixed");
    assert_eq!(profiles[0]["view_admission"]["mode"], "admitted-only");
    assert_eq!(profiles[0]["viewer_score"]["enabled"], Value::Bool(true));
    assert_eq!(
        profiles[0]["viewer_score"]["mode"],
        Value::String("bounded-bonus-penalty".to_string())
    );
    assert_eq!(profiles[1]["profile_id"], "stable");
    assert_eq!(profiles[1]["viewer_score"]["enabled"], Value::Bool(false));
}

#[test]
fn head_profile_inspect_json_reports_requested_named_profile() {
    let policy = json!({
        "merge_rule": "manual-reviewed",
        "preferred_branches": ["main"]
    });
    let stable = head_profile(hash_json(&policy), 18);
    let mut preview = head_profile(hash_json(&policy), 30);
    preview["editor_admission"] = json!({
        "mode": "admitted-only",
        "admitted_keys": ["key:editor-preview"]
    });
    preview["view_admission"] = json!({
        "mode": "admitted-only",
        "admitted_keys": ["key:view-preview"]
    });
    preview["viewer_score"] = bounded_viewer_score_profile();
    let input = write_input_file(
        "head-profile-inspect",
        "input.json",
        json!({
            "profiles": named_profiles(&[
                ("stable", stable),
                ("preview", preview)
            ]),
            "revisions": [],
            "views": [],
            "critical_violations": []
        }),
    );

    let output = run_mycel(&[
        "head",
        "profile",
        "inspect",
        "--input",
        &path_arg(&input.path),
        "--profile-id",
        "preview",
        "--json",
    ]);

    assert_success(&output);
    let json = parse_json_stdout(&output);
    assert_eq!(json["status"], "ok");
    assert_eq!(json["requested_profile_id"], "preview");
    assert_eq!(json["available_profile_ids"], json!(["preview", "stable"]));
    assert_eq!(json["profile"]["profile_id"], "preview");
    assert_eq!(json["profile"]["source"], "named");
    assert_eq!(json["profile"]["effective_selection_time"], 30);
    assert_eq!(json["profile"]["editor_admission"]["mode"], "admitted-only");
    assert_eq!(
        json["profile"]["editor_admission"]["admitted_keys"],
        json!(["key:editor-preview"])
    );
    assert_eq!(
        json["profile"]["view_admission"]["admitted_keys"],
        json!(["key:view-preview"])
    );
    assert_eq!(
        json["profile"]["viewer_score"]["enabled"],
        Value::Bool(true)
    );
}

#[test]
fn head_profile_inspect_requires_profile_id_for_multi_profile_bundle() {
    let policy = json!({
        "merge_rule": "manual-reviewed",
        "preferred_branches": ["main"]
    });
    let input = write_input_file(
        "head-profile-missing-profile-id",
        "input.json",
        json!({
            "profiles": named_profiles(&[
                ("stable", head_profile(hash_json(&policy), 18)),
                ("preview", head_profile(hash_json(&policy), 30))
            ]),
            "revisions": [],
            "views": [],
            "critical_violations": []
        }),
    );

    let output = run_mycel(&[
        "head",
        "profile",
        "inspect",
        "--input",
        &path_arg(&input.path),
    ]);

    assert_exit_code(&output, 1);
    assert_stdout_contains(&output, "Head profile: failed");
    assert_stdout_contains(&output, "- available profiles: preview, stable");
    assert_stdout_contains(
        &output,
        "- retry with one of: --profile-id preview | --profile-id stable",
    );
    assert_stderr_contains(
        &output,
        "head input declares multiple named profiles; pass --profile-id (preview, stable)",
    );
}

#[test]
fn head_profile_inspect_json_reads_default_profile_from_fixture() {
    let output = run_mycel(&[
        "head",
        "profile",
        "inspect",
        "--input",
        "viewer-score-channels",
        "--json",
    ]);

    assert_success(&output);
    let json = parse_json_stdout(&output);
    assert_eq!(json["status"], "ok");
    assert_eq!(json["requested_profile_id"], Value::Null);
    assert_eq!(json["available_profile_ids"], json!([]));
    assert_eq!(json["profile"]["profile_id"], Value::Null);
    assert_eq!(json["profile"]["source"], "default");
    assert_eq!(
        json["profile"]["viewer_score"]["enabled"],
        Value::Bool(true)
    );
    assert_eq!(
        json["notes"][0],
        "selected the default fixed reader profile from top-level 'profile'"
    );
}
