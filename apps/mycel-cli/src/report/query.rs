use super::*;
use std::fs;

fn metadata_string(report: &Report, key: &str) -> Option<String> {
    report
        .metadata
        .as_ref()
        .and_then(|metadata| metadata.get(key))
        .and_then(|value| value.as_str())
        .map(ToOwned::to_owned)
}

fn metadata_string_vec(report: &Report, key: &str) -> Vec<String> {
    report
        .metadata
        .as_ref()
        .and_then(|metadata| metadata.get(key))
        .and_then(|value| value.as_array())
        .map(|values| {
            values
                .iter()
                .filter_map(|value| value.as_str().map(ToOwned::to_owned))
                .collect::<Vec<_>>()
        })
        .unwrap_or_default()
}

fn metadata_array_len(report: &Report, key: &str) -> usize {
    report
        .metadata
        .as_ref()
        .and_then(|metadata| metadata.get(key))
        .and_then(|value| value.as_array())
        .map(Vec::len)
        .unwrap_or(0)
}

fn is_report_schema_file(path: &Path) -> bool {
    path.file_name().and_then(|name| name.to_str()) == Some("report.schema.json")
}

fn is_json_file(path: &Path) -> bool {
    path.extension().and_then(|extension| extension.to_str()) == Some("json")
}

pub(super) fn inspect_report(target: PathBuf) -> InspectedReport {
    let mut inspected = InspectedReport {
        summary: ReportInspectSummary::new(target.clone()),
        report: None,
        events: Vec::new(),
        failures: Vec::new(),
    };

    if !target.exists() {
        inspected
            .summary
            .push_error(format!("report path does not exist: {}", target.display()));
        return inspected;
    }

    if target.is_dir() {
        inspected.summary.push_error(format!(
            "report inspect target is a directory: {}",
            target.display()
        ));
        return inspected;
    }

    if is_report_schema_file(&target) {
        inspected
            .summary
            .push_error("report schema files are not inspect targets");
        return inspected;
    }

    let content = match fs::read_to_string(&target) {
        Ok(content) => content,
        Err(err) => {
            inspected
                .summary
                .push_error(format!("failed to read report file: {err}"));
            return inspected;
        }
    };

    let report: Report = match serde_json::from_str(&content) {
        Ok(report) => report,
        Err(err) => {
            inspected
                .summary
                .push_error(format!("failed to parse report JSON: {err}"));
            return inspected;
        }
    };

    inspected.summary.run_id = Some(report.run_id.clone());
    inspected.summary.topology_id = Some(report.topology_id.clone());
    inspected.summary.fixture_id = Some(report.fixture_id.clone());
    inspected.summary.test_id = report.test_id.clone();
    inspected.summary.execution_mode = report.execution_mode.clone();
    inspected.summary.started_at = report.started_at.clone();
    inspected.summary.finished_at = report.finished_at.clone();
    inspected.summary.validation_status = metadata_string(&report, "validation_status");
    inspected.summary.deterministic_seed = metadata_string(&report, "deterministic_seed");
    inspected.summary.seed_source = metadata_string(&report, "seed_source");
    inspected.summary.result = Some(report.result.clone());
    inspected.summary.peer_count = report.peers.len();
    inspected.summary.event_count = report.events.len();
    inspected.summary.failure_count = report.failures.len();
    inspected.summary.verified_object_count = report
        .summary
        .as_ref()
        .and_then(|report_summary| report_summary.verified_object_count);
    inspected.summary.rejected_object_count = report
        .summary
        .as_ref()
        .and_then(|report_summary| report_summary.rejected_object_count);
    inspected.summary.matched_expected_outcomes = report
        .summary
        .as_ref()
        .map(|report_summary| report_summary.matched_expected_outcomes.clone())
        .unwrap_or_default();
    inspected.summary.scheduled_peer_order = metadata_string_vec(&report, "scheduled_peer_order");
    inspected.summary.fault_plan_count = metadata_array_len(&report, "fault_plan");
    inspected.events = report.events.clone();
    inspected.failures = report.failures.clone();
    inspected.report = Some(report);

    inspected
}

fn collect_report_targets(target: &Path) -> Result<Vec<PathBuf>, String> {
    if !target.exists() {
        return Err(format!(
            "report list target does not exist: {}",
            target.display()
        ));
    }

    if target.is_file() {
        if is_report_schema_file(target) {
            return Err("report schema files are not list targets".to_string());
        }
        if !is_json_file(target) {
            return Err(format!(
                "report list target is not a JSON file: {}",
                target.display()
            ));
        }
        return Ok(vec![target.to_path_buf()]);
    }

    if !target.is_dir() {
        return Err(format!(
            "report list target is not a file or directory: {}",
            target.display()
        ));
    }

    let mut targets = Vec::new();
    collect_report_targets_from_dir(target, &mut targets)?;
    targets.sort();
    Ok(targets)
}

