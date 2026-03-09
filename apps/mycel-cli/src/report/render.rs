use super::diff::{format_report_diff_value, format_report_event_text, report_diff_exit_code};
use super::query::{inspect_report, latest_report_path};
use super::*;

fn print_report_query_text_header(summary: ReportQuerySummaryView<'_>) {
    println!("reports root: {}", summary.root.display());
    println!("status: {}", summary.status);
    if let Some(result_filter) = summary.result_filter {
        println!("result filter: {}", result_filter.as_str());
    }
    if let Some(validation_status_filter) = summary.validation_status_filter {
        println!(
            "validation status filter: {}",
            validation_status_filter.as_str()
        );
    }
    println!("reports: {}", summary.report_count);
    println!("valid reports: {}", summary.valid_report_count);
    println!("invalid reports: {}", summary.invalid_report_count);
}

fn format_trace_identity_text(identity: &ReportEventTraceIdentity) -> String {
    let phase = identity.phase.as_deref().unwrap_or("*");
    let action = identity.action.as_deref().unwrap_or("*");
    let node = identity.node_id.as_deref().unwrap_or("*");
    let object_ids = if identity.object_ids.is_empty() {
        "*".to_string()
    } else {
        identity.object_ids.join(",")
    };
    format!(
        "phase={phase} action={action} node={node} objects={object_ids} occurrence={}",
        identity.occurrence
    )
}

fn report_query_summary_json(
    summary: ReportQuerySummaryView<'_>,
) -> serde_json::Map<String, serde_json::Value> {
    let mut json = serde_json::Map::new();
    json.insert("root".to_string(), serde_json::json!(summary.root));
    json.insert("status".to_string(), serde_json::json!(summary.status));
    json.insert(
        "result_filter".to_string(),
        serde_json::json!(summary.result_filter.map(ReportResultFilter::as_str)),
    );
    json.insert(
        "validation_status_filter".to_string(),
        serde_json::json!(summary
            .validation_status_filter
            .map(ReportValidationStatusFilter::as_str)),
    );
    json.insert(
        "report_count".to_string(),
        serde_json::json!(summary.report_count),
    );
    json.insert(
        "valid_report_count".to_string(),
        serde_json::json!(summary.valid_report_count),
    );
    json.insert(
        "invalid_report_count".to_string(),
        serde_json::json!(summary.invalid_report_count),
    );
    json.insert("errors".to_string(), serde_json::json!(summary.errors));
    json
}

fn report_list_entry_json(report: &ReportListEntry) -> serde_json::Value {
    serde_json::json!({
        "path": report.path,
        "status": report.status,
        "run_id": report.run_id,
        "topology_id": report.topology_id,
        "fixture_id": report.fixture_id,
        "test_id": report.test_id,
        "started_at": report.started_at,
        "finished_at": report.finished_at,
        "validation_status": report.validation_status,
        "result": report.result,
        "peer_count": report.peer_count,
        "event_count": report.event_count,
        "failure_count": report.failure_count,
        "parse_error": report.parse_error,
    })
}

fn finish_report_query_text(command: &str, summary: ReportQuerySummaryView<'_>) -> i32 {
    if summary.errors.is_empty() {
        println!("{command}: {}", summary.status);
        0
    } else {
        println!("{command}: failed");
        for error in summary.errors {
            emit_error_line(error);
        }
        1
    }
}

fn finish_report_query_paths(summary: ReportQuerySummaryView<'_>) -> i32 {
    if summary.errors.is_empty() {
        0
    } else {
        for error in summary.errors {
            emit_error_line(error);
        }
        1
    }
}

