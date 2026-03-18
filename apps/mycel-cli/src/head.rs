use std::path::PathBuf;

use clap::{Args, Subcommand, ValueEnum};
use mycel_core::head::{
    inspect_heads_from_path, inspect_heads_from_store_path, render_head_from_path,
    render_head_from_store_path, HeadInspectSummary, HeadRenderSummary,
};

use crate::{emit_error_line, CliError};

#[derive(Args)]
pub(crate) struct HeadCliArgs {
    #[command(subcommand)]
    command: Option<HeadSubcommand>,
}

#[derive(Subcommand)]
enum HeadSubcommand {
    #[command(about = "Inspect one document's accepted head")]
    Inspect(HeadInspectCliArgs),
    #[command(about = "Render one document's accepted text state")]
    Render(HeadRenderCliArgs),
    #[command(external_subcommand)]
    External(Vec<String>),
}

#[derive(Args)]
struct HeadInspectCliArgs {
    #[arg(
        value_name = "DOC_ID",
        help = "Document identifier to inspect",
        required = true,
        allow_hyphen_values = true
    )]
    doc_id: String,
    #[arg(
        long,
        value_name = "PATH_OR_FIXTURE",
        help = "Input bundle path or repo fixture name",
        required = true
    )]
    input: String,
    #[arg(
        long,
        value_name = "STORE_ROOT",
        help = "Load selector revisions and views from a persisted store index"
    )]
    store_root: Option<String>,
    #[arg(
        long,
        value_name = "PROFILE_ID",
        help = "Select one named fixed reader profile from the input bundle"
    )]
    profile_id: Option<String>,
    #[arg(long, help = "Emit machine-readable head inspection output")]
    json: bool,
    #[arg(
        long,
        value_enum,
        default_value_t = HeadOutputMode::Human,
        help = "Text output style for non-JSON output"
    )]
    output_mode: HeadOutputMode,
    #[arg(hide = true, allow_hyphen_values = true)]
    extra: Vec<String>,
}

#[derive(Args)]
struct HeadRenderCliArgs {
    #[arg(
        value_name = "DOC_ID",
        help = "Document identifier to render",
        required = true,
        allow_hyphen_values = true
    )]
    doc_id: String,
    #[arg(
        long,
        value_name = "PATH_OR_FIXTURE",
        help = "Input bundle path or repo fixture name",
        required = true
    )]
    input: String,
    #[arg(
        long,
        value_name = "STORE_ROOT",
        help = "Load selector and replay objects from a persisted store index"
    )]
    store_root: Option<String>,
    #[arg(
        long,
        value_name = "PROFILE_ID",
        help = "Select one named fixed reader profile from the input bundle"
    )]
    profile_id: Option<String>,
    #[arg(long, help = "Emit machine-readable accepted-head render output")]
    json: bool,
    #[arg(
        long,
        value_enum,
        default_value_t = HeadOutputMode::Human,
        help = "Text output style for non-JSON output"
    )]
    output_mode: HeadOutputMode,
    #[arg(hide = true, allow_hyphen_values = true)]
    extra: Vec<String>,
}

#[derive(Copy, Clone, Debug, Eq, PartialEq, ValueEnum)]
enum HeadOutputMode {
    Human,
    Debug,
}

