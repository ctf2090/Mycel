use std::fs;
use std::path::Path;

use ed25519_dalek::SigningKey;
use mycel_core::author::signer_id;
use serde_json::{json, Value};

mod common;

use common::{
    assert_empty_stderr, assert_exit_code, assert_json_status, assert_stderr_contains,
    assert_success, create_temp_dir, prefixed_hash_for_test, run_mycel, signed_test_object,
    stdout_text,
};

fn path_arg(path: &Path) -> String {
    path.to_string_lossy().into_owned()
}

fn signing_key() -> SigningKey {
    SigningKey::from_bytes(&[7u8; 32])
}

fn signed_object(value: Value, signer_field: &str, id_field: &str, id_prefix: &str) -> Value {
    let signing_key = signing_key();
    signed_test_object(value, &signing_key, signer_field, id_field, id_prefix)
}

fn profile_id(policy: &Value) -> String {
    prefixed_hash_for_test(policy, "hash")
}

struct StoreFixtureInfo {
    source_dir: common::TempDir,
    store_dir: common::TempDir,
    signer: String,
    revision_id: String,
    view_id: String,
    profile_id: String,
}

struct RelatedGovernanceFixtureInfo {
    store_dir: common::TempDir,
    view_a1_id: String,
    view_a2_id: String,
    view_b1_id: String,
}

fn build_store_with_view() -> StoreFixtureInfo {
    let source_dir = create_temp_dir("store-index-source");
    let store_dir = create_temp_dir("store-index-root");
    let patch_path = source_dir.path().join("patch.json");
    let revision_path = source_dir.path().join("revision.json");
    let view_path = source_dir.path().join("view.json");

    let patch = signed_object(
        json!({
            "type": "patch",
            "version": "mycel/0.1",
            "doc_id": "doc:index",
            "base_revision": "rev:genesis-null",
            "timestamp": 1u64,
            "ops": []
        }),
        "author",
        "patch_id",
        "patch",
    );
    fs::write(
        &patch_path,
        serde_json::to_string_pretty(&patch).expect("patch should serialize"),
    )
    .expect("patch should write");

    let state_hash = prefixed_hash_for_test(&json!({"doc_id": "doc:index", "blocks": []}), "hash");
    let revision = signed_object(
        json!({
            "type": "revision",
            "version": "mycel/0.1",
            "doc_id": "doc:index",
            "parents": [],
            "patches": [patch["patch_id"].as_str().expect("patch id should exist")],
            "state_hash": state_hash,
            "timestamp": 2u64
        }),
        "author",
        "revision_id",
        "rev",
    );
    let revision_id = revision["revision_id"]
        .as_str()
        .expect("revision id should exist")
        .to_string();
    fs::write(
        &revision_path,
        serde_json::to_string_pretty(&revision).expect("revision should serialize"),
    )
    .expect("revision should write");

    let policy = json!({
        "accept_keys": [signer_id(&signing_key())],
        "merge_rule": "manual-reviewed",
        "preferred_branches": ["main"]
    });
    let profile_id = profile_id(&policy);
    let view = signed_object(
        json!({
            "type": "view",
            "version": "mycel/0.1",
            "documents": {
                "doc:index": revision["revision_id"].as_str().expect("revision id should exist")
            },
            "policy": policy,
            "timestamp": 3u64
        }),
        "maintainer",
        "view_id",
        "view",
    );
    let view_id = view["view_id"]
        .as_str()
        .expect("view id should exist")
        .to_string();
    fs::write(
        &view_path,
        serde_json::to_string_pretty(&view).expect("view should serialize"),
    )
    .expect("view should write");

    let ingest = run_mycel(&[
        "store",
        "ingest",
        &path_arg(source_dir.path()),
        "--into",
        &path_arg(store_dir.path()),
    ]);
    assert_success(&ingest);

    StoreFixtureInfo {
        source_dir,
        store_dir,
        signer: signer_id(&signing_key()),
        revision_id,
        view_id,
        profile_id,
    }
}

