use std::fs;
use std::path::{Path, PathBuf};

use ed25519_dalek::SigningKey;
use mycel_core::author::signer_id;
use serde_json::{json, Value};

mod common;

use common::{
    assert_exit_code, assert_stderr_contains, assert_stdout_contains, assert_success,
    create_temp_dir, parse_json_stdout, recompute_test_object_id, run_mycel, sign_test_value,
};

fn path_arg(path: &Path) -> String {
    path.to_string_lossy().into_owned()
}

fn signing_key(seed: u8) -> SigningKey {
    SigningKey::from_bytes(&[seed; 32])
}

fn signed_view(
    signing_key: &SigningKey,
    policy: &Value,
    documents: Value,
    timestamp: u64,
) -> Value {
    let mut value = json!({
        "type": "view",
        "version": "mycel/0.1",
        "maintainer": signer_id(signing_key),
        "documents": documents,
        "policy": policy,
        "timestamp": timestamp
    });
    let id = recompute_test_object_id(&value, "view_id", "view");
    value["view_id"] = Value::String(id);
    value["signature"] = Value::String(sign_test_value(signing_key, &value));
    value
}

fn signed_patch(signing_key: &SigningKey, doc_id: &str) -> Value {
    let mut value = json!({
        "type": "patch",
        "version": "mycel/0.1",
        "doc_id": doc_id,
        "base_revision": "rev:bbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbb",
        "author": signer_id(signing_key),
        "timestamp": 1u64,
        "ops": []
    });
    let id = recompute_test_object_id(&value, "patch_id", "patch");
    value["patch_id"] = Value::String(id);
    value["signature"] = Value::String(sign_test_value(signing_key, &value));
    value
}

fn documents_value(doc_id: &str, revision_id: &str) -> Value {
    json!({
        doc_id: revision_id
    })
}

fn write_json_file(prefix: &str, name: &str, value: &Value) -> (common::TempDir, PathBuf) {
    let dir = create_temp_dir(prefix);
    let path = dir.path().join(name);
    fs::write(
        &path,
        serde_json::to_string_pretty(value).expect("value should serialize"),
    )
    .expect("value should write");
    (dir, path)
}

fn publish_view(source_path: &Path, store_root: &str) -> Value {
    let output = run_mycel(&[
        "view",
        "publish",
        &path_arg(source_path),
        "--into",
        store_root,
        "--json",
    ]);
    assert_success(&output);
    parse_json_stdout(&output)
}

fn rewrite_store_manifest(store_root: &str, update: impl FnOnce(&mut Value)) {
    let manifest_path = Path::new(store_root).join("indexes").join("manifest.json");
    let mut manifest: Value =
        serde_json::from_str(&fs::read_to_string(&manifest_path).expect("manifest should read"))
            .expect("manifest should parse");
    update(&mut manifest);
    fs::write(
        &manifest_path,
        serde_json::to_string_pretty(&manifest).expect("manifest should serialize"),
    )
    .expect("manifest should write");
}

#[test]
fn view_publish_json_writes_verified_view_into_store() {
    let store_dir = create_temp_dir("view-publish-store");
    let store_root = path_arg(store_dir.path());
    let init = run_mycel(&["store", "init", &store_root, "--json"]);
    assert_success(&init);

    let maintainer = signing_key(41);
    let policy = json!({
        "accept_keys": [signer_id(&maintainer)],
        "merge_rule": "manual-reviewed",
        "preferred_branches": ["main"]
    });
    let view = signed_view(
        &maintainer,
        &policy,
        documents_value(
            "doc:view-publish",
            "rev:aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa",
        ),
        10,
    );
    let (_source_dir, source_path) = write_json_file("view-publish-source", "view.json", &view);

    let json = publish_view(&source_path, &store_root);
    assert_eq!(json["status"], "ok");
    assert_eq!(json["view_id"], view["view_id"]);
    assert_eq!(json["maintainer"], view["maintainer"]);
    assert_eq!(
        json["documents"]["doc:view-publish"],
        "rev:aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa"
    );
    assert_eq!(json["created"], Value::Bool(true));
    assert!(
        json["profile_id"]
            .as_str()
            .is_some_and(|value| value.starts_with("hash:")),
        "expected hashed profile id, published summary: {json}",
    );
    assert_eq!(
        json["maintainer_view_ids"],
        json!([view["view_id"].as_str().expect("view id should exist")])
    );
    assert_eq!(
        json["profile_view_ids"],
        json!([view["view_id"].as_str().expect("view id should exist")])
    );
    assert_eq!(
        json["document_view_ids"]["doc:view-publish"],
        json!([view["view_id"].as_str().expect("view id should exist")])
    );
    assert!(
        json["notes"]
            .as_array()
            .is_some_and(|notes| notes.iter().any(|note| note
                == "related maintainer/profile/document view IDs come from persisted governance indexes")),
        "expected persisted-governance note in publish summary: {json}",
    );

    let inspect = run_mycel(&[
        "view",
        "inspect",
        view["view_id"].as_str().expect("view id should exist"),
        "--store-root",
        &store_root,
        "--json",
    ]);
    assert_success(&inspect);
    let inspect_json = parse_json_stdout(&inspect);
    assert_eq!(inspect_json["status"], "ok");
    assert_eq!(inspect_json["view_id"], view["view_id"]);
    assert_eq!(
        inspect_json["profile_heads"]["doc:view-publish"],
        json!(["rev:aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa"])
    );
    assert_eq!(
        inspect_json["maintainer_view_ids"],
        json!([view["view_id"].as_str().expect("view id should exist")])
    );
    assert_eq!(
        inspect_json["profile_view_ids"],
        json!([view["view_id"].as_str().expect("view id should exist")])
    );
    assert_eq!(
        inspect_json["document_view_ids"]["doc:view-publish"],
        json!([view["view_id"].as_str().expect("view id should exist")])
    );
}