fn print_head_inspect_debug(summary: &HeadInspectSummary) -> i32 {
    println!("input path: {}", summary.input_path.display());
    println!("doc id: {}", summary.doc_id);
    if let Some(profile_id) = &summary.profile_id {
        println!("profile id: {profile_id}");
    }
    if !summary.available_profile_ids.is_empty() {
        println!(
            "available profiles: {}",
            summary.available_profile_ids.join(", ")
        );
    }
    if let Some(effective_selection_time) = summary.effective_selection_time {
        println!("effective selection time: {effective_selection_time}");
    }
    if let Some(selector_epoch) = summary.selector_epoch {
        println!("selector epoch: {selector_epoch}");
    }
    println!("verified revisions: {}", summary.verified_revision_count);
    println!("verified views: {}", summary.verified_view_count);
    if summary.viewer_signal_count > 0 {
        println!("viewer signals: {}", summary.viewer_signal_count);
    }
    println!("status: {}", summary.status);

    for head in &summary.eligible_heads {
        if head.viewer_bonus > 0 || head.viewer_penalty > 0 {
            println!(
                "eligible head: {} timestamp={} score={} supporters={} maintainer_score={} viewer_bonus={} viewer_penalty={}",
                head.revision_id,
                head.revision_timestamp,
                head.selector_score,
                head.supporter_count,
                head.maintainer_score,
                head.viewer_bonus,
                head.viewer_penalty
            );
        } else {
            println!(
                "eligible head: {} timestamp={} score={} supporters={}",
                head.revision_id,
                head.revision_timestamp,
                head.selector_score,
                head.supporter_count
            );
        }
    }

    if let Some(selected_head) = &summary.selected_head {
        println!("selected head: {selected_head}");
    }
    if let Some(tie_break_reason) = &summary.tie_break_reason {
        println!("tie break reason: {tie_break_reason}");
    }
    for channel in &summary.viewer_score_channels {
        if channel.viewer_bonus > 0
            || channel.viewer_penalty > 0
            || channel.challenge_review_pressure > 0
            || channel.challenge_freeze_pressure > 0
        {
            println!(
                "viewer channel: {} maintainer_score={} bonus={} penalty={} challenges={} review_pressure={} freeze_pressure={} review_state={}",
                channel.revision_id,
                channel.maintainer_score,
                channel.viewer_bonus,
                channel.viewer_penalty,
                channel.challenge_signal_count,
                channel.challenge_review_pressure,
                channel.challenge_freeze_pressure,
                serde_json::to_string(&channel.viewer_review_state)
                    .unwrap_or_else(|_| "\"unknown\"".to_string())
                    .trim_matches('"')
            );
        }
    }
    for trace in &summary.decision_trace {
        println!("trace: {}: {}", trace.step, trace.detail);
    }
    for note in &summary.notes {
        println!("note: {note}");
    }

    if summary.is_ok() {
        println!("head inspection: ok");
        0
    } else {
        println!("head inspection: failed");
        for error in &summary.errors {
            emit_error_line(error);
        }
        1
    }
}

fn print_head_inspect_human(summary: &HeadInspectSummary) -> i32 {
    println!(
        "Head inspection: {}",
        if summary.is_ok() { "ok" } else { "failed" }
    );
    println!();
    println!("Document");
    println!("- doc: {}", summary.doc_id);
    println!("- input: {}", summary.input_path.display());
    if let Some(profile_id) = &summary.profile_id {
        println!("- profile: {profile_id}");
    }
    if let Some(effective_selection_time) = summary.effective_selection_time {
        println!("- effective selection time: {effective_selection_time}");
    }
    if let Some(selector_epoch) = summary.selector_epoch {
        println!("- selector epoch: {selector_epoch}");
    }
    println!("- verified revisions: {}", summary.verified_revision_count);
    println!("- verified views: {}", summary.verified_view_count);
    if summary.viewer_signal_count > 0 {
        println!("- viewer signals: {}", summary.viewer_signal_count);
    }
    println!("- status: {}", summary.status);

    if !summary.eligible_heads.is_empty() {
        println!();
        println!("Candidates");
        for head in &summary.eligible_heads {
            let selected_marker = summary
                .selected_head
                .as_ref()
                .is_some_and(|selected| selected == &head.revision_id);
            println!(
                "- {}{}",
                head.revision_id,
                if selected_marker { " (selected)" } else { "" }
            );
            println!(
                "  maintainer support={} maintainer score={} viewer bonus={} viewer penalty={} final score={} timestamp={}",
                head.supporter_count,
                head.maintainer_score,
                head.viewer_bonus,
                head.viewer_penalty,
                head.selector_score,
                head.revision_timestamp
            );
        }
    }

    if !summary.viewer_score_channels.is_empty() {
        println!();
        println!("Viewer Effects");
        for channel in &summary.viewer_score_channels {
            if channel.viewer_bonus > 0
                || channel.viewer_penalty > 0
                || channel.challenge_review_pressure > 0
                || channel.challenge_freeze_pressure > 0
            {
                println!("- {}", channel.revision_id);
                println!(
                    "  maintainer score={} bonus={} penalty={} challenges={} review pressure={} freeze pressure={} state={}",
                    channel.maintainer_score,
                    channel.viewer_bonus,
                    channel.viewer_penalty,
                    channel.challenge_signal_count,
                    channel.challenge_review_pressure,
                    channel.challenge_freeze_pressure,
                    serde_json::to_string(&channel.viewer_review_state)
                        .unwrap_or_else(|_| "\"unknown\"".to_string())
                        .trim_matches('"')
                );
            }
        }
    }

    println!();
    println!("Decision");
    if let Some(selected_head) = &summary.selected_head {
        println!("- selected head: {selected_head}");
    }
    if let Some(tie_break_reason) = &summary.tie_break_reason {
        println!("- reason: {tie_break_reason}");
    }
    if let Some(selection_trace) = summary
        .decision_trace
        .iter()
        .find(|trace| trace.step == "selected_head")
    {
        println!("- trace: {}", selection_trace.detail);
    }

    if !summary.notes.is_empty() {
        println!();
        println!("Notes");
        for note in &summary.notes {
            println!("- {note}");
        }
    }

    if summary.is_ok() {
        0
    } else {
        for error in &summary.errors {
            emit_error_line(error);
        }
        1
    }
}

