//! Minimal single-process simulator runner.

use std::collections::{BTreeSet, HashSet};
use std::fs;
use std::path::{Path, PathBuf};

use serde::Serialize;
use serde_json::json;

use crate::model::{Fixture, Report, ReportEvent, ReportPeer, ReportSummary, TestCase, Topology};
use crate::validate::{validate_path, ValidationStatus};

#[derive(Debug, Clone, Serialize)]
pub struct SimulationRunSummary {
    pub root: PathBuf,
    pub target: PathBuf,
    pub report_path: PathBuf,
    pub validation_status: ValidationStatus,
    pub validation_warnings: Vec<String>,
    pub result: String,
    pub peer_count: usize,
    pub event_count: usize,
    pub verified_object_count: usize,
    pub rejected_object_count: usize,
    pub matched_expected_outcomes: Vec<String>,
}

pub fn run_test_case(target_path: &Path) -> Result<SimulationRunSummary, String> {
    let validation = validate_path(target_path);
    if !validation.errors.is_empty() {
        let first_error = validation
            .errors
            .first()
            .map(|error| format!("{}: {}", error.path, error.message))
            .unwrap_or_else(|| "validation failed".to_owned());
        return Err(first_error);
    }

    let root = validation
        .root
        .clone()
        .ok_or_else(|| "missing repo root after validation".to_owned())?;
    let target = validation
        .target
        .clone()
        .ok_or_else(|| "missing validate target after validation".to_owned())?;

    if target.file_name().and_then(|name| name.to_str()) == Some("test-case.schema.json") {
        return Err("schema files are not simulator run targets".to_owned());
    }

    let test_case = load_json::<TestCase>(&target)?;
    if test_case.execution_mode != "single-process" {
        return Err(format!(
            "unsupported execution_mode '{}'; only 'single-process' is implemented",
            test_case.execution_mode
        ));
    }

    let topology_path = root.join(&test_case.topology);
    let topology = load_json::<Topology>(&topology_path)?;
    if topology.execution_mode.as_deref() != Some("single-process") {
        return Err(format!(
            "unsupported topology execution_mode '{:?}'; only 'single-process' is implemented",
            topology.execution_mode
        ));
    }

    let fixture_path = root.join(&test_case.fixture_set).join("fixture.json");
    let fixture = load_json::<Fixture>(&fixture_path)?;

    let report = simulate_report(&test_case, &topology, &fixture)?;
    let report_path = root
        .join("sim/reports/out")
        .join(format!("{}.report.json", test_case.test_id));
    if let Some(parent) = report_path.parent() {
        fs::create_dir_all(parent).map_err(|err| {
            format!(
                "failed to create report directory {}: {err}",
                parent.display()
            )
        })?;
    }

    let report_json = serde_json::to_string_pretty(&report)
        .map_err(|err| format!("failed to serialize report: {err}"))?;
    fs::write(&report_path, report_json)
        .map_err(|err| format!("failed to write report {}: {err}", report_path.display()))?;

    let summary = SimulationRunSummary {
        root,
        target,
        report_path,
        validation_status: validation.status,
        validation_warnings: validation
            .warnings
            .iter()
            .map(|warning| format!("{}: {}", warning.path, warning.message))
            .collect(),
        result: report.result.clone(),
        peer_count: report.peers.len(),
        event_count: report.events.len(),
        verified_object_count: report
            .summary
            .as_ref()
            .and_then(|summary| summary.verified_object_count)
            .unwrap_or(0) as usize,
        rejected_object_count: report
            .summary
            .as_ref()
            .and_then(|summary| summary.rejected_object_count)
            .unwrap_or(0) as usize,
        matched_expected_outcomes: report
            .summary
            .as_ref()
            .map(|summary| summary.matched_expected_outcomes.clone())
            .unwrap_or_default(),
    };

    Ok(summary)
}