fn build_store_with_related_views() -> RelatedGovernanceFixtureInfo {
    let source_dir = create_temp_dir("store-index-related-source");
    let store_dir = create_temp_dir("store-index-related-root");

    let policy_a = json!({
        "accept_keys": [signer_id(&signing_key())],
        "merge_rule": "manual-reviewed",
        "preferred_branches": ["main"]
    });
    let policy_b = json!({
        "accept_keys": [signer_id(&SigningKey::from_bytes(&[8u8; 32]))],
        "merge_rule": "manual-reviewed",
        "preferred_branches": ["main"]
    });

    let view_a1 = signed_object(
        json!({
            "type": "view",
            "version": "mycel/0.1",
            "documents": {
                "doc:alpha": "rev:aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa",
                "doc:beta": "rev:bbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbb"
            },
            "policy": policy_a,
            "timestamp": 10u64
        }),
        "maintainer",
        "view_id",
        "view",
    );
    let view_a2 = signed_object(
        json!({
            "type": "view",
            "version": "mycel/0.1",
            "documents": {
                "doc:alpha": "rev:cccccccccccccccccccccccccccccccccccccccccccccccccccccccccccccccc"
            },
            "policy": policy_a,
            "timestamp": 11u64
        }),
        "maintainer",
        "view_id",
        "view",
    );
    let view_b1 = signed_object(
        json!({
            "type": "view",
            "version": "mycel/0.1",
            "documents": {
                "doc:beta": "rev:dddddddddddddddddddddddddddddddddddddddddddddddddddddddddddddddd"
            },
            "policy": policy_b,
            "timestamp": 12u64
        }),
        "maintainer",
        "view_id",
        "view",
    );

    let view_a1_id = view_a1["view_id"]
        .as_str()
        .expect("view a1 id should exist")
        .to_string();
    let view_a2_id = view_a2["view_id"]
        .as_str()
        .expect("view a2 id should exist")
        .to_string();
    let view_b1_id = view_b1["view_id"]
        .as_str()
        .expect("view b1 id should exist")
        .to_string();

    fs::write(
        source_dir.path().join("view-a1.json"),
        serde_json::to_string_pretty(&view_a1).expect("view a1 should serialize"),
    )
    .expect("view a1 should write");
    fs::write(
        source_dir.path().join("view-a2.json"),
        serde_json::to_string_pretty(&view_a2).expect("view a2 should serialize"),
    )
    .expect("view a2 should write");
    fs::write(
        source_dir.path().join("view-b1.json"),
        serde_json::to_string_pretty(&view_b1).expect("view b1 should serialize"),
    )
    .expect("view b1 should write");

    let ingest = run_mycel(&[
        "store",
        "ingest",
        &path_arg(source_dir.path()),
        "--into",
        &path_arg(store_dir.path()),
    ]);
    assert_success(&ingest);

    RelatedGovernanceFixtureInfo {
        store_dir,
        view_a1_id,
        view_a2_id,
        view_b1_id,
    }
}

