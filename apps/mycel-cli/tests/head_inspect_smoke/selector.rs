use super::*;

#[test]
fn head_inspect_uses_effective_weight_in_selector_score() {
    let doc_id = "doc:weighted";
    let revision_author = signing_key(21);
    let maintainer_a = signing_key(31);
    let maintainer_b = signing_key(32);
    let maintainer_c = signing_key(33);
    let policy = json!({
        "accept_keys": [
            signer_id(&maintainer_a),
            signer_id(&maintainer_b),
            signer_id(&maintainer_c)
        ],
        "merge_rule": "manual-reviewed",
        "preferred_branches": ["main"]
    });
    let revision_a = signed_revision(&revision_author, doc_id, vec![], 10, "hash:weighted-a");
    let revision_b = signed_revision(&revision_author, doc_id, vec![], 20, "hash:weighted-b");
    let bundle = json!({
        "profile": {
            "policy_hash": hash_json(&policy),
            "effective_selection_time": 250,
            "epoch_seconds": 100,
            "epoch_zero_timestamp": 0,
            "admission_window_epochs": 2,
            "min_valid_views_for_admission": 1,
            "min_valid_views_per_epoch": 2,
            "weight_cap_per_key": 3
        },
        "revisions": [revision_a.clone(), revision_b.clone()],
        "views": [
            signed_view(
                &maintainer_a,
                &policy,
                documents_value(doc_id, &revision_a["revision_id"]),
                10
            ),
            signed_view(
                &maintainer_b,
                &policy,
                documents_value(doc_id, &revision_b["revision_id"]),
                12
            ),
            signed_view(
                &maintainer_a,
                &policy,
                documents_value(doc_id, &revision_a["revision_id"]),
                110
            ),
            signed_view(
                &maintainer_a,
                &policy,
                documents_value(doc_id, &revision_a["revision_id"]),
                120
            ),
            signed_view(
                &maintainer_c,
                &policy,
                documents_value(doc_id, &revision_b["revision_id"]),
                220
            ),
            signed_view(
                &maintainer_a,
                &policy,
                documents_value(doc_id, &revision_a["revision_id"]),
                230
            ),
            signed_view(
                &maintainer_b,
                &policy,
                documents_value(doc_id, &revision_b["revision_id"]),
                240
            )
        ],
        "critical_violations": []
    });
    let input = write_input_file("head-inspect-weighted", "input.json", bundle);
    let output = run_mycel(&[
        "head",
        "inspect",
        doc_id,
        "--input",
        &path_arg(&input.path),
        "--json",
    ]);

    assert_success(&output);
    let json = parse_json_stdout(&output);
    assert_eq!(json["status"], Value::String("ok".to_string()));
    assert_eq!(json["selected_head"], revision_a["revision_id"]);
    assert_eq!(
        json["tie_break_reason"],
        Value::String("higher_selector_score".to_string())
    );
    let eligible_heads = json["eligible_heads"]
        .as_array()
        .expect("eligible_heads should be array");
    let selected = eligible_heads
        .iter()
        .find(|entry| entry["revision_id"] == revision_a["revision_id"])
        .expect("selected head summary should exist");
    let alternative = eligible_heads
        .iter()
        .find(|entry| entry["revision_id"] == revision_b["revision_id"])
        .expect("alternative head summary should exist");
    assert_eq!(selected["weighted_support"], 2);
    assert_eq!(selected["supporter_count"], 1);
    assert_eq!(alternative["weighted_support"], 1);
    let effective_weights = json["effective_weights"]
        .as_array()
        .expect("effective_weights should be array");
    let promoted = effective_weights
        .iter()
        .find(|entry| entry["effective_weight"] == Value::from(2))
        .expect("expected promoted effective weight entry");
    assert_eq!(promoted["admitted"], Value::Bool(true));
    assert!(
        promoted["valid_view_counts"]
            .as_array()
            .is_some_and(|counts| counts.iter().any(|entry| {
                entry["epoch"] == Value::from(1) && entry["count"] == Value::from(2)
            })),
        "expected epoch 1 valid_view_counts entry, stdout: {}",
        stdout_text(&output)
    );
    let maintainer_support = json["maintainer_support"]
        .as_array()
        .expect("maintainer_support should be array");
    assert!(
        maintainer_support.iter().any(|entry| {
            entry["revision_id"] == revision_a["revision_id"]
                && entry["effective_weight"] == Value::from(2)
        }),
        "expected weighted maintainer_support entry, stdout: {}",
        stdout_text(&output)
    );
    assert!(
        json["decision_trace"]
            .as_array()
            .is_some_and(|trace| trace.iter().any(|entry| {
                entry["step"].as_str() == Some("effective_weight")
                    && entry["detail"].as_str().is_some_and(|detail| {
                        detail.contains("admitted=2")
                            && detail.contains("zero_weight=1")
                            && detail.contains("max_effective_weight=2")
                    })
            })),
        "expected effective_weight trace entry, stdout: {}",
        stdout_text(&output)
    );
}

