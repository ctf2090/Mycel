use std::path::PathBuf;

use clap::{Args, Subcommand};
use mycel_core::verify::{
    inspect_object_path, verify_object_path, ObjectInspectionSummary, ObjectVerificationSummary,
};

use crate::{emit_error_line, CliError};

#[derive(Args)]
pub(crate) struct ObjectCliArgs {
    #[command(subcommand)]
    command: Option<ObjectSubcommand>,
}

#[derive(Subcommand)]
enum ObjectSubcommand {
    #[command(about = "Inspect one object file without verifying signatures")]
    Inspect(ObjectInspectCliArgs),
    #[command(about = "Verify one object file")]
    Verify(ObjectVerifyCliArgs),
    #[command(external_subcommand)]
    External(Vec<String>),
}

#[derive(Args)]
struct ObjectInspectCliArgs {
    #[arg(
        value_name = "PATH",
        help = "Object file to inspect",
        required = true,
        allow_hyphen_values = true
    )]
    target: String,
    #[arg(long, help = "Emit machine-readable object inspection output")]
    json: bool,
    #[arg(hide = true, allow_hyphen_values = true)]
    extra: Vec<String>,
}

#[derive(Args)]
struct ObjectVerifyCliArgs {
    #[arg(
        value_name = "PATH",
        help = "Object file to verify",
        required = true,
        allow_hyphen_values = true
    )]
    target: String,
    #[arg(long, help = "Emit machine-readable object verification output")]
    json: bool,
    #[arg(hide = true, allow_hyphen_values = true)]
    extra: Vec<String>,
}

fn print_object_inspection_text(summary: &ObjectInspectionSummary) -> i32 {
    println!("object path: {}", summary.path.display());
    if let Some(object_type) = &summary.object_type {
        println!("object type: {object_type}");
    }
    if let Some(version) = &summary.version {
        println!("version: {version}");
    }
    if let Some(signature_rule) = &summary.signature_rule {
        println!("signature rule: {signature_rule}");
    }
    if let Some(signer_field) = &summary.signer_field {
        println!("signer field: {signer_field}");
    }
    if let Some(signer) = &summary.signer {
        println!("signer: {signer}");
    }
    if let Some(declared_id_field) = &summary.declared_id_field {
        println!("declared id field: {declared_id_field}");
    }
    if let Some(declared_id) = &summary.declared_id {
        println!("declared id: {declared_id}");
    }
    println!(
        "has signature: {}",
        if summary.has_signature { "yes" } else { "no" }
    );
    if !summary.top_level_keys.is_empty() {
        println!("top-level keys: {}", summary.top_level_keys.join(", "));
    }
    println!("status: {}", summary.status);

    for note in &summary.notes {
        println!("note: {note}");
    }

    if summary.is_failed() {
        println!("inspection: failed");
        for error in &summary.errors {
            emit_error_line(error);
        }
        1
    } else {
        println!("inspection: {}", summary.status);
        0
    }
}

fn print_object_inspection_json(summary: &ObjectInspectionSummary) -> Result<i32, CliError> {
    match serde_json::to_string_pretty(summary) {
        Ok(json) => {
            println!("{json}");
            if summary.is_failed() {
                Ok(1)
            } else {
                Ok(0)
            }
        }
        Err(source) => Err(CliError::serialization("object inspection summary", source)),
    }
}

fn object_inspect(target: PathBuf, json: bool) -> Result<i32, CliError> {
    let summary = inspect_object_path(&target);
    if json {
        print_object_inspection_json(&summary)
    } else {
        Ok(print_object_inspection_text(&summary))
    }
}

fn print_object_verification_text(summary: &ObjectVerificationSummary) -> i32 {
    println!("object path: {}", summary.path.display());
    if let Some(object_type) = &summary.object_type {
        println!("object type: {object_type}");
    }
    if let Some(signature_rule) = &summary.signature_rule {
        println!("signature rule: {signature_rule}");
    }
    if let Some(signer_field) = &summary.signer_field {
        println!("signer field: {signer_field}");
    }
    if let Some(signer) = &summary.signer {
        println!("signer: {signer}");
    }
    if let Some(signature_verification) = &summary.signature_verification {
        println!("signature verification: {signature_verification}");
    }
    if let Some(declared_id) = &summary.declared_id {
        println!("declared id: {declared_id}");
    }
    if let Some(recomputed_id) = &summary.recomputed_id {
        println!("recomputed id: {recomputed_id}");
    }
    if let Some(declared_state_hash) = &summary.declared_state_hash {
        println!("declared state hash: {declared_state_hash}");
    }
    if let Some(recomputed_state_hash) = &summary.recomputed_state_hash {
        println!("recomputed state hash: {recomputed_state_hash}");
    }
    if let Some(state_hash_verification) = &summary.state_hash_verification {
        println!("state hash verification: {state_hash_verification}");
    }
    println!("status: {}", summary.status);

    for note in &summary.notes {
        println!("note: {note}");
    }

    if summary.is_ok() {
        println!("verification: ok");
        0
    } else {
        println!("verification: failed");
        for error in &summary.errors {
            emit_error_line(error);
        }
        1
    }
}

fn print_object_verification_json(summary: &ObjectVerificationSummary) -> Result<i32, CliError> {
    match serde_json::to_string_pretty(summary) {
        Ok(json) => {
            println!("{json}");
            if summary.is_ok() {
                Ok(0)
            } else {
                Ok(1)
            }
        }
        Err(source) => Err(CliError::serialization(
            "object verification summary",
            source,
        )),
    }
}

fn object_verify(target: PathBuf, json: bool) -> Result<i32, CliError> {
    let summary = verify_object_path(&target);
    if json {
        print_object_verification_json(&summary)
    } else {
        Ok(print_object_verification_text(&summary))
    }
}

fn unexpected_extra(extra: &[String], context: &str) -> Option<String> {
    extra
        .first()
        .map(|arg| format!("unexpected {context} argument: {arg}"))
}

pub(crate) fn handle_object_command(command: ObjectCliArgs) -> Result<i32, CliError> {
    match command.command {
        Some(ObjectSubcommand::Inspect(args)) => {
            if let Some(message) = unexpected_extra(&args.extra, "object inspect") {
                return Err(CliError::usage(message));
            }

            object_inspect(PathBuf::from(args.target), args.json)
        }
        Some(ObjectSubcommand::Verify(args)) => {
            if let Some(message) = unexpected_extra(&args.extra, "object verify") {
                return Err(CliError::usage(message));
            }

            object_verify(PathBuf::from(args.target), args.json)
        }
        Some(ObjectSubcommand::External(args)) => {
            let other = args.first().map(String::as_str).unwrap_or("<unknown>");
            Err(CliError::usage(format!(
                "unknown object subcommand: {other}"
            )))
        }
        None => Err(CliError::usage("missing object subcommand")),
    }
}
