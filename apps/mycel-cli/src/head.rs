use std::path::PathBuf;

use clap::{Args, Subcommand, ValueEnum};
use mycel_core::head::{
    inspect_head_profile_from_path, inspect_heads_from_path, inspect_heads_from_store_path,
    list_head_profiles_from_path, render_head_from_path, render_head_from_store_path,
    HeadInspectSummary, HeadProfileInspectSummary, HeadProfileListSummary, HeadProfileSummary,
    HeadRenderSummary,
};
use serde::Serialize;

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
    #[command(about = "List or inspect fixed reader profiles from a head input bundle")]
    Profile(HeadProfileCliArgs),
    #[command(external_subcommand)]
    External(Vec<String>),
}

#[derive(Args)]
struct HeadProfileCliArgs {
    #[command(subcommand)]
    command: Option<HeadProfileSubcommand>,
}

#[derive(Subcommand)]
enum HeadProfileSubcommand {
    #[command(about = "List fixed reader profiles declared by a head input bundle")]
    List(HeadProfileListCliArgs),
    #[command(about = "Inspect one fixed reader profile from a head input bundle")]
    Inspect(HeadProfileInspectCliArgs),
    #[command(external_subcommand)]
    External(Vec<String>),
}

#[derive(Args)]
struct HeadProfileListCliArgs {
    #[arg(
        long,
        value_name = "PATH_OR_FIXTURE",
        help = "Input bundle path or repo fixture name",
        required = true
    )]
    input: String,
    #[arg(long, help = "Emit machine-readable profile listing output")]
    json: bool,
    #[arg(hide = true, allow_hyphen_values = true)]
    extra: Vec<String>,
}

#[derive(Args)]
struct HeadProfileInspectCliArgs {
    #[arg(
        long,
        value_name = "PATH_OR_FIXTURE",
        help = "Input bundle path or repo fixture name",
        required = true
    )]
    input: String,
    #[arg(
        long,
        value_name = "PROFILE_ID",
        help = "Select one named fixed reader profile from the input bundle"
    )]
    profile_id: Option<String>,
    #[arg(long, help = "Emit machine-readable profile inspection output")]
    json: bool,
    #[arg(hide = true, allow_hyphen_values = true)]
    extra: Vec<String>,
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

fn humanize_tokenized_label(value: &str) -> String {
    value.replace(['-', '_'], " ")
}

fn human_head_status(status: &str) -> String {
    match status {
        "ok" => "selection succeeded".to_string(),
        "ok-with-viewer-review-delay" => {
            "selection succeeded after delaying candidates under viewer review".to_string()
        }
        "ok-with-viewer-freeze-block" => {
            "selection succeeded after blocking candidates under viewer freeze pressure".to_string()
        }
        "ok-with-viewer-review-delay-and-freeze-block" => {
            "selection succeeded after delaying reviewed candidates and blocking frozen candidates"
                .to_string()
        }
        "blocked-by-viewer-review-delay" => {
            "selection was blocked because all candidates were under viewer review".to_string()
        }
        "blocked-by-viewer-freeze-block" => {
            "selection was blocked because all candidates were frozen by viewer pressure"
                .to_string()
        }
        "blocked-by-viewer-review-delay-and-freeze-block" => {
            "selection was blocked because all candidates were delayed or frozen by viewer pressure"
                .to_string()
        }
        "failed" => "selection failed".to_string(),
        other => humanize_tokenized_label(other),
    }
}

fn human_tie_break_reason(reason: &str) -> String {
    match reason {
        "higher_selector_score" => "higher selector score".to_string(),
        "higher_selector_score_after_viewer-freeze-block" => {
            "higher selector score after another candidate was frozen".to_string()
        }
        "newer_revision_timestamp_or_lexicographic_tiebreak" => {
            "newer revision timestamp, with lexicographic fallback".to_string()
        }
        "newer_revision_timestamp_or_lexicographic_tiebreak_after_viewer-review-delay" => {
            "newer revision timestamp after candidates under viewer review were delayed"
                .to_string()
        }
        "newer_revision_timestamp_or_lexicographic_tiebreak_after_viewer-freeze-block" => {
            "newer revision timestamp after another candidate was frozen".to_string()
        }
        "newer_revision_timestamp_or_lexicographic_tiebreak_after_viewer-review-delay-and-freeze-block" => {
            "newer revision timestamp after other candidates were delayed or frozen".to_string()
        }
        other => humanize_tokenized_label(other),
    }
}