pub(super) fn print_report_text(summary: &ReportInspectSummary) -> i32 {
    println!("report path: {}", summary.path.display());
    if let Some(run_id) = &summary.run_id {
        println!("run id: {run_id}");
    }
    if let Some(topology_id) = &summary.topology_id {
        println!("topology id: {topology_id}");
    }
    if let Some(fixture_id) = &summary.fixture_id {
        println!("fixture id: {fixture_id}");
    }
    if let Some(test_id) = &summary.test_id {
        println!("test id: {test_id}");
    }
    if let Some(execution_mode) = &summary.execution_mode {
        println!("execution mode: {execution_mode}");
    }
    if let Some(started_at) = &summary.started_at {
        println!("started at: {started_at}");
    }
    if let Some(finished_at) = &summary.finished_at {
        println!("finished at: {finished_at}");
    }
    if let Some(validation_status) = &summary.validation_status {
        println!("validation status: {validation_status}");
    }
    if let Some(deterministic_seed) = &summary.deterministic_seed {
        println!("deterministic seed: {deterministic_seed}");
    }
    if let Some(seed_source) = &summary.seed_source {
        println!("seed source: {seed_source}");
    }
    if let Some(result) = &summary.result {
        println!("result: {result}");
    }
    println!("status: {}", summary.status);
    println!("peers: {}", summary.peer_count);
    println!("events: {}", summary.event_count);
    println!("failures: {}", summary.failure_count);
    if let Some(verified_object_count) = summary.verified_object_count {
        println!("verified objects: {verified_object_count}");
    }
    if let Some(rejected_object_count) = summary.rejected_object_count {
        println!("rejected objects: {rejected_object_count}");
    }
    println!("fault plan entries: {}", summary.fault_plan_count);
    if !summary.scheduled_peer_order.is_empty() {
        println!(
            "scheduled peer order: {}",
            summary.scheduled_peer_order.join(" -> ")
        );
    }
    if !summary.matched_expected_outcomes.is_empty() {
        println!(
            "matched expected outcomes: {}",
            summary.matched_expected_outcomes.join(", ")
        );
    }

    if summary.is_ok() {
        println!("report inspection: ok");
        0
    } else {
        println!("report inspection: failed");
        for error in &summary.errors {
            emit_error_line(error);
        }
        1
    }
}

pub(super) fn print_report_list_text(summary: &ReportListSummary) -> i32 {
    let query_summary = ReportQuerySummaryView::from(summary);
    print_report_query_text_header(query_summary);

    for report in &summary.reports {
        print!("report: {} status={}", report.path.display(), report.status);
        if let Some(run_id) = &report.run_id {
            print!(" run_id={run_id}");
        }
        if let Some(finished_at) = &report.finished_at {
            print!(" finished_at={finished_at}");
        }
        if let Some(result) = &report.result {
            print!(" result={result}");
        }
        if let Some(validation_status) = &report.validation_status {
            print!(" validation_status={validation_status}");
        }
        if let Some(parse_error) = &report.parse_error {
            print!(" parse_error={parse_error}");
        }
        println!();
    }

    finish_report_query_text("report listing", query_summary)
}

pub(super) fn print_report_latest_text(summary: &ReportLatestSummary) -> i32 {
    let query_summary = ReportQuerySummaryView::from(summary);
    print_report_query_text_header(query_summary);

    if let Some(selected) = &summary.selected {
        println!("selected report: {}", selected.path.display());
        if let Some(run_id) = &selected.run_id {
            println!("run id: {run_id}");
        }
        if let Some(fixture_id) = &selected.fixture_id {
            println!("fixture id: {fixture_id}");
        }
        if let Some(test_id) = &selected.test_id {
            println!("test id: {test_id}");
        }
        if let Some(started_at) = &selected.started_at {
            println!("started at: {started_at}");
        }
        if let Some(finished_at) = &selected.finished_at {
            println!("finished at: {finished_at}");
        }
        if let Some(validation_status) = &selected.validation_status {
            println!("validation status: {validation_status}");
        }
        if let Some(result) = &selected.result {
            println!("result: {result}");
        }
        println!("peers: {}", selected.peer_count);
        println!("events: {}", selected.event_count);
        println!("failures: {}", selected.failure_count);
    }

    finish_report_query_text("report latest", query_summary)
}

