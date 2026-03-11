use super::fixtures::*;
use super::*;

#[test]
fn object_kind_round_trips_from_strings() {
    let kind = "revision"
        .parse::<ObjectKind>()
        .expect("revision should parse");
    assert_eq!(kind, ObjectKind::Revision);
    assert_eq!(kind.to_string(), "revision");
}

#[test]
fn patch_schema_requires_signature_and_derived_id() {
    let schema = object_schema("patch").expect("patch schema should exist");
    assert_eq!(schema.kind, ObjectKind::Patch);
    assert_eq!(schema.signature_rule, SignatureRule::Required);
    assert_eq!(schema.signer_field, Some("author"));
    assert_eq!(schema.logical_id_field(), None);
    assert_eq!(schema.derived_id(), Some(("patch_id", "patch")));
}

#[test]
fn document_schema_uses_doc_id_as_logical_id() {
    let schema = object_schema("document").expect("document schema should exist");
    assert_eq!(schema.kind, ObjectKind::Document);
    assert_eq!(schema.signature_rule, SignatureRule::Forbidden);
    assert_eq!(schema.logical_id_field(), Some("doc_id"));
    assert_eq!(schema.derived_id(), None);
}

#[test]
fn unknown_object_kind_has_no_schema() {
    assert!(object_schema("unknown-object").is_none());
}

#[test]
fn parse_object_envelope_exposes_schema_and_fields() {
    let value = json!({
        "type": "view",
        "version": "mycel/0.1",
        "view_id": "view:test",
        "maintainer": "pk:ed25519:test"
    });

    let envelope = parse_object_envelope(&value).expect("view envelope should parse");
    assert_eq!(envelope.kind(), ObjectKind::View);
    assert_eq!(envelope.object_type(), "view");
    assert_eq!(envelope.schema().signer_field, Some("maintainer"));
    assert_eq!(
        envelope.optional_string_field("version"),
        Ok(Some("mycel/0.1"))
    );
    assert_eq!(envelope.signer(), Ok(Some("pk:ed25519:test")));
    assert_eq!(envelope.logical_id(), Ok(None));
    assert_eq!(envelope.declared_id(), Ok(Some("view:test")));
}

#[test]
fn parse_document_envelope_exposes_logical_id() {
    let value = json!({
        "type": "document",
        "version": "mycel/0.1",
        "doc_id": "doc:test"
    });

    let envelope = parse_object_envelope(&value).expect("document envelope should parse");
    assert_eq!(envelope.kind(), ObjectKind::Document);
    assert_eq!(envelope.logical_id(), Ok(Some("doc:test")));
    assert_eq!(envelope.declared_id(), Ok(None));
}

#[test]
fn parse_patch_envelope_exposes_typed_derived_id() {
    let value = json!({
        "type": "patch",
        "version": "mycel/0.1",
        "patch_id": "patch:test",
        "author": "pk:ed25519:test"
    });

    let envelope = parse_object_envelope(&value).expect("patch envelope should parse");
    let declared = envelope
        .declared_derived_id()
        .expect("patch derived ID should parse")
        .expect("patch should expose a derived ID");
    assert_eq!(declared.field, "patch_id");
    assert_eq!(declared.prefix, "patch");
    assert_eq!(declared.value, "patch:test");
    assert_eq!(envelope.declared_id(), Ok(Some("patch:test")));
}

#[test]
fn parse_revision_envelope_exposes_typed_derived_id() {
    let value = json!({
        "type": "revision",
        "version": "mycel/0.1",
        "revision_id": "rev:test",
        "author": "pk:ed25519:test"
    });

    let envelope = parse_object_envelope(&value).expect("revision envelope should parse");
    let declared = envelope
        .declared_derived_id()
        .expect("revision derived ID should parse")
        .expect("revision should expose a derived ID");
    assert_eq!(declared.field, "revision_id");
    assert_eq!(declared.prefix, "rev");
    assert_eq!(declared.value, "rev:test");
}

#[test]
fn parse_view_envelope_exposes_typed_derived_id() {
    let value = json!({
        "type": "view",
        "version": "mycel/0.1",
        "view_id": "view:test",
        "maintainer": "pk:ed25519:test"
    });

    let envelope = parse_object_envelope(&value).expect("view envelope should parse");
    let declared = envelope
        .declared_derived_id()
        .expect("view derived ID should parse")
        .expect("view should expose a derived ID");
    assert_eq!(declared.field, "view_id");
    assert_eq!(declared.prefix, "view");
    assert_eq!(declared.value, "view:test");
}