#[test]
fn store_index_json_reads_persisted_manifest() {
    let fixture = build_store_with_view();
    let signer = fixture.signer.clone();
    let profile_id = fixture.profile_id.clone();

    let output = run_mycel(&[
        "store",
        "index",
        &path_arg(fixture.store_dir.path()),
        "--json",
    ]);

    assert_success(&output);
    let json = assert_json_status(&output, "ok");
    assert_eq!(json["stored_object_count"], 3);
    assert!(
        json["object_ids_by_type"]["patch"]
            .as_array()
            .is_some_and(|values| values.len() == 1),
        "expected patch index, stdout: {}",
        stdout_text(&output)
    );
    assert!(
        json["doc_revisions"]["doc:index"]
            .as_array()
            .is_some_and(|values| values.len() == 1),
        "expected doc revision index, stdout: {}",
        stdout_text(&output)
    );
    assert!(
        json["author_patches"][signer.as_str()]
            .as_array()
            .is_some_and(|values| values.len() == 1),
        "expected author patch index, stdout: {}",
        stdout_text(&output)
    );
    assert!(
        json["maintainer_views"][signer.as_str()]
            .as_array()
            .is_some_and(|values| values.len() == 1),
        "expected maintainer view index, stdout: {}",
        stdout_text(&output)
    );
    assert!(
        json["profile_views"][profile_id.as_str()]
            .as_array()
            .is_some_and(|values| values.len() == 1),
        "expected profile view index, stdout: {}",
        stdout_text(&output)
    );
    assert!(
        json["document_views"]["doc:index"]
            .as_array()
            .is_some_and(|values| values.len() == 1),
        "expected document view index, stdout: {}",
        stdout_text(&output)
    );
    assert!(
        json["profile_heads"][profile_id.as_str()]["doc:index"]
            .as_array()
            .is_some_and(|values| values.len() == 1),
        "expected profile head index, stdout: {}",
        stdout_text(&output)
    );
}

#[test]
fn store_index_json_filters_common_indexes() {
    let fixture = build_store_with_view();
    let signer = fixture.signer.clone();
    let profile_id = fixture.profile_id.clone();

    let output = run_mycel(&[
        "store",
        "index",
        &path_arg(fixture.store_dir.path()),
        "--doc-id",
        "doc:index",
        "--author",
        &signer,
        "--maintainer",
        &signer,
        "--profile-id",
        &profile_id,
        "--object-type",
        "patch",
        "--json",
    ]);

    assert_success(&output);
    let json = assert_json_status(&output, "ok");
    assert_eq!(json["filters"]["doc_id"], "doc:index");
    assert_eq!(json["filters"]["author"], signer);
    assert_eq!(json["filters"]["maintainer"], fixture.signer);
    assert_eq!(json["filters"]["profile_id"], profile_id);
    assert_eq!(json["filters"]["object_type"], "patch");
    assert_eq!(
        json["object_ids_by_type"]
            .as_object()
            .map(|values| values.len()),
        Some(1)
    );
    assert!(
        json["object_ids_by_type"]["patch"]
            .as_array()
            .is_some_and(|values| values.len() == 1),
        "expected filtered patch object index, stdout: {}",
        stdout_text(&output)
    );
    assert_eq!(
        json["doc_revisions"].as_object().map(|values| values.len()),
        Some(1)
    );
    assert_eq!(
        json["author_patches"]
            .as_object()
            .map(|values| values.len()),
        Some(1)
    );
    assert_eq!(
        json["profile_heads"].as_object().map(|values| values.len()),
        Some(1)
    );
    assert_eq!(
        json["maintainer_views"]
            .as_object()
            .map(|values| values.len()),
        Some(1)
    );
    assert_eq!(
        json["profile_views"].as_object().map(|values| values.len()),
        Some(1)
    );
    assert_eq!(
        json["document_views"]
            .as_object()
            .map(|values| values.len()),
        Some(1)
    );
    assert_eq!(
        json["view_governance"]
            .as_array()
            .map(|values| values.len()),
        Some(1)
    );
}