pub(super) fn print_report_latest_json(summary: &ReportLatestSummary) -> Result<i32, CliError> {
    let mut json = report_query_summary_json(ReportQuerySummaryView::from(summary));
    json.insert(
        "selected".to_string(),
        serde_json::json!(summary.selected.as_ref().map(report_list_entry_json)),
    );

    match serde_json::to_string_pretty(&json) {
        Ok(json) => {
            println!("{json}");
            if summary.is_ok() {
                Ok(0)
            } else {
                Ok(1)
            }
        }
        Err(source) => Err(CliError::serialization("latest report summary", source)),
    }
}

pub(super) fn print_report_list_json(summary: &ReportListSummary) -> Result<i32, CliError> {
    let reports = summary
        .reports
        .iter()
        .map(report_list_entry_json)
        .collect::<Vec<_>>();
    let mut json = report_query_summary_json(ReportQuerySummaryView::from(summary));
    json.insert("reports".to_string(), serde_json::json!(reports));

    match serde_json::to_string_pretty(&json) {
        Ok(json) => {
            println!("{json}");
            if summary.is_ok() {
                Ok(0)
            } else {
                Ok(1)
            }
        }
        Err(source) => Err(CliError::serialization("report listing summary", source)),
    }
}

pub(super) fn print_report_list_paths(summary: &ReportListSummary) -> i32 {
    for report in &summary.reports {
        if report.status == "ok" {
            println!("{}", report.path.display());
        }
    }

    finish_report_query_paths(ReportQuerySummaryView::from(summary))
}

pub(super) fn print_report_latest_path(summary: &ReportLatestSummary) -> i32 {
    match latest_report_path(summary) {
        Some(path) => {
            println!("{}", path.display());
            0
        }
        None => finish_report_query_paths(ReportQuerySummaryView::from(summary)),
    }
}

pub(super) fn print_report_stats_text(summary: &ReportStatsSummary) -> i32 {
    let query_summary = ReportQuerySummaryView::from(summary);
    print_report_query_text_header(query_summary);
    println!("result counts:");
    if summary.result_counts.is_empty() {
        println!("  (none)");
    } else {
        for (result, count) in &summary.result_counts {
            println!("  {result}: {count}");
        }
    }
    println!("validation status counts:");
    if summary.validation_status_counts.is_empty() {
        println!("  (none)");
    } else {
        for (status, count) in &summary.validation_status_counts {
            println!("  {status}: {count}");
        }
    }
    if let Some(latest_finished_at) = &summary.latest_finished_at {
        println!("latest finished at: {latest_finished_at}");
    }
    if let Some(latest_valid_report) = &summary.latest_valid_report {
        println!(
            "latest valid report: {}",
            latest_valid_report.path.display()
        );
        if let Some(run_id) = &latest_valid_report.run_id {
            println!("latest run id: {run_id}");
        }
        if let Some(result) = &latest_valid_report.result {
            println!("latest result: {result}");
        }
        if let Some(validation_status) = &latest_valid_report.validation_status {
            println!("latest validation status: {validation_status}");
        }
    }

    finish_report_query_text("report stats", query_summary)
}

pub(super) fn print_report_stats_json(summary: &ReportStatsSummary) -> Result<i32, CliError> {
    let mut json = report_query_summary_json(ReportQuerySummaryView::from(summary));
    json.insert(
        "result_counts".to_string(),
        serde_json::json!(summary.result_counts),
    );
    json.insert(
        "validation_status_counts".to_string(),
        serde_json::json!(summary.validation_status_counts),
    );
    json.insert(
        "latest_finished_at".to_string(),
        serde_json::json!(summary.latest_finished_at),
    );
    json.insert(
        "latest_valid_report".to_string(),
        serde_json::json!(summary.latest_valid_report.as_ref().map(|report| {
            serde_json::json!({
                "path": report.path,
                "run_id": report.run_id,
                "finished_at": report.finished_at,
                "result": report.result,
                "validation_status": report.validation_status,
            })
        })),
    );

    render_report_stats_json(json, summary.is_ok())
}