#[test]
fn head_inspect_applies_bounded_viewer_score_channels() {
    let doc_id = "doc:viewer-score";
    let revision_author = signing_key(91);
    let maintainer_a = signing_key(92);
    let maintainer_b = signing_key(93);
    let policy = json!({
        "accept_keys": [
            signer_id(&maintainer_a),
            signer_id(&maintainer_b)
        ],
        "merge_rule": "manual-reviewed",
        "preferred_branches": ["main"]
    });
    let revision_a = signed_revision(&revision_author, doc_id, vec![], 10, "hash:viewer-score-a");
    let revision_b = signed_revision(&revision_author, doc_id, vec![], 20, "hash:viewer-score-b");
    let mut profile = head_profile(hash_json(&policy), 250);
    profile["viewer_score"] = bounded_viewer_score_profile();
    let mut challenge = viewer_signal(
        "signal-challenge",
        104,
        &revision_b["revision_id"],
        "challenge",
        "high",
        100,
        400,
    );
    challenge["evidence_ref"] = Value::String("evidence:challenge-1".to_string());
    let bundle = json!({
        "profile": profile,
        "revisions": [revision_a.clone(), revision_b.clone()],
        "views": [
            signed_view(
                &maintainer_a,
                &policy,
                documents_value(doc_id, &revision_a["revision_id"]),
                100
            ),
            signed_view(
                &maintainer_b,
                &policy,
                documents_value(doc_id, &revision_b["revision_id"]),
                110
            )
        ],
        "viewer_signals": [
            viewer_signal(
                "signal-approval-low",
                101,
                &revision_a["revision_id"],
                "approval",
                "low",
                100,
                400
            ),
            viewer_signal(
                "signal-approval-high",
                102,
                &revision_a["revision_id"],
                "approval",
                "high",
                100,
                400
            ),
            viewer_signal(
                "signal-objection-medium",
                103,
                &revision_b["revision_id"],
                "objection",
                "medium",
                100,
                400
            ),
            challenge
        ],
        "critical_violations": []
    });
    let input = write_input_file("head-inspect-viewer-score", "input.json", bundle);
    let output = run_mycel(&[
        "head",
        "inspect",
        doc_id,
        "--input",
        &path_arg(&input.path),
        "--json",
    ]);

    assert_success(&output);
    let json = parse_json_stdout(&output);
    assert_eq!(json["selected_head"], revision_a["revision_id"]);
    assert_eq!(json["viewer_signal_count"], Value::from(4));
    let eligible_heads = json["eligible_heads"]
        .as_array()
        .expect("eligible_heads should be array");
    let selected = eligible_heads
        .iter()
        .find(|entry| entry["revision_id"] == revision_a["revision_id"])
        .expect("selected viewer-scored head should exist");
    let alternative = eligible_heads
        .iter()
        .find(|entry| entry["revision_id"] == revision_b["revision_id"])
        .expect("alternative viewer-scored head should exist");
    assert_eq!(selected["maintainer_score"], Value::from(1));
    assert_eq!(selected["weighted_support"], Value::from(1));
    assert_eq!(selected["viewer_bonus"], Value::from(2));
    assert_eq!(selected["viewer_penalty"], Value::from(0));
    assert_eq!(selected["selector_score"], Value::from(3));
    assert_eq!(alternative["maintainer_score"], Value::from(1));
    assert_eq!(alternative["viewer_bonus"], Value::from(0));
    assert_eq!(alternative["viewer_penalty"], Value::from(2));
    assert_eq!(alternative["selector_score"], Value::from(0));

    let viewer_signals = json["viewer_signals"]
        .as_array()
        .expect("viewer_signals should be array");
    assert_eq!(viewer_signals.len(), 4);
    let challenge_entry = viewer_signals
        .iter()
        .find(|entry| entry["signal_type"] == Value::String("challenge".to_string()))
        .expect("challenge signal summary should exist");
    assert_eq!(challenge_entry["selector_eligible"], Value::Bool(true));
    assert_eq!(challenge_entry["effective_signal_weight"], Value::from(0));
    assert_eq!(
        challenge_entry["signal_status"],
        Value::String("active".to_string())
    );

    let viewer_score_channels = json["viewer_score_channels"]
        .as_array()
        .expect("viewer_score_channels should be array");
    let selected_channel = viewer_score_channels
        .iter()
        .find(|entry| entry["revision_id"] == revision_a["revision_id"])
        .expect("selected viewer score channel should exist");
    assert_eq!(selected_channel["maintainer_score"], Value::from(1));
    assert_eq!(selected_channel["viewer_bonus"], Value::from(2));
    assert_eq!(selected_channel["viewer_penalty"], Value::from(0));
    assert_eq!(selected_channel["approval_signal_count"], Value::from(2));
    assert_eq!(selected_channel["challenge_signal_count"], Value::from(0));
    assert_eq!(
        selected_channel["challenge_review_pressure"],
        Value::from(0)
    );
    assert_eq!(
        selected_channel["challenge_freeze_pressure"],
        Value::from(0)
    );
    assert_eq!(
        selected_channel["viewer_review_state"],
        Value::String("none".to_string())
    );
    assert_eq!(selected_channel["selector_score"], Value::from(3));
    let alternative_channel = viewer_score_channels
        .iter()
        .find(|entry| entry["revision_id"] == revision_b["revision_id"])
        .expect("alternative viewer score channel should exist");
    assert_eq!(alternative_channel["viewer_bonus"], Value::from(0));
    assert_eq!(alternative_channel["viewer_penalty"], Value::from(2));
    assert_eq!(
        alternative_channel["objection_signal_count"],
        Value::from(1)
    );
    assert_eq!(
        alternative_channel["challenge_signal_count"],
        Value::from(1)
    );
    assert_eq!(
        alternative_channel["challenge_review_pressure"],
        Value::from(2)
    );
    assert_eq!(
        alternative_channel["challenge_freeze_pressure"],
        Value::from(2)
    );
    assert_eq!(
        alternative_channel["viewer_review_state"],
        Value::String("freeze-pressure".to_string())
    );
    assert_eq!(alternative_channel["selector_score"], Value::from(0));

    assert!(
        json["decision_trace"]
            .as_array()
            .is_some_and(|trace| trace.iter().any(|entry| {
                entry["step"].as_str() == Some("viewer_score_channels")
                    && entry["detail"].as_str().is_some_and(|detail| {
                        detail.contains("mode=bounded-bonus-penalty")
                            && detail.contains("signals=4")
                            && detail.contains("contributing=3")
                            && detail.contains("bonus_cap=2")
                            && detail.contains("penalty_cap=2")
                    })
            })),
        "expected viewer_score_channels trace entry, stdout: {}",
        stdout_text(&output)
    );
}