#[test]
fn store_index_json_filters_by_revision_and_view() {
    let fixture = build_store_with_view();
    let signer = fixture.signer.clone();
    let profile_id = fixture.profile_id.clone();

    let output = run_mycel(&[
        "store",
        "index",
        &path_arg(fixture.store_dir.path()),
        "--revision-id",
        &fixture.revision_id,
        "--view-id",
        &fixture.view_id,
        "--json",
    ]);

    assert_success(&output);
    let json = assert_json_status(&output, "ok");
    assert_eq!(json["filters"]["revision_id"], fixture.revision_id);
    assert_eq!(json["filters"]["view_id"], fixture.view_id);
    assert!(
        json["revision_parents"][fixture.revision_id]
            .as_array()
            .is_some_and(|values| values.is_empty()),
        "expected revision parent entry, stdout: {}",
        stdout_text(&output)
    );
    assert_eq!(
        json["view_governance"]
            .as_array()
            .map(|values| values.len()),
        Some(1)
    );
    assert_eq!(
        json["maintainer_views"][signer.as_str()]
            .as_array()
            .map(|values| values.len()),
        Some(1)
    );
    assert_eq!(
        json["profile_views"][profile_id.as_str()]
            .as_array()
            .map(|values| values.len()),
        Some(1)
    );
    assert_eq!(
        json["document_views"]["doc:index"]
            .as_array()
            .map(|values| values.len()),
        Some(1)
    );
    assert_eq!(json["view_governance"][0]["view_id"], fixture.view_id);
    assert!(
        json["profile_heads"][profile_id.as_str()]["doc:index"]
            .as_array()
            .is_some_and(|values| values.len() == 1),
        "expected filtered profile head index, stdout: {}",
        stdout_text(&output)
    );
}

#[test]
fn store_index_text_reports_summary() {
    let fixture = build_store_with_view();

    let output = run_mycel(&[
        "store",
        "index",
        &path_arg(fixture.store_dir.path()),
        "--doc-id",
        "doc:index",
    ]);

    assert_success(&output);
    assert_empty_stderr(&output);
    let stdout = stdout_text(&output);
    assert!(
        stdout.contains("document revision indexes: 1"),
        "stdout: {stdout}"
    );
    assert!(
        stdout.contains("filter doc_id: doc:index"),
        "stdout: {stdout}"
    );
    assert!(stdout.contains("store index: ok"), "stdout: {stdout}");
}

#[test]
fn store_index_path_only_prints_manifest_path() {
    let fixture = build_store_with_view();

    let output = run_mycel(&[
        "store",
        "index",
        &path_arg(fixture.store_dir.path()),
        "--path-only",
    ]);

    assert_success(&output);
    assert_empty_stderr(&output);
    assert_eq!(
        stdout_text(&output).trim(),
        fixture
            .store_dir
            .path()
            .join("indexes")
            .join("manifest.json")
            .to_string_lossy()
    );
    let _ = fixture.source_dir.path();
}

#[test]
fn store_index_path_only_rejects_json() {
    let fixture = build_store_with_view();
    let output = run_mycel(&[
        "store",
        "index",
        &path_arg(fixture.store_dir.path()),
        "--path-only",
        "--json",
    ]);

    assert_exit_code(&output, 2);
    assert_stderr_contains(
        &output,
        "store index --path-only cannot be used with --json",
    );
}

#[test]
fn store_index_filters_only_json_emits_query_metadata() {
    let fixture = build_store_with_view();
    let output = run_mycel(&[
        "store",
        "index",
        &path_arg(fixture.store_dir.path()),
        "--doc-id",
        "doc:index",
        "--head-only",
        "--filters-only",
        "--json",
    ]);

    assert_success(&output);
    let json = assert_json_status(&output, "ok");
    let object = json
        .as_object()
        .expect("filters-only output should be a JSON object");
    assert_eq!(json["filters"]["doc_id"], "doc:index");
    assert_eq!(json["projection"], "head-only");
    assert!(
        !object.contains_key("doc_revisions"),
        "filters-only output should omit full indexes, stdout: {}",
        stdout_text(&output)
    );
    assert!(
        !object.contains_key("profile_heads"),
        "filters-only output should omit profile heads, stdout: {}",
        stdout_text(&output)
    );
}