pub(super) fn print_report_stats_counts_json(
    summary: &ReportStatsSummary,
) -> Result<i32, CliError> {
    let mut json = report_query_summary_json(ReportQuerySummaryView::from(summary));
    json.insert(
        "result_counts".to_string(),
        serde_json::json!(summary.result_counts),
    );
    json.insert(
        "validation_status_counts".to_string(),
        serde_json::json!(summary.validation_status_counts),
    );
    render_report_stats_json(json, summary.is_ok())
}

fn render_report_stats_json(
    json: serde_json::Map<String, serde_json::Value>,
    ok: bool,
) -> Result<i32, CliError> {
    match serde_json::to_string_pretty(&json) {
        Ok(json) => {
            println!("{json}");
            if ok {
                Ok(0)
            } else {
                Ok(1)
            }
        }
        Err(source) => Err(CliError::serialization("report statistics summary", source)),
    }
}

pub(super) fn print_report_stats_latest_path(summary: &ReportStatsSummary) -> i32 {
    match summary.latest_valid_report.as_ref() {
        Some(report) => {
            println!("{}", report.path.display());
            0
        }
        None => {
            if !summary.errors.is_empty() {
                return finish_report_query_paths(ReportQuerySummaryView::from(summary));
            }

            emit_error_line(
                ReportQuery::new(summary.result_filter, summary.validation_status_filter)
                    .describe_missing(),
            );
            1
        }
    }
}

pub(super) fn print_report_stats_full_latest(
    summary: &ReportStatsSummary,
) -> Result<i32, CliError> {
    match summary.latest_valid_report.as_ref() {
        Some(report) => {
            let inspected = inspect_report(report.path.clone());
            match inspected.report.as_ref() {
                Some(report) => print_report_full_json(&inspected.summary, report),
                None => print_report_stats_json(summary),
            }
        }
        None => {
            if !summary.errors.is_empty() {
                return print_report_stats_json(summary);
            }

            let error = ReportQuery::new(summary.result_filter, summary.validation_status_filter)
                .describe_missing();
            let mut json = report_query_summary_json(ReportQuerySummaryView::from(summary));
            json.insert("status".to_string(), serde_json::json!("failed"));
            json.insert(
                "result_counts".to_string(),
                serde_json::json!(summary.result_counts),
            );
            json.insert(
                "validation_status_counts".to_string(),
                serde_json::json!(summary.validation_status_counts),
            );
            json.insert(
                "latest_finished_at".to_string(),
                serde_json::json!(summary.latest_finished_at),
            );
            json.insert(
                "latest_valid_report".to_string(),
                serde_json::json!(serde_json::Value::Null),
            );
            json.insert("errors".to_string(), serde_json::json!([error]));
            render_report_stats_json(json, false)
        }
    }
}

pub(super) fn print_report_summary_json(summary: &ReportInspectSummary) -> Result<i32, CliError> {
    let json = serde_json::json!({
        "path": summary.path,
        "status": summary.status,
        "run_id": summary.run_id,
        "topology_id": summary.topology_id,
        "fixture_id": summary.fixture_id,
        "test_id": summary.test_id,
        "execution_mode": summary.execution_mode,
        "started_at": summary.started_at,
        "finished_at": summary.finished_at,
        "validation_status": summary.validation_status,
        "deterministic_seed": summary.deterministic_seed,
        "seed_source": summary.seed_source,
        "result": summary.result,
        "peer_count": summary.peer_count,
        "event_count": summary.event_count,
        "failure_count": summary.failure_count,
        "verified_object_count": summary.verified_object_count,
        "rejected_object_count": summary.rejected_object_count,
        "matched_expected_outcomes": summary.matched_expected_outcomes,
        "scheduled_peer_order": summary.scheduled_peer_order,
        "fault_plan_count": summary.fault_plan_count,
        "errors": summary.errors,
    });

    match serde_json::to_string_pretty(&json) {
        Ok(json) => {
            println!("{json}");
            if summary.is_ok() {
                Ok(0)
            } else {
                Ok(1)
            }
        }
        Err(source) => Err(CliError::serialization("report inspection summary", source)),
    }
}

