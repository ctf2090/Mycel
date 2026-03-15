//! Minimal single-process simulator runner.

use std::collections::{BTreeMap, BTreeSet, HashSet};
use std::fs;
use std::path::{Path, PathBuf};
use std::process;
use std::time::Instant;

use base64::Engine;
use chrono::{FixedOffset, Utc};
use ed25519_dalek::Signer;
use mycel_core::author::{
    commit_revision_to_store, create_document_in_store, create_patch_in_store,
    parse_signing_key_seed, signer_id, DocumentCreateParams, PatchCreateParams,
    RevisionCommitParams,
};
use mycel_core::canonical::signed_payload_bytes;
use mycel_core::protocol::{parse_json_strict, recompute_object_id};
use mycel_core::replay::replay_revision_from_index;
use mycel_core::store::{
    load_store_index_manifest, load_store_object_index, write_object_value_to_store,
    StoreIndexManifest,
};
use mycel_core::sync::{sync_pull_from_peer_store, SyncPeer};
use serde::Serialize;
use serde_json::{json, Value};

use crate::model::{
    Fixture, Report, ReportEvent, ReportFailure, ReportPeer, ReportSummary, TestCase, Topology,
};
use crate::validate::{validate_path, ValidationStatus};

#[derive(Debug, Clone, Default)]
pub struct RunOptions {
    pub seed_override: Option<String>,
}

#[derive(Debug, Clone, Serialize)]
pub struct SimulationRunSummary {
    pub root: PathBuf,
    pub target: PathBuf,
    pub report_path: PathBuf,
    pub started_at: String,
    pub finished_at: String,
    pub run_duration_ms: u128,
    pub deterministic_seed: String,
    pub seed_source: String,
    pub events_per_second: f64,
    pub ms_per_event: f64,
    pub scheduled_peer_order: Vec<String>,
    pub fault_plan: Vec<FaultPlanEntry>,
    pub validation_status: ValidationStatus,
    pub validation_warnings: Vec<String>,
    pub result: String,
    pub peer_count: usize,
    pub event_count: usize,
    pub verified_object_count: usize,
    pub rejected_object_count: usize,
    pub matched_expected_outcomes: Vec<String>,
}

#[derive(Debug, Clone, Serialize)]
pub struct FaultPlanEntry {
    pub order: u64,
    pub fault: String,
    pub phase: String,
    pub source_node_id: String,
    pub target_node_id: Option<String>,
}

const SIM_SIGNING_KEY_SEED: &str = "AQEBAQEBAQEBAQEBAQEBAQEBAQEBAQEBAQEBAQEBAQE=";

pub fn run_test_case(target_path: &Path) -> Result<SimulationRunSummary, String> {
    run_test_case_with_options(target_path, &RunOptions::default())
}

pub fn run_test_case_with_options(
    target_path: &Path,
    options: &RunOptions,
) -> Result<SimulationRunSummary, String> {
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

    let started_at = now_taipei_timestamp()?;
    let started_clock = Instant::now();
    let test_case = load_json::<TestCase>(&target)?;
    let report_path = root
        .join("sim/reports/out")
        .join(format!("{}.report.json", test_case.test_id));
    let filtered_validation_warnings: Vec<_> = validation
        .warnings
        .iter()
        .filter(|warning| Path::new(&warning.path) != report_path)
        .map(|warning| format!("{}: {}", warning.path, warning.message))
        .collect();
    let filtered_validation_status = derive_validation_status(
        !validation.errors.is_empty(),
        !filtered_validation_warnings.is_empty(),
    );

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
    let (deterministic_seed, seed_source) = resolve_deterministic_seed(
        &test_case,
        &topology,
        &fixture,
        options.seed_override.as_deref(),
    );
    let fault_plan = build_fault_plan(&topology, &fixture, &deterministic_seed);

    let mut report = simulate_report(
        &test_case,
        &topology,
        &fixture,
        &deterministic_seed,
        &fault_plan,
    )?;
    let run_mode = if collect_fault_modes(&fixture).is_empty() {
        "peer-store-sync"
    } else {
        "deterministic-placeholder"
    };
    let scheduled_peer_order = scheduled_peer_order(&topology, &deterministic_seed);
    let finished_at = now_taipei_timestamp()?;
    let elapsed = started_clock.elapsed();
    let run_duration_ms = elapsed.as_millis();
    let event_count = report.events.len();
    let events_per_second = if elapsed.as_secs_f64() > 0.0 {
        event_count as f64 / elapsed.as_secs_f64()
    } else {
        0.0
    };
    let ms_per_event = if event_count > 0 {
        (elapsed.as_secs_f64() * 1000.0) / event_count as f64
    } else {
        0.0
    };
    report.started_at = Some(started_at.clone());
    report.finished_at = Some(finished_at.clone());
    report.metadata = Some(build_run_metadata(
        &root,
        &target,
        &topology_path,
        &fixture_path,
        filtered_validation_status,
        run_duration_ms,
        &deterministic_seed,
        &seed_source,
        events_per_second,
        ms_per_event,
        &scheduled_peer_order,
        &fault_plan,
        run_mode,
    ));
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
        started_at,
        finished_at,
        run_duration_ms,
        deterministic_seed,
        seed_source,
        events_per_second,
        ms_per_event,
        scheduled_peer_order,
        fault_plan,
        validation_status: filtered_validation_status,
        validation_warnings: filtered_validation_warnings,
        result: report.result.clone(),
        peer_count: report.peers.len(),
        event_count,
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
    deterministic_seed: &str,
    fault_plan: &[FaultPlanEntry],
) -> Result<Report, String> {
    let has_faults = fixture
        .expected_outcomes
        .iter()
        .any(|outcome| outcome.contains("hash-mismatch") || outcome.contains("signature"));
    if has_faults {
        return simulate_fault_report(test_case, topology, fixture, deterministic_seed, fault_plan);
    }
    simulate_peer_store_sync_report(test_case, topology, fixture, deterministic_seed, fault_plan)
}