#[test]
fn store_index_counts_only_json_emits_section_counts() {
    let fixture = build_store_with_view();
    let output = run_mycel(&[
        "store",
        "index",
        &path_arg(fixture.store_dir.path()),
        "--counts-only",
        "--json",
    ]);

    assert_success(&output);
    let json = assert_json_status(&output, "ok");
    let object = json
        .as_object()
        .expect("counts-only output should be a JSON object");
    assert_eq!(json["stored_object_count"], 3);
    assert_eq!(json["object_type_index_count"], 3);
    assert_eq!(json["document_revision_index_count"], 1);
    assert_eq!(json["revision_parent_index_count"], 1);
    assert_eq!(json["author_patch_index_count"], 1);
    assert_eq!(json["view_governance_record_count"], 1);
    assert_eq!(json["maintainer_view_index_count"], 1);
    assert_eq!(json["profile_view_index_count"], 1);
    assert_eq!(json["document_view_index_count"], 1);
    assert_eq!(json["profile_head_index_count"], 1);
    assert!(
        !object.contains_key("object_ids_by_type"),
        "counts-only output should omit full indexes, stdout: {}",
        stdout_text(&output)
    );
}

#[test]
fn store_index_manifest_only_json_emits_manifest_metadata() {
    let fixture = build_store_with_view();
    let output = run_mycel(&[
        "store",
        "index",
        &path_arg(fixture.store_dir.path()),
        "--manifest-only",
        "--json",
    ]);

    assert_success(&output);
    let json = assert_json_status(&output, "ok");
    let object = json
        .as_object()
        .expect("manifest-only output should be a JSON object");
    assert_eq!(json["version"], "mycel-store-index/0.1");
    assert_eq!(json["stored_object_count"], 3);
    assert_eq!(json["object_type_count"], 3);
    assert_eq!(
        json["manifest_path"],
        fixture
            .store_dir
            .path()
            .join("indexes")
            .join("manifest.json")
            .to_string_lossy()
            .as_ref()
    );
    assert!(
        !object.contains_key("filters"),
        "manifest-only output should omit query filters, stdout: {}",
        stdout_text(&output)
    );
}

#[test]
fn store_index_doc_only_json_prunes_other_sections() {
    let fixture = build_store_with_view();
    let output = run_mycel(&[
        "store",
        "index",
        &path_arg(fixture.store_dir.path()),
        "--doc-only",
        "--json",
    ]);

    assert_success(&output);
    let json = assert_json_status(&output, "ok");
    assert_eq!(json["projection"], "doc-only");
    assert_eq!(
        json["doc_revisions"].as_object().map(|values| values.len()),
        Some(1)
    );
    assert_eq!(
        json["revision_parents"]
            .as_object()
            .map(|values| values.len()),
        Some(0)
    );
    assert_eq!(
        json["view_governance"]
            .as_array()
            .map(|values| values.len()),
        Some(0)
    );
    assert_eq!(
        json["maintainer_views"]
            .as_object()
            .map(|values| values.len()),
        Some(0)
    );
    assert_eq!(
        json["profile_views"].as_object().map(|values| values.len()),
        Some(0)
    );
    assert_eq!(
        json["document_views"]
            .as_object()
            .map(|values| values.len()),
        Some(0)
    );
    assert_eq!(
        json["profile_heads"].as_object().map(|values| values.len()),
        Some(0)
    );
}

#[test]
fn store_index_governance_only_json_prunes_non_governance_sections() {
    let fixture = build_store_with_view();
    let output = run_mycel(&[
        "store",
        "index",
        &path_arg(fixture.store_dir.path()),
        "--governance-only",
        "--json",
    ]);

    assert_success(&output);
    let json = assert_json_status(&output, "ok");
    assert_eq!(json["projection"], "governance-only");
    assert_eq!(
        json["view_governance"]
            .as_array()
            .map(|values| values.len()),
        Some(1)
    );
    assert_eq!(
        json["maintainer_views"]
            .as_object()
            .map(|values| values.len()),
        Some(1)
    );
    assert_eq!(
        json["profile_views"].as_object().map(|values| values.len()),
        Some(1)
    );
    assert_eq!(
        json["document_views"]
            .as_object()
            .map(|values| values.len()),
        Some(1)
    );
    assert_eq!(
        json["profile_heads"].as_object().map(|values| values.len()),
        Some(1)
    );
    assert_eq!(
        json["doc_revisions"].as_object().map(|values| values.len()),
        Some(0)
    );
    assert_eq!(
        json["revision_parents"]
            .as_object()
            .map(|values| values.len()),
        Some(0)
    );
    assert_eq!(
        json["view_governance"][0]["maintainer_view_ids"],
        json!([fixture.view_id.clone()])
    );
    assert_eq!(
        json["view_governance"][0]["profile_view_ids"],
        json!([fixture.view_id.clone()])
    );
    assert_eq!(
        json["view_governance"][0]["document_view_ids"]["doc:index"],
        json!([fixture.view_id])
    );
}

