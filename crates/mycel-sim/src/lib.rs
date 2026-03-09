//! Simulator-facing library for fixture, topology, and test orchestration.
//!
//! This crate owns the Rust-side domain model around the existing simulator
//! scaffold under `fixtures/` and `sim/`.

pub mod manifest;
pub mod model;
pub mod validate;

use mycel_core::protocol::ProtocolVersion;

pub fn simulator_banner() -> String {
    let version = ProtocolVersion::default();
    format!(
        "Mycel simulator scaffold (core {}, wire {})",
        version.core, version.wire
    )
}