#[test]
fn parse_snapshot_envelope_exposes_typed_derived_id() {
    let value = json!({
        "type": "snapshot",
        "version": "mycel/0.1",
        "snapshot_id": "snap:test",
        "created_by": "pk:ed25519:test"
    });

    let envelope = parse_object_envelope(&value).expect("snapshot envelope should parse");
    let declared = envelope
        .declared_derived_id()
        .expect("snapshot derived ID should parse")
        .expect("snapshot should expose a derived ID");
    assert_eq!(declared.field, "snapshot_id");
    assert_eq!(declared.prefix, "snap");
    assert_eq!(declared.value, "snap:test");
}

#[test]
fn parse_block_envelope_reports_wrong_logical_id_type() {
    let value = json!({
        "type": "block",
        "version": "mycel/0.1",
        "block_id": 7
    });

    let envelope = parse_object_envelope(&value).expect("block envelope should parse");
    assert_eq!(envelope.kind(), ObjectKind::Block);
    assert_eq!(envelope.logical_id(), Err(StringFieldError::WrongType));
}

#[test]
fn parse_document_envelope_reports_wrong_logical_id_type() {
    let value = json!({
        "type": "document",
        "version": "mycel/0.1",
        "doc_id": 7
    });

    let envelope = parse_object_envelope(&value).expect("document envelope should parse");
    assert_eq!(envelope.kind(), ObjectKind::Document);
    assert_eq!(envelope.logical_id(), Err(StringFieldError::WrongType));
}

#[test]
fn parse_patch_envelope_reports_wrong_derived_id_type() {
    let value = json!({
        "type": "patch",
        "version": "mycel/0.1",
        "patch_id": 7,
        "author": "pk:ed25519:test"
    });

    let envelope = parse_object_envelope(&value).expect("patch envelope should parse");
    assert_eq!(
        envelope.declared_derived_id(),
        Err(StringFieldError::WrongType)
    );
}

#[test]
fn parse_object_envelope_rejects_non_object_values() {
    let value = json!(["not-an-object"]);
    assert_eq!(
        parse_object_envelope(&value).unwrap_err(),
        ParseObjectEnvelopeError::TopLevelNotObject
    );
}

#[test]
fn required_string_field_reports_missing_and_wrong_type() {
    let value = json!({
        "type": "document",
        "version": 1
    });

    let envelope = parse_object_envelope(&value).expect("document should parse");
    assert_eq!(
        envelope.optional_string_field("version"),
        Err(StringFieldError::WrongType)
    );
    assert_eq!(
        envelope.required_string_field("missing"),
        Err(StringFieldError::Missing)
    );
}

#[rstest]
#[case("document", "doc_id", json!(7), "top-level 'doc_id' must be a string")]
#[case("block", "block_id", json!(7), "top-level 'block_id' must be a string")]
#[case("patch", "patch_id", json!(7), "top-level 'patch_id' must be a string")]
#[case("revision", "revision_id", json!(7), "top-level 'revision_id' must be a string")]
#[case("view", "view_id", json!(7), "top-level 'view_id' must be a string")]
#[case("snapshot", "snapshot_id", json!(7), "top-level 'snapshot_id' must be a string")]
fn parse_object_rejects_non_string_id_field(
    #[case] kind: &str,
    #[case] id_field: &str,
    #[case] invalid_value: Value,
    #[case] expected_error: &str,
) {
    let mut value = strict_id_case_value(kind);
    value[id_field] = invalid_value;

    assert_eq!(parse_strict_id_case(kind, &value), expected_error);
}

#[rstest]
#[case(
    "document",
    "doc_id",
    "revision:test",
    "top-level 'doc_id' must use 'doc:' prefix"
)]
#[case(
    "block",
    "block_id",
    "paragraph-1",
    "top-level 'block_id' must use 'blk:' prefix"
)]
#[case(
    "patch",
    "patch_id",
    "rev:test",
    "top-level 'patch_id' must use 'patch:' prefix"
)]
#[case(
    "revision",
    "revision_id",
    "patch:test",
    "top-level 'revision_id' must use 'rev:' prefix"
)]
#[case(
    "view",
    "view_id",
    "snap:test",
    "top-level 'view_id' must use 'view:' prefix"
)]
#[case(
    "snapshot",
    "snapshot_id",
    "view:test",
    "top-level 'snapshot_id' must use 'snap:' prefix"
)]

fn parse_object_rejects_wrong_id_prefix(
    #[case] kind: &str,
    #[case] id_field: &str,
    #[case] invalid_value: &str,
    #[case] expected_error: &str,
) {
    let mut value = strict_id_case_value(kind);
    value[id_field] = json!(invalid_value);

    assert_eq!(parse_strict_id_case(kind, &value), expected_error);
}