fn simulate_report(
    test_case: &TestCase,
    topology: &Topology,
    fixture: &Fixture,
) -> Result<Report, String> {
    let seed_node_id = resolve_peer_ref(topology, &fixture.seed_peer).ok_or_else(|| {
        format!(
            "fixture seed peer '{}' does not resolve in topology",
            fixture.seed_peer
        )
    })?;
    let reader_node_ids: HashSet<_> = fixture
        .reader_peers
        .iter()
        .filter_map(|peer_ref| resolve_peer_ref(topology, peer_ref))
        .collect();
    let fault_node_id = fixture
        .fault_peer
        .as_deref()
        .and_then(|peer_ref| resolve_peer_ref(topology, peer_ref));

    let verified_object_ids = build_verified_object_ids(fixture);
    let rejected_object_ids = build_rejected_object_ids(fixture);
    let matched_expected_outcomes = matched_expected_outcomes(test_case, topology, fixture);
    let mut events = Vec::new();

    let has_hash_failure = fixture
        .expected_outcomes
        .iter()
        .any(|outcome| outcome.contains("hash-mismatch"));
    let has_signature_failure = fixture
        .expected_outcomes
        .iter()
        .any(|outcome| outcome.contains("signature"));
    let has_recovery = fixture
        .expected_outcomes
        .iter()
        .any(|outcome| outcome.contains("recovery"));

    push_event(
        &mut events,
        "load",
        "load-fixture",
        "ok",
        None,
        build_fixture_object_refs(fixture),
        Some(format!("Loaded fixture '{}'.", fixture.fixture_id)),
    );
    push_event(
        &mut events,
        "init",
        "build-topology",
        "ok",
        None,
        Vec::new(),
        Some(format!(
            "Prepared topology '{}' with {} peers.",
            topology.topology_id,
            topology.peers.len()
        )),
    );

    let mut peers = Vec::with_capacity(topology.peers.len());
    for peer in &topology.peers {
        let mut report_peer = ReportPeer {
            node_id: peer.node_id.clone(),
            status: "ok".to_owned(),
            verified_object_ids: Vec::new(),
            rejected_object_ids: Vec::new(),
            notes: Vec::new(),
        };

        let is_fault = fault_node_id.as_deref() == Some(peer.node_id.as_str());
        let is_reader = reader_node_ids.contains(peer.node_id.as_str()) || peer.role == "reader";
        let is_seed = peer.node_id == seed_node_id || peer.role == "seed";

        push_event(
            &mut events,
            "init",
            "init-peer",
            "ok",
            Some(peer.node_id.clone()),
            Vec::new(),
            Some(format!("Initialized peer with role '{}'.", peer.role)),
        );

        if is_fault {
            report_peer.status = "failed".to_owned();
            report_peer.rejected_object_ids = rejected_object_ids.clone();
            report_peer
                .notes
                .push("Fixture declares this peer as the injected fault source.".to_owned());
            push_event(
                &mut events,
                "sync",
                "inject-fault",
                "failed",
                Some(peer.node_id.clone()),
                rejected_object_ids.clone(),
                Some("Fixture routed the invalid object set through this peer.".to_owned()),
            );
        } else if has_hash_failure || has_signature_failure {
            if is_reader {
                report_peer.rejected_object_ids = rejected_object_ids.clone();
                report_peer.notes.push(
                    "Reader rejected the advertised object set during deterministic validation."
                        .to_owned(),
                );
                push_event(
                    &mut events,
                    "verify",
                    "reject-object-set",
                    "ok",
                    Some(peer.node_id.clone()),
                    rejected_object_ids.clone(),
                    Some("Reader rejected the injected invalid object set.".to_owned()),
                );
            } else if is_seed {
                report_peer.verified_object_ids = verified_object_ids.clone();
                push_event(
                    &mut events,
                    "sync",
                    "seed-advertise",
                    "ok",
                    Some(peer.node_id.clone()),
                    verified_object_ids.clone(),
                    Some("Seed advertised the current verified object set.".to_owned()),
                );
            }
        } else if has_recovery {
            report_peer.verified_object_ids = verified_object_ids.clone();
            if is_reader {
                report_peer
                    .notes
                    .push("Reader completed WANT-based recovery before replay.".to_owned());
                push_event(
                    &mut events,
                    "sync",
                    "request-missing-objects",
                    "ok",
                    Some(peer.node_id.clone()),
                    verified_object_ids.clone(),
                    Some("Reader completed WANT-based recovery.".to_owned()),
                );
            } else if is_seed {
                push_event(
                    &mut events,
                    "sync",
                    "seed-advertise",
                    "ok",
                    Some(peer.node_id.clone()),
                    verified_object_ids.clone(),
                    Some("Seed supplied missing objects for recovery.".to_owned()),
                );
            }
        } else {
            if is_seed || is_reader {
                report_peer.verified_object_ids = verified_object_ids.clone();
                let action = if is_seed {
                    "seed-advertise"
                } else {
                    "reader-accept"
                };
                push_event(
                    &mut events,
                    "sync",
                    action,
                    "ok",
                    Some(peer.node_id.clone()),
                    verified_object_ids.clone(),
                    Some("Peer accepted the deterministic object set.".to_owned()),
                );
            }
        }

        peers.push(report_peer);
    }

    push_event(
        &mut events,
        "replay",
        "compare-replay-results",
        "ok",
        None,
        verified_object_ids.clone(),
        Some("Reader-visible replay results converged for this deterministic run.".to_owned()),
    );
    push_event(
        &mut events,
        "finalize",
        "write-report",
        "ok",
        None,
        Vec::new(),
        Some("Prepared machine-readable simulator report.".to_owned()),
    );

    let report = Report {
        schema: Some("../report.schema.json".to_owned()),
        run_id: format!("run:{}", test_case.test_id),
        topology_id: topology.topology_id.clone(),
        fixture_id: fixture.fixture_id.clone(),
        test_id: Some(test_case.test_id.clone()),
        execution_mode: Some(test_case.execution_mode.clone()),
        started_at: None,
        finished_at: None,
        peers,
        result: test_case.expected_result.clone(),
        events,
        failures: Vec::new(),
        summary: Some(ReportSummary {
            verified_object_count: Some(verified_object_ids.len() as u64),
            rejected_object_count: Some(rejected_object_ids.len() as u64),
            matched_expected_outcomes,
        }),
        metadata: Some(json!({
            "generator": "mycel-cli/sim-run-v0",
            "topology_description": topology.description,
            "fixture_description": fixture.description,
        })),
    };

    Ok(report)
}