pub(super) fn print_report_events_text(
    summary: &ReportInspectSummary,
    events: &[ReportEvent],
) -> i32 {
    println!("report path: {}", summary.path.display());
    if let Some(run_id) = &summary.run_id {
        println!("run id: {run_id}");
    }
    println!("status: {}", summary.status);
    println!("events: {}", events.len());
    for event in events {
        println!(
            "event #{} phase={} action={} outcome={}",
            event.step, event.phase, event.action, event.outcome
        );
        if let Some(node_id) = &event.node_id {
            println!("  node: {node_id}");
        }
        if !event.object_ids.is_empty() {
            println!("  object ids: {}", event.object_ids.join(", "));
        }
        if let Some(detail) = &event.detail {
            println!("  detail: {detail}");
        }
    }

    if summary.is_ok() {
        println!("report inspection: ok");
        0
    } else {
        println!("report inspection: failed");
        for error in &summary.errors {
            emit_error_line(error);
        }
        1
    }
}

pub(super) fn print_report_events_json(
    summary: &ReportInspectSummary,
    events: &[ReportEvent],
) -> Result<i32, CliError> {
    let json = serde_json::json!({
        "path": summary.path,
        "status": summary.status,
        "run_id": summary.run_id,
        "result": summary.result,
        "event_count": events.len(),
        "events": events,
        "errors": summary.errors,
    });

    match serde_json::to_string_pretty(&json) {
        Ok(json) => {
            println!("{json}");
            if summary.is_ok() {
                Ok(0)
            } else {
                Ok(1)
            }
        }
        Err(source) => Err(CliError::serialization(
            "report event inspection summary",
            source,
        )),
    }
}

pub(super) fn print_report_failures_text(
    summary: &ReportInspectSummary,
    failures: &[ReportFailure],
) -> i32 {
    println!("report path: {}", summary.path.display());
    if let Some(run_id) = &summary.run_id {
        println!("run id: {run_id}");
    }
    println!("status: {}", summary.status);
    println!("failures: {}", failures.len());
    for failure in failures {
        println!("failure {}: {}", failure.failure_id, failure.description);
        if let Some(node_id) = &failure.node_id {
            println!("  node: {node_id}");
        }
        if let Some(severity) = &failure.severity {
            println!("  severity: {severity}");
        }
    }

    if summary.is_ok() {
        println!("report inspection: ok");
        0
    } else {
        println!("report inspection: failed");
        for error in &summary.errors {
            emit_error_line(error);
        }
        1
    }
}

pub(super) fn print_report_failures_json(
    summary: &ReportInspectSummary,
    failures: &[ReportFailure],
) -> Result<i32, CliError> {
    let json = serde_json::json!({
        "path": summary.path,
        "status": summary.status,
        "run_id": summary.run_id,
        "result": summary.result,
        "failure_count": failures.len(),
        "failures": failures,
        "errors": summary.errors,
    });

    match serde_json::to_string_pretty(&json) {
        Ok(json) => {
            println!("{json}");
            if summary.is_ok() {
                Ok(0)
            } else {
                Ok(1)
            }
        }
        Err(source) => Err(CliError::serialization(
            "report failure inspection summary",
            source,
        )),
    }
}