#[test]
fn view_publish_reports_existing_view_on_repeat_publish() {
    let store_dir = create_temp_dir("view-publish-repeat-store");
    let store_root = path_arg(store_dir.path());
    let init = run_mycel(&["store", "init", &store_root, "--json"]);
    assert_success(&init);

    let maintainer = signing_key(42);
    let policy = json!({
        "accept_keys": [signer_id(&maintainer)],
        "merge_rule": "manual-reviewed",
        "preferred_branches": ["main"]
    });
    let view = signed_view(
        &maintainer,
        &policy,
        documents_value(
            "doc:view-repeat",
            "rev:cccccccccccccccccccccccccccccccccccccccccccccccccccccccccccccccc",
        ),
        11,
    );
    let (_source_dir, source_path) = write_json_file("view-publish-repeat", "view.json", &view);

    let first = run_mycel(&[
        "view",
        "publish",
        &path_arg(&source_path),
        "--into",
        &store_root,
        "--json",
    ]);
    assert_success(&first);

    let second = run_mycel(&[
        "view",
        "publish",
        &path_arg(&source_path),
        "--into",
        &store_root,
        "--json",
    ]);
    assert_success(&second);
    let json = parse_json_stdout(&second);
    assert_eq!(json["status"], "ok");
    assert_eq!(json["created"], Value::Bool(false));
}

#[test]
fn view_list_json_filters_governance_records() {
    let store_dir = create_temp_dir("view-list-store");
    let store_root = path_arg(store_dir.path());
    let init = run_mycel(&["store", "init", &store_root, "--json"]);
    assert_success(&init);

    let maintainer_a = signing_key(51);
    let maintainer_b = signing_key(52);
    let policy_a = json!({
        "accept_keys": [signer_id(&maintainer_a)],
        "merge_rule": "manual-reviewed",
        "preferred_branches": ["main"]
    });
    let policy_b = json!({
        "accept_keys": [signer_id(&maintainer_b)],
        "merge_rule": "manual-reviewed",
        "preferred_branches": ["stable"]
    });

    let view_a1 = signed_view(
        &maintainer_a,
        &policy_a,
        documents_value(
            "doc:alpha",
            "rev:1111111111111111111111111111111111111111111111111111111111111111",
        ),
        10,
    );
    let view_a2 = signed_view(
        &maintainer_a,
        &policy_a,
        documents_value(
            "doc:beta",
            "rev:2222222222222222222222222222222222222222222222222222222222222222",
        ),
        11,
    );
    let view_b1 = signed_view(
        &maintainer_b,
        &policy_b,
        json!({
            "doc:alpha": "rev:3333333333333333333333333333333333333333333333333333333333333333",
            "doc:gamma": "rev:4444444444444444444444444444444444444444444444444444444444444444"
        }),
        12,
    );

    let (_dir_a1, path_a1) = write_json_file("view-list-a1", "view-a1.json", &view_a1);
    let (_dir_a2, path_a2) = write_json_file("view-list-a2", "view-a2.json", &view_a2);
    let (_dir_b1, path_b1) = write_json_file("view-list-b1", "view-b1.json", &view_b1);

    let publish_a1 = publish_view(&path_a1, &store_root);
    let _publish_a2 = publish_view(&path_a2, &store_root);
    let publish_b1 = publish_view(&path_b1, &store_root);

    let all = run_mycel(&["view", "list", "--store-root", &store_root, "--json"]);
    assert_success(&all);
    let all_json = parse_json_stdout(&all);
    assert_eq!(all_json["record_count"], 3);
    assert!(all_json["records"][0]["maintainer_view_ids"]
        .as_array()
        .is_some_and(|values| !values.is_empty()));
    assert!(all_json["records"][0]["profile_view_ids"]
        .as_array()
        .is_some_and(|values| !values.is_empty()));
    assert!(all_json["records"][0]["document_view_ids"]
        .as_object()
        .is_some_and(|values| !values.is_empty()));

    let by_profile = run_mycel(&[
        "view",
        "list",
        "--store-root",
        &store_root,
        "--profile-id",
        publish_a1["profile_id"]
            .as_str()
            .expect("profile id should exist"),
        "--json",
    ]);
    assert_success(&by_profile);
    let by_profile_json = parse_json_stdout(&by_profile);
    assert_eq!(by_profile_json["record_count"], 2);

    let by_maintainer = run_mycel(&[
        "view",
        "list",
        "--store-root",
        &store_root,
        "--maintainer",
        view_b1["maintainer"]
            .as_str()
            .expect("maintainer should exist"),
        "--json",
    ]);
    assert_success(&by_maintainer);
    let by_maintainer_json = parse_json_stdout(&by_maintainer);
    assert_eq!(by_maintainer_json["record_count"], 1);
    assert_eq!(
        by_maintainer_json["records"][0]["view_id"],
        view_b1["view_id"]
    );

    let by_doc = run_mycel(&[
        "view",
        "list",
        "--store-root",
        &store_root,
        "--doc-id",
        "doc:alpha",
        "--json",
    ]);
    assert_success(&by_doc);
    let by_doc_json = parse_json_stdout(&by_doc);
    assert_eq!(by_doc_json["record_count"], 2);

    let by_revision = run_mycel(&[
        "view",
        "list",
        "--store-root",
        &store_root,
        "--revision-id",
        "rev:2222222222222222222222222222222222222222222222222222222222222222",
        "--json",
    ]);
    assert_success(&by_revision);
    let by_revision_json = parse_json_stdout(&by_revision);
    assert_eq!(by_revision_json["record_count"], 1);
    assert_eq!(
        by_revision_json["records"][0]["view_id"],
        view_a2["view_id"]
    );

    let by_view_id = run_mycel(&[
        "view",
        "list",
        "--store-root",
        &store_root,
        "--view-id",
        publish_b1["view_id"]
            .as_str()
            .expect("view id should exist"),
        "--json",
    ]);
    assert_success(&by_view_id);
    let by_view_id_json = parse_json_stdout(&by_view_id);
    assert_eq!(by_view_id_json["record_count"], 1);
    assert_eq!(by_view_id_json["records"][0]["view_id"], view_b1["view_id"]);
    assert_eq!(
        by_view_id_json["records"][0]["maintainer_view_ids"],
        json!([publish_b1["view_id"]
            .as_str()
            .expect("view id should exist")])
    );
}

