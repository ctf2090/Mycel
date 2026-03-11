use super::fixtures::*;
use super::*;

#[test]
fn protocol_spec_examples_parse() {
    parse_document_object(&protocol_spec_document_example())
        .expect("document example should parse");
    parse_block_object(&protocol_spec_block_example()).expect("block example should parse");
    parse_patch_object(&protocol_spec_patch_example()).expect("patch example should parse");
    parse_revision_object(&protocol_spec_revision_example())
        .expect("revision example should parse");
    parse_revision_object(&protocol_spec_merge_revision_example())
        .expect("merge revision example should parse");
    parse_view_object(&protocol_spec_view_example()).expect("view example should parse");
    parse_snapshot_object(&protocol_spec_snapshot_example())
        .expect("snapshot example should parse");
}

#[rstest]
#[case(protocol_spec_patch_example(), "patch_id", "patch")]
#[case(protocol_spec_revision_example(), "revision_id", "rev")]
#[case(protocol_spec_merge_revision_example(), "revision_id", "rev")]
#[case(protocol_spec_view_example(), "view_id", "view")]
#[case(protocol_spec_snapshot_example(), "snapshot_id", "snap")]
fn protocol_spec_examples_recompute_derived_ids(
    #[case] value: Value,
    #[case] id_field: &str,
    #[case] prefix: &str,
) {
    let recomputed =
        recompute_object_id(&value, id_field, prefix).expect("derived ID should recompute");

    assert!(recomputed.starts_with(&format!("{prefix}:")));
    assert_ne!(
        recomputed,
        value[id_field]
            .as_str()
            .expect("example derived ID should be present"),
    );
}

#[rstest]
#[case(wire_protocol_hello_example())]
#[case(wire_protocol_manifest_example())]
#[case(wire_protocol_heads_example())]
#[case(wire_protocol_want_example())]
#[case(wire_protocol_object_example())]
#[case(wire_protocol_snapshot_offer_example())]
#[case(wire_protocol_view_announce_example())]
#[case(wire_protocol_bye_example())]
#[case(wire_protocol_error_example())]
fn wire_protocol_spec_examples_have_valid_envelope_shape(#[case] value: Value) {
    let (message_type, payload) =
        parse_wire_envelope_example(&value).expect("wire example envelope should parse");

    validate_wire_payload_example(&message_type, &payload)
        .expect("wire example payload should validate");
}

#[test]
fn concrete_wire_object_payload_matches_recomputed_object_id_and_hash() {
    let value = concrete_wire_object_example();
    let (message_type, payload) =
        parse_wire_envelope_example(&value).expect("concrete wire envelope should parse");

    assert_eq!(message_type.as_str(), "OBJECT");
    validate_wire_payload_example(&message_type, &payload)
        .expect("concrete wire payload shape should validate");
    validate_wire_object_payload_behavior(&payload)
        .expect("concrete wire OBJECT behavior should validate");
}

#[test]
fn concrete_wire_object_payload_rejects_object_id_mismatch() {
    let mut value = concrete_wire_object_example();
    value["payload"]["object_id"] = Value::String("patch:mismatch".to_string());
    let (_, payload) =
        parse_wire_envelope_example(&value).expect("mismatched wire envelope should parse");

    let error = validate_wire_object_payload_behavior(&payload).unwrap_err();

    assert!(error.contains("OBJECT payload object_id"));
    assert!(error.contains("does not match recomputed"));
}