fn selector_score_formula(maintainer_score: u64, viewer_bonus: u64, viewer_penalty: u64) -> String {
    format!(
        "{maintainer_score} + {viewer_bonus} - {viewer_penalty} = {}",
        maintainer_score
            .saturating_add(viewer_bonus)
            .saturating_sub(viewer_penalty)
    )
}

fn profile_retry_hint(available_profile_ids: &[String], errors: &[String]) -> Option<String> {
    if available_profile_ids.is_empty() {
        return None;
    }
    if !errors.iter().any(|error| error.contains("--profile-id")) {
        return None;
    }

    let examples = available_profile_ids
        .iter()
        .map(|profile_id| format!("--profile-id {profile_id}"))
        .collect::<Vec<_>>()
        .join(" | ");
    Some(format!("retry with one of: {examples}"))
}

fn has_viewer_effects(
    viewer_bonus: u64,
    viewer_penalty: u64,
    challenge_review_pressure: u64,
    challenge_freeze_pressure: u64,
) -> bool {
    viewer_bonus > 0
        || viewer_penalty > 0
        || challenge_review_pressure > 0
        || challenge_freeze_pressure > 0
}

fn viewer_review_state_text(review_state: &impl Serialize) -> String {
    serde_json::to_string(review_state)
        .unwrap_or_else(|_| "\"unknown\"".to_string())
        .trim_matches('"')
        .to_string()
}

fn head_profile_label(profile: &HeadProfileSummary) -> &str {
    profile.profile_id.as_deref().unwrap_or("default")
}

fn summarize_profile_admitted_keys(keys: &[String]) -> String {
    if keys.is_empty() {
        "none".to_string()
    } else {
        keys.join(", ")
    }
}

fn viewer_score_mode_text(profile: &HeadProfileSummary) -> String {
    if profile.viewer_score.enabled {
        humanize_tokenized_label(&viewer_review_state_text(&profile.viewer_score.mode))
    } else {
        "disabled".to_string()
    }
}

fn print_head_inspect_summary_debug(summary: &HeadInspectSummary) {
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
}

fn print_head_debug_candidate(summary: &HeadInspectSummary) {
    for head in &summary.eligible_heads {
        if has_viewer_effects(head.viewer_bonus, head.viewer_penalty, 0, 0) {
            println!(
                "eligible head: {} timestamp={} score={} supporters={} maintainer_score={} viewer_bonus={} viewer_penalty={} score_formula=\"{}\"",
                head.revision_id,
                head.revision_timestamp,
                head.selector_score,
                head.supporter_count,
                head.maintainer_score,
                head.viewer_bonus,
                head.viewer_penalty,
                selector_score_formula(
                    head.maintainer_score,
                    head.viewer_bonus,
                    head.viewer_penalty
                )
            );
        } else {
            println!(
                "eligible head: {} timestamp={} score={} supporters={} score_formula=\"{}\"",
                head.revision_id,
                head.revision_timestamp,
                head.selector_score,
                head.supporter_count,
                selector_score_formula(
                    head.maintainer_score,
                    head.viewer_bonus,
                    head.viewer_penalty
                )
            );
        }
    }
}

fn print_head_debug_viewer_channels(summary: &HeadInspectSummary) {
    for channel in &summary.viewer_score_channels {
        if has_viewer_effects(
            channel.viewer_bonus,
            channel.viewer_penalty,
            channel.challenge_review_pressure,
            channel.challenge_freeze_pressure,
        ) {
            println!(
                "viewer channel: {} maintainer_score={} bonus={} penalty={} challenges={} review_pressure={} freeze_pressure={} review_state={}",
                channel.revision_id,
                channel.maintainer_score,
                channel.viewer_bonus,
                channel.viewer_penalty,
                channel.challenge_signal_count,
                channel.challenge_review_pressure,
                channel.challenge_freeze_pressure,
                viewer_review_state_text(&channel.viewer_review_state)
            );
        }
    }
}