#[test]
fn head_inspect_gates_low_confidence_challenge_from_review_path() {
    let doc_id = "doc:viewer-low-confidence-challenge";
    let revision_author = signing_key(141);
    let maintainer_a = signing_key(142);
    let policy = json!({
        "accept_keys": [signer_id(&maintainer_a)],
        "merge_rule": "manual-reviewed",
        "preferred_branches": ["main"]
    });
    let revision_a = signed_revision(
        &revision_author,
        doc_id,
        vec![],
        10,
        "hash:viewer-low-confidence",
    );
    let mut profile = head_profile(hash_json(&policy), 250);
    profile["viewer_score"] = bounded_viewer_score_profile();
    let mut challenge = viewer_signal(
        "signal-challenge-low",
        143,
        &revision_a["revision_id"],
        "challenge",
        "low",
        100,
        400,
    );
    challenge["evidence_ref"] = Value::String("evidence:challenge-low".to_string());
    let bundle = json!({
        "profile": profile,
        "revisions": [revision_a.clone()],
        "views": [
            signed_view(
                &maintainer_a,
                &policy,
                documents_value(doc_id, &revision_a["revision_id"]),
                100
            )
        ],
        "viewer_signals": [challenge],
        "critical_violations": []
    });
    let input = write_input_file(
        "head-inspect-viewer-low-confidence-challenge",
        "input.json",
        bundle,
    );
    let output = run_mycel(&[
        "head",
        "inspect",
        doc_id,
        "--input",
        &path_arg(&input.path),
        "--json",
    ]);

    assert_success(&output);
    let json = parse_json_stdout(&output);
    assert_eq!(json["selected_head"], revision_a["revision_id"]);
    let viewer_signals = json["viewer_signals"]
        .as_array()
        .expect("viewer_signals should be array");
    let challenge_entry = viewer_signals
        .iter()
        .find(|entry| entry["signal_id"] == Value::String("signal-challenge-low".to_string()))
        .expect("low-confidence challenge summary should exist");
    assert_eq!(challenge_entry["selector_eligible"], Value::Bool(false));
    assert_eq!(challenge_entry["effective_signal_weight"], Value::from(0));

    let viewer_score_channels = json["viewer_score_channels"]
        .as_array()
        .expect("viewer_score_channels should be array");
    let channel = viewer_score_channels
        .iter()
        .find(|entry| entry["revision_id"] == revision_a["revision_id"])
        .expect("viewer score channel should exist");
    assert_eq!(channel["challenge_signal_count"], Value::from(0));
    assert_eq!(channel["challenge_review_pressure"], Value::from(0));
    assert_eq!(channel["challenge_freeze_pressure"], Value::from(0));
    assert_eq!(
        channel["viewer_review_state"],
        Value::String("none".to_string())
    );
}

