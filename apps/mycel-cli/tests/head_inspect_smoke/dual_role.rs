use super::*;
use serde_json::Value;

fn store_backed_dual_role_profile(
    editor_only_author: &SigningKey,
    view_only_author: &SigningKey,
) -> Value {
    let policy = json!({
        "merge_rule": "manual-reviewed",
        "preferred_branches": ["main"]
    });
    let mut profile = head_profile(hash_json(&policy), 30);
    profile["editor_admission"] = json!({
        "mode": "mixed",
        "admitted_keys": [signer_id(editor_only_author)]
    });
    profile["view_admission"] = json!({
        "mode": "admitted-only",
        "admitted_keys": [signer_id(view_only_author)]
    });
    profile
}

#[test]
fn head_inspect_store_backed_separates_editor_and_view_admission() {
    let doc_id = "doc:store-backed-dual-role-separation";
    let editor_only_author = signing_key(211);
    let view_only_author = signing_key(212);
    let policy = json!({
        "merge_rule": "manual-reviewed",
        "preferred_branches": ["main"]
    });
    let state_hash = empty_document_state_hash(doc_id);
    let editor_revision = signed_revision(&editor_only_author, doc_id, vec![], 10, &state_hash);
    let view_revision = signed_revision(&view_only_author, doc_id, vec![], 20, &state_hash);
    let editor_view = signed_view(
        &editor_only_author,
        &policy,
        documents_value(doc_id, &editor_revision["revision_id"]),
        25,
    );
    let view_view = signed_view(
        &view_only_author,
        &policy,
        documents_value(doc_id, &view_revision["revision_id"]),
        26,
    );
    let store_dir = build_store_from_objects(&[
        editor_revision.clone(),
        view_revision.clone(),
        editor_view,
        view_view,
    ]);
    let input = write_input_file(
        "head-inspect-store-backed-dual-role-separation",
        "input.json",
        json!({
            "profile": store_backed_dual_role_profile(&editor_only_author, &view_only_author),
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
}

#[test]
fn head_render_store_backed_separates_editor_and_view_admission() {
    let doc_id = "doc:render-store-backed-dual-role-separation";
    let editor_only_author = signing_key(213);
    let view_only_author = signing_key(214);
    let policy = json!({
        "merge_rule": "manual-reviewed",
        "preferred_branches": ["main"]
    });
    let editor_patch = signed_patch(
        &editor_only_author,
        doc_id,
        "rev:genesis-null",
        10,
        json!([
            {
                "op": "insert_block",
                "new_block": {
                    "block_id": "blk:render-editor-only-001",
                    "block_type": "paragraph",
                    "content": "Editor-only line",
                    "attrs": {},
                    "children": []
                }
            }
        ]),
    );
    let editor_revision = signed_revision_with_patches(
        &editor_only_author,
        doc_id,
        vec![],
        vec![editor_patch["patch_id"]
            .as_str()
            .expect("patch id should exist")
            .to_string()],
        11,
        &document_state_hash(
            doc_id,
            vec![json!({
                "block_id": "blk:render-editor-only-001",
                "block_type": "paragraph",
                "content": "Editor-only line",
                "attrs": {},
                "children": []
            })],
        ),
    );
    let view_patch = signed_patch(
        &view_only_author,
        doc_id,
        "rev:genesis-null",
        20,
        json!([
            {
                "op": "insert_block",
                "new_block": {
                    "block_id": "blk:render-view-only-001",
                    "block_type": "paragraph",
                    "content": "View-only line",
                    "attrs": {},
                    "children": []
                }
            }
        ]),
    );
    let view_revision = signed_revision_with_patches(
        &view_only_author,
        doc_id,
        vec![],
        vec![view_patch["patch_id"]
            .as_str()
            .expect("patch id should exist")
            .to_string()],
        21,
        &document_state_hash(
            doc_id,
            vec![json!({
                "block_id": "blk:render-view-only-001",
                "block_type": "paragraph",
                "content": "View-only line",
                "attrs": {},
                "children": []
            })],
        ),
    );
    let editor_view = signed_view(
        &editor_only_author,
        &policy,
        documents_value(doc_id, &editor_revision["revision_id"]),
        25,
    );
    let view_view = signed_view(
        &view_only_author,
        &policy,
        documents_value(doc_id, &view_revision["revision_id"]),
        26,
    );
    let store_dir = build_store_from_objects(&[
        editor_patch,
        editor_revision,
        view_patch,
        view_revision.clone(),
        editor_view,
        view_view,
    ]);
    let input = write_input_file(
        "head-render-store-backed-dual-role-separation",
        "input.json",
        json!({
            "profile": store_backed_dual_role_profile(&editor_only_author, &view_only_author),
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
        &path_arg(store_dir.path()),
        "--json",
    ]);

    assert_success(&output);
    let json = parse_json_stdout(&output);
    assert_eq!(json["selected_head"], view_revision["revision_id"]);
    assert_eq!(
        json["rendered_text"],
        Value::String("View-only line".to_string())
    );
    assert_eq!(json["rendered_block_count"], Value::from(1));
}
