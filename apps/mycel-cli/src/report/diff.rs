use super::*;

fn report_diff_side(summary: &ReportInspectSummary) -> ReportDiffSideSummary {
    ReportDiffSideSummary {
        path: summary.path.clone(),
        run_id: summary.run_id.clone(),
        topology_id: summary.topology_id.clone(),
        fixture_id: summary.fixture_id.clone(),
        test_id: summary.test_id.clone(),
        execution_mode: summary.execution_mode.clone(),
        started_at: summary.started_at.clone(),
        finished_at: summary.finished_at.clone(),
        validation_status: summary.validation_status.clone(),
        deterministic_seed: summary.deterministic_seed.clone(),
        seed_source: summary.seed_source.clone(),
        result: summary.result.clone(),
        peer_count: summary.peer_count,
        event_count: summary.event_count,
        failure_count: summary.failure_count,
        verified_object_count: summary.verified_object_count,
        rejected_object_count: summary.rejected_object_count,
        matched_expected_outcomes: summary.matched_expected_outcomes.clone(),
        scheduled_peer_order: summary.scheduled_peer_order.clone(),
        fault_plan_count: summary.fault_plan_count,
        errors: summary.errors.clone(),
    }
}

fn push_report_diff_if_changed(
    differences: &mut Vec<ReportDiffEntry>,
    field: &str,
    left: serde_json::Value,
    right: serde_json::Value,
) {
    if left != right {
        differences.push(ReportDiffEntry {
            field: field.to_string(),
            left,
            right,
        });
    }
}

fn ignored_field_names(ignore_fields: &[ReportDiffIgnoreField]) -> Vec<String> {
    ignore_fields
        .iter()
        .map(|field| field.as_str().to_string())
        .collect()
}

fn selected_field_names(fields: &[ReportDiffIgnoreField]) -> Vec<String> {
    fields
        .iter()
        .map(|field| field.as_str().to_string())
        .collect()
}

fn summary_field_ignored(
    ignore_fields: &[ReportDiffIgnoreField],
    field: ReportDiffIgnoreField,
) -> bool {
    ignore_fields.contains(&field)
}

fn diff_field_selected(fields: &[ReportDiffIgnoreField], field: ReportDiffIgnoreField) -> bool {
    fields.is_empty() || fields.contains(&field)
}

fn push_report_diff_field(
    differences: &mut Vec<ReportDiffEntry>,
    fields: &[ReportDiffIgnoreField],
    ignore_fields: &[ReportDiffIgnoreField],
    ignore_field: ReportDiffIgnoreField,
    field: &str,
    left: serde_json::Value,
    right: serde_json::Value,
) {
    if diff_field_selected(fields, ignore_field)
        && !summary_field_ignored(ignore_fields, ignore_field)
    {
        push_report_diff_if_changed(differences, field, left, right);
    }
}

fn normalized_event_value(
    event: &ReportEvent,
    fields: &[ReportDiffIgnoreField],
    ignore_fields: &[ReportDiffIgnoreField],
) -> serde_json::Value {
    let mut value = serde_json::to_value(event).unwrap_or_else(|_| serde_json::Value::Null);
    let Some(object) = value.as_object_mut() else {
        return value;
    };

    object.remove("step");

    if !diff_field_selected(fields, ReportDiffIgnoreField::EventPhase)
        || ignore_fields.contains(&ReportDiffIgnoreField::EventPhase)
    {
        object.remove("phase");
    }
    if !diff_field_selected(fields, ReportDiffIgnoreField::EventAction)
        || ignore_fields.contains(&ReportDiffIgnoreField::EventAction)
    {
        object.remove("action");
    }
    if !diff_field_selected(fields, ReportDiffIgnoreField::EventOutcome)
        || ignore_fields.contains(&ReportDiffIgnoreField::EventOutcome)
    {
        object.remove("outcome");
    }
    if !diff_field_selected(fields, ReportDiffIgnoreField::EventNodeId)
        || ignore_fields.contains(&ReportDiffIgnoreField::EventNodeId)
    {
        object.remove("node_id");
    }
    if !diff_field_selected(fields, ReportDiffIgnoreField::EventObjectIds)
        || ignore_fields.contains(&ReportDiffIgnoreField::EventObjectIds)
    {
        object.remove("object_ids");
    }
    if !diff_field_selected(fields, ReportDiffIgnoreField::EventDetail)
        || ignore_fields.contains(&ReportDiffIgnoreField::EventDetail)
    {
        object.remove("detail");
    }

    value
}

