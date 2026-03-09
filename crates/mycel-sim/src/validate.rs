//! Validation logic for fixture, peer, topology, test-case, and report inputs.

use std::collections::{HashMap, HashSet};
use std::fs;
use std::path::{Path, PathBuf};

use crate::model::{Fixture, Peer, Report, TestCase, Topology};

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ValidationError {
    pub path: String,
    pub message: String,
}

#[derive(Debug, Clone, Default)]
pub struct ValidationSummary {
    pub fixture_count: usize,
    pub peer_count: usize,
    pub topology_count: usize,
    pub test_case_count: usize,
    pub report_count: usize,
    pub errors: Vec<ValidationError>,
}

impl ValidationSummary {
    pub fn is_ok(&self) -> bool {
        self.errors.is_empty()
    }
}

#[derive(Debug, Clone)]
struct NamedFixture {
    path: PathBuf,
    value: Fixture,
}

#[derive(Debug, Clone)]
struct NamedPeer {
    path: PathBuf,
    value: Peer,
}

#[derive(Debug, Clone)]
struct NamedTopology {
    path: PathBuf,
    value: Topology,
}

#[derive(Debug, Clone)]
struct NamedTestCase {
    path: PathBuf,
    value: TestCase,
}

#[derive(Debug, Clone)]
struct NamedReport {
    path: PathBuf,
    value: Report,
}

pub fn validate_repo(root: &Path) -> ValidationSummary {
    let mut summary = ValidationSummary::default();

    let fixtures = load_fixtures(root, &mut summary);
    let peers = load_peers(root, &mut summary);
    let topologies = load_topologies(root, &mut summary);
    let test_cases = load_test_cases(root, &mut summary);
    let reports = load_reports(root, &mut summary);

    summary.fixture_count = fixtures.len();
    summary.peer_count = peers.len();
    summary.topology_count = topologies.len();
    summary.test_case_count = test_cases.len();
    summary.report_count = reports.len();

    validate_peers(&peers, &mut summary);
    validate_topologies(&topologies, &mut summary);
    validate_test_cases(root, &fixtures, &topologies, &test_cases, &mut summary);
    validate_reports(&fixtures, &topologies, &test_cases, &reports, &mut summary);

    summary
}

fn load_json<T: serde::de::DeserializeOwned>(
    path: &Path,
    summary: &mut ValidationSummary,
) -> Option<T> {
    match fs::read_to_string(path) {
        Ok(content) => match serde_json::from_str::<T>(&content) {
            Ok(value) => Some(value),
            Err(err) => {
                push_error(summary, path, format!("invalid JSON content: {err}"));
                None
            }
        },
        Err(err) => {
            push_error(summary, path, format!("failed to read file: {err}"));
            None
        }
    }
}

fn load_fixtures(root: &Path, summary: &mut ValidationSummary) -> Vec<NamedFixture> {
    let base = root.join("fixtures/object-sets");
    let mut items = Vec::new();

    for dir in read_dir_paths(&base, summary) {
        if dir.is_dir() {
            let file = dir.join("fixture.json");
            if file.exists() {
                if let Some(value) = load_json::<Fixture>(&file, summary) {
                    items.push(NamedFixture { path: file, value });
                }
            }
        }
    }

    items
}

fn load_peers(root: &Path, summary: &mut ValidationSummary) -> Vec<NamedPeer> {
    let base = root.join("sim/peers");
    load_json_files::<Peer>(&base, summary)
        .into_iter()
        .map(|(path, value)| NamedPeer { path, value })
        .collect()
}

fn load_topologies(root: &Path, summary: &mut ValidationSummary) -> Vec<NamedTopology> {
    let base = root.join("sim/topologies");
    load_json_files::<Topology>(&base, summary)
        .into_iter()
        .map(|(path, value)| NamedTopology { path, value })
        .collect()
}

fn load_test_cases(root: &Path, summary: &mut ValidationSummary) -> Vec<NamedTestCase> {
    let base = root.join("sim/tests");
    load_json_files::<TestCase>(&base, summary)
        .into_iter()
        .map(|(path, value)| NamedTestCase { path, value })
        .collect()
}

fn load_reports(root: &Path, summary: &mut ValidationSummary) -> Vec<NamedReport> {
    let base = root.join("sim/reports");
    load_json_files::<Report>(&base, summary)
        .into_iter()
        .map(|(path, value)| NamedReport { path, value })
        .collect()
}