#[test]
fn view_list_json_supports_sorting_time_windows_and_grouped_summaries() {
    let store_dir = create_temp_dir("view-list-summary-store");
    let store_root = path_arg(store_dir.path());
    let init = run_mycel(&["store", "init", &store_root, "--json"]);
    assert_success(&init);

    let maintainer_a = signing_key(61);
    let maintainer_b = signing_key(62);
    let policy_a = json!({
        "accept_keys": [signer_id(&maintainer_a)],
        "merge_rule": "manual-reviewed",
        "preferred_branches": ["main"]
    });
    let policy_b = json!({
        "accept_keys": [signer_id(&maintainer_b)],
        "merge_rule": "manual-reviewed",
        "preferred_branches": ["stable"]
    });

    let view_a1 = signed_view(
        &maintainer_a,
        &policy_a,
        json!({
            "doc:alpha": "rev:aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa",
            "doc:beta": "rev:bbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbb"
        }),
        10,
    );
    let view_a2 = signed_view(
        &maintainer_a,
        &policy_a,
        documents_value(
            "doc:alpha",
            "rev:cccccccccccccccccccccccccccccccccccccccccccccccccccccccccccccccc",
        ),
        20,
    );
    let view_b1 = signed_view(
        &maintainer_b,
        &policy_b,
        documents_value(
            "doc:gamma",
            "rev:dddddddddddddddddddddddddddddddddddddddddddddddddddddddddddddddd",
        ),
        30,
    );

    let (_dir_a1, path_a1) = write_json_file("view-list-summary-a1", "view-a1.json", &view_a1);
    let (_dir_a2, path_a2) = write_json_file("view-list-summary-a2", "view-a2.json", &view_a2);
    let (_dir_b1, path_b1) = write_json_file("view-list-summary-b1", "view-b1.json", &view_b1);

    let publish_a1 = publish_view(&path_a1, &store_root);
    let publish_b1 = publish_view(&path_b1, &store_root);
    let publish_a2 = publish_view(&path_a2, &store_root);

    let output = run_mycel(&[
        "view",
        "list",
        "--store-root",
        &store_root,
        "--timestamp-min",
        "15",
        "--timestamp-max",
        "30",
        "--sort",
        "timestamp-desc",
        "--group-by",
        "profile-id",
        "--group-by",
        "maintainer",
        "--group-by",
        "doc-id",
        "--json",
    ]);
    assert_success(&output);

    let json = parse_json_stdout(&output);
    assert_eq!(json["status"], "ok");
    assert_eq!(json["sort"], "timestamp-desc");
    assert_eq!(json["record_count"], 2);
    assert_eq!(json["filters"]["timestamp_min"], 15);
    assert_eq!(json["filters"]["timestamp_max"], 30);
    assert_eq!(json["records"][0]["timestamp"], 30);
    assert_eq!(json["records"][1]["timestamp"], 20);
    assert_eq!(json["records"][0]["view_id"], view_b1["view_id"]);
    assert_eq!(json["records"][1]["view_id"], view_a2["view_id"]);
    assert_eq!(
        json["records"][0]["current_profile_view_id"],
        view_b1["view_id"]
    );
    assert_eq!(
        json["records"][1]["current_profile_view_id"],
        view_a2["view_id"]
    );
    assert_eq!(
        json["records"][1]["current_profile_document_view_ids"]["doc:alpha"],
        view_a2["view_id"]
    );
    assert_eq!(
        json["records"][1]["current_profile_document_view_ids"]["doc:beta"],
        view_a1["view_id"]
    );

    let groups = json["groups"]
        .as_array()
        .expect("groups should be an array");
    assert_eq!(groups.len(), 3);

    let by_profile = groups
        .iter()
        .find(|group| group["group_by"] == "profile-id")
        .expect("expected profile-id grouping");
    assert_eq!(
        by_profile["groups"]
            .as_array()
            .expect("profile groups")
            .len(),
        2
    );
    assert_eq!(by_profile["groups"][0]["record_count"], 1);
    assert_eq!(by_profile["groups"][1]["record_count"], 1);

    let by_maintainer = groups
        .iter()
        .find(|group| group["group_by"] == "maintainer")
        .expect("expected maintainer grouping");
    assert_eq!(
        by_maintainer["groups"]
            .as_array()
            .expect("maintainer groups")
            .len(),
        2
    );
    assert_eq!(by_maintainer["groups"][0]["latest_timestamp"], 30);
    assert_eq!(by_maintainer["groups"][1]["latest_timestamp"], 20);

    let by_doc = groups
        .iter()
        .find(|group| group["group_by"] == "doc-id")
        .expect("expected doc-id grouping");
    let doc_groups = by_doc["groups"].as_array().expect("doc groups");
    assert_eq!(doc_groups.len(), 2);
    let alpha_group = doc_groups
        .iter()
        .find(|group| group["key"] == "doc:alpha")
        .expect("expected doc:alpha group");
    assert_eq!(alpha_group["record_count"], 1);
    let gamma_group = doc_groups
        .iter()
        .find(|group| group["key"] == "doc:gamma")
        .expect("expected doc:gamma group");
    assert_eq!(gamma_group["latest_timestamp"], 30);

    let by_profile_only = run_mycel(&[
        "view",
        "list",
        "--store-root",
        &store_root,
        "--profile-id",
        publish_a1["profile_id"]
            .as_str()
            .expect("profile id should exist"),
        "--sort",
        "profile-id",
        "--json",
    ]);
    assert_success(&by_profile_only);
    let by_profile_only_json = parse_json_stdout(&by_profile_only);
    assert_eq!(by_profile_only_json["record_count"], 2);
    assert_eq!(by_profile_only_json["records"][0]["timestamp"], 20);
    assert_eq!(by_profile_only_json["records"][1]["timestamp"], 10);
    assert_eq!(
        by_profile_only_json["records"][0]["current_profile_view_id"],
        publish_a2["view_id"]
    );
    assert_eq!(
        by_profile_only_json["records"][0]["current_profile_document_view_ids"]["doc:alpha"],
        publish_a2["view_id"]
    );
    assert_eq!(
        by_profile_only_json["records"][0]["current_profile_document_view_ids"]["doc:beta"],
        view_a1["view_id"]
    );

    let by_profile_only_text = run_mycel(&[
        "view",
        "list",
        "--store-root",
        &store_root,
        "--profile-id",
        publish_a1["profile_id"]
            .as_str()
            .expect("profile id should exist"),
        "--sort",
        "profile-id",
    ]);
    assert_success(&by_profile_only_text);
    assert_stdout_contains(
        &by_profile_only_text,
        &format!(
            "  current profile view id: {}",
            publish_a2["view_id"]
                .as_str()
                .expect("view a2 id should exist")
        ),
    );
    assert_stdout_contains(
        &by_profile_only_text,
        &format!(
            "  current profile document view: doc:beta -> {}",
            view_a1["view_id"]
                .as_str()
                .expect("view a1 id should exist")
        ),
    );

    let by_view_id = run_mycel(&[
        "view",
        "list",
        "--store-root",
        &store_root,
        "--view-id",
        publish_b1["view_id"]
            .as_str()
            .expect("view id should exist"),
        "--timestamp-min",
        "31",
    ]);
    assert_exit_code(&by_view_id, 0);
    assert_stdout_contains(&by_view_id, "record count: 0");
    assert_stdout_contains(&by_view_id, "view list: ok");
}