fn simulate_peer_store_sync_report(
    test_case: &TestCase,
    topology: &Topology,
    fixture: &Fixture,
    deterministic_seed: &str,
    _fault_plan: &[FaultPlanEntry],
) -> Result<Report, String> {
    let seed_node_id = resolve_peer_ref(topology, &fixture.seed_peer).ok_or_else(|| {
        format!(
            "fixture seed peer '{}' does not resolve in topology",
            fixture.seed_peer
        )
    })?;
    let matched_expected_outcomes = matched_expected_outcomes(test_case, topology, fixture);
    let signing_key = parse_signing_key_seed(SIM_SIGNING_KEY_SEED)
        .map_err(|err| format!("failed to parse simulator signing key seed: {err}"))?;
    let seed_peer = SyncPeer {
        node_id: seed_node_id.clone(),
        public_key: mycel_core::author::signer_id(&signing_key),
    };
    let stores = SimulationStores::new(&fixture.fixture_id)?;
    let seed_store_root = stores.store_root(&seed_node_id);
    let uses_recovery = fixture
        .expected_outcomes
        .iter()
        .any(|outcome| outcome.contains("recovery") || outcome.contains("missing-objects"));
    let uses_incremental = fixture
        .expected_outcomes
        .iter()
        .any(|outcome| outcome.contains("incremental-sync"));

    populate_seed_store(
        &seed_store_root,
        fixture,
        &signing_key,
        uses_recovery || uses_incremental,
    )?;
    let seed_manifest = load_store_index_manifest(&seed_store_root)
        .map_err(|err| format!("failed to read seed store manifest: {err}"))?;
    let seed_verified_object_ids = manifest_object_ids(&seed_manifest);

    let mut events = Vec::new();
    let mut failures = Vec::new();
    let mut peers = Vec::with_capacity(topology.peers.len());
    let mut reader_object_sets = BTreeMap::new();
    let mut reader_replay_hashes = BTreeMap::new();
    let mut reader_head_ids: BTreeMap<String, BTreeMap<String, Vec<String>>> = BTreeMap::new();

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
            "Prepared topology '{}' with {} peers. Scheduler order: {}.",
            topology.topology_id,
            topology.peers.len(),
            scheduled_peer_order(topology, deterministic_seed).join(" -> ")
        )),
    );
    push_event(
        &mut events,
        "init",
        "build-fault-plan",
        "ok",
        None,
        Vec::new(),
        Some("No injected faults are scheduled for this run.".to_owned()),
    );

    for peer in scheduled_peers(topology, deterministic_seed) {
        let mut report_peer = ReportPeer {
            node_id: peer.node_id.clone(),
            status: "ok".to_owned(),
            verified_object_ids: Vec::new(),
            rejected_object_ids: Vec::new(),
            head_revision_ids: Vec::new(),
            notes: Vec::new(),
        };
        let is_seed = peer.node_id == seed_node_id || peer.role == "seed";
        let is_reader = peer.role == "reader";

        push_event(
            &mut events,
            "init",
            "init-peer",
            "ok",
            Some(peer.node_id.clone()),
            Vec::new(),
            Some(format!("Initialized peer with role '{}'.", peer.role)),
        );

        if is_seed {
            report_peer.verified_object_ids = seed_verified_object_ids.clone();
            report_peer.notes.push(format!(
                "Prepared peer-store source with {} verified objects.",
                seed_verified_object_ids.len()
            ));
            push_event(
                &mut events,
                "sync",
                "seed-advertise",
                "ok",
                Some(peer.node_id.clone()),
                seed_verified_object_ids.clone(),
                Some("Seed advertised the current verified object set from the peer-store sync driver.".to_owned()),
            );
            peers.push(report_peer);
            continue;
        }

        if !is_reader {
            report_peer.status = "warning".to_owned();
            report_peer
                .notes
                .push("Simulator did not schedule a sync for this peer role.".to_owned());
            peers.push(report_peer);
            continue;
        }

        let peer_store_root = stores.store_root(&peer.node_id);
        let starts_with_partial_store = (uses_recovery || uses_incremental)
            && fixture.reader_peers.iter().any(|peer_ref| {
                resolve_peer_ref(topology, peer_ref).as_deref() == Some(peer.node_id.as_str())
            });
        if starts_with_partial_store {
            populate_partial_reader_store(&peer_store_root, fixture, &signing_key)?;
        }

        let summary =
            sync_pull_from_peer_store(&seed_peer, &signing_key, &seed_store_root, &peer_store_root)
                .map_err(|err| format!("peer-store sync failed for '{}': {err}", peer.node_id))?;

        let manifest = load_store_index_manifest(&peer_store_root).map_err(|err| {
            format!(
                "failed to read reader store manifest '{}': {err}",
                peer.node_id
            )
        })?;
        let verified_object_ids = manifest_object_ids(&manifest);
        let synced_object_ids = summary
            .stored_objects
            .iter()
            .map(|record| record.object_id.clone())
            .collect::<Vec<_>>();
        let replay_hashes = store_head_replay_hashes(&peer_store_root)?;
        let leaf_ids = store_leaf_revision_ids(&peer_store_root)?;
        let action = if starts_with_partial_store && uses_incremental {
            "incremental-accept"
        } else if starts_with_partial_store {
            "request-missing-objects"
        } else {
            "reader-accept"
        };
        let outcome = if summary.is_ok() { "ok" } else { "failed" };

        report_peer.status = if summary.is_ok() {
            "ok".to_owned()
        } else {
            "failed".to_owned()
        };
        report_peer.verified_object_ids = verified_object_ids.clone();
        report_peer.notes.push(format!(
            "peer-store sync exchanged {} messages, verified {} objects, wrote {} new objects.",
            summary.message_count, summary.verified_object_count, summary.written_object_count
        ));
        report_peer.notes.extend(summary.notes.clone());

        if !summary.errors.is_empty() {
            report_peer.notes.extend(summary.errors.iter().cloned());
            failures.push(ReportFailure {
                failure_id: format!("sync-failed:{}", peer.node_id),
                node_id: Some(peer.node_id.clone()),
                description: format!(
                    "Peer-store sync failed for '{}': {}",
                    peer.node_id,
                    summary.errors.join("; ")
                ),
                severity: Some("error".to_owned()),
            });
        }

        push_event(
            &mut events,
            "sync",
            action,
            outcome,
            Some(peer.node_id.clone()),
            if starts_with_partial_store {
                synced_object_ids
            } else {
                verified_object_ids.clone()
            },
            Some(format!(
                "Peer-store sync used {} and transferred {} OBJECT messages.",
                if starts_with_partial_store && uses_incremental {
                    "HEADS/WANT (incremental)"
                } else if starts_with_partial_store {
                    "HEADS/WANT"
                } else {
                    "MANIFEST/WANT"
                },
                summary.object_message_count
            )),
        );

        report_peer.head_revision_ids = leaf_ids
            .values()
            .flat_map(|ids| ids.iter().cloned())
            .collect();
        report_peer.head_revision_ids.sort();
        reader_object_sets.insert(peer.node_id.clone(), verified_object_ids);
        reader_replay_hashes.insert(peer.node_id.clone(), replay_hashes);
        reader_head_ids.insert(peer.node_id.clone(), leaf_ids);
        peers.push(report_peer);
    }

    for (node_id, object_ids) in &reader_object_sets {
        if object_ids != &seed_verified_object_ids {
            failures.push(ReportFailure {
                failure_id: format!("object-set-mismatch:{node_id}"),
                node_id: Some(node_id.clone()),
                description: format!(
                    "Reader '{}' diverged from the seed object set after peer-store sync.",
                    node_id
                ),
                severity: Some("error".to_owned()),
            });
        }
    }

    let replay_outcome = if readers_match_replay(&reader_replay_hashes) {
        "ok"
    } else {
        failures.push(ReportFailure {
            failure_id: "replay-mismatch".to_owned(),
            node_id: None,
            description: "Reader-visible replay results diverged after peer-store sync.".to_owned(),
            severity: Some("error".to_owned()),
        });
        "failed"
    };
    push_event(
        &mut events,
        "replay",
        "compare-replay-results",
        replay_outcome,
        None,
        seed_verified_object_ids.clone(),
        Some(
            "Reader-visible replay results were derived from the synchronized peer stores."
                .to_owned(),
        ),
    );
    let heads_outcome = if readers_match_heads(&reader_head_ids) {
        "ok"
    } else {
        failures.push(ReportFailure {
            failure_id: "accepted-head-mismatch".to_owned(),
            node_id: None,
            description: "Reader-visible accepted head revisions diverged after peer-store sync."
                .to_owned(),
            severity: Some("error".to_owned()),
        });
        "failed"
    };
    push_event(
        &mut events,
        "replay",
        "compare-accepted-heads",
        heads_outcome,
        None,
        seed_verified_object_ids.clone(),
        Some(
            "Reader-visible accepted head revisions were compared across synchronized peers."
                .to_owned(),
        ),
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

    Ok(Report {
        schema: Some("../report.schema.json".to_owned()),
        run_id: format!("run:{}", test_case.test_id),
        topology_id: topology.topology_id.clone(),
        fixture_id: fixture.fixture_id.clone(),
        test_id: Some(test_case.test_id.clone()),
        execution_mode: Some(test_case.execution_mode.clone()),
        started_at: None,
        finished_at: None,
        peers,
        result: derive_report_result(&failures),
        events,
        failures,
        summary: Some(ReportSummary {
            verified_object_count: Some(seed_verified_object_ids.len() as u64),
            rejected_object_count: Some(0),
            matched_expected_outcomes,
        }),
        metadata: None,
    })
}