fn collect_report_diff_errors(
    left: &ReportDiffSideSummary,
    right: &ReportDiffSideSummary,
) -> Vec<String> {
    let mut errors = Vec::new();
    errors.extend(
        left.errors
            .iter()
            .map(|message| format!("left report: {message}")),
    );
    errors.extend(
        right
            .errors
            .iter()
            .map(|message| format!("right report: {message}")),
    );
    errors
}

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
struct EventTraceIdentityKey {
    phase: Option<String>,
    action: Option<String>,
    node_id: Option<String>,
    object_ids: Vec<String>,
}

#[derive(Debug, Clone)]
struct IndexedReportEvent {
    identity: ReportEventTraceIdentity,
    event: ReportEvent,
}

fn identity_field_retained(
    fields: &[ReportDiffIgnoreField],
    ignore_fields: &[ReportDiffIgnoreField],
    field: ReportDiffIgnoreField,
) -> bool {
    !ignore_fields.contains(&field) && (fields.is_empty() || fields.contains(&field))
}

fn event_trace_identity_key(
    event: &ReportEvent,
    fields: &[ReportDiffIgnoreField],
    ignore_fields: &[ReportDiffIgnoreField],
) -> EventTraceIdentityKey {
    EventTraceIdentityKey {
        phase: identity_field_retained(fields, ignore_fields, ReportDiffIgnoreField::EventPhase)
            .then(|| event.phase.clone()),
        action: identity_field_retained(fields, ignore_fields, ReportDiffIgnoreField::EventAction)
            .then(|| event.action.clone()),
        node_id: identity_field_retained(fields, ignore_fields, ReportDiffIgnoreField::EventNodeId)
            .then(|| event.node_id.clone())
            .flatten(),
        object_ids: if identity_field_retained(
            fields,
            ignore_fields,
            ReportDiffIgnoreField::EventObjectIds,
        ) {
            event.object_ids.clone()
        } else {
            Vec::new()
        },
    }
}

fn index_report_events(
    events: Vec<ReportEvent>,
    fields: &[ReportDiffIgnoreField],
    ignore_fields: &[ReportDiffIgnoreField],
) -> BTreeMap<(EventTraceIdentityKey, usize), IndexedReportEvent> {
    let mut occurrences = BTreeMap::<EventTraceIdentityKey, usize>::new();
    let mut indexed = BTreeMap::new();

    for event in events {
        let key = event_trace_identity_key(&event, fields, ignore_fields);
        let occurrence = {
            let count = occurrences.entry(key.clone()).or_insert(0);
            *count += 1;
            *count
        };
        indexed.insert(
            (key.clone(), occurrence),
            IndexedReportEvent {
                identity: ReportEventTraceIdentity {
                    phase: key.phase.clone(),
                    action: key.action.clone(),
                    node_id: key.node_id.clone(),
                    object_ids: key.object_ids.clone(),
                    occurrence,
                },
                event,
            },
        );
    }

    indexed
}