#[test]
fn head_inspect_debug_text_reports_viewer_channels_without_overloading_trace() {
    let doc_id = "doc:viewer-score-text";
    let revision_author = signing_key(111);
    let maintainer_a = signing_key(112);
    let maintainer_b = signing_key(113);
    let policy = json!({
        "accept_keys": [
            signer_id(&maintainer_a),
            signer_id(&maintainer_b)
        ],
        "merge_rule": "manual-reviewed",
        "preferred_branches": ["main"]
    });
    let revision_a = signed_revision(
        &revision_author,
        doc_id,
        vec![],
        10,
        "hash:viewer-score-text-a",
    );
    let revision_b = signed_revision(
        &revision_author,
        doc_id,
        vec![],
        20,
        "hash:viewer-score-text-b",
    );
    let mut profile = head_profile(hash_json(&policy), 250);
    profile["viewer_score"] = bounded_viewer_score_profile();
    let mut challenge = viewer_signal(
        "signal-challenge-text",
        114,
        &revision_b["revision_id"],
        "challenge",
        "high",
        100,
        400,
    );
    challenge["evidence_ref"] = Value::String("evidence:challenge-text".to_string());
    let bundle = json!({
        "profile": profile,
        "revisions": [revision_a.clone(), revision_b.clone()],
        "views": [
            signed_view(
                &maintainer_a,
                &policy,
                documents_value(doc_id, &revision_a["revision_id"]),
                100
            ),
            signed_view(
                &maintainer_b,
                &policy,
                documents_value(doc_id, &revision_b["revision_id"]),
                110
            )
        ],
        "viewer_signals": [
            viewer_signal(
                "signal-approval-text",
                115,
                &revision_a["revision_id"],
                "approval",
                "medium",
                100,
                400
            ),
            challenge
        ],
        "critical_violations": []
    });
    let input = write_input_file("head-inspect-viewer-score-text", "input.json", bundle);
    let output = run_mycel(&[
        "head",
        "inspect",
        doc_id,
        "--input",
        &path_arg(&input.path),
        "--output-mode",
        "debug",
    ]);

    assert_success(&output);
    assert_stdout_contains(&output, "viewer signals: 2");
    assert_stdout_contains(&output, "viewer channel: ");
    assert_stdout_contains(&output, "review_pressure=2");
    assert_stdout_contains(&output, "freeze_pressure=2");
    assert_stdout_contains(&output, "review_state=freeze-pressure");
    assert_stdout_contains(&output, "score_formula=\"1 + 2 - 0 = 3\"");
    assert!(
        !stdout_text(&output).contains("trace: viewer_signal_id"),
        "expected trace to stay high-level, stdout: {}",
        stdout_text(&output)
    );
}

#[test]
fn head_inspect_human_mode_groups_summary_candidates_and_decision() {
    let output = run_mycel(&[
        "head",
        "inspect",
        "doc:sample",
        "--input",
        "viewer-score-channels",
    ]);

    assert_success(&output);
    assert_stdout_contains(&output, "Head inspection: ok");
    assert_stdout_contains(&output, "Document");
    assert_stdout_contains(&output, "Candidates");
    assert_stdout_contains(&output, "(selected)");
    assert_stdout_contains(&output, "Viewer Effects");
    assert_stdout_contains(&output, "Decision");
    assert_stdout_contains(
        &output,
        "status: selection succeeded after blocking candidates under viewer freeze pressure",
    );
    assert_stdout_contains(&output, "selector score: 4");
    assert_stdout_contains(&output, "score formula: 2 + 2 - 0 = 4");
    assert_stdout_contains(
        &output,
        "reason: higher selector score after another candidate was frozen",
    );
    assert_stdout_contains(&output, "state=freeze pressure");
    assert!(
        !stdout_text(&output).contains("trace: selector_epoch"),
        "expected human mode to avoid full debug trace, stdout: {}",
        stdout_text(&output)
    );
}

#[test]
fn head_inspect_skips_candidate_delayed_by_viewer_review_pressure() {
    let doc_id = "doc:viewer-review-skip";
    let revision_author = signing_key(116);
    let maintainer_a = signing_key(117);
    let maintainer_b = signing_key(118);
    let policy = json!({
        "accept_keys": [
            signer_id(&maintainer_a),
            signer_id(&maintainer_b)
        ],
        "merge_rule": "manual-reviewed",
        "preferred_branches": ["main"]
    });
    let revision_a = signed_revision(&revision_author, doc_id, vec![], 10, "hash:viewer-review-a");
    let revision_b = signed_revision(&revision_author, doc_id, vec![], 20, "hash:viewer-review-b");
    let mut profile = head_profile(hash_json(&policy), 250);
    profile["viewer_score"] = bounded_viewer_score_profile();
    let mut challenge = viewer_signal(
        "signal-review-selected",
        119,
        &revision_a["revision_id"],
        "challenge",
        "medium",
        100,
        400,
    );
    challenge["evidence_ref"] = Value::String("evidence:review-selected".to_string());
    let bundle = json!({
        "profile": profile,
        "revisions": [revision_a.clone(), revision_b.clone()],
        "views": [
            signed_view(
                &maintainer_a,
                &policy,
                documents_value(doc_id, &revision_a["revision_id"]),
                100
            ),
            signed_view(
                &maintainer_b,
                &policy,
                documents_value(doc_id, &revision_a["revision_id"]),
                110
            )
        ],
        "viewer_signals": [challenge],
        "critical_violations": []
    });
    let input = write_input_file("head-inspect-viewer-review-skip", "input.json", bundle);
    let output = run_mycel(&[
        "head",
        "inspect",
        doc_id,
        "--input",
        &path_arg(&input.path),
        "--json",
    ]);

    assert_success(&output);
    let json = parse_json_stdout(&output);
    assert_eq!(
        json["status"],
        Value::String("ok-with-viewer-review-delay".to_string())
    );
    assert_eq!(json["selected_head"], revision_b["revision_id"]);
    assert_eq!(
        json["tie_break_reason"],
        Value::String(
            "newer_revision_timestamp_or_lexicographic_tiebreak_after_viewer-review-delay"
                .to_string(),
        )
    );
    assert!(
        json["notes"]
            .as_array()
            .is_some_and(|notes| notes.iter().any(|entry| {
                entry.as_str().is_some_and(|message| {
                    message.contains("review pressure delays candidate activation")
                        && message.contains(
                            revision_a["revision_id"]
                                .as_str()
                                .expect("revision id should exist"),
                        )
                })
            })),
        "expected review delay note, stdout: {}",
        stdout_text(&output)
    );
    assert!(
        json["decision_trace"]
            .as_array()
            .is_some_and(|trace| trace.iter().any(|entry| {
                entry["step"].as_str() == Some("viewer_review")
                    && entry["detail"].as_str().is_some_and(|detail| {
                        detail.contains("delayed_candidates=1")
                            && detail.contains("active_candidates=1")
                            && detail.contains(
                                revision_a["revision_id"]
                                    .as_str()
                                    .expect("revision id should exist"),
                            )
                    })
            })),
        "expected viewer_review trace entry, stdout: {}",
        stdout_text(&output)
    );
    let viewer_score_channels = json["viewer_score_channels"]
        .as_array()
        .expect("viewer_score_channels should be array");
    let delayed_channel = viewer_score_channels
        .iter()
        .find(|entry| entry["revision_id"] == revision_a["revision_id"])
        .expect("delayed candidate channel should exist");
    assert_eq!(
        delayed_channel["viewer_review_state"],
        Value::String("review-pressure".to_string())
    );
}