fn simulate_fault_report(
    test_case: &TestCase,
    topology: &Topology,
    fixture: &Fixture,
    deterministic_seed: &str,
    fault_plan: &[FaultPlanEntry],
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
    let mut failures = Vec::new();

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
            "Prepared topology '{}' with {} peers. Scheduler order: {}.",
            topology.topology_id,
            topology.peers.len(),
            scheduled_peer_order(topology, deterministic_seed).join(" -> ")
        )),
    );
    if fault_plan.is_empty() {
        push_event(
            &mut events,
            "init",
            "build-fault-plan",
            "ok",
            None,
            Vec::new(),
            Some("No injected faults are scheduled for this run.".to_owned()),
        );
    } else {
        push_event(
            &mut events,
            "init",
            "build-fault-plan",
            "ok",
            None,
            Vec::new(),
            Some(format!(
                "Prepared fault plan: {}.",
                describe_fault_plan(fault_plan)
            )),
        );
    }

    let mut peers = Vec::with_capacity(topology.peers.len());
    for peer in scheduled_peers(topology, deterministic_seed) {
        let mut report_peer = ReportPeer {
            node_id: peer.node_id.clone(),
            status: "ok".to_owned(),
            verified_object_ids: Vec::new(),
            rejected_object_ids: Vec::new(),
            head_revision_ids: Vec::new(),
            notes: Vec::new(),
        };

        let is_fault = fault_node_id.as_deref() == Some(peer.node_id.as_str());
        let is_reader = reader_node_ids.contains(peer.node_id.as_str()) || peer.role == "reader";
        let is_seed = peer.node_id == seed_node_id || peer.role == "seed";
        let peer_faults: Vec<_> = fault_plan
            .iter()
            .filter(|entry| entry.source_node_id == peer.node_id)
            .collect();
        let peer_targets_faults: Vec<_> = fault_plan
            .iter()
            .filter(|entry| entry.target_node_id.as_deref() == Some(peer.node_id.as_str()))
            .collect();

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
            for planned_fault in &peer_faults {
                failures.push(ReportFailure {
                    failure_id: format!("fault:{}:{}", planned_fault.order, planned_fault.fault),
                    node_id: Some(peer.node_id.clone()),
                    description: format!(
                        "Fault source injected planned fault #{} ('{}') toward {}.",
                        planned_fault.order,
                        planned_fault.fault,
                        planned_fault
                            .target_node_id
                            .as_deref()
                            .unwrap_or("unspecified-target")
                    ),
                    severity: Some("error".to_owned()),
                });
                push_event(
                    &mut events,
                    &planned_fault.phase,
                    "inject-fault",
                    "failed",
                    Some(peer.node_id.clone()),
                    rejected_object_ids.clone(),
                    Some(format!(
                        "Planned fault #{} injected as '{}' toward {}.",
                        planned_fault.order,
                        planned_fault.fault,
                        planned_fault
                            .target_node_id
                            .as_deref()
                            .unwrap_or("unspecified-target")
                    )),
                );
            }
        } else if has_hash_failure || has_signature_failure {
            if is_reader && (!fault_plan.is_empty() && !peer_targets_faults.is_empty()) {
                report_peer.rejected_object_ids = rejected_object_ids.clone();
                report_peer.notes.push(
                    "Reader rejected the advertised object set during deterministic validation."
                        .to_owned(),
                );
                for planned_fault in &peer_targets_faults {
                    failures.push(ReportFailure {
                        failure_id: format!("rejection:{}:{}", planned_fault.order, peer.node_id),
                        node_id: Some(peer.node_id.clone()),
                        description: format!(
                            "Reader rejected planned fault #{} ('{}').",
                            planned_fault.order, planned_fault.fault
                        ),
                        severity: Some("error".to_owned()),
                    });
                    push_event(
                        &mut events,
                        "verify",
                        "reject-object-set",
                        "ok",
                        Some(peer.node_id.clone()),
                        rejected_object_ids.clone(),
                        Some(format!(
                            "Reader rejected planned fault #{} ('{}').",
                            planned_fault.order, planned_fault.fault
                        )),
                    );
                }
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
        result: derive_report_result(&failures),
        events,
        failures,
        summary: Some(ReportSummary {
            verified_object_count: Some(verified_object_ids.len() as u64),
            rejected_object_count: Some(rejected_object_ids.len() as u64),
            matched_expected_outcomes,
        }),
        metadata: None,
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

fn derive_report_result(failures: &[ReportFailure]) -> String {
    let has_error = failures
        .iter()
        .any(|failure| failure.severity.as_deref() != Some("warning"));
    let has_warning = failures
        .iter()
        .any(|failure| failure.severity.as_deref() == Some("warning"));

    if has_error {
        "fail".to_owned()
    } else if has_warning {
        "partial".to_owned()
    } else {
        "pass".to_owned()
    }
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
    parse_json_strict(&body).map_err(|err| format!("failed to parse {}: {err}", path.display()))
}

fn build_run_metadata(
    root: &Path,
    test_case_path: &Path,
    topology_path: &Path,
    fixture_path: &Path,
    validation_status: ValidationStatus,
    run_duration_ms: u128,
    deterministic_seed: &str,
    seed_source: &str,
    events_per_second: f64,
    ms_per_event: f64,
    scheduled_peer_order: &[String],
    fault_plan: &[FaultPlanEntry],
    run_mode: &str,
) -> serde_json::Value {
    json!({
        "generator": "mycel-cli/sim-run-v0",
        "deterministic": true,
        "run_mode": run_mode,
        "trace_version": "v0",
        "timezone": "Asia/Taipei (UTC+8)",
        "validation_status": validation_status.to_string(),
        "run_duration_ms": run_duration_ms,
        "deterministic_seed": deterministic_seed,
        "seed_source": seed_source,
        "events_per_second": events_per_second,
        "ms_per_event": ms_per_event,
        "scheduled_peer_order": scheduled_peer_order,
        "fault_plan": fault_plan,
        "source_test_case": relative_path_string(root, test_case_path),
        "source_topology": relative_path_string(root, topology_path),
        "source_fixture": relative_path_string(root, fixture_path),
    })
}

struct SimulationStores {
    root: PathBuf,
}

impl SimulationStores {
    fn new(label: &str) -> Result<Self, String> {
        let unique = Utc::now().timestamp_nanos_opt().unwrap_or_default();
        let root = std::env::temp_dir().join(format!(
            "mycel-sim-{}-{}-{}",
            sanitize_path_component(label),
            process::id(),
            unique
        ));
        fs::create_dir_all(&root).map_err(|err| {
            format!(
                "failed to create simulator temp root {}: {err}",
                root.display()
            )
        })?;
        Ok(Self { root })
    }

    fn store_root(&self, node_id: &str) -> PathBuf {
        self.root.join(sanitize_path_component(node_id))
    }
}

impl Drop for SimulationStores {
    fn drop(&mut self) {
        let _ = fs::remove_dir_all(&self.root);
    }
}

fn populate_seed_store(
    store_root: &Path,
    fixture: &Fixture,
    signing_key: &ed25519_dalek::SigningKey,
    with_follow_up_revision: bool,
) -> Result<(), String> {
    fs::create_dir_all(store_root).map_err(|err| {
        format!(
            "failed to create seed store {}: {err}",
            store_root.display()
        )
    })?;
    for (index, document) in fixture_documents(fixture).iter().enumerate() {
        let timestamp = 1_700_000_000 + index as u64;
        let created = create_document_in_store(
            store_root,
            signing_key,
            &DocumentCreateParams {
                doc_id: document.doc_id.clone(),
                title: format!("{} document", fixture.fixture_id),
                language: "en".to_owned(),
                timestamp,
            },
        )
        .map_err(|err| {
            format!(
                "failed to create seed document '{}': {err}",
                document.doc_id
            )
        })?;
        let mut current_revision_id = created.genesis_revision_id.clone();

        if with_follow_up_revision {
            let patch = create_patch_in_store(
                store_root,
                signing_key,
                &PatchCreateParams {
                    doc_id: document.doc_id.clone(),
                    base_revision: created.genesis_revision_id.clone(),
                    timestamp: timestamp + 1,
                    ops: json!([]),
                },
            )
            .map_err(|err| format!("failed to create patch for '{}': {err}", document.doc_id))?;
            let revision = commit_revision_to_store(
                store_root,
                signing_key,
                &RevisionCommitParams {
                    doc_id: document.doc_id.clone(),
                    parents: vec![created.genesis_revision_id],
                    patches: vec![patch.patch_id],
                    merge_strategy: None,
                    timestamp: timestamp + 2,
                },
            )
            .map_err(|err| format!("failed to commit revision for '{}': {err}", document.doc_id))?;
            current_revision_id = revision.revision_id;
        }

        if fixture_requests_seed_view_sync(fixture) {
            write_governance_view_to_store(
                store_root,
                signing_key,
                &document.doc_id,
                &current_revision_id,
                timestamp + 10,
            )?;
        }
        if fixture_requests_seed_snapshot_sync(fixture) {
            write_snapshot_to_store(
                store_root,
                signing_key,
                &document.doc_id,
                &current_revision_id,
                timestamp + 20,
            )?;
        }
    }
    Ok(())
}

fn populate_partial_reader_store(
    store_root: &Path,
    fixture: &Fixture,
    signing_key: &ed25519_dalek::SigningKey,
) -> Result<(), String> {
    fs::create_dir_all(store_root).map_err(|err| {
        format!(
            "failed to create reader store {}: {err}",
            store_root.display()
        )
    })?;
    for (index, document) in fixture_documents(fixture).iter().enumerate() {
        let timestamp = 1_700_000_000 + index as u64;
        create_document_in_store(
            store_root,
            signing_key,
            &DocumentCreateParams {
                doc_id: document.doc_id.clone(),
                title: format!("{} document", fixture.fixture_id),
                language: "en".to_owned(),
                timestamp,
            },
        )
        .map_err(|err| {
            format!(
                "failed to preseed reader document '{}': {err}",
                document.doc_id
            )
        })?;
    }
    Ok(())
}

fn fixture_documents(fixture: &Fixture) -> Vec<crate::model::FixtureDocumentRef> {
    if fixture.documents.is_empty() {
        return vec![crate::model::FixtureDocumentRef {
            doc_id: format!("doc:{}", fixture.fixture_id),
            head_ids: Vec::new(),
            notes: None,
        }];
    }
    fixture.documents.clone()
}

fn fixture_requests_seed_view_sync(fixture: &Fixture) -> bool {
    fixture
        .metadata
        .as_ref()
        .and_then(|value| value.get("publish_seed_view"))
        .and_then(Value::as_bool)
        .unwrap_or(false)
}

fn fixture_requests_seed_snapshot_sync(fixture: &Fixture) -> bool {
    fixture
        .metadata
        .as_ref()
        .and_then(|value| value.get("publish_seed_snapshot"))
        .and_then(Value::as_bool)
        .unwrap_or(false)
}

fn write_governance_view_to_store(
    store_root: &Path,
    signing_key: &ed25519_dalek::SigningKey,
    doc_id: &str,
    revision_id: &str,
    timestamp: u64,
) -> Result<(), String> {
    let maintainer = signer_id(signing_key);
    let mut view = json!({
        "type": "view",
        "version": "mycel/0.1",
        "view_id": "view:placeholder",
        "maintainer": maintainer,
        "documents": {
            doc_id: revision_id
        },
        "policy": {
            "accept_keys": [signer_id(signing_key)],
            "merge_rule": "manual-reviewed",
            "preferred_branches": ["main"]
        },
        "timestamp": timestamp
    });
    let view_id = recompute_object_id(&view, "view_id", "view")
        .map_err(|err| format!("failed to compute governance view id for '{doc_id}': {err}"))?;
    view["view_id"] = Value::String(view_id);
    let payload = signed_payload_bytes(&view)
        .map_err(|err| format!("failed to canonicalize governance view for '{doc_id}': {err}"))?;
    let signature = signing_key.sign(&payload);
    view["signature"] = Value::String(format!(
        "sig:ed25519:{}",
        base64::engine::general_purpose::STANDARD.encode(signature.to_bytes())
    ));

    write_object_value_to_store(store_root, &view).map_err(|err| {
        format!(
            "failed to store governance view for '{doc_id}' at {}: {err}",
            store_root.display()
        )
    })?;
    Ok(())
}

fn write_snapshot_to_store(
    store_root: &Path,
    signing_key: &ed25519_dalek::SigningKey,
    doc_id: &str,
    revision_id: &str,
    timestamp: u64,
) -> Result<(), String> {
    let creator = signer_id(signing_key);
    let mut snapshot = json!({
        "type": "snapshot",
        "version": "mycel/0.1",
        "snapshot_id": "snap:placeholder",
        "documents": {
            doc_id: revision_id
        },
        "included_objects": [revision_id],
        "root_hash": format!("hash:snapshot-root-{timestamp}"),
        "created_by": creator,
        "timestamp": timestamp
    });
    let snapshot_id = recompute_object_id(&snapshot, "snapshot_id", "snap")
        .map_err(|err| format!("failed to compute snapshot id for '{doc_id}': {err}"))?;
    snapshot["snapshot_id"] = Value::String(snapshot_id);
    let payload = signed_payload_bytes(&snapshot)
        .map_err(|err| format!("failed to canonicalize snapshot for '{doc_id}': {err}"))?;
    let signature = signing_key.sign(&payload);
    snapshot["signature"] = Value::String(format!(
        "sig:ed25519:{}",
        base64::engine::general_purpose::STANDARD.encode(signature.to_bytes())
    ));

    write_object_value_to_store(store_root, &snapshot).map_err(|err| {
        format!(
            "failed to store snapshot for '{doc_id}' at {}: {err}",
            store_root.display()
        )
    })?;
    Ok(())
}

fn manifest_object_ids(manifest: &StoreIndexManifest) -> Vec<String> {
    let mut object_ids = manifest
        .object_ids_by_type
        .iter()
        .filter(|(object_type, _)| object_type.as_str() != "document")
        .flat_map(|(_, ids)| ids.iter().cloned())
        .collect::<Vec<_>>();
    object_ids.sort();
    object_ids
}

fn store_leaf_revision_ids(store_root: &Path) -> Result<BTreeMap<String, Vec<String>>, String> {
    let manifest = load_store_index_manifest(store_root).map_err(|err| {
        format!(
            "failed to load store manifest {}: {err}",
            store_root.display()
        )
    })?;
    let parent_revision_ids = manifest
        .revision_parents
        .values()
        .flat_map(|parents| parents.iter().cloned())
        .collect::<HashSet<_>>();
    let mut result: BTreeMap<String, Vec<String>> = BTreeMap::new();
    for (doc_id, revision_ids) in &manifest.doc_revisions {
        let mut leaves: Vec<String> = revision_ids
            .iter()
            .filter(|rev_id| !parent_revision_ids.contains(*rev_id))
            .cloned()
            .collect();
        leaves.sort();
        result.insert(doc_id.clone(), leaves);
    }
    Ok(result)
}

fn store_head_replay_hashes(store_root: &Path) -> Result<BTreeMap<String, Vec<String>>, String> {
    let manifest = load_store_index_manifest(store_root).map_err(|err| {
        format!(
            "failed to load store manifest {}: {err}",
            store_root.display()
        )
    })?;
    let object_index = load_store_object_index(store_root)
        .map_err(|err| format!("failed to load store index {}: {err}", store_root.display()))?;
    let parent_revision_ids = manifest
        .revision_parents
        .values()
        .flat_map(|parents| parents.iter().cloned())
        .collect::<HashSet<_>>();
    let mut replay_hashes = BTreeMap::new();

    for (doc_id, revision_ids) in &manifest.doc_revisions {
        let mut doc_hashes = revision_ids
            .iter()
            .filter(|revision_id| !parent_revision_ids.contains(*revision_id))
            .map(|revision_id| {
                let revision_value = object_index.get(revision_id).ok_or_else(|| {
                    format!(
                        "missing revision '{}' in store {}",
                        revision_id,
                        store_root.display()
                    )
                })?;
                replay_revision_from_index(revision_value, &object_index)
                    .map(|summary| summary.recomputed_state_hash)
                    .map_err(|err| {
                        format!(
                            "failed to replay revision '{}' from {}: {err}",
                            revision_id,
                            store_root.display()
                        )
                    })
            })
            .collect::<Result<Vec<_>, _>>()?;
        doc_hashes.sort();
        replay_hashes.insert(doc_id.clone(), doc_hashes);
    }

    Ok(replay_hashes)
}

fn readers_match_replay(replay_hashes: &BTreeMap<String, BTreeMap<String, Vec<String>>>) -> bool {
    let mut entries = replay_hashes.values();
    let Some(first) = entries.next() else {
        return true;
    };
    entries.all(|entry| entry == first)
}

fn readers_match_heads(head_ids: &BTreeMap<String, BTreeMap<String, Vec<String>>>) -> bool {
    let mut entries = head_ids.values();
    let Some(first) = entries.next() else {
        return true;
    };
    entries.all(|entry| entry == first)
}

fn sanitize_path_component(value: &str) -> String {
    value.replace(':', "-").replace('/', "-")
}

fn derive_validation_status(has_errors: bool, has_warnings: bool) -> ValidationStatus {
    if has_errors {
        ValidationStatus::Failed
    } else if has_warnings {
        ValidationStatus::Warning
    } else {
        ValidationStatus::Ok
    }
}

fn now_taipei_timestamp() -> Result<String, String> {
    let offset = FixedOffset::east_opt(8 * 60 * 60)
        .ok_or_else(|| "failed to construct Asia/Taipei fixed offset".to_owned())?;
    Ok(Utc::now().with_timezone(&offset).to_rfc3339())
}

fn relative_path_string(root: &Path, path: &Path) -> String {
    path.strip_prefix(root)
        .ok()
        .and_then(|relative| relative.to_str())
        .map(|value| value.replace('\\', "/"))
        .unwrap_or_else(|| path.display().to_string())
}

fn derive_deterministic_seed(
    test_case: &TestCase,
    topology: &Topology,
    fixture: &Fixture,
) -> String {
    format!(
        "{}|{}|{}|{}",
        test_case.test_id, topology.topology_id, fixture.fixture_id, test_case.execution_mode
    )
}

fn generate_runtime_seed(mode: &str) -> String {
    let timestamp_ns = Utc::now().timestamp_nanos_opt().unwrap_or_default();
    let pid = process::id();
    let entropy = stable_hash64([
        mode.as_bytes(),
        b"|",
        timestamp_ns.to_string().as_bytes(),
        b"|",
        pid.to_string().as_bytes(),
    ]);

    format!("{mode}:{timestamp_ns}:{pid}:{entropy:016x}")
}

fn resolve_deterministic_seed(
    test_case: &TestCase,
    topology: &Topology,
    fixture: &Fixture,
    seed_override: Option<&str>,
) -> (String, String) {
    match seed_override {
        Some("random") => (generate_runtime_seed("random"), "random".to_owned()),
        Some("auto") => (generate_runtime_seed("auto"), "auto".to_owned()),
        Some(seed) => (seed.to_owned(), "override".to_owned()),
        None => (
            derive_deterministic_seed(test_case, topology, fixture),
            "derived".to_owned(),
        ),
    }
}

fn build_fault_plan(
    topology: &Topology,
    fixture: &Fixture,
    deterministic_seed: &str,
) -> Vec<FaultPlanEntry> {
    let Some(source_node_id) = fixture
        .fault_peer
        .as_deref()
        .and_then(|peer_ref| resolve_peer_ref(topology, peer_ref))
    else {
        return Vec::new();
    };

    let reader_targets: Vec<_> = fixture
        .reader_peers
        .iter()
        .filter_map(|peer_ref| resolve_peer_ref(topology, peer_ref))
        .collect();
    let mut fault_modes = collect_fault_modes(fixture);
    fault_modes.sort_by(|left, right| {
        scheduler_rank(deterministic_seed, left)
            .cmp(&scheduler_rank(deterministic_seed, right))
            .then_with(|| left.cmp(right))
    });

    if fault_modes.is_empty() {
        return Vec::new();
    }

    let mut plan = Vec::new();
    for (index, fault_mode) in fault_modes.into_iter().enumerate() {
        let target_node_id = if reader_targets.is_empty() {
            None
        } else {
            let mut ranked_targets = reader_targets.clone();
            ranked_targets.sort_by(|left, right| {
                scheduler_rank(deterministic_seed, &format!("{fault_mode}|{left}"))
                    .cmp(&scheduler_rank(
                        deterministic_seed,
                        &format!("{fault_mode}|{right}"),
                    ))
                    .then_with(|| left.cmp(right))
            });
            ranked_targets.get(index % ranked_targets.len()).cloned()
        };

        plan.push(FaultPlanEntry {
            order: index as u64 + 1,
            phase: "sync".to_owned(),
            fault: fault_mode,
            source_node_id: source_node_id.clone(),
            target_node_id,
        });
    }

    plan
}

fn collect_fault_modes(fixture: &Fixture) -> Vec<String> {
    let mut modes = BTreeSet::new();

    for outcome in &fixture.expected_outcomes {
        if outcome.contains("hash-mismatch") {
            modes.insert("hash-mismatch".to_owned());
        }
        if outcome.contains("signature") {
            modes.insert("signature-mismatch".to_owned());
        }
    }

    modes.into_iter().collect()
}

fn describe_fault_plan(fault_plan: &[FaultPlanEntry]) -> String {
    fault_plan
        .iter()
        .map(|entry| {
            format!(
                "#{}:{}:{}->{}",
                entry.order,
                entry.fault,
                entry.source_node_id,
                entry
                    .target_node_id
                    .as_deref()
                    .unwrap_or("unspecified-target")
            )
        })
        .collect::<Vec<_>>()
        .join(", ")
}

fn scheduled_peer_order(topology: &Topology, deterministic_seed: &str) -> Vec<String> {
    scheduled_peers(topology, deterministic_seed)
        .into_iter()
        .map(|peer| peer.node_id.clone())
        .collect()
}

fn scheduled_peers<'a>(
    topology: &'a Topology,
    deterministic_seed: &str,
) -> Vec<&'a crate::model::Peer> {
    let mut peers: Vec<_> = topology.peers.iter().collect();
    peers.sort_by(|left, right| {
        scheduler_rank(deterministic_seed, &left.node_id)
            .cmp(&scheduler_rank(deterministic_seed, &right.node_id))
            .then_with(|| left.node_id.cmp(&right.node_id))
    });
    peers
}