pub(super) fn print_report_full_json(
    summary: &ReportInspectSummary,
    report: &Report,
) -> Result<i32, CliError> {
    if !summary.is_ok() {
        return print_report_summary_json(summary);
    }

    match serde_json::to_string_pretty(report) {
        Ok(json) => {
            println!("{json}");
            Ok(0)
        }
        Err(source) => Err(CliError::serialization("full report JSON", source)),
    }
}

pub(super) fn print_report_diff_text(summary: &ReportDiffSummary, fail_on_diff: bool) -> i32 {
    println!("left report: {}", summary.left.path.display());
    println!("right report: {}", summary.right.path.display());
    println!("comparison: {}", summary.comparison);
    println!("difference count: {}", summary.difference_count);
    if !summary.selected_fields.is_empty() {
        println!("selected fields: {}", summary.selected_fields.join(", "));
    }
    if !summary.ignored_fields.is_empty() {
        println!("ignored fields: {}", summary.ignored_fields.join(", "));
    }

    for difference in &summary.differences {
        println!(
            "difference {}: left={} right={}",
            difference.field,
            format_report_diff_value(&difference.left),
            format_report_diff_value(&difference.right)
        );
    }

    let exit_code = report_diff_exit_code(&summary.status, &summary.comparison, fail_on_diff);

    if summary.status == "failed" {
        println!("report diff: failed");
        for error in &summary.errors {
            emit_error_line(error);
        }
    } else {
        println!("report diff: {}", summary.comparison);
    }

    exit_code
}

pub(super) fn print_report_event_diff_text(
    summary: &ReportEventDiffSummary,
    fail_on_diff: bool,
) -> i32 {
    println!("left report: {}", summary.left.path.display());
    println!("right report: {}", summary.right.path.display());
    println!("comparison: {}", summary.comparison);
    println!("event difference count: {}", summary.event_difference_count);
    if !summary.selected_fields.is_empty() {
        println!("selected fields: {}", summary.selected_fields.join(", "));
    }
    if !summary.ignored_fields.is_empty() {
        println!("ignored fields: {}", summary.ignored_fields.join(", "));
    }

    for difference in &summary.event_differences {
        println!(
            "event step {}: {} ({})",
            difference.step,
            difference.change,
            format_trace_identity_text(&difference.trace_identity)
        );
        if difference.left_step != difference.right_step {
            if let Some(left_step) = difference.left_step {
                println!("  left step: {left_step}");
            }
            if let Some(right_step) = difference.right_step {
                println!("  right step: {right_step}");
            }
        }
        if let Some(left) = &difference.left {
            println!("  left: {}", format_report_event_text(left));
        }
        if let Some(right) = &difference.right {
            println!("  right: {}", format_report_event_text(right));
        }
    }

    let exit_code = report_diff_exit_code(&summary.status, &summary.comparison, fail_on_diff);

    if summary.status == "failed" {
        println!("report diff: failed");
        for error in &summary.errors {
            emit_error_line(error);
        }
    } else {
        println!("report diff: {}", summary.comparison);
    }

    exit_code
}

pub(super) fn print_report_diff_json(
    summary: &ReportDiffSummary,
    fail_on_diff: bool,
) -> Result<i32, CliError> {
    match serde_json::to_string_pretty(summary) {
        Ok(json) => {
            println!("{json}");
            Ok(report_diff_exit_code(
                &summary.status,
                &summary.comparison,
                fail_on_diff,
            ))
        }
        Err(source) => Err(CliError::serialization("report diff summary", source)),
    }
}

pub(super) fn print_report_event_diff_json(
    summary: &ReportEventDiffSummary,
    fail_on_diff: bool,
) -> Result<i32, CliError> {
    match serde_json::to_string_pretty(summary) {
        Ok(json) => {
            println!("{json}");
            Ok(report_diff_exit_code(
                &summary.status,
                &summary.comparison,
                fail_on_diff,
            ))
        }
        Err(source) => Err(CliError::serialization("report event diff summary", source)),
    }
}