#[test]
fn head_inspect_fails_when_all_candidates_are_delayed_by_viewer_review_pressure() {
    let doc_id = "doc:viewer-review-fail";
    let revision_author = signing_key(125);
    let maintainer_a = signing_key(126);
    let policy = json!({
        "accept_keys": [signer_id(&maintainer_a)],
        "merge_rule": "manual-reviewed",
        "preferred_branches": ["main"]
    });
    let revision_a = signed_revision(
        &revision_author,
        doc_id,
        vec![],
        10,
        "hash:viewer-review-fail-a",
    );
    let mut profile = head_profile(hash_json(&policy), 250);
    profile["viewer_score"] = bounded_viewer_score_profile();
    let mut challenge = viewer_signal(
        "signal-review-only",
        127,
        &revision_a["revision_id"],
        "challenge",
        "medium",
        100,
        400,
    );
    challenge["evidence_ref"] = Value::String("evidence:review-only".to_string());
    let bundle = json!({
        "profile": profile,
        "revisions": [revision_a.clone()],
        "views": [
            signed_view(
                &maintainer_a,
                &policy,
                documents_value(doc_id, &revision_a["revision_id"]),
                100
            )
        ],
        "viewer_signals": [challenge],
        "critical_violations": []
    });
    let input = write_input_file("head-inspect-viewer-review-fail", "input.json", bundle);
    let output = run_mycel(&[
        "head",
        "inspect",
        doc_id,
        "--input",
        &path_arg(&input.path),
        "--json",
    ]);

    assert_exit_code(&output, 1);
    let json = parse_json_stdout(&output);
    assert_eq!(
        json["status"],
        Value::String("blocked-by-viewer-review-delay".to_string())
    );
    assert_eq!(json["selected_head"], Value::Null);
    assert!(
        json["errors"]
            .as_array()
            .is_some_and(|errors| errors.iter().any(|entry| {
                entry.as_str().is_some_and(|message| {
                    message.contains("NO_ACTIVE_HEAD_AFTER_VIEWER_REVIEW_OR_FREEZE")
                })
            })),
        "expected viewer review/freeze failure error, stdout: {}",
        stdout_text(&output)
    );
}