fn collect_report_targets_from_dir(
    target: &Path,
    targets: &mut Vec<PathBuf>,
) -> Result<(), String> {
    let mut entries = fs::read_dir(target)
        .map_err(|err| {
            format!(
                "failed to read report directory {}: {err}",
                target.display()
            )
        })?
        .collect::<Result<Vec<_>, _>>()
        .map_err(|err| {
            format!(
                "failed to read report directory {}: {err}",
                target.display()
            )
        })?;
    entries.sort_by_key(|entry| entry.path());

    for entry in entries {
        let path = entry.path();
        if path.is_dir() {
            collect_report_targets_from_dir(&path, targets)?;
            continue;
        }
        if is_json_file(&path) && !is_report_schema_file(&path) {
            targets.push(path);
        }
    }

    Ok(())
}

fn build_report_list_entry(path: PathBuf) -> ReportListEntry {
    let inspected = inspect_report(path.clone());
    let parse_error = (!inspected.summary.is_ok()).then(|| inspected.summary.errors.join("; "));

    ReportListEntry {
        path,
        status: if parse_error.is_some() {
            "failed".to_string()
        } else {
            "ok".to_string()
        },
        run_id: inspected.summary.run_id,
        topology_id: inspected.summary.topology_id,
        fixture_id: inspected.summary.fixture_id,
        test_id: inspected.summary.test_id,
        started_at: inspected.summary.started_at,
        finished_at: inspected.summary.finished_at,
        validation_status: inspected.summary.validation_status,
        result: inspected.summary.result,
        peer_count: inspected.summary.peer_count,
        event_count: inspected.summary.event_count,
        failure_count: inspected.summary.failure_count,
        parse_error,
    }
}

fn list_reports(target: PathBuf) -> ReportListSummary {
    let mut summary = ReportListSummary::new(target.clone());

    let targets = match collect_report_targets(&target) {
        Ok(targets) => targets,
        Err(message) => {
            summary.push_error(message);
            return summary;
        }
    };

    for report_target in targets {
        summary.push_report(build_report_list_entry(report_target));
    }

    summary
}

fn apply_report_query(mut summary: ReportListSummary, query: ReportQuery) -> ReportListSummary {
    summary.result_filter = query.result_filter;
    summary.validation_status_filter = query.validation_status_filter;

    if query == ReportQuery::default() {
        return summary;
    }

    summary
        .reports
        .retain(|report| report.status != "ok" || query.matches_valid_report(report));
    summary.report_count = summary.reports.len();
    summary.valid_report_count = summary
        .reports
        .iter()
        .filter(|report| report.status == "ok")
        .count();
    summary.invalid_report_count = summary
        .reports
        .iter()
        .filter(|report| report.status != "ok")
        .count();
    summary.refresh_status();
    summary
}

pub(super) fn query_reports(target: PathBuf, query: ReportQuery) -> ReportListSummary {
    apply_report_query(list_reports(target), query)
}

fn latest_report_sort_key(report: &ReportListEntry) -> (Option<&str>, Option<&str>, String) {
    (
        report.finished_at.as_deref(),
        report.started_at.as_deref(),
        report.path.to_string_lossy().into_owned(),
    )
}

pub(super) fn latest_report(summary: ReportListSummary) -> ReportLatestSummary {
    if !summary.is_ok() {
        return ReportLatestSummary {
            root: summary.root,
            status: "failed".to_string(),
            result_filter: summary.result_filter,
            validation_status_filter: summary.validation_status_filter,
            report_count: summary.report_count,
            valid_report_count: summary.valid_report_count,
            invalid_report_count: summary.invalid_report_count,
            selected: None,
            errors: summary.errors,
        };
    }

    if summary.report_count == 0 {
        let query = ReportQuery::new(summary.result_filter, summary.validation_status_filter);
        return ReportLatestSummary {
            root: summary.root,
            status: "failed".to_string(),
            result_filter: summary.result_filter,
            validation_status_filter: summary.validation_status_filter,
            report_count: 0,
            valid_report_count: 0,
            invalid_report_count: 0,
            selected: None,
            errors: vec![if query == ReportQuery::default() {
                "no reports found under target".to_string()
            } else {
                query.describe_missing()
            }],
        };
    }

    let selected = summary
        .reports
        .iter()
        .filter(|report| report.status == "ok")
        .max_by_key(|report| latest_report_sort_key(report))
        .cloned();

    match selected {
        Some(selected) => ReportLatestSummary {
            root: summary.root,
            status: if summary.invalid_report_count > 0 {
                "warning".to_string()
            } else {
                "ok".to_string()
            },
            result_filter: summary.result_filter,
            validation_status_filter: summary.validation_status_filter,
            report_count: summary.report_count,
            valid_report_count: summary.valid_report_count,
            invalid_report_count: summary.invalid_report_count,
            selected: Some(selected),
            errors: Vec::new(),
        },
        None => ReportLatestSummary {
            root: summary.root,
            status: "failed".to_string(),
            result_filter: summary.result_filter,
            validation_status_filter: summary.validation_status_filter,
            report_count: summary.report_count,
            valid_report_count: summary.valid_report_count,
            invalid_report_count: summary.invalid_report_count,
            selected: None,
            errors: vec![
                ReportQuery::new(summary.result_filter, summary.validation_status_filter)
                    .describe_missing(),
            ],
        },
    }
}

