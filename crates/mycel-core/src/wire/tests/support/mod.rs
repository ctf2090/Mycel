use std::fs;
use std::path::PathBuf;

use base64::Engine;
use ed25519_dalek::{Signer, SigningKey};
use proptest::prelude::*;
use serde_json::{json, Value};

use crate::canonical::{signed_payload_bytes, wire_envelope_signed_payload_bytes};
use crate::protocol::{recompute_declared_object_identity, recompute_object_id};
use crate::replay::{compute_state_hash, DocumentState};

mod common;
mod messages;
mod objects;
mod session;
mod strategies;

use common::sign_wire_value;
pub(super) use common::{
    empty_state_hash, sender_public_key, sign_object_value, signing_key, temp_dir,
};
pub(super) use messages::{
    hello_envelope_with, signed_bye_message, signed_error_message, signed_heads_message,
    signed_hello_message, signed_hello_message_with_capabilities, signed_manifest_message,
    signed_manifest_message_with_capabilities, signed_manifest_message_with_heads,
    signed_snapshot_offer_message, signed_view_announce_message, signed_want_message,
};
pub(super) use objects::{
    signed_object_message, signed_patch_object_message, signed_revision_object_message,
    valid_object_payload_for_proptests,
};
pub(super) use session::{patch_revision_graph, registered_session};
pub(super) use strategies::{
    invalid_canonical_object_id_strategy, invalid_object_type_strategy,
    invalid_wire_timestamp_strategy, valid_wire_timestamp_strategy,
};