#[test]
fn head_inspect_skips_candidate_blocked_by_viewer_freeze_pressure() {
    let doc_id = "doc:viewer-freeze-skip";
    let revision_author = signing_key(121);
    let maintainer_a = signing_key(122);
    let maintainer_b = signing_key(123);
    let policy = json!({
        "accept_keys": [
            signer_id(&maintainer_a),
            signer_id(&maintainer_b)
        ],
        "merge_rule": "manual-reviewed",
        "preferred_branches": ["main"]
    });
    let revision_a = signed_revision(&revision_author, doc_id, vec![], 10, "hash:viewer-freeze-a");
    let revision_b = signed_revision(&revision_author, doc_id, vec![], 20, "hash:viewer-freeze-b");
    let mut profile = head_profile(hash_json(&policy), 250);
    profile["viewer_score"] = bounded_viewer_score_profile();
    let mut challenge = viewer_signal(
        "signal-freeze-selected",
        124,
        &revision_a["revision_id"],
        "challenge",
        "high",
        100,
        400,
    );
    challenge["evidence_ref"] = Value::String("evidence:freeze-selected".to_string());
    let bundle = json!({
        "profile": profile,
        "revisions": [revision_a.clone(), revision_b.clone()],
        "views": [
            signed_view(
                &maintainer_a,
                &policy,
                documents_value(doc_id, &revision_a["revision_id"]),
                100
            ),
            signed_view(
                &maintainer_b,
                &policy,
                documents_value(doc_id, &revision_a["revision_id"]),
                110
            )
        ],
        "viewer_signals": [challenge],
        "critical_violations": []
    });
    let input = write_input_file("head-inspect-viewer-freeze-skip", "input.json", bundle);
    let output = run_mycel(&[
        "head",
        "inspect",
        doc_id,
        "--input",
        &path_arg(&input.path),
        "--json",
    ]);

    assert_success(&output);
    let json = parse_json_stdout(&output);
    assert_eq!(
        json["status"],
        Value::String("ok-with-viewer-freeze-block".to_string())
    );
    assert_eq!(json["selected_head"], revision_b["revision_id"]);
    assert_eq!(
        json["tie_break_reason"],
        Value::String(
            "newer_revision_timestamp_or_lexicographic_tiebreak_after_viewer-freeze-block"
                .to_string()
        )
    );
    assert!(
        json["notes"]
            .as_array()
            .is_some_and(|notes| notes.iter().any(|entry| {
                entry.as_str().is_some_and(|message| {
                    message.contains("temporary freeze blocks candidate activation")
                        && message.contains(
                            revision_a["revision_id"]
                                .as_str()
                                .expect("revision id should exist"),
                        )
                })
            })),
        "expected temporary freeze note, stdout: {}",
        stdout_text(&output)
    );
    assert!(
        json["decision_trace"]
            .as_array()
            .is_some_and(|trace| trace.iter().any(|entry| {
                entry["step"].as_str() == Some("viewer_freeze")
                    && entry["detail"].as_str().is_some_and(|detail| {
                        detail.contains("blocked_candidates=1")
                            && detail.contains("active_candidates=1")
                            && detail.contains(
                                revision_a["revision_id"]
                                    .as_str()
                                    .expect("revision id should exist"),
                            )
                    })
            })),
        "expected viewer_freeze trace entry, stdout: {}",
        stdout_text(&output)
    );
    let viewer_score_channels = json["viewer_score_channels"]
        .as_array()
        .expect("viewer_score_channels should be array");
    let blocked_channel = viewer_score_channels
        .iter()
        .find(|entry| entry["revision_id"] == revision_a["revision_id"])
        .expect("blocked candidate channel should exist");
    assert_eq!(
        blocked_channel["viewer_review_state"],
        Value::String("freeze-pressure".to_string())
    );
}

#[test]
fn head_inspect_fails_when_all_candidates_are_blocked_by_viewer_freeze_pressure() {
    let doc_id = "doc:viewer-freeze-fail";
    let revision_author = signing_key(131);
    let maintainer_a = signing_key(132);
    let policy = json!({
        "accept_keys": [signer_id(&maintainer_a)],
        "merge_rule": "manual-reviewed",
        "preferred_branches": ["main"]
    });
    let revision_a = signed_revision(
        &revision_author,
        doc_id,
        vec![],
        10,
        "hash:viewer-freeze-fail-a",
    );
    let mut profile = head_profile(hash_json(&policy), 250);
    profile["viewer_score"] = bounded_viewer_score_profile();
    let mut challenge = viewer_signal(
        "signal-freeze-only",
        133,
        &revision_a["revision_id"],
        "challenge",
        "high",
        100,
        400,
    );
    challenge["evidence_ref"] = Value::String("evidence:freeze-only".to_string());
    let bundle = json!({
        "profile": profile,
        "revisions": [revision_a.clone()],
        "views": [
            signed_view(
                &maintainer_a,
                &policy,
                documents_value(doc_id, &revision_a["revision_id"]),
                100
            )
        ],
        "viewer_signals": [challenge],
        "critical_violations": []
    });
    let input = write_input_file("head-inspect-viewer-freeze-fail", "input.json", bundle);
    let output = run_mycel(&[
        "head",
        "inspect",
        doc_id,
        "--input",
        &path_arg(&input.path),
        "--json",
    ]);

    assert_exit_code(&output, 1);
    let json = parse_json_stdout(&output);
    assert_eq!(
        json["status"],
        Value::String("blocked-by-viewer-freeze-block".to_string())
    );
    assert_eq!(json["selected_head"], Value::Null);
    assert!(
        json["errors"]
            .as_array()
            .is_some_and(|errors| errors.iter().any(|entry| {
                entry.as_str().is_some_and(|message| {
                    message.contains("NO_ACTIVE_HEAD_AFTER_VIEWER_REVIEW_OR_FREEZE")
                })
            })),
        "expected viewer freeze failure error, stdout: {}",
        stdout_text(&output)
    );
}