fn load_json_files<T: serde::de::DeserializeOwned>(
    base: &Path,
    summary: &mut ValidationSummary,
) -> Vec<(PathBuf, T)> {
    let mut items = Vec::new();

    for path in read_dir_paths(base, summary) {
        if !path.is_file() {
            continue;
        }
        let Some(name) = path.file_name().and_then(|name| name.to_str()) else {
            continue;
        };
        if !name.ends_with(".json") || name.ends_with(".schema.json") {
            continue;
        }
        if let Some(value) = load_json::<T>(&path, summary) {
            items.push((path, value));
        }
    }

    items
}

fn read_dir_paths(base: &Path, summary: &mut ValidationSummary) -> Vec<PathBuf> {
    match fs::read_dir(base) {
        Ok(entries) => entries
            .filter_map(|entry| entry.ok().map(|e| e.path()))
            .collect(),
        Err(err) => {
            push_error(summary, base, format!("failed to read directory: {err}"));
            Vec::new()
        }
    }
}

fn validate_peers(peers: &[NamedPeer], summary: &mut ValidationSummary) {
    let mut seen = HashSet::new();

    for peer in peers {
        let node_id = &peer.value.node_id;
        if !seen.insert(node_id.clone()) {
            push_error(
                summary,
                &peer.path,
                format!("duplicate standalone peer node_id: {node_id}"),
            );
        }
    }
}

fn validate_topologies(topologies: &[NamedTopology], summary: &mut ValidationSummary) {
    for topology in topologies {
        let mut node_ids = HashSet::new();
        for peer in &topology.value.peers {
            if !node_ids.insert(peer.node_id.clone()) {
                push_error(
                    summary,
                    &topology.path,
                    format!(
                        "duplicate topology peer node_id in {}: {}",
                        topology.value.topology_id, peer.node_id
                    ),
                );
            }
        }

        for peer in &topology.value.peers {
            for bootstrap in &peer.bootstrap_peers {
                if !node_ids.contains(bootstrap) {
                    push_error(
                        summary,
                        &topology.path,
                        format!(
                            "unresolved bootstrap peer '{}' in topology {}",
                            bootstrap, topology.value.topology_id
                        ),
                    );
                }
            }
        }
    }
}

fn validate_test_cases(
    root: &Path,
    fixtures: &[NamedFixture],
    topologies: &[NamedTopology],
    test_cases: &[NamedTestCase],
    summary: &mut ValidationSummary,
) {
    let fixture_by_id: HashMap<_, _> = fixtures
        .iter()
        .map(|fixture| (fixture.value.fixture_id.clone(), fixture))
        .collect();
    let topology_by_id: HashMap<_, _> = topologies
        .iter()
        .map(|topology| (topology.value.topology_id.clone(), topology))
        .collect();
    let topology_by_rel_path: HashMap<_, _> = topologies
        .iter()
        .filter_map(|topology| relative_display(root, &topology.path).map(|path| (path, topology)))
        .collect();

    for test_case in test_cases {
        let topology_file = root.join(&test_case.value.topology);
        if !topology_file.exists() {
            push_error(
                summary,
                &test_case.path,
                format!(
                    "missing referenced topology file: {}",
                    test_case.value.topology
                ),
            );
            continue;
        }

        let Some(topology) = topology_by_rel_path
            .get(test_case.value.topology.as_str())
            .or_else(|| topology_by_id.get(test_case.value.topology.as_str()))
        else {
            push_error(
                summary,
                &test_case.path,
                format!(
                    "referenced topology was not loaded successfully: {}",
                    test_case.value.topology
                ),
            );
            continue;
        };

        let fixture_dir = root.join(&test_case.value.fixture_set);
        let fixture_file = fixture_dir.join("fixture.json");
        if !fixture_file.exists() {
            push_error(
                summary,
                &test_case.path,
                format!(
                    "missing referenced fixture file: {}",
                    fixture_file.display()
                ),
            );
            continue;
        }

        let fixture_key = fixture_dir
            .file_name()
            .and_then(|name| name.to_str())
            .unwrap_or_default();

        let Some(fixture) = fixture_by_id.get(fixture_key) else {
            push_error(
                summary,
                &test_case.path,
                format!(
                    "referenced fixture was not loaded successfully: {}",
                    test_case.value.fixture_set
                ),
            );
            continue;
        };

        if let Some(mode) = &topology.value.execution_mode {
            if mode != &test_case.value.execution_mode {
                push_error(
                    summary,
                    &test_case.path,
                    format!(
                        "test-case execution_mode '{}' does not match topology execution_mode '{}'",
                        test_case.value.execution_mode, mode
                    ),
                );
            }
        }

        let allowed_outcomes: HashSet<_> = fixture
            .value
            .expected_outcomes
            .iter()
            .chain(topology.value.expected_outcomes.iter())
            .collect();

        for outcome in &test_case.value.expected_outcomes {
            if !allowed_outcomes.contains(&outcome) {
                push_error(
                    summary,
                    &test_case.path,
                    format!(
                        "test-case expected outcome '{}' is not declared by fixture or topology",
                        outcome
                    ),
                );
            }
        }

        validate_fixture_topology_mapping(fixture, topology, summary);
    }
}