fn print_head_inspect_text(summary: &HeadInspectSummary, output_mode: HeadOutputMode) -> i32 {
    match output_mode {
        HeadOutputMode::Human => print_head_inspect_human(summary),
        HeadOutputMode::Debug => print_head_inspect_debug(summary),
    }
}

fn print_head_inspect_json(summary: &HeadInspectSummary) -> Result<i32, CliError> {
    match serde_json::to_string_pretty(summary) {
        Ok(json) => {
            println!("{json}");
            if summary.is_ok() {
                Ok(0)
            } else {
                Ok(1)
            }
        }
        Err(source) => Err(CliError::serialization("head inspection summary", source)),
    }
}

fn print_head_render_debug(summary: &HeadRenderSummary) -> i32 {
    println!("input path: {}", summary.input_path.display());
    if let Some(store_root) = &summary.store_root {
        println!("store root: {}", store_root.display());
    }
    println!("doc id: {}", summary.doc_id);
    if let Some(profile_id) = &summary.profile_id {
        println!("profile id: {profile_id}");
    }
    if !summary.available_profile_ids.is_empty() {
        println!(
            "available profiles: {}",
            summary.available_profile_ids.join(", ")
        );
    }
    if let Some(effective_selection_time) = summary.effective_selection_time {
        println!("effective selection time: {effective_selection_time}");
    }
    if let Some(selected_head) = &summary.selected_head {
        println!("selected head: {selected_head}");
    }
    if let Some(recomputed_state_hash) = &summary.recomputed_state_hash {
        println!("recomputed state hash: {recomputed_state_hash}");
    }
    println!("rendered blocks: {}", summary.rendered_block_count);
    if !summary.rendered_text.is_empty() {
        println!("rendered text:");
        println!("{}", summary.rendered_text);
    }
    for note in &summary.notes {
        println!("note: {note}");
    }

    if summary.is_ok() {
        println!("head render: ok");
        0
    } else {
        println!("head render: failed");
        for error in &summary.errors {
            emit_error_line(error);
        }
        1
    }
}