#[test]
fn store_index_governance_only_json_embeds_related_view_context_per_record() {
    let fixture = build_store_with_related_views();
    let output = run_mycel(&[
        "store",
        "index",
        &path_arg(fixture.store_dir.path()),
        "--governance-only",
        "--view-id",
        &fixture.view_a1_id,
        "--json",
    ]);

    assert_success(&output);
    let json = assert_json_status(&output, "ok");
    assert_eq!(
        json["view_governance"]
            .as_array()
            .map(|values| values.len()),
        Some(1)
    );
    assert_eq!(
        json["view_governance"][0]["maintainer_view_ids"],
        json!([fixture.view_a1_id, fixture.view_b1_id, fixture.view_a2_id])
    );
    assert_eq!(
        json["view_governance"][0]["profile_view_ids"],
        json!([fixture.view_a1_id, fixture.view_a2_id])
    );
    assert_eq!(
        json["view_governance"][0]["document_view_ids"]["doc:alpha"],
        json!([fixture.view_a1_id, fixture.view_a2_id])
    );
    assert_eq!(
        json["view_governance"][0]["document_view_ids"]["doc:beta"],
        json!([fixture.view_a1_id, fixture.view_b1_id])
    );
}

#[test]
fn store_index_governance_only_text_reports_related_view_context() {
    let fixture = build_store_with_related_views();
    let output = run_mycel(&[
        "store",
        "index",
        &path_arg(fixture.store_dir.path()),
        "--governance-only",
        "--view-id",
        &fixture.view_a1_id,
    ]);

    assert_success(&output);
    assert_empty_stderr(&output);
    let stdout = stdout_text(&output);
    assert!(
        stdout.contains(&format!("view governance record: {}", fixture.view_a1_id)),
        "stdout: {stdout}"
    );
    assert!(
        stdout.contains(&format!(
            "  maintainer related views: {}, {}, {}",
            fixture.view_a1_id, fixture.view_b1_id, fixture.view_a2_id
        )),
        "stdout: {stdout}"
    );
    assert!(
        stdout.contains(&format!(
            "  document related views: doc:beta -> {}, {}",
            fixture.view_a1_id, fixture.view_b1_id
        )),
        "stdout: {stdout}"
    );
}

#[test]
fn store_index_head_only_json_prunes_non_head_sections() {
    let fixture = build_store_with_view();
    let output = run_mycel(&[
        "store",
        "index",
        &path_arg(fixture.store_dir.path()),
        "--head-only",
        "--json",
    ]);

    assert_success(&output);
    let json = assert_json_status(&output, "ok");
    assert_eq!(json["projection"], "head-only");
    assert_eq!(
        json["profile_heads"].as_object().map(|values| values.len()),
        Some(1)
    );
    assert_eq!(
        json["author_patches"]
            .as_object()
            .map(|values| values.len()),
        Some(0)
    );
    assert_eq!(
        json["view_governance"]
            .as_array()
            .map(|values| values.len()),
        Some(0)
    );
    assert_eq!(
        json["maintainer_views"]
            .as_object()
            .map(|values| values.len()),
        Some(0)
    );
    assert_eq!(
        json["profile_views"].as_object().map(|values| values.len()),
        Some(0)
    );
    assert_eq!(
        json["document_views"]
            .as_object()
            .map(|values| values.len()),
        Some(0)
    );
}