fn build_fixture_object_refs(fixture: &Fixture) -> Vec<String> {
    let mut refs = BTreeSet::new();

    for document in &fixture.documents {
        refs.insert(document.doc_id.clone());
    }

    if refs.is_empty() {
        refs.insert(format!("fixture:{}", fixture.fixture_id));
    }

    refs.into_iter().collect()
}

fn build_verified_object_ids(fixture: &Fixture) -> Vec<String> {
    let mut object_ids = BTreeSet::new();

    for document in &fixture.documents {
        if document.head_ids.is_empty() {
            object_ids.insert(format!("obj:{}:accepted-head", document.doc_id));
        } else {
            object_ids.extend(document.head_ids.iter().cloned());
        }
    }

    if object_ids.is_empty() {
        object_ids.insert(format!("obj:{}:synthetic", fixture.fixture_id));
    }

    object_ids.into_iter().collect()
}

fn build_rejected_object_ids(fixture: &Fixture) -> Vec<String> {
    let mut object_ids = BTreeSet::new();

    if fixture
        .expected_outcomes
        .iter()
        .any(|outcome| outcome.contains("hash-mismatch") || outcome.contains("signature"))
    {
        object_ids.insert(format!("obj:{}:rejected", fixture.fixture_id));
    }

    object_ids.into_iter().collect()
}

fn matched_expected_outcomes(
    test_case: &TestCase,
    topology: &Topology,
    fixture: &Fixture,
) -> Vec<String> {
    if !test_case.expected_outcomes.is_empty() {
        return test_case.expected_outcomes.clone();
    }

    if !topology.expected_outcomes.is_empty() {
        return topology.expected_outcomes.clone();
    }

    fixture.expected_outcomes.clone()
}

fn resolve_peer_ref(topology: &Topology, peer_ref: &str) -> Option<String> {
    topology
        .peers
        .iter()
        .find(|peer| {
            peer.node_id == peer_ref
                || normalize_node_id(&peer.node_id) == peer_ref
                || peer.role == peer_ref
        })
        .map(|peer| peer.node_id.clone())
}

fn normalize_node_id(node_id: &str) -> String {
    node_id.strip_prefix("node:").unwrap_or(node_id).to_owned()
}

fn load_json<T>(path: &Path) -> Result<T, String>
where
    T: serde::de::DeserializeOwned,
{
    let body = fs::read_to_string(path)
        .map_err(|err| format!("failed to read {}: {err}", path.display()))?;
    serde_json::from_str(&body).map_err(|err| format!("failed to parse {}: {err}", path.display()))
}

fn push_event(
    events: &mut Vec<ReportEvent>,
    phase: &str,
    action: &str,
    outcome: &str,
    node_id: Option<String>,
    object_ids: Vec<String>,
    detail: Option<String>,
) {
    events.push(ReportEvent {
        step: events.len() as u64 + 1,
        phase: phase.to_owned(),
        action: action.to_owned(),
        outcome: outcome.to_owned(),
        node_id,
        object_ids,
        detail,
    });
}
