use proptest::prelude::*;
use serde_json::{json, Value};

use super::*;

proptest! {
    #[test]
    fn validate_wire_timestamp_accepts_generated_rfc3339_shapes(
        timestamp in valid_wire_timestamp_strategy()
    ) {
        prop_assert!(validate_wire_timestamp(&timestamp).is_ok());
    }

    #[test]
    fn validate_wire_timestamp_rejects_generated_non_rfc3339_shapes(
        timestamp in invalid_wire_timestamp_strategy()
    ) {
        prop_assert_eq!(
            validate_wire_timestamp(&timestamp).unwrap_err(),
            "wire envelope 'timestamp' must use RFC 3339 format"
        );
    }

    #[test]
    fn validate_wire_envelope_accepts_generated_hello_top_level_shapes(
        timestamp in valid_wire_timestamp_strategy()
    ) {
        let value = hello_envelope_with(&timestamp);
        prop_assert!(validate_wire_envelope(&value).is_ok());
        let envelope = validate_wire_envelope(&value)
            .expect("validated generated HELLO envelope should parse on the happy path");
        prop_assert_eq!(envelope.message_type(), WireMessageType::Hello);
        prop_assert_eq!(envelope.from(), "node:alpha");
    }

    #[test]
    fn parse_wire_envelope_rejects_invalid_top_level_fields(
        timestamp in valid_wire_timestamp_strategy(),
        invalid_case in prop_oneof![
            Just("bad_msg_id"),
            Just("bad_from"),
            Just("missing_payload"),
            Just("payload_not_object"),
            Just("bad_sig"),
        ]
    ) {
        let mut value = hello_envelope_with(&timestamp);
        match invalid_case {
            "bad_msg_id" => value["msg_id"] = Value::String("hello-proptest-001".to_owned()),
            "bad_from" => value["from"] = Value::String("alpha".to_owned()),
            "missing_payload" => {
                let object = value
                    .as_object_mut()
                    .expect("hello envelope helper should return an object");
                object.remove("payload");
            }
            "payload_not_object" => value["payload"] = Value::String("not-an-object".to_owned()),
            "bad_sig" => value["sig"] = Value::String("placeholder".to_owned()),
            _ => unreachable!("invalid_case strategy produced unexpected discriminator"),
        }

        prop_assert!(parse_wire_envelope(&value).is_err());
    }

    #[test]
    fn validate_wire_object_payload_behavior_rejects_generated_identity_mismatches(
        mismatch_case in prop_oneof![Just("object_id"), Just("hash")]
    ) {
        let mut payload = valid_object_payload_for_proptests();
        match mismatch_case {
            "object_id" => payload["object_id"] = Value::String("patch:wrong-object".to_owned()),
            "hash" => payload["hash"] = Value::String("hash:wrong-hash".to_owned()),
            _ => unreachable!("mismatch_case strategy produced unexpected discriminator"),
        }

        let payload_object = payload
            .as_object()
            .expect("valid OBJECT payload helper should return an object");
        prop_assert!(validate_wire_payload(WireMessageType::Object, payload_object).is_ok());
        prop_assert!(validate_wire_object_payload_behavior(payload_object).is_err());
    }

    #[test]
    fn validate_wire_payload_accepts_generated_want_max_items(
        max_items in any::<u64>()
    ) {
        let payload = json!({
            "objects": ["rev:test"],
            "max_items": max_items
        });
        let payload_object = payload
            .as_object()
            .expect("generated WANT payload should be an object");
        prop_assert!(validate_wire_payload(WireMessageType::Want, payload_object).is_ok());
    }

    #[test]
    fn validate_wire_payload_rejects_generated_invalid_want_max_items(
        invalid_max_items in prop_oneof![
            any::<i64>().prop_filter("negative integers only", |value| *value < 0).prop_map(Value::from),
            any::<bool>().prop_map(Value::from),
            ".*".prop_map(Value::from)
        ]
    ) {
        let payload = json!({
            "objects": ["rev:test"],
            "max_items": invalid_max_items
        });
        let payload_object = payload
            .as_object()
            .expect("generated WANT payload should be an object");
        prop_assert!(validate_wire_payload(WireMessageType::Want, payload_object).is_err());
    }

    #[test]
    fn validate_wire_payload_rejects_generated_invalid_object_encoding(
        invalid_encoding in ".*".prop_filter("encoding must differ from json", |value| value != "json")
    ) {
        let mut payload = valid_object_payload_for_proptests();
        payload["encoding"] = Value::String(invalid_encoding);

        let payload_object = payload
            .as_object()
            .expect("valid OBJECT payload helper should return an object");
        prop_assert_eq!(
            validate_wire_payload(WireMessageType::Object, payload_object).unwrap_err(),
            "OBJECT payload 'encoding' must equal 'json'"
        );
    }

    #[test]
    fn validate_wire_payload_rejects_generated_invalid_object_hash_algorithm(
        invalid_hash_alg in ".*".prop_filter("hash_alg must differ from sha256", |value| value != "sha256")
    ) {
        let mut payload = valid_object_payload_for_proptests();
        payload["hash_alg"] = Value::String(invalid_hash_alg);

        let payload_object = payload
            .as_object()
            .expect("valid OBJECT payload helper should return an object");
        prop_assert_eq!(
            validate_wire_payload(WireMessageType::Object, payload_object).unwrap_err(),
            "OBJECT payload 'hash_alg' must equal 'sha256'"
        );
    }

    #[test]
    fn validate_wire_payload_rejects_generated_unsupported_object_type(
        invalid_object_type in invalid_object_type_strategy()
    ) {
        let mut payload = valid_object_payload_for_proptests();
        payload["object_type"] = Value::String(invalid_object_type);

        let payload_object = payload
            .as_object()
            .expect("valid OBJECT payload helper should return an object");
        prop_assert_eq!(
            validate_wire_payload(WireMessageType::Object, payload_object).unwrap_err(),
            "OBJECT payload 'object_type' must be a supported object type"
        );
    }

    #[test]
    fn validate_wire_payload_rejects_generated_invalid_canonical_object_id(
        invalid_object_id in invalid_canonical_object_id_strategy()
    ) {
        let mut payload = valid_object_payload_for_proptests();
        payload["object_id"] = Value::String(invalid_object_id);

        let payload_object = payload
            .as_object()
            .expect("valid OBJECT payload helper should return an object");
        prop_assert_eq!(
            validate_wire_payload(WireMessageType::Object, payload_object).unwrap_err(),
            "top-level 'object_id' must use a canonical object ID prefix"
        );
    }

    #[test]
    fn validate_wire_payload_rejects_generated_non_object_object_body(
        invalid_body in prop_oneof![
            any::<bool>().prop_map(Value::from),
            any::<i64>().prop_map(Value::from),
            ".*".prop_map(Value::from),
            prop::collection::vec(any::<u8>(), 0..8).prop_map(|bytes| {
                Value::Array(bytes.into_iter().map(Value::from).collect())
            }),
        ]
    ) {
        let mut payload = valid_object_payload_for_proptests();
        payload["body"] = invalid_body;

        let payload_object = payload
            .as_object()
            .expect("valid OBJECT payload helper should return an object");
        prop_assert_eq!(
            validate_wire_payload(WireMessageType::Object, payload_object).unwrap_err(),
            "top-level 'body' must be an object"
        );
    }

    #[test]
    fn validate_wire_object_payload_behavior_rejects_generated_object_type_body_mismatches(
        mismatched_object_type in prop_oneof![Just("revision"), Just("view")]
    ) {
        let mut payload = valid_object_payload_for_proptests();
        payload["object_type"] = Value::String(mismatched_object_type.to_owned());

        let payload_object = payload
            .as_object()
            .expect("valid OBJECT payload helper should return an object");
        prop_assert!(validate_wire_payload(WireMessageType::Object, payload_object).is_ok());

        let error = validate_wire_object_payload_behavior(payload_object)
            .expect_err("mismatched object_type should fail OBJECT payload behavior validation");
        prop_assert!(
            error.contains("OBJECT body type 'patch' does not match object_type"),
            "unexpected mismatch error: {error}"
        );
        prop_assert!(
            error.contains(mismatched_object_type),
            "mismatch error should mention generated object_type, got: {error}"
        );
    }
}