fn validate_fixture_topology_mapping(
    fixture: &NamedFixture,
    topology: &NamedTopology,
    summary: &mut ValidationSummary,
) {
    let mut aliases = HashSet::new();
    for peer in &topology.value.peers {
        aliases.insert(peer.node_id.clone());
        aliases.insert(normalize_node_id(&peer.node_id));
        aliases.insert(peer.role.clone());
    }

    let required_refs = std::iter::once(&fixture.value.seed_peer)
        .chain(fixture.value.reader_peers.iter())
        .chain(fixture.value.fault_peer.iter());

    for peer_ref in required_refs {
        if !aliases.contains(peer_ref) {
            push_error(
                summary,
                &fixture.path,
                format!(
                    "fixture peer reference '{}' does not map to topology '{}'",
                    peer_ref, topology.value.topology_id
                ),
            );
        }
    }
}

fn validate_reports(
    fixtures: &[NamedFixture],
    topologies: &[NamedTopology],
    test_cases: &[NamedTestCase],
    reports: &[NamedReport],
    summary: &mut ValidationSummary,
) {
    let fixture_by_id: HashMap<_, _> = fixtures
        .iter()
        .map(|fixture| (fixture.value.fixture_id.clone(), fixture))
        .collect();
    let topology_by_id: HashMap<_, _> = topologies
        .iter()
        .map(|topology| (topology.value.topology_id.clone(), topology))
        .collect();
    let test_by_id: HashMap<_, _> = test_cases
        .iter()
        .map(|test| (test.value.test_id.clone(), test))
        .collect();

    for report in reports {
        if !fixture_by_id.contains_key(report.value.fixture_id.as_str()) {
            push_error(
                summary,
                &report.path,
                format!(
                    "report fixture_id '{}' does not match any loaded fixture",
                    report.value.fixture_id
                ),
            );
        }

        if !topology_by_id.contains_key(report.value.topology_id.as_str()) {
            push_error(
                summary,
                &report.path,
                format!(
                    "report topology_id '{}' does not match any loaded topology",
                    report.value.topology_id
                ),
            );
        }

        if let Some(test_id) = &report.value.test_id {
            if let Some(test_case) = test_by_id.get(test_id) {
                if let Some(mode) = &report.value.execution_mode {
                    if &test_case.value.execution_mode != mode {
                        push_error(
                            summary,
                            &report.path,
                            format!(
                                "report execution_mode '{}' does not match test-case execution_mode '{}'",
                                mode, test_case.value.execution_mode
                            ),
                        );
                    }
                }

                if report.value.fixture_id != fixture_dir_name(&test_case.value.fixture_set) {
                    push_error(
                        summary,
                        &report.path,
                        format!(
                            "report fixture_id '{}' does not match test-case fixture_set '{}'",
                            report.value.fixture_id, test_case.value.fixture_set
                        ),
                    );
                }

                if let Some(summary_block) = &report.value.summary {
                    let expected: HashSet<_> = test_case
                        .value
                        .expected_outcomes
                        .iter()
                        .map(String::as_str)
                        .collect();
                    for matched in &summary_block.matched_expected_outcomes {
                        if !expected.contains(matched.as_str()) {
                            push_error(
                                summary,
                                &report.path,
                                format!(
                                    "report matched outcome '{}' is not declared in test-case '{}'",
                                    matched, test_id
                                ),
                            );
                        }
                    }
                }
            } else {
                push_error(
                    summary,
                    &report.path,
                    format!(
                        "report test_id '{}' does not match any loaded test case",
                        test_id
                    ),
                );
            }
        }
    }
}

fn fixture_dir_name(path: &str) -> String {
    Path::new(path)
        .file_name()
        .and_then(|name| name.to_str())
        .unwrap_or_default()
        .to_owned()
}

fn normalize_node_id(node_id: &str) -> String {
    node_id.strip_prefix("node:").unwrap_or(node_id).to_owned()
}

fn relative_display(root: &Path, path: &Path) -> Option<String> {
    path.strip_prefix(root)
        .ok()
        .and_then(|relative| relative.to_str().map(|s| s.replace('\\', "/")))
}

fn push_error(summary: &mut ValidationSummary, path: &Path, message: String) {
    summary.errors.push(ValidationError {
        path: path.display().to_string(),
        message,
    });
}
