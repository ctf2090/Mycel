#[allow(unused_imports)]
use serde_json::{json, Value};

#[allow(unused_imports)]
use super::{
    derive_wire_object_payload_identity, parse_wire_envelope, validate_wire_envelope,
    validate_wire_object_payload_behavior, validate_wire_payload, verify_wire_envelope_signature,
    WireMessageType,
};
#[allow(unused_imports)]
use crate::protocol::recompute_declared_object_identity;

#[path = "tests/envelope.rs"]
mod envelope;
#[path = "tests/object_payload.rs"]
mod object_payload;
#[path = "tests/property.rs"]
mod property;
#[path = "tests/session.rs"]
mod session;
#[path = "tests/support/mod.rs"]
mod support;

#[allow(unused_imports)]
use support::*;