pub(super) fn diff_reports(
    left_path: PathBuf,
    right_path: PathBuf,
    fields: &[ReportDiffIgnoreField],
    ignore_fields: &[ReportDiffIgnoreField],
) -> ReportDiffSummary {
    let left_inspected = inspect_report(left_path);
    let right_inspected = inspect_report(right_path);

    let left = report_diff_side(&left_inspected.summary);
    let right = report_diff_side(&right_inspected.summary);

    let errors = collect_report_diff_errors(&left, &right);

    if !errors.is_empty() {
        return ReportDiffSummary {
            status: "failed".to_string(),
            comparison: "failed".to_string(),
            difference_count: 0,
            selected_fields: selected_field_names(fields),
            ignored_fields: ignored_field_names(ignore_fields),
            left,
            right,
            differences: Vec::new(),
            errors,
        };
    }

    let mut differences = Vec::new();
    push_report_diff_field(
        &mut differences,
        fields,
        ignore_fields,
        ReportDiffIgnoreField::RunId,
        "run_id",
        serde_json::json!(left.run_id),
        serde_json::json!(right.run_id),
    );
    push_report_diff_field(
        &mut differences,
        fields,
        ignore_fields,
        ReportDiffIgnoreField::TopologyId,
        "topology_id",
        serde_json::json!(left.topology_id),
        serde_json::json!(right.topology_id),
    );
    push_report_diff_field(
        &mut differences,
        fields,
        ignore_fields,
        ReportDiffIgnoreField::FixtureId,
        "fixture_id",
        serde_json::json!(left.fixture_id),
        serde_json::json!(right.fixture_id),
    );
    push_report_diff_field(
        &mut differences,
        fields,
        ignore_fields,
        ReportDiffIgnoreField::TestId,
        "test_id",
        serde_json::json!(left.test_id),
        serde_json::json!(right.test_id),
    );
    push_report_diff_field(
        &mut differences,
        fields,
        ignore_fields,
        ReportDiffIgnoreField::ExecutionMode,
        "execution_mode",
        serde_json::json!(left.execution_mode),
        serde_json::json!(right.execution_mode),
    );
    push_report_diff_field(
        &mut differences,
        fields,
        ignore_fields,
        ReportDiffIgnoreField::StartedAt,
        "started_at",
        serde_json::json!(left.started_at),
        serde_json::json!(right.started_at),
    );
    push_report_diff_field(
        &mut differences,
        fields,
        ignore_fields,
        ReportDiffIgnoreField::FinishedAt,
        "finished_at",
        serde_json::json!(left.finished_at),
        serde_json::json!(right.finished_at),
    );
    push_report_diff_field(
        &mut differences,
        fields,
        ignore_fields,
        ReportDiffIgnoreField::ValidationStatus,
        "validation_status",
        serde_json::json!(left.validation_status),
        serde_json::json!(right.validation_status),
    );
    push_report_diff_field(
        &mut differences,
        fields,
        ignore_fields,
        ReportDiffIgnoreField::DeterministicSeed,
        "deterministic_seed",
        serde_json::json!(left.deterministic_seed),
        serde_json::json!(right.deterministic_seed),
    );
    push_report_diff_field(
        &mut differences,
        fields,
        ignore_fields,
        ReportDiffIgnoreField::SeedSource,
        "seed_source",
        serde_json::json!(left.seed_source),
        serde_json::json!(right.seed_source),
    );
    push_report_diff_field(
        &mut differences,
        fields,
        ignore_fields,
        ReportDiffIgnoreField::Result,
        "result",
        serde_json::json!(left.result),
        serde_json::json!(right.result),
    );
    push_report_diff_field(
        &mut differences,
        fields,
        ignore_fields,
        ReportDiffIgnoreField::PeerCount,
        "peer_count",
        serde_json::json!(left.peer_count),
        serde_json::json!(right.peer_count),
    );
    push_report_diff_field(
        &mut differences,
        fields,
        ignore_fields,
        ReportDiffIgnoreField::EventCount,
        "event_count",
        serde_json::json!(left.event_count),
        serde_json::json!(right.event_count),
    );
    push_report_diff_field(
        &mut differences,
        fields,
        ignore_fields,
        ReportDiffIgnoreField::FailureCount,
        "failure_count",
        serde_json::json!(left.failure_count),
        serde_json::json!(right.failure_count),
    );
    push_report_diff_field(
        &mut differences,
        fields,
        ignore_fields,
        ReportDiffIgnoreField::VerifiedObjectCount,
        "verified_object_count",
        serde_json::json!(left.verified_object_count),
        serde_json::json!(right.verified_object_count),
    );
    push_report_diff_field(
        &mut differences,
        fields,
        ignore_fields,
        ReportDiffIgnoreField::RejectedObjectCount,
        "rejected_object_count",
        serde_json::json!(left.rejected_object_count),
        serde_json::json!(right.rejected_object_count),
    );
    push_report_diff_field(
        &mut differences,
        fields,
        ignore_fields,
        ReportDiffIgnoreField::MatchedExpectedOutcomes,
        "matched_expected_outcomes",
        serde_json::json!(left.matched_expected_outcomes),
        serde_json::json!(right.matched_expected_outcomes),
    );
    push_report_diff_field(
        &mut differences,
        fields,
        ignore_fields,
        ReportDiffIgnoreField::ScheduledPeerOrder,
        "scheduled_peer_order",
        serde_json::json!(left.scheduled_peer_order),
        serde_json::json!(right.scheduled_peer_order),
    );
    push_report_diff_field(
        &mut differences,
        fields,
        ignore_fields,
        ReportDiffIgnoreField::FaultPlanCount,
        "fault_plan_count",
        serde_json::json!(left.fault_plan_count),
        serde_json::json!(right.fault_plan_count),
    );

    let comparison = if differences.is_empty() {
        "match"
    } else {
        "different"
    };

    ReportDiffSummary {
        status: "ok".to_string(),
        comparison: comparison.to_string(),
        difference_count: differences.len(),
        selected_fields: selected_field_names(fields),
        ignored_fields: ignored_field_names(ignore_fields),
        left,
        right,
        differences,
        errors: Vec::new(),
    }
}