pub(super) fn latest_report_path(summary: &ReportLatestSummary) -> Option<PathBuf> {
    summary
        .selected
        .as_ref()
        .map(|selected| selected.path.clone())
}

pub(super) fn summarize_reports(summary: ReportListSummary) -> ReportStatsSummary {
    let mut result_counts = BTreeMap::new();
    let mut validation_status_counts = BTreeMap::new();
    let latest_valid_report = summary
        .reports
        .iter()
        .filter(|report| report.status == "ok")
        .max_by_key(|report| latest_report_sort_key(report))
        .cloned();

    for report in summary
        .reports
        .iter()
        .filter(|report| report.status == "ok")
    {
        let result_key = report.result.as_deref().unwrap_or("unknown").to_string();
        *result_counts.entry(result_key).or_insert(0) += 1;

        let validation_status_key = report
            .validation_status
            .as_deref()
            .unwrap_or("unknown")
            .to_string();
        *validation_status_counts
            .entry(validation_status_key)
            .or_insert(0) += 1;
    }

    let latest_finished_at = latest_valid_report
        .as_ref()
        .and_then(|report| report.finished_at.clone());
    let latest_valid_report = latest_valid_report.map(|report| ReportStatsLatestReport {
        path: report.path,
        run_id: report.run_id,
        finished_at: report.finished_at,
        result: report.result,
        validation_status: report.validation_status,
    });

    ReportStatsSummary {
        root: summary.root,
        status: summary.status,
        result_filter: summary.result_filter,
        validation_status_filter: summary.validation_status_filter,
        report_count: summary.report_count,
        valid_report_count: summary.valid_report_count,
        invalid_report_count: summary.invalid_report_count,
        result_counts,
        validation_status_counts,
        latest_finished_at,
        latest_valid_report,
        errors: summary.errors,
    }
}

pub(super) fn filter_events(
    events: &[ReportEvent],
    filters: &ReportInspectFilters,
) -> Vec<ReportEvent> {
    let filtered: Vec<_> = events
        .iter()
        .filter(|event| {
            filters
                .phase
                .as_deref()
                .is_none_or(|phase| event.phase == phase)
        })
        .filter(|event| {
            filters
                .action
                .as_deref()
                .is_none_or(|action| event.action == action)
        })
        .filter(|event| {
            filters
                .outcome
                .as_deref()
                .is_none_or(|outcome| event.outcome == outcome)
        })
        .filter(|event| filters.step.is_none_or(|step| event.step == step))
        .filter(|event| {
            filters
                .step_range
                .is_none_or(|(start, end)| start <= event.step && event.step <= end)
        })
        .filter(|event| {
            filters.node.as_deref().is_none_or(|node| {
                event
                    .node_id
                    .as_deref()
                    .is_some_and(|event_node_id| event_node_id == node)
            })
        })
        .cloned()
        .collect();

    let filtered = match filters.first {
        Some(first) => filtered.into_iter().take(first).collect(),
        None => filtered,
    };

    match filters.last {
        Some(last) => {
            let skip = filtered.len().saturating_sub(last);
            filtered.into_iter().skip(skip).collect()
        }
        None => filtered,
    }
}

pub(super) fn filter_failures(
    failures: &[ReportFailure],
    filters: &ReportInspectFilters,
) -> Vec<ReportFailure> {
    failures
        .iter()
        .filter(|failure| {
            filters.node.as_deref().is_none_or(|node| {
                failure
                    .node_id
                    .as_deref()
                    .is_some_and(|failure_node_id| failure_node_id == node)
            })
        })
        .cloned()
        .collect()
}