#[test]
fn head_inspect_penalizes_critical_violations() {
    let doc_id = "doc:penalty";
    let revision_author = signing_key(41);
    let maintainer_a = signing_key(51);
    let maintainer_b = signing_key(52);
    let policy = json!({
        "accept_keys": [
            signer_id(&maintainer_a),
            signer_id(&maintainer_b)
        ],
        "merge_rule": "manual-reviewed",
        "preferred_branches": ["main"]
    });
    let revision_a = signed_revision(&revision_author, doc_id, vec![], 10, "hash:penalty-a");
    let revision_b = signed_revision(&revision_author, doc_id, vec![], 20, "hash:penalty-b");
    let bundle = json!({
        "profile": {
            "policy_hash": hash_json(&policy),
            "effective_selection_time": 250,
            "epoch_seconds": 100,
            "epoch_zero_timestamp": 0,
            "admission_window_epochs": 2,
            "min_valid_views_for_admission": 1,
            "min_valid_views_per_epoch": 2,
            "weight_cap_per_key": 3
        },
        "revisions": [revision_a.clone(), revision_b.clone()],
        "views": [
            signed_view(
                &maintainer_a,
                &policy,
                documents_value(doc_id, &revision_a["revision_id"]),
                10
            ),
            signed_view(
                &maintainer_b,
                &policy,
                documents_value(doc_id, &revision_b["revision_id"]),
                12
            ),
            signed_view(
                &maintainer_a,
                &policy,
                documents_value(doc_id, &revision_a["revision_id"]),
                110
            ),
            signed_view(
                &maintainer_a,
                &policy,
                documents_value(doc_id, &revision_a["revision_id"]),
                120
            ),
            signed_view(
                &maintainer_b,
                &policy,
                documents_value(doc_id, &revision_b["revision_id"]),
                210
            ),
            signed_view(
                &maintainer_b,
                &policy,
                documents_value(doc_id, &revision_b["revision_id"]),
                220
            )
        ],
        "critical_violations": [
            critical_violation(&maintainer_a, 150, "equivocated view publication")
        ]
    });
    let input = write_input_file("head-inspect-penalty", "input.json", bundle);
    let output = run_mycel(&[
        "head",
        "inspect",
        doc_id,
        "--input",
        &path_arg(&input.path),
        "--json",
    ]);

    assert_success(&output);
    let json = parse_json_stdout(&output);
    assert_eq!(json["selected_head"], revision_b["revision_id"]);
    let eligible_heads = json["eligible_heads"]
        .as_array()
        .expect("eligible_heads should be array");
    let penalized = eligible_heads
        .iter()
        .find(|entry| entry["revision_id"] == revision_a["revision_id"])
        .expect("penalized head summary should exist");
    let surviving = eligible_heads
        .iter()
        .find(|entry| entry["revision_id"] == revision_b["revision_id"])
        .expect("surviving head summary should exist");
    assert_eq!(penalized["weighted_support"], 0);
    assert_eq!(surviving["weighted_support"], 1);
    let critical_violations = json["critical_violations"]
        .as_array()
        .expect("critical_violations should be array");
    assert_eq!(critical_violations.len(), 1);
    assert_eq!(critical_violations[0]["selector_epoch"], Value::from(1));
    assert_eq!(
        critical_violations[0]["reason"],
        Value::String("equivocated view publication".to_string())
    );
    let effective_weights = json["effective_weights"]
        .as_array()
        .expect("effective_weights should be array");
    let penalized_weight = effective_weights
        .iter()
        .find(|entry| {
            entry["critical_violation_counts"]
                .as_array()
                .is_some_and(|counts| {
                    counts.iter().any(|count| {
                        count["epoch"] == Value::from(1) && count["count"] == Value::from(1)
                    })
                })
        })
        .expect("expected penalized effective weight entry");
    assert_eq!(penalized_weight["admitted"], Value::Bool(false));
    assert_eq!(penalized_weight["effective_weight"], Value::from(0));
    let maintainer_support = json["maintainer_support"]
        .as_array()
        .expect("maintainer_support should be array");
    assert!(
        maintainer_support
            .iter()
            .all(|entry| entry["effective_weight"] != Value::from(0)),
        "expected penalized maintainer to be absent from maintainer_support, stdout: {}",
        stdout_text(&output)
    );
    assert!(
        json["decision_trace"]
            .as_array()
            .is_some_and(|trace| trace.iter().any(|entry| {
                entry["step"].as_str() == Some("effective_weight")
                    && entry["detail"].as_str().is_some_and(|detail| {
                        detail.contains("penalized=1")
                            && detail.contains("zero_weight=1")
                            && detail.contains("max_effective_weight=1")
                    })
            })),
        "expected penalty trace entry, stdout: {}",
        stdout_text(&output)
    );
    assert!(
        json["decision_trace"]
            .as_array()
            .is_some_and(|trace| trace.iter().any(|entry| {
                entry["step"].as_str() == Some("critical_violations")
                    && entry["detail"]
                        .as_str()
                        .is_some_and(|detail| detail == "count=1 affected_maintainers=1")
            })),
        "expected critical_violations trace summary, stdout: {}",
        stdout_text(&output)
    );
}