pub(super) fn diff_report_events(
    left_path: PathBuf,
    right_path: PathBuf,
    fields: &[ReportDiffIgnoreField],
    ignore_fields: &[ReportDiffIgnoreField],
) -> ReportEventDiffSummary {
    let left_inspected = inspect_report(left_path);
    let right_inspected = inspect_report(right_path);

    let left = report_diff_side(&left_inspected.summary);
    let right = report_diff_side(&right_inspected.summary);

    let errors = collect_report_diff_errors(&left, &right);
    if !errors.is_empty() {
        return ReportEventDiffSummary {
            status: "failed".to_string(),
            comparison: "failed".to_string(),
            event_difference_count: 0,
            selected_fields: selected_field_names(fields),
            ignored_fields: ignored_field_names(ignore_fields),
            left,
            right,
            event_differences: Vec::new(),
            errors,
        };
    }

    let left_by_identity = index_report_events(left_inspected.events, fields, ignore_fields);
    let right_by_identity = index_report_events(right_inspected.events, fields, ignore_fields);

    let all_identities = left_by_identity
        .keys()
        .chain(right_by_identity.keys())
        .cloned()
        .collect::<std::collections::BTreeSet<_>>();

    let mut event_differences = Vec::new();
    for identity_key in all_identities {
        let left_event = left_by_identity.get(&identity_key).cloned();
        let right_event = right_by_identity.get(&identity_key).cloned();
        let trace_identity = left_event
            .as_ref()
            .map(|indexed| indexed.identity.clone())
            .or_else(|| right_event.as_ref().map(|indexed| indexed.identity.clone()))
            .expect("event identity set should contain one side");
        let left_step = left_event.as_ref().map(|indexed| indexed.event.step);
        let right_step = right_event.as_ref().map(|indexed| indexed.event.step);
        let step = left_step.or(right_step).unwrap_or_default();
        match (&left_event, &right_event) {
            (Some(left_event), Some(right_event))
                if normalized_event_value(&left_event.event, fields, ignore_fields)
                    != normalized_event_value(&right_event.event, fields, ignore_fields) =>
            {
                event_differences.push(ReportEventDiffEntry {
                    step,
                    left_step,
                    right_step,
                    trace_identity,
                    change: "changed".to_string(),
                    left: Some(left_event.event.clone()),
                    right: Some(right_event.event.clone()),
                });
            }
            (Some(_), None) => {
                event_differences.push(ReportEventDiffEntry {
                    step,
                    left_step,
                    right_step,
                    trace_identity,
                    change: "left_only".to_string(),
                    left: left_event.map(|indexed| indexed.event),
                    right: None,
                });
            }
            (None, Some(_)) => {
                event_differences.push(ReportEventDiffEntry {
                    step,
                    left_step,
                    right_step,
                    trace_identity,
                    change: "right_only".to_string(),
                    left: None,
                    right: right_event.map(|indexed| indexed.event),
                });
            }
            _ => {}
        }
    }

    let comparison = if event_differences.is_empty() {
        "match"
    } else {
        "different"
    };

    ReportEventDiffSummary {
        status: "ok".to_string(),
        comparison: comparison.to_string(),
        event_difference_count: event_differences.len(),
        selected_fields: selected_field_names(fields),
        ignored_fields: ignored_field_names(ignore_fields),
        left,
        right,
        event_differences,
        errors: Vec::new(),
    }
}

pub(super) fn format_report_diff_value(value: &serde_json::Value) -> String {
    match value {
        serde_json::Value::String(text) => format!("{text:?}"),
        _ => value.to_string(),
    }
}

pub(super) fn format_report_event_text(event: &ReportEvent) -> String {
    let node = event.node_id.as_deref().unwrap_or("-");
    let detail = event.detail.as_deref().unwrap_or("-");
    format!(
        "phase={} action={} outcome={} node={} objects={} detail={detail:?}",
        event.phase,
        event.action,
        event.outcome,
        node,
        event.object_ids.len(),
    )
}

pub(super) fn report_diff_exit_code(status: &str, comparison: &str, fail_on_diff: bool) -> i32 {
    if status == "failed" || (fail_on_diff && comparison == "different") {
        1
    } else {
        0
    }
}