#[test]
fn view_list_json_supports_limit_latest_per_profile_and_summary_only() {
    let store_dir = create_temp_dir("view-list-projection-store");
    let store_root = path_arg(store_dir.path());
    let init = run_mycel(&["store", "init", &store_root, "--json"]);
    assert_success(&init);

    let maintainer_a = signing_key(71);
    let maintainer_b = signing_key(72);
    let policy_a = json!({
        "accept_keys": [signer_id(&maintainer_a)],
        "merge_rule": "manual-reviewed",
        "preferred_branches": ["main"]
    });
    let policy_b = json!({
        "accept_keys": [signer_id(&maintainer_b)],
        "merge_rule": "manual-reviewed",
        "preferred_branches": ["stable"]
    });

    let view_a1 = signed_view(
        &maintainer_a,
        &policy_a,
        documents_value(
            "doc:alpha",
            "rev:1111111111111111111111111111111111111111111111111111111111111111",
        ),
        10,
    );
    let view_a2 = signed_view(
        &maintainer_a,
        &policy_a,
        documents_value(
            "doc:beta",
            "rev:2222222222222222222222222222222222222222222222222222222222222222",
        ),
        30,
    );
    let view_b1 = signed_view(
        &maintainer_b,
        &policy_b,
        documents_value(
            "doc:gamma",
            "rev:3333333333333333333333333333333333333333333333333333333333333333",
        ),
        20,
    );

    let (_dir_a1, path_a1) = write_json_file("view-list-projection-a1", "view-a1.json", &view_a1);
    let (_dir_a2, path_a2) = write_json_file("view-list-projection-a2", "view-a2.json", &view_a2);
    let (_dir_b1, path_b1) = write_json_file("view-list-projection-b1", "view-b1.json", &view_b1);

    let publish_a1 = publish_view(&path_a1, &store_root);
    let _publish_a2 = publish_view(&path_a2, &store_root);
    let _publish_b1 = publish_view(&path_b1, &store_root);

    let latest_per_profile = run_mycel(&[
        "view",
        "list",
        "--store-root",
        &store_root,
        "--latest-per-profile",
        "--sort",
        "timestamp-desc",
        "--json",
    ]);
    assert_success(&latest_per_profile);
    let latest_per_profile_json = parse_json_stdout(&latest_per_profile);
    assert_eq!(latest_per_profile_json["record_count"], 2);
    assert_eq!(latest_per_profile_json["latest_per_profile"], true);
    assert_eq!(latest_per_profile_json["records"][0]["timestamp"], 30);
    assert_eq!(latest_per_profile_json["records"][1]["timestamp"], 20);
    assert_eq!(
        latest_per_profile_json["records"][0]["profile_id"],
        publish_a1["profile_id"]
    );
    assert_ne!(
        latest_per_profile_json["records"][0]["profile_id"],
        latest_per_profile_json["records"][1]["profile_id"]
    );

    let summary_only = run_mycel(&[
        "view",
        "list",
        "--store-root",
        &store_root,
        "--latest-per-profile",
        "--sort",
        "timestamp-desc",
        "--limit",
        "1",
        "--summary-only",
        "--group-by",
        "profile-id",
        "--json",
    ]);
    assert_success(&summary_only);
    let summary_only_json = parse_json_stdout(&summary_only);
    assert_eq!(summary_only_json["record_count"], 1);
    assert_eq!(summary_only_json["summary_only"], true);
    assert_eq!(summary_only_json["limit"], 1);
    assert_eq!(summary_only_json["latest_per_profile"], true);
    assert_eq!(
        summary_only_json["records"]
            .as_array()
            .expect("records should be an array")
            .len(),
        0
    );
    let grouped = summary_only_json["groups"]
        .as_array()
        .expect("groups should be an array");
    assert_eq!(grouped.len(), 1);
    assert_eq!(grouped[0]["group_by"], "profile-id");
    assert_eq!(grouped[0]["groups"][0]["record_count"], 1);
    assert_eq!(grouped[0]["groups"][0]["latest_timestamp"], 30);
    assert_eq!(
        grouped[0]["groups"][0]["key"],
        publish_a1["profile_id"]
            .as_str()
            .expect("profile id should exist")
    );
}