#[test]
fn head_inspect_admitted_only_editor_policy_filters_non_admitted_candidate_heads() {
    let doc_id = "doc:editor-admitted-only";
    let admitted_author = signing_key(61);
    let non_admitted_author = signing_key(62);
    let policy = json!({
        "merge_rule": "manual-reviewed",
        "preferred_branches": ["main"]
    });
    let admitted_revision =
        signed_revision(&admitted_author, doc_id, vec![], 10, "hash:editor-admitted");
    let non_admitted_revision = signed_revision(
        &non_admitted_author,
        doc_id,
        vec![],
        20,
        "hash:editor-non-admitted",
    );
    let mut profile = head_profile(hash_json(&policy), 1200);
    profile["editor_admission"] = json!({
        "mode": "admitted-only",
        "admitted_keys": [signer_id(&admitted_author)]
    });
    let bundle = json!({
        "profile": profile,
        "revisions": [admitted_revision.clone(), non_admitted_revision.clone()],
        "views": [],
        "critical_violations": []
    });
    let input = write_input_file("head-inspect-editor-admitted-only", "input.json", bundle);
    let output = run_mycel(&[
        "head",
        "inspect",
        doc_id,
        "--input",
        &path_arg(&input.path),
        "--json",
    ]);

    assert_success(&output);
    let json = parse_json_stdout(&output);
    assert_eq!(json["selected_head"], admitted_revision["revision_id"]);
    let editor_candidates = json["editor_candidates"]
        .as_array()
        .expect("editor_candidates should be array");
    assert!(
        editor_candidates.iter().any(|entry| {
            entry["revision_id"] == admitted_revision["revision_id"]
                && entry["editor_admitted"] == Value::Bool(true)
                && entry["candidate_eligible"] == Value::Bool(true)
                && entry["formal_candidate"] == Value::Bool(true)
        }),
        "expected admitted editor candidate summary, stdout: {}",
        stdout_text(&output)
    );
    assert!(
        editor_candidates.iter().any(|entry| {
            entry["revision_id"] == non_admitted_revision["revision_id"]
                && entry["editor_admitted"] == Value::Bool(false)
                && entry["candidate_eligible"] == Value::Bool(false)
                && entry["formal_candidate"] == Value::Bool(false)
        }),
        "expected filtered editor candidate summary, stdout: {}",
        stdout_text(&output)
    );
    let eligible_heads = json["eligible_heads"]
        .as_array()
        .expect("eligible_heads should be array");
    assert_eq!(eligible_heads.len(), 1);
    assert_eq!(
        eligible_heads[0]["revision_id"],
        admitted_revision["revision_id"]
    );
    assert_eq!(
        eligible_heads[0]["author"],
        Value::String(signer_id(&admitted_author))
    );
    assert_eq!(eligible_heads[0]["editor_admitted"], Value::Bool(true));
    assert_eq!(eligible_heads[0]["formal_candidate"], Value::Bool(true));
    assert!(
        json["decision_trace"]
            .as_array()
            .is_some_and(|trace| trace.iter().any(|entry| {
                entry["step"].as_str() == Some("editor_admission")
                    && entry["detail"].as_str().is_some_and(|detail| {
                        detail.contains("mode=admitted-only")
                            && detail.contains("structural_heads=2")
                            && detail.contains("eligible=1")
                            && detail.contains("formal=1")
                    })
            })),
        "expected editor_admission trace entry, stdout: {}",
        stdout_text(&output)
    );
}

#[test]
fn head_inspect_mixed_editor_policy_marks_formal_candidates_without_filtering_selection() {
    let doc_id = "doc:editor-mixed";
    let admitted_author = signing_key(71);
    let non_admitted_author = signing_key(72);
    let policy = json!({
        "merge_rule": "manual-reviewed",
        "preferred_branches": ["main"]
    });
    let admitted_revision = signed_revision(
        &admitted_author,
        doc_id,
        vec![],
        10,
        "hash:editor-mixed-admitted",
    );
    let non_admitted_revision = signed_revision(
        &non_admitted_author,
        doc_id,
        vec![],
        20,
        "hash:editor-mixed-non-admitted",
    );
    let mut profile = head_profile(hash_json(&policy), 1200);
    profile["editor_admission"] = json!({
        "mode": "mixed",
        "admitted_keys": [signer_id(&admitted_author)]
    });
    let bundle = json!({
        "profile": profile,
        "revisions": [admitted_revision.clone(), non_admitted_revision.clone()],
        "views": [],
        "critical_violations": []
    });
    let input = write_input_file("head-inspect-editor-mixed", "input.json", bundle);
    let output = run_mycel(&[
        "head",
        "inspect",
        doc_id,
        "--input",
        &path_arg(&input.path),
        "--json",
    ]);

    assert_success(&output);
    let json = parse_json_stdout(&output);
    assert_eq!(json["selected_head"], non_admitted_revision["revision_id"]);
    let eligible_heads = json["eligible_heads"]
        .as_array()
        .expect("eligible_heads should be array");
    assert_eq!(eligible_heads.len(), 2);
    assert!(
        eligible_heads.iter().any(|entry| {
            entry["revision_id"] == admitted_revision["revision_id"]
                && entry["formal_candidate"] == Value::Bool(true)
        }),
        "expected admitted formal candidate head, stdout: {}",
        stdout_text(&output)
    );
    assert!(
        eligible_heads.iter().any(|entry| {
            entry["revision_id"] == non_admitted_revision["revision_id"]
                && entry["editor_admitted"] == Value::Bool(false)
                && entry["formal_candidate"] == Value::Bool(false)
        }),
        "expected mixed-mode informal candidate head, stdout: {}",
        stdout_text(&output)
    );
    assert!(
        json["decision_trace"]
            .as_array()
            .is_some_and(|trace| trace.iter().any(|entry| {
                entry["step"].as_str() == Some("editor_admission")
                    && entry["detail"].as_str().is_some_and(|detail| {
                        detail.contains("mode=mixed")
                            && detail.contains("structural_heads=2")
                            && detail.contains("eligible=2")
                            && detail.contains("formal=1")
                    })
            })),
        "expected mixed editor_admission trace entry, stdout: {}",
        stdout_text(&output)
    );
}