#[test]
fn store_index_patches_only_json_prunes_non_patch_sections() {
    let fixture = build_store_with_view();
    let output = run_mycel(&[
        "store",
        "index",
        &path_arg(fixture.store_dir.path()),
        "--patches-only",
        "--json",
    ]);

    assert_success(&output);
    let json = assert_json_status(&output, "ok");
    assert_eq!(json["projection"], "patches-only");
    assert_eq!(
        json["author_patches"]
            .as_object()
            .map(|values| values.len()),
        Some(1)
    );
    assert_eq!(
        json["profile_heads"].as_object().map(|values| values.len()),
        Some(0)
    );
    assert_eq!(
        json["doc_revisions"].as_object().map(|values| values.len()),
        Some(0)
    );
}

#[test]
fn store_index_parents_only_text_reports_projection() {
    let fixture = build_store_with_view();
    let output = run_mycel(&[
        "store",
        "index",
        &path_arg(fixture.store_dir.path()),
        "--parents-only",
    ]);

    assert_success(&output);
    assert_empty_stderr(&output);
    let stdout = stdout_text(&output);
    assert!(
        stdout.contains("projection: parents-only"),
        "stdout: {stdout}"
    );
    assert!(
        stdout.contains("revision parent indexes: 1"),
        "stdout: {stdout}"
    );
    assert!(
        stdout.contains("document revision indexes: 0"),
        "stdout: {stdout}"
    );
    assert!(
        stdout.contains("view governance records: 0"),
        "stdout: {stdout}"
    );
    assert!(
        stdout.contains("maintainer view indexes: 0"),
        "stdout: {stdout}"
    );
}

#[test]
fn store_index_empty_query_fails_without_empty_ok() {
    let fixture = build_store_with_view();
    let output = run_mycel(&[
        "store",
        "index",
        &path_arg(fixture.store_dir.path()),
        "--doc-id",
        "doc:missing",
        "--json",
    ]);

    assert_exit_code(&output, 1);
    let json = assert_json_status(&output, "empty");
    assert_eq!(
        json["doc_revisions"].as_object().map(|values| values.len()),
        Some(0)
    );
}

#[test]
fn store_index_empty_query_succeeds_with_empty_ok() {
    let fixture = build_store_with_view();
    let output = run_mycel(&[
        "store",
        "index",
        &path_arg(fixture.store_dir.path()),
        "--doc-id",
        "doc:missing",
        "--empty-ok",
        "--json",
    ]);

    assert_success(&output);
    let json = assert_json_status(&output, "ok");
    assert_eq!(
        json["doc_revisions"].as_object().map(|values| values.len()),
        Some(0)
    );
}

#[test]
fn store_index_rejects_multiple_projection_flags() {
    let fixture = build_store_with_view();
    let output = run_mycel(&[
        "store",
        "index",
        &path_arg(fixture.store_dir.path()),
        "--doc-only",
        "--head-only",
        "--governance-only",
    ]);

    assert_exit_code(&output, 2);
    assert_stderr_contains(
        &output,
        "store index projection flags are mutually exclusive",
    );
}

#[test]
fn store_index_rejects_multiple_output_modes() {
    let fixture = build_store_with_view();
    let output = run_mycel(&[
        "store",
        "index",
        &path_arg(fixture.store_dir.path()),
        "--filters-only",
        "--counts-only",
    ]);

    assert_exit_code(&output, 2);
    assert_stderr_contains(
        &output,
        "store index output mode flags are mutually exclusive",
    );
}

#[test]
fn store_index_missing_manifest_fails_cleanly() {
    let store_dir = create_temp_dir("store-index-missing");
    let output = run_mycel(&["store", "index", &path_arg(store_dir.path())]);

    assert_exit_code(&output, 2);
    assert_stderr_contains(&output, "failed to read store index manifest");
}
