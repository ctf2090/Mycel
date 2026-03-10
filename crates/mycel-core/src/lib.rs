//! Core library for Mycel protocol-facing logic.
//!
//! This crate is intentionally small at first. It defines shared identities and
//! workspace-level constants that both the simulator and CLI can depend on.

pub mod head;
pub mod protocol;
pub mod replay;
pub mod verify;

pub const WORKSPACE_NAME: &str = "mycel";

pub fn workspace_banner() -> &'static str {
    "Mycel Rust workspace"
}