fn print_head_render_human(summary: &HeadRenderSummary) -> i32 {
    println!(
        "Head render: {}",
        if summary.is_ok() { "ok" } else { "failed" }
    );
    println!();
    println!("Document");
    println!("- doc: {}", summary.doc_id);
    println!("- input: {}", summary.input_path.display());
    if let Some(store_root) = &summary.store_root {
        println!("- store root: {}", store_root.display());
    }
    if let Some(profile_id) = &summary.profile_id {
        println!("- profile: {profile_id}");
    }
    if let Some(effective_selection_time) = summary.effective_selection_time {
        println!("- effective selection time: {effective_selection_time}");
    }
    if let Some(selected_head) = &summary.selected_head {
        println!("- selected head: {selected_head}");
    }
    if let Some(recomputed_state_hash) = &summary.recomputed_state_hash {
        println!("- recomputed state hash: {recomputed_state_hash}");
    }
    println!("- rendered blocks: {}", summary.rendered_block_count);
    if !summary.rendered_text.is_empty() {
        println!();
        println!("Rendered Text");
        println!("{}", summary.rendered_text);
    }
    if !summary.notes.is_empty() {
        println!();
        println!("Notes");
        for note in &summary.notes {
            println!("- {note}");
        }
    }

    if summary.is_ok() {
        0
    } else {
        for error in &summary.errors {
            emit_error_line(error);
        }
        1
    }
}

fn print_head_render_text(summary: &HeadRenderSummary, output_mode: HeadOutputMode) -> i32 {
    match output_mode {
        HeadOutputMode::Human => print_head_render_human(summary),
        HeadOutputMode::Debug => print_head_render_debug(summary),
    }
}

fn print_head_render_json(summary: &HeadRenderSummary) -> Result<i32, CliError> {
    match serde_json::to_string_pretty(summary) {
        Ok(json) => {
            println!("{json}");
            if summary.is_ok() {
                Ok(0)
            } else {
                Ok(1)
            }
        }
        Err(source) => Err(CliError::serialization("head render summary", source)),
    }
}

fn head_inspect(
    doc_id: String,
    input_path: PathBuf,
    store_root: Option<PathBuf>,
    profile_id: Option<String>,
    json: bool,
    output_mode: HeadOutputMode,
) -> Result<i32, CliError> {
    let summary = match store_root {
        Some(store_root) => {
            inspect_heads_from_store_path(&input_path, &store_root, &doc_id, profile_id.as_deref())
        }
        None => inspect_heads_from_path(&input_path, &doc_id, profile_id.as_deref()),
    };
    if json {
        print_head_inspect_json(&summary)
    } else {
        Ok(print_head_inspect_text(&summary, output_mode))
    }
}

fn head_render(
    doc_id: String,
    input_path: PathBuf,
    store_root: Option<PathBuf>,
    profile_id: Option<String>,
    json: bool,
    output_mode: HeadOutputMode,
) -> Result<i32, CliError> {
    let summary = match store_root {
        Some(store_root) => {
            render_head_from_store_path(&input_path, &store_root, &doc_id, profile_id.as_deref())
        }
        None => render_head_from_path(&input_path, &doc_id, profile_id.as_deref()),
    };
    if json {
        print_head_render_json(&summary)
    } else {
        Ok(print_head_render_text(&summary, output_mode))
    }
}

fn unexpected_extra(extra: &[String], context: &str) -> Option<String> {
    extra
        .first()
        .map(|arg| format!("unexpected {context} argument: {arg}"))
}

pub(crate) fn handle_head_command(command: HeadCliArgs) -> Result<i32, CliError> {
    match command.command {
        Some(HeadSubcommand::Inspect(args)) => {
            if let Some(message) = unexpected_extra(&args.extra, "head inspect") {
                return Err(CliError::usage(message));
            }

            head_inspect(
                args.doc_id,
                PathBuf::from(args.input),
                args.store_root.map(PathBuf::from),
                args.profile_id,
                args.json,
                args.output_mode,
            )
        }
        Some(HeadSubcommand::Render(args)) => {
            if let Some(message) = unexpected_extra(&args.extra, "head render") {
                return Err(CliError::usage(message));
            }

            head_render(
                args.doc_id,
                PathBuf::from(args.input),
                args.store_root.map(PathBuf::from),
                args.profile_id,
                args.json,
                args.output_mode,
            )
        }
        Some(HeadSubcommand::External(args)) => {
            let other = args.first().map(String::as_str).unwrap_or("<unknown>");
            Err(CliError::usage(format!("unknown head subcommand: {other}")))
        }
        None => Err(CliError::usage("missing head subcommand")),
    }
}