#[test]
fn view_list_current_profile_fields_fall_back_to_latest_indexes_when_current_governance_is_missing()
{
    let store_dir = create_temp_dir("view-list-current-fallback-store");
    let store_root = path_arg(store_dir.path());
    let init = run_mycel(&["store", "init", &store_root, "--json"]);
    assert_success(&init);

    let maintainer = signing_key(81);
    let policy = json!({
        "accept_keys": [signer_id(&maintainer)],
        "merge_rule": "manual-reviewed",
        "preferred_branches": ["main"]
    });
    let view_a1 = signed_view(
        &maintainer,
        &policy,
        documents_value(
            "doc:alpha",
            "rev:aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa",
        ),
        10,
    );
    let view_a2 = signed_view(
        &maintainer,
        &policy,
        json!({
            "doc:alpha": "rev:cccccccccccccccccccccccccccccccccccccccccccccccccccccccccccccccc",
            "doc:beta": "rev:bbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbb"
        }),
        20,
    );

    let (_dir_a1, path_a1) =
        write_json_file("view-list-current-fallback-a1", "view-a1.json", &view_a1);
    let (_dir_a2, path_a2) =
        write_json_file("view-list-current-fallback-a2", "view-a2.json", &view_a2);

    let publish_a1 = publish_view(&path_a1, &store_root);
    let publish_a2 = publish_view(&path_a2, &store_root);

    rewrite_store_manifest(&store_root, |manifest| {
        manifest["current_governance"] = json!({});
    });

    let output = run_mycel(&[
        "view",
        "list",
        "--store-root",
        &store_root,
        "--profile-id",
        publish_a1["profile_id"]
            .as_str()
            .expect("profile id should exist"),
        "--sort",
        "profile-id",
        "--json",
    ]);
    assert_success(&output);
    let json = parse_json_stdout(&output);
    assert_eq!(json["record_count"], 2);
    assert_eq!(
        json["records"][0]["current_profile_view_id"],
        publish_a2["view_id"]
    );
    assert_eq!(
        json["records"][0]["current_profile_document_view_ids"]["doc:alpha"],
        publish_a2["view_id"]
    );
    assert_eq!(
        json["records"][0]["current_profile_document_view_ids"]["doc:beta"],
        publish_a2["view_id"]
    );
}

#[test]
fn view_publish_rejects_non_view_object() {
    let store_dir = create_temp_dir("view-publish-invalid-store");
    let store_root = path_arg(store_dir.path());
    let init = run_mycel(&["store", "init", &store_root, "--json"]);
    assert_success(&init);

    let patch = signed_patch(&signing_key(43), "doc:not-view");
    let (_source_dir, source_path) =
        write_json_file("view-publish-invalid-source", "patch.json", &patch);

    let output = run_mycel(&[
        "view",
        "publish",
        &path_arg(&source_path),
        "--into",
        &store_root,
    ]);

    assert_exit_code(&output, 1);
    assert_stdout_contains(&output, "view publish: failed");
    assert_stderr_contains(&output, "view publish source is not a valid view object");
}

#[test]
fn view_list_rejects_inverted_timestamp_window() {
    let store_dir = create_temp_dir("view-list-bad-window-store");
    let store_root = path_arg(store_dir.path());
    let init = run_mycel(&["store", "init", &store_root, "--json"]);
    assert_success(&init);

    let output = run_mycel(&[
        "view",
        "list",
        "--store-root",
        &store_root,
        "--timestamp-min",
        "20",
        "--timestamp-max",
        "10",
    ]);

    assert_exit_code(&output, 2);
    assert_stderr_contains(
        &output,
        "view list timestamp-min cannot be greater than timestamp-max",
    );
}

#[test]
fn view_inspect_reports_missing_view_id() {
    let store_dir = create_temp_dir("view-inspect-missing-store");
    let store_root = path_arg(store_dir.path());
    let init = run_mycel(&["store", "init", &store_root, "--json"]);
    assert_success(&init);

    let output = run_mycel(&[
        "view",
        "inspect",
        "view:missing",
        "--store-root",
        &store_root,
    ]);

    assert_exit_code(&output, 1);
    assert_stdout_contains(&output, "view inspection: failed");
    assert_stderr_contains(&output, "was not found in persisted governance indexes");
}