fn print_head_inspect_summary_human(summary: &HeadInspectSummary) {
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
    if !summary.available_profile_ids.is_empty() {
        println!(
            "- available profiles: {}",
            summary.available_profile_ids.join(", ")
        );
        if let Some(hint) = profile_retry_hint(&summary.available_profile_ids, &summary.errors) {
            println!("- {hint}");
        }
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
    println!("- status: {}", human_head_status(&summary.status));
}

fn print_head_human_candidates(summary: &HeadInspectSummary) {
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
}

fn print_head_human_viewer_effects(summary: &HeadInspectSummary) {
    if !summary.viewer_score_channels.is_empty() {
        println!();
        println!("Viewer Effects");
        for channel in &summary.viewer_score_channels {
            if has_viewer_effects(
                channel.viewer_bonus,
                channel.viewer_penalty,
                channel.challenge_review_pressure,
                channel.challenge_freeze_pressure,
            ) {
                println!("- {}", channel.revision_id);
                println!(
                    "  maintainer score={} bonus={} penalty={} challenges={} review pressure={} freeze pressure={} state={}",
                    channel.maintainer_score,
                    channel.viewer_bonus,
                    channel.viewer_penalty,
                    channel.challenge_signal_count,
                    channel.challenge_review_pressure,
                    channel.challenge_freeze_pressure,
                    humanize_tokenized_label(&viewer_review_state_text(&channel.viewer_review_state))
                );
            }
        }
    }
}

fn print_head_human_decision(summary: &HeadInspectSummary) {
    println!();
    println!("Decision");
    if let Some(selected_head) = &summary.selected_head {
        println!("- selected head: {selected_head}");
        if let Some(selected) = summary
            .eligible_heads
            .iter()
            .find(|head| &head.revision_id == selected_head)
        {
            println!("- selector score: {}", selected.selector_score);
            println!(
                "- score formula: {}",
                selector_score_formula(
                    selected.maintainer_score,
                    selected.viewer_bonus,
                    selected.viewer_penalty
                )
            );
        }
    }
    if let Some(tie_break_reason) = &summary.tie_break_reason {
        println!("- reason: {}", human_tie_break_reason(tie_break_reason));
    }
    if let Some(selection_trace) = summary
        .decision_trace
        .iter()
        .find(|trace| trace.step == "selected_head")
    {
        println!("- trace: {}", selection_trace.detail);
    }
}

fn print_head_inspect_debug(summary: &HeadInspectSummary) -> i32 {
    print_head_inspect_summary_debug(summary);
    print_head_debug_candidate(summary);

    if let Some(selected_head) = &summary.selected_head {
        println!("selected head: {selected_head}");
    }
    if let Some(tie_break_reason) = &summary.tie_break_reason {
        println!("tie break reason: {tie_break_reason}");
    }
    print_head_debug_viewer_channels(summary);
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
    print_head_inspect_summary_human(summary);
    print_head_human_candidates(summary);
    print_head_human_viewer_effects(summary);
    print_head_human_decision(summary);

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

fn print_head_profile_entry_human(profile: &HeadProfileSummary) {
    println!("- {}", head_profile_label(profile));
    println!(
        "  source={} policy hash={} selector epoch={}",
        profile.source, profile.policy_hash, profile.selector_epoch
    );
    println!(
        "  effective selection time={} epoch seconds={} epoch zero timestamp={}",
        profile.effective_selection_time, profile.epoch_seconds, profile.epoch_zero_timestamp
    );
    println!(
        "  admission window epochs={} min valid views for admission={} min valid views per epoch={} weight cap per key={}",
        profile.admission_window_epochs,
        profile.min_valid_views_for_admission,
        profile.min_valid_views_per_epoch,
        profile.weight_cap_per_key
    );
    println!(
        "  editor admission={} admitted keys={}",
        profile.editor_admission.mode,
        summarize_profile_admitted_keys(&profile.editor_admission.admitted_keys)
    );
    println!(
        "  view admission={} admitted keys={}",
        profile.view_admission.mode,
        summarize_profile_admitted_keys(&profile.view_admission.admitted_keys)
    );
    if profile.viewer_score.enabled {
        println!(
            "  viewer score={} bonus cap={} penalty cap={} signal weight cap={} admission required={} min identity tier={} min reputation band={}",
            viewer_score_mode_text(profile),
            profile.viewer_score.bonus_cap,
            profile.viewer_score.penalty_cap,
            profile.viewer_score.signal_weight_cap,
            profile.viewer_score.admission_required,
            viewer_review_state_text(&profile.viewer_score.min_identity_tier),
            viewer_review_state_text(&profile.viewer_score.min_reputation_band)
        );
    } else {
        println!("  viewer score=disabled");
    }
}

fn print_head_profile_list_human(summary: &HeadProfileListSummary) -> i32 {
    println!(
        "Head profiles: {}",
        if summary.is_ok() { "ok" } else { "failed" }
    );
    println!();
    println!("Input");
    println!("- input: {}", summary.input_path.display());
    println!("- profile count: {}", summary.profile_count);
    if !summary.available_profile_ids.is_empty() {
        println!(
            "- available profiles: {}",
            summary.available_profile_ids.join(", ")
        );
    }
    if !summary.profiles.is_empty() {
        println!();
        println!("Profiles");
        for profile in &summary.profiles {
            print_head_profile_entry_human(profile);
        }
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

fn print_head_profile_list_json(summary: &HeadProfileListSummary) -> Result<i32, CliError> {
    match serde_json::to_string_pretty(summary) {
        Ok(json) => {
            println!("{json}");
            if summary.is_ok() {
                Ok(0)
            } else {
                Ok(1)
            }
        }
        Err(source) => Err(CliError::serialization("head profile list summary", source)),
    }
}

fn print_head_profile_inspect_human(summary: &HeadProfileInspectSummary) -> i32 {
    println!(
        "Head profile: {}",
        if summary.is_ok() { "ok" } else { "failed" }
    );
    println!();
    println!("Input");
    println!("- input: {}", summary.input_path.display());
    if let Some(requested_profile_id) = &summary.requested_profile_id {
        println!("- requested profile: {requested_profile_id}");
    }
    if !summary.available_profile_ids.is_empty() {
        println!(
            "- available profiles: {}",
            summary.available_profile_ids.join(", ")
        );
        if let Some(hint) = profile_retry_hint(&summary.available_profile_ids, &summary.errors) {
            println!("- {hint}");
        }
    }
    if let Some(profile) = &summary.profile {
        println!();
        println!("Profile");
        print_head_profile_entry_human(profile);
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

fn print_head_profile_inspect_json(summary: &HeadProfileInspectSummary) -> Result<i32, CliError> {
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
            "head profile inspect summary",
            source,
        )),
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
    if !summary.available_profile_ids.is_empty() {
        println!(
            "- available profiles: {}",
            summary.available_profile_ids.join(", ")
        );
        if let Some(hint) = profile_retry_hint(&summary.available_profile_ids, &summary.errors) {
            println!("- {hint}");
        }
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

fn head_profile_list(input_path: PathBuf, json: bool) -> Result<i32, CliError> {
    let summary = list_head_profiles_from_path(&input_path);
    if json {
        print_head_profile_list_json(&summary)
    } else {
        Ok(print_head_profile_list_human(&summary))
    }
}

fn head_profile_inspect(
    input_path: PathBuf,
    profile_id: Option<String>,
    json: bool,
) -> Result<i32, CliError> {
    let summary = inspect_head_profile_from_path(&input_path, profile_id.as_deref());
    if json {
        print_head_profile_inspect_json(&summary)
    } else {
        Ok(print_head_profile_inspect_human(&summary))
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
        Some(HeadSubcommand::Profile(args)) => match args.command {
            Some(HeadProfileSubcommand::List(args)) => {
                if let Some(message) = unexpected_extra(&args.extra, "head profile list") {
                    return Err(CliError::usage(message));
                }

                head_profile_list(PathBuf::from(args.input), args.json)
            }
            Some(HeadProfileSubcommand::Inspect(args)) => {
                if let Some(message) = unexpected_extra(&args.extra, "head profile inspect") {
                    return Err(CliError::usage(message));
                }

                head_profile_inspect(PathBuf::from(args.input), args.profile_id, args.json)
            }
            Some(HeadProfileSubcommand::External(args)) => {
                let other = args.first().map(String::as_str).unwrap_or("<unknown>");
                Err(CliError::usage(format!(
                    "unknown head profile subcommand: {other}"
                )))
            }
            None => Err(CliError::usage("missing head profile subcommand")),
        },
        Some(HeadSubcommand::External(args)) => {
            let other = args.first().map(String::as_str).unwrap_or("<unknown>");
            Err(CliError::usage(format!("unknown head subcommand: {other}")))
        }
        None => Err(CliError::usage("missing head subcommand")),
    }
}