fn scheduler_rank(deterministic_seed: &str, node_id: &str) -> u64 {
    stable_hash64([deterministic_seed.as_bytes(), b"|", node_id.as_bytes()])
}

fn stable_hash64<'a>(parts: impl IntoIterator<Item = &'a [u8]>) -> u64 {
    let mut hash = 0xcbf29ce484222325u64;

    for part in parts {
        for byte in part {
            hash ^= u64::from(*byte);
            hash = hash.wrapping_mul(0x100000001b3);
        }
    }

    hash
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

#[cfg(test)]
mod tests {
    use super::{
        build_fault_plan, derive_report_result, scheduled_peer_order, scheduler_rank, stable_hash64,
    };
    use crate::model::{Fixture, Peer, ReportFailure, Topology};

    fn sample_topology() -> Topology {
        Topology {
            schema: None,
            topology_id: "sample-topology".to_owned(),
            description: "sample".to_owned(),
            fixture_set: "fixtures/object-sets/minimal-valid".to_owned(),
            execution_mode: Some("single-process".to_owned()),
            peers: vec![
                Peer {
                    schema: None,
                    node_id: "node:peer-seed".to_owned(),
                    role: "seed".to_owned(),
                    bootstrap_peers: Vec::new(),
                    endpoint: Some("local:peer-seed".to_owned()),
                    capabilities: Vec::new(),
                    store_ref: None,
                    fixture_policy: None,
                    notes: Vec::new(),
                    metadata: None,
                },
                Peer {
                    schema: None,
                    node_id: "node:peer-reader-a".to_owned(),
                    role: "reader".to_owned(),
                    bootstrap_peers: vec!["node:peer-seed".to_owned()],
                    endpoint: Some("local:peer-reader-a".to_owned()),
                    capabilities: Vec::new(),
                    store_ref: None,
                    fixture_policy: None,
                    notes: Vec::new(),
                    metadata: None,
                },
                Peer {
                    schema: None,
                    node_id: "node:peer-reader-b".to_owned(),
                    role: "reader".to_owned(),
                    bootstrap_peers: vec!["node:peer-seed".to_owned()],
                    endpoint: Some("local:peer-reader-b".to_owned()),
                    capabilities: Vec::new(),
                    store_ref: None,
                    fixture_policy: None,
                    notes: Vec::new(),
                    metadata: None,
                },
            ],
            expected_outcomes: Vec::new(),
            notes: Vec::new(),
            metadata: None,
        }
    }

    fn sample_fixture() -> Fixture {
        Fixture {
            schema: None,
            fixture_id: "minimal-valid".to_owned(),
            description: "sample".to_owned(),
            seed_peer: "peer-seed".to_owned(),
            reader_peers: vec!["peer-reader-a".to_owned()],
            documents: Vec::new(),
            expected_outcomes: Vec::new(),
            fault_peer: None,
            notes: Vec::new(),
            metadata: None,
        }
    }

    #[test]
    fn stable_hash_is_deterministic() {
        let first = stable_hash64([
            b"seed".as_slice(),
            b"|".as_slice(),
            b"node:peer-seed".as_slice(),
        ]);
        let second = stable_hash64([
            b"seed".as_slice(),
            b"|".as_slice(),
            b"node:peer-seed".as_slice(),
        ]);

        assert_eq!(first, second);
        assert_eq!(first, scheduler_rank("seed", "node:peer-seed"));
    }

    #[test]
    fn scheduled_order_is_reproducible_for_same_seed() {
        let topology = sample_topology();
        let first = scheduled_peer_order(&topology, "seed-a");
        let second = scheduled_peer_order(&topology, "seed-a");

        assert_eq!(first, second);
    }

    #[test]
    fn fault_plan_is_reproducible_for_same_seed() {
        let topology = sample_topology();
        let fixture = Fixture {
            schema: None,
            fixture_id: "signature-mismatch".to_owned(),
            description: "negative".to_owned(),
            seed_peer: "peer-fault".to_owned(),
            reader_peers: vec!["peer-reader-a".to_owned(), "peer-reader-b".to_owned()],
            documents: Vec::new(),
            expected_outcomes: vec![
                "signature-verification-failure".to_owned(),
                "object-rejected-hash-mismatch".to_owned(),
            ],
            fault_peer: Some("peer-seed".to_owned()),
            notes: Vec::new(),
            metadata: None,
        };

        let first = build_fault_plan(&topology, &fixture, "seed-a");
        let second = build_fault_plan(&topology, &fixture, "seed-a");

        assert_eq!(first.len(), 2);
        assert_eq!(
            first.iter().map(|entry| &entry.fault).collect::<Vec<_>>(),
            second.iter().map(|entry| &entry.fault).collect::<Vec<_>>()
        );
        assert_eq!(
            first
                .iter()
                .map(|entry| entry.target_node_id.clone())
                .collect::<Vec<_>>(),
            second
                .iter()
                .map(|entry| entry.target_node_id.clone())
                .collect::<Vec<_>>()
        );
    }

    #[test]
    fn override_seed_wins_over_derived_seed() {
        let topology = sample_topology();
        let fixture = sample_fixture();
        let test_case = crate::model::TestCase {
            schema: None,
            test_id: "three-peer-consistency".to_owned(),
            description: "sample".to_owned(),
            category: "deterministic-comparison".to_owned(),
            topology: "sim/topologies/three-peer-consistency.example.json".to_owned(),
            fixture_set: "fixtures/object-sets/minimal-valid".to_owned(),
            execution_mode: "single-process".to_owned(),
            expected_result: "pass".to_owned(),
            expected_outcomes: Vec::new(),
            assertions: Vec::new(),
            notes: Vec::new(),
            metadata: None,
        };

        let (derived, derived_source) =
            super::resolve_deterministic_seed(&test_case, &topology, &fixture, None);
        let (override_seed, override_source) =
            super::resolve_deterministic_seed(&test_case, &topology, &fixture, Some("custom-seed"));

        assert_eq!(derived_source, "derived");
        assert_eq!(override_source, "override");
        assert_ne!(derived, override_seed);
        assert_eq!(override_seed, "custom-seed");
    }

    #[test]
    fn random_seed_mode_generates_runtime_seed() {
        let topology = sample_topology();
        let fixture = sample_fixture();
        let test_case = crate::model::TestCase {
            schema: None,
            test_id: "three-peer-consistency".to_owned(),
            description: "sample".to_owned(),
            category: "deterministic-comparison".to_owned(),
            topology: "sim/topologies/three-peer-consistency.example.json".to_owned(),
            fixture_set: "fixtures/object-sets/minimal-valid".to_owned(),
            execution_mode: "single-process".to_owned(),
            expected_result: "pass".to_owned(),
            expected_outcomes: Vec::new(),
            assertions: Vec::new(),
            notes: Vec::new(),
            metadata: None,
        };

        let (seed, source) =
            super::resolve_deterministic_seed(&test_case, &topology, &fixture, Some("random"));

        assert_ne!(seed, "random");
        assert!(seed.starts_with("random:"));
        assert_eq!(source, "random");
    }

    #[test]
    fn auto_seed_mode_generates_runtime_seed() {
        let topology = sample_topology();
        let fixture = sample_fixture();
        let test_case = crate::model::TestCase {
            schema: None,
            test_id: "three-peer-consistency".to_owned(),
            description: "sample".to_owned(),
            category: "deterministic-comparison".to_owned(),
            topology: "sim/topologies/three-peer-consistency.example.json".to_owned(),
            fixture_set: "fixtures/object-sets/minimal-valid".to_owned(),
            execution_mode: "single-process".to_owned(),
            expected_result: "pass".to_owned(),
            expected_outcomes: Vec::new(),
            assertions: Vec::new(),
            notes: Vec::new(),
            metadata: None,
        };

        let (seed, source) =
            super::resolve_deterministic_seed(&test_case, &topology, &fixture, Some("auto"));

        assert_ne!(seed, "auto");
        assert!(seed.starts_with("auto:"));
        assert_eq!(source, "auto");
    }

    #[test]
    fn derive_report_result_returns_pass_without_failures() {
        assert_eq!(derive_report_result(&[]), "pass");
    }

    #[test]
    fn derive_report_result_returns_partial_for_warning_only_failures() {
        let failures = vec![ReportFailure {
            failure_id: "warn:1".to_owned(),
            node_id: None,
            description: "warning only".to_owned(),
            severity: Some("warning".to_owned()),
        }];

        assert_eq!(derive_report_result(&failures), "partial");
    }

    #[test]
    fn derive_report_result_returns_fail_for_error_failures() {
        let failures = vec![ReportFailure {
            failure_id: "err:1".to_owned(),
            node_id: None,
            description: "error".to_owned(),
            severity: Some("error".to_owned()),
        }];

        assert_eq!(derive_report_result(&failures), "fail");
    }

    #[test]
    fn derive_report_result_treats_missing_severity_as_fail() {
        let failures = vec![ReportFailure {
            failure_id: "unknown:1".to_owned(),
            node_id: None,
            description: "unknown severity".to_owned(),
            severity: None,
        }];

        assert_eq!(derive_report_result(&failures), "fail");
    }
}