#[test]
fn view_current_json_reports_profile_current_governance_state() {
    let store_dir = create_temp_dir("view-current-profile-store");
    let store_root = path_arg(store_dir.path());
    let init = run_mycel(&["store", "init", &store_root, "--json"]);
    assert_success(&init);

    let maintainer_a = signing_key(71);
    let maintainer_b = signing_key(72);
    let policy = json!({
        "accept_keys": [signer_id(&maintainer_a)],
        "merge_rule": "manual-reviewed",
        "preferred_branches": ["main"]
    });

    let view_a1 = signed_view(
        &maintainer_a,
        &policy,
        json!({
            "doc:alpha": "rev:1111111111111111111111111111111111111111111111111111111111111111",
            "doc:beta": "rev:2222222222222222222222222222222222222222222222222222222222222222"
        }),
        10,
    );
    let view_a2 = signed_view(
        &maintainer_b,
        &policy,
        json!({
            "doc:alpha": "rev:3333333333333333333333333333333333333333333333333333333333333333"
        }),
        20,
    );

    let (_dir_a1, path_a1) = write_json_file("view-current-profile-a1", "view-a1.json", &view_a1);
    let (_dir_a2, path_a2) = write_json_file("view-current-profile-a2", "view-a2.json", &view_a2);

    publish_view(&path_a1, &store_root);
    let publish_a2 = publish_view(&path_a2, &store_root);

    let current = run_mycel(&[
        "view",
        "current",
        "--store-root",
        &store_root,
        "--profile-id",
        publish_a2["profile_id"]
            .as_str()
            .expect("profile id should exist"),
        "--json",
    ]);
    assert_success(&current);
    let current_json = parse_json_stdout(&current);

    assert_eq!(current_json["status"], "ok");
    assert_eq!(current_json["current_view_id"], publish_a2["view_id"]);
    assert_eq!(
        current_json["profile_current_view_id"],
        publish_a2["view_id"]
    );
    assert_eq!(current_json["timestamp"], json!(20));
    assert_eq!(current_json["maintainer"], view_a2["maintainer"]);
    assert_eq!(
        current_json["current_profile_document_view_ids"]["doc:alpha"],
        publish_a2["view_id"]
    );
    assert_eq!(
        current_json["current_profile_document_view_ids"]["doc:beta"],
        view_a1["view_id"]
    );
    assert_eq!(
        current_json["current_documents"][0]["doc_id"],
        json!("doc:alpha")
    );
    assert_eq!(
        current_json["current_documents"][0]["current_view_id"],
        publish_a2["view_id"]
    );
    assert_eq!(
        current_json["current_documents"][0]["current_revision_id"],
        json!("rev:3333333333333333333333333333333333333333333333333333333333333333")
    );
    assert_eq!(
        current_json["current_documents"][1]["doc_id"],
        json!("doc:beta")
    );
    assert_eq!(
        current_json["current_documents"][1]["current_view_id"],
        view_a1["view_id"]
    );
    assert_eq!(
        current_json["current_documents"][1]["current_revision_id"],
        json!("rev:2222222222222222222222222222222222222222222222222222222222222222")
    );
    assert_eq!(
        current_json["profile_heads"]["doc:alpha"],
        json!([
            "rev:1111111111111111111111111111111111111111111111111111111111111111",
            "rev:3333333333333333333333333333333333333333333333333333333333333333"
        ])
    );
    assert_eq!(
        current_json["profile_heads"]["doc:beta"],
        json!(["rev:2222222222222222222222222222222222222222222222222222222222222222"])
    );
    assert!(
        current_json["notes"].as_array().is_some_and(|notes| notes.iter().any(|note| {
            note == "profile head IDs come from persisted governance head indexes for the selected profile"
        })),
        "expected persisted profile-head note in current summary: {current_json}",
    );
}

#[test]
fn view_current_profile_falls_back_to_latest_indexes_when_current_governance_is_missing() {
    let store_dir = create_temp_dir("view-current-profile-fallback-store");
    let store_root = path_arg(store_dir.path());
    let init = run_mycel(&["store", "init", &store_root, "--json"]);
    assert_success(&init);

    let maintainer_a = signing_key(75);
    let maintainer_b = signing_key(76);
    let policy = json!({
        "accept_keys": [signer_id(&maintainer_a)],
        "merge_rule": "manual-reviewed",
        "preferred_branches": ["main"]
    });

    let view_a1 = signed_view(
        &maintainer_a,
        &policy,
        json!({
            "doc:alpha": "rev:1111111111111111111111111111111111111111111111111111111111111111",
            "doc:beta": "rev:2222222222222222222222222222222222222222222222222222222222222222"
        }),
        10,
    );
    let view_a2 = signed_view(
        &maintainer_b,
        &policy,
        json!({
            "doc:alpha": "rev:3333333333333333333333333333333333333333333333333333333333333333"
        }),
        20,
    );

    let (_dir_a1, path_a1) =
        write_json_file("view-current-profile-fallback-a1", "view-a1.json", &view_a1);
    let (_dir_a2, path_a2) =
        write_json_file("view-current-profile-fallback-a2", "view-a2.json", &view_a2);

    publish_view(&path_a1, &store_root);
    let publish_a2 = publish_view(&path_a2, &store_root);

    rewrite_store_manifest(&store_root, |manifest| {
        manifest["current_governance"] = json!({});
    });

    let current = run_mycel(&[
        "view",
        "current",
        "--store-root",
        &store_root,
        "--profile-id",
        publish_a2["profile_id"]
            .as_str()
            .expect("profile id should exist"),
        "--json",
    ]);
    assert_success(&current);
    let current_json = parse_json_stdout(&current);

    assert_eq!(current_json["status"], "ok");
    assert_eq!(current_json["current_view_id"], publish_a2["view_id"]);
    assert_eq!(
        current_json["profile_current_view_id"],
        publish_a2["view_id"]
    );
    assert_eq!(current_json["timestamp"], json!(20));
    assert_eq!(current_json["maintainer"], view_a2["maintainer"]);
    assert_eq!(
        current_json["current_profile_document_view_ids"]["doc:alpha"],
        publish_a2["view_id"]
    );
    assert_eq!(
        current_json["current_profile_document_view_ids"]["doc:beta"],
        view_a1["view_id"]
    );
}

#[test]
fn view_current_doc_scope_falls_back_to_latest_indexes_when_current_governance_is_missing() {
    let store_dir = create_temp_dir("view-current-doc-fallback-store");
    let store_root = path_arg(store_dir.path());
    let init = run_mycel(&["store", "init", &store_root, "--json"]);
    assert_success(&init);

    let maintainer = signing_key(77);
    let policy = json!({
        "accept_keys": [signer_id(&maintainer)],
        "merge_rule": "manual-reviewed",
        "preferred_branches": ["main"]
    });

    let view_a1 = signed_view(
        &maintainer,
        &policy,
        json!({
            "doc:alpha": "rev:aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa",
            "doc:beta": "rev:bbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbb"
        }),
        10,
    );
    let view_a2 = signed_view(
        &maintainer,
        &policy,
        json!({
            "doc:alpha": "rev:cccccccccccccccccccccccccccccccccccccccccccccccccccccccccccccccc"
        }),
        20,
    );

    let (_dir_a1, path_a1) =
        write_json_file("view-current-doc-fallback-a1", "view-a1.json", &view_a1);
    let (_dir_a2, path_a2) =
        write_json_file("view-current-doc-fallback-a2", "view-a2.json", &view_a2);

    let publish_a1 = publish_view(&path_a1, &store_root);
    let publish_a2 = publish_view(&path_a2, &store_root);

    rewrite_store_manifest(&store_root, |manifest| {
        manifest["current_governance"] = json!({});
    });

    let current = run_mycel(&[
        "view",
        "current",
        "--store-root",
        &store_root,
        "--profile-id",
        publish_a2["profile_id"]
            .as_str()
            .expect("profile id should exist"),
        "--doc-id",
        "doc:beta",
        "--json",
    ]);
    assert_success(&current);
    let current_json = parse_json_stdout(&current);

    assert_eq!(current_json["status"], "ok");
    assert_eq!(current_json["current_view_id"], publish_a1["view_id"]);
    assert_eq!(
        current_json["profile_current_view_id"],
        publish_a2["view_id"]
    );
    assert_eq!(
        current_json["current_document_revision_id"],
        json!("rev:bbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbb")
    );
    assert_eq!(current_json["maintainer"], view_a1["maintainer"]);
    assert_eq!(current_json["timestamp"], json!(10));
}

#[test]
fn view_current_json_reports_doc_scoped_current_governance_state() {
    let store_dir = create_temp_dir("view-current-doc-store");
    let store_root = path_arg(store_dir.path());
    let init = run_mycel(&["store", "init", &store_root, "--json"]);
    assert_success(&init);

    let maintainer = signing_key(73);
    let policy = json!({
        "accept_keys": [signer_id(&maintainer)],
        "merge_rule": "manual-reviewed",
        "preferred_branches": ["main"]
    });

    let view_a1 = signed_view(
        &maintainer,
        &policy,
        json!({
            "doc:alpha": "rev:aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa",
            "doc:beta": "rev:bbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbb"
        }),
        10,
    );
    let view_a2 = signed_view(
        &maintainer,
        &policy,
        json!({
            "doc:alpha": "rev:cccccccccccccccccccccccccccccccccccccccccccccccccccccccccccccccc"
        }),
        20,
    );

    let (_dir_a1, path_a1) = write_json_file("view-current-doc-a1", "view-a1.json", &view_a1);
    let (_dir_a2, path_a2) = write_json_file("view-current-doc-a2", "view-a2.json", &view_a2);

    let publish_a1 = publish_view(&path_a1, &store_root);
    let publish_a2 = publish_view(&path_a2, &store_root);

    let current = run_mycel(&[
        "view",
        "current",
        "--store-root",
        &store_root,
        "--profile-id",
        publish_a2["profile_id"]
            .as_str()
            .expect("profile id should exist"),
        "--doc-id",
        "doc:beta",
        "--json",
    ]);
    assert_success(&current);
    let current_json = parse_json_stdout(&current);

    assert_eq!(current_json["status"], "ok");
    assert_eq!(current_json["current_view_id"], publish_a1["view_id"]);
    assert_eq!(
        current_json["profile_current_view_id"],
        publish_a2["view_id"]
    );
    assert_eq!(
        current_json["current_document_revision_id"],
        json!("rev:bbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbb")
    );
    assert_eq!(
        current_json["documents"]["doc:beta"],
        json!("rev:bbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbb")
    );
    assert_eq!(
        current_json["current_documents"][1]["doc_id"],
        json!("doc:beta")
    );
    assert_eq!(
        current_json["current_documents"][1]["current_view_id"],
        publish_a1["view_id"]
    );
    assert_eq!(
        current_json["profile_heads"]["doc:alpha"],
        json!([
            "rev:aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa",
            "rev:cccccccccccccccccccccccccccccccccccccccccccccccccccccccccccccccc"
        ])
    );
    assert_eq!(
        current_json["profile_heads"]["doc:beta"],
        json!(["rev:bbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbb"])
    );
}

#[test]
fn view_current_reports_missing_profile_or_doc_cleanly() {
    let store_dir = create_temp_dir("view-current-missing-store");
    let store_root = path_arg(store_dir.path());
    let init = run_mycel(&["store", "init", &store_root, "--json"]);
    assert_success(&init);

    let missing_profile = run_mycel(&[
        "view",
        "current",
        "--store-root",
        &store_root,
        "--profile-id",
        "hash:missing",
    ]);
    assert_exit_code(&missing_profile, 1);
    assert_stdout_contains(&missing_profile, "view current: failed");
    assert_stderr_contains(
        &missing_profile,
        "was not found in persisted current governance state",
    );

    let maintainer = signing_key(74);
    let policy = json!({
        "accept_keys": [signer_id(&maintainer)],
        "merge_rule": "manual-reviewed",
        "preferred_branches": ["main"]
    });
    let view = signed_view(
        &maintainer,
        &policy,
        documents_value(
            "doc:known",
            "rev:dddddddddddddddddddddddddddddddddddddddddddddddddddddddddddddddd",
        ),
        10,
    );
    let (_dir, path) = write_json_file("view-current-missing-doc", "view.json", &view);
    let publish = publish_view(&path, &store_root);

    let missing_doc = run_mycel(&[
        "view",
        "current",
        "--store-root",
        &store_root,
        "--profile-id",
        publish["profile_id"]
            .as_str()
            .expect("profile id should exist"),
        "--doc-id",
        "doc:missing",
    ]);
    assert_exit_code(&missing_doc, 1);
    assert_stdout_contains(&missing_doc, "view current: failed");
    assert_stderr_contains(
        &missing_doc,
        "was not found in persisted current governance state for profile",
    );
}

#[test]
fn view_inspect_json_reports_related_governance_indexes() {
    let store_dir = create_temp_dir("view-inspect-related-store");
    let store_root = path_arg(store_dir.path());
    let init = run_mycel(&["store", "init", &store_root, "--json"]);
    assert_success(&init);

    let maintainer_a = signing_key(61);
    let maintainer_b = signing_key(62);
    let policy_a = json!({
        "accept_keys": [signer_id(&maintainer_a)],
        "merge_rule": "manual-reviewed",
        "preferred_branches": ["main"]
    });
    let policy_b = json!({
        "accept_keys": [signer_id(&maintainer_b)],
        "merge_rule": "manual-reviewed",
        "preferred_branches": ["stable"]
    });

    let view_a1 = signed_view(
        &maintainer_a,
        &policy_a,
        json!({
            "doc:alpha": "rev:1111111111111111111111111111111111111111111111111111111111111111",
            "doc:beta": "rev:2222222222222222222222222222222222222222222222222222222222222222"
        }),
        10,
    );
    let view_a2 = signed_view(
        &maintainer_a,
        &policy_a,
        json!({
            "doc:alpha": "rev:3333333333333333333333333333333333333333333333333333333333333333"
        }),
        11,
    );
    let view_b1 = signed_view(
        &maintainer_b,
        &policy_b,
        json!({
            "doc:beta": "rev:4444444444444444444444444444444444444444444444444444444444444444"
        }),
        12,
    );

    let (_dir_a1, path_a1) = write_json_file("view-inspect-related-a1", "view-a1.json", &view_a1);
    let (_dir_a2, path_a2) = write_json_file("view-inspect-related-a2", "view-a2.json", &view_a2);
    let (_dir_b1, path_b1) = write_json_file("view-inspect-related-b1", "view-b1.json", &view_b1);

    let publish_a1 = publish_view(&path_a1, &store_root);
    let publish_a2 = publish_view(&path_a2, &store_root);
    let publish_b1 = publish_view(&path_b1, &store_root);

    let inspect = run_mycel(&[
        "view",
        "inspect",
        view_a1["view_id"].as_str().expect("view id should exist"),
        "--store-root",
        &store_root,
        "--json",
    ]);
    assert_success(&inspect);
    let inspect_json = parse_json_stdout(&inspect);

    assert_eq!(
        inspect_json["maintainer_view_ids"],
        json!([
            publish_a1["view_id"]
                .as_str()
                .expect("view a1 id should exist"),
            publish_a2["view_id"]
                .as_str()
                .expect("view a2 id should exist")
        ])
    );
    assert_eq!(
        inspect_json["profile_view_ids"],
        json!([
            publish_a1["view_id"]
                .as_str()
                .expect("view a1 id should exist"),
            publish_a2["view_id"]
                .as_str()
                .expect("view a2 id should exist")
        ])
    );
    assert_eq!(
        inspect_json["document_view_ids"]["doc:alpha"],
        json!([
            publish_a1["view_id"]
                .as_str()
                .expect("view a1 id should exist"),
            publish_a2["view_id"]
                .as_str()
                .expect("view a2 id should exist")
        ])
    );
    assert_eq!(
        inspect_json["document_view_ids"]["doc:beta"],
        json!([
            publish_a1["view_id"]
                .as_str()
                .expect("view a1 id should exist"),
            publish_b1["view_id"]
                .as_str()
                .expect("view b1 id should exist")
        ])
    );
    assert_eq!(inspect_json["timestamp"], json!(10));
    assert_eq!(
        inspect_json["current_profile_view_id"],
        publish_a2["view_id"]
    );
    assert_eq!(
        inspect_json["current_profile_document_view_ids"]["doc:alpha"],
        publish_a2["view_id"]
    );
    assert_eq!(
        inspect_json["current_profile_document_view_ids"]["doc:beta"],
        publish_a1["view_id"]
    );
}
