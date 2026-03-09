//! Validation logic for fixture, peer, topology, test-case, and report inputs.

use std::collections::{HashMap, HashSet};
use std::fmt;
use std::fs;
use std::path::{Path, PathBuf};

use serde::Serialize;

use crate::model::{Fixture, Peer, Report, TestCase, Topology};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
#[serde(rename_all = "lowercase")]
pub enum ValidationStatus {
    Ok,
    Warning,
    Failed,
}

impl Default for ValidationStatus {
    fn default() -> Self {
        Self::Ok
    }
}

impl fmt::Display for ValidationStatus {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let value = match self {
            Self::Ok => "ok",
            Self::Warning => "warning",
            Self::Failed => "failed",
        };

        f.write_str(value)
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct ValidationMessage {
    pub path: String,
    pub message: String,
}

#[derive(Debug, Clone, Default, Serialize)]
pub struct ValidationSummary {
    pub root: Option<PathBuf>,
    pub target: Option<PathBuf>,
    pub status: ValidationStatus,
    pub fixture_count: usize,
    pub peer_count: usize,
    pub topology_count: usize,
    pub test_case_count: usize,
    pub report_count: usize,
    pub errors: Vec<ValidationMessage>,
    pub warnings: Vec<ValidationMessage>,
}

impl ValidationSummary {
    pub fn is_ok(&self) -> bool {
        self.errors.is_empty()
    }

    pub fn has_warnings(&self) -> bool {
        !self.warnings.is_empty()
    }

    pub fn refresh_status(&mut self) {
        self.status = if !self.errors.is_empty() {
            ValidationStatus::Failed
        } else if !self.warnings.is_empty() {
            ValidationStatus::Warning
        } else {
            ValidationStatus::Ok
        };
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

#[derive(Debug, Clone)]
enum ValidationTarget {
    Repo,
    Fixture(PathBuf),
    Peer(PathBuf),
    Topology(PathBuf),
    TestCase(PathBuf),
    Report(PathBuf),
    FixturesDir(PathBuf),
    PeersDir(PathBuf),
    TopologiesDir(PathBuf),
    TestsDir(PathBuf),
    ReportsDir(PathBuf),
}

#[derive(Debug, Clone)]
struct ValidationInput {
    fixtures: Vec<NamedFixture>,
    peers: Vec<NamedPeer>,
    topologies: Vec<NamedTopology>,
    test_cases: Vec<NamedTestCase>,
    reports: Vec<NamedReport>,
}

pub fn validate_repo(root: &Path) -> ValidationSummary {
    let normalized_root = normalize_input_path(root);
    let target = ValidationTarget::Repo;
    validate_from_target(&normalized_root, &normalized_root, &target)
}

pub fn validate_path(target_path: &Path) -> ValidationSummary {
    let normalized_target = normalize_input_path(target_path);
    let mut summary = ValidationSummary::default();
    let Some(root) = find_repo_root(&normalized_target) else {
        push_error(
            &mut summary,
            &normalized_target,
            "could not find repository root containing Cargo.toml, fixtures/, and sim/".to_owned(),
        );
        summary.target = Some(normalized_target);
        summary.refresh_status();
        return summary;
    };

    let target = match detect_target(&root, &normalized_target) {
        Ok(target) => target,
        Err(message) => {
            push_error(&mut summary, &normalized_target, message);
            summary.root = Some(root);
            summary.target = Some(normalized_target);
            summary.refresh_status();
            return summary;
        }
    };

    validate_from_target(&root, &normalized_target, &target)
}

fn validate_from_target(
    root: &Path,
    target_path: &Path,
    target: &ValidationTarget,
) -> ValidationSummary {
    let mut summary = ValidationSummary {
        root: Some(root.to_path_buf()),
        target: Some(target_path.to_path_buf()),
        ..ValidationSummary::default()
    };

    let input = load_all(root, &mut summary);
    let scoped = scope_input(root, &input, target);

    summary.fixture_count = scoped.fixtures.len();
    summary.peer_count = scoped.peers.len();
    summary.topology_count = scoped.topologies.len();
    summary.test_case_count = scoped.test_cases.len();
    summary.report_count = scoped.reports.len();

    validate_peers(&scoped.peers, &mut summary);
    validate_topologies(&scoped.topologies, &mut summary);
    validate_test_cases(
        root,
        &scoped.fixtures,
        &scoped.topologies,
        &scoped.test_cases,
        &mut summary,
    );
    validate_reports(
        &scoped.fixtures,
        &scoped.topologies,
        &scoped.test_cases,
        &scoped.reports,
        &mut summary,
    );

    validate_peer_topology_usage(&scoped.peers, &scoped.topologies, &mut summary);
    validate_quality_hints(
        &scoped.peers,
        &scoped.test_cases,
        &scoped.reports,
        &mut summary,
    );
    summary.refresh_status();

    summary
}

fn load_all(root: &Path, summary: &mut ValidationSummary) -> ValidationInput {
    ValidationInput {
        fixtures: load_fixtures(root, summary),
        peers: load_peers(root, summary),
        topologies: load_topologies(root, summary),
        test_cases: load_test_cases(root, summary),
        reports: load_reports(root, summary),
    }
}

fn scope_input(root: &Path, input: &ValidationInput, target: &ValidationTarget) -> ValidationInput {
    match target {
        ValidationTarget::Repo => input.clone(),
        ValidationTarget::Fixture(path) => scope_for_fixture(root, input, path),
        ValidationTarget::Peer(path) => ValidationInput {
            fixtures: Vec::new(),
            peers: filter_by_path(&input.peers, path),
            topologies: Vec::new(),
            test_cases: Vec::new(),
            reports: Vec::new(),
        },
        ValidationTarget::Topology(path) => scope_for_topology(root, input, path),
        ValidationTarget::TestCase(path) => scope_for_test_case(root, input, path),
        ValidationTarget::Report(path) => scope_for_report(input, path),
        ValidationTarget::FixturesDir(path) => scope_for_fixtures_dir(root, input, path),
        ValidationTarget::PeersDir(path) => ValidationInput {
            fixtures: Vec::new(),
            peers: filter_by_dir(&input.peers, path),
            topologies: Vec::new(),
            test_cases: Vec::new(),
            reports: Vec::new(),
        },
        ValidationTarget::TopologiesDir(path) => scope_for_topologies_dir(root, input, path),
        ValidationTarget::TestsDir(path) => scope_for_tests_dir(root, input, path),
        ValidationTarget::ReportsDir(path) => scope_for_reports_dir(input, path),
    }
}

fn scope_for_fixture(root: &Path, input: &ValidationInput, path: &Path) -> ValidationInput {
    let fixtures = filter_by_path(&input.fixtures, path);
    let fixture_ids: HashSet<_> = fixtures
        .iter()
        .map(|fixture| fixture.value.fixture_id.as_str())
        .collect();

    let topologies: Vec<_> = input
        .topologies
        .iter()
        .filter(|topology| {
            fixture_ids.contains(fixture_dir_name(&topology.value.fixture_set).as_str())
        })
        .cloned()
        .collect();
    let topology_ids: HashSet<_> = topologies
        .iter()
        .map(|topology| topology.value.topology_id.as_str())
        .collect();
    let topology_paths: HashSet<_> = topologies
        .iter()
        .filter_map(|topology| relative_display(root, &topology.path))
        .collect();

    let test_cases: Vec<_> = input
        .test_cases
        .iter()
        .filter(|test_case| {
            fixture_ids.contains(fixture_dir_name(&test_case.value.fixture_set).as_str())
                || topology_ids.contains(test_case.value.topology.as_str())
                || topology_paths.contains(test_case.value.topology.as_str())
        })
        .cloned()
        .collect();
    let test_ids: HashSet<_> = test_cases
        .iter()
        .map(|test_case| test_case.value.test_id.as_str())
        .collect();

    let reports: Vec<_> = input
        .reports
        .iter()
        .filter(|report| {
            fixture_ids.contains(report.value.fixture_id.as_str())
                || topology_ids.contains(report.value.topology_id.as_str())
                || report
                    .value
                    .test_id
                    .as_deref()
                    .is_some_and(|test_id| test_ids.contains(test_id))
        })
        .cloned()
        .collect();

    ValidationInput {
        fixtures,
        peers: Vec::new(),
        topologies,
        test_cases,
        reports,
    }
}

fn scope_for_topology(root: &Path, input: &ValidationInput, path: &Path) -> ValidationInput {
    let topologies = filter_by_path(&input.topologies, path);
    let topology_ids: HashSet<_> = topologies
        .iter()
        .map(|topology| topology.value.topology_id.as_str())
        .collect();
    let topology_paths: HashSet<_> = topologies
        .iter()
        .filter_map(|topology| relative_display(root, &topology.path))
        .collect();
    let fixture_ids: HashSet<_> = topologies
        .iter()
        .map(|topology| fixture_dir_name(&topology.value.fixture_set))
        .collect();

    let fixtures: Vec<_> = input
        .fixtures
        .iter()
        .filter(|fixture| fixture_ids.contains(&fixture.value.fixture_id))
        .cloned()
        .collect();
    let test_cases: Vec<_> = input
        .test_cases
        .iter()
        .filter(|test_case| {
            topology_ids.contains(test_case.value.topology.as_str())
                || topology_paths.contains(test_case.value.topology.as_str())
        })
        .cloned()
        .collect();
    let test_ids: HashSet<_> = test_cases
        .iter()
        .map(|test_case| test_case.value.test_id.as_str())
        .collect();
    let reports: Vec<_> = input
        .reports
        .iter()
        .filter(|report| {
            topology_ids.contains(report.value.topology_id.as_str())
                || report
                    .value
                    .test_id
                    .as_deref()
                    .is_some_and(|test_id| test_ids.contains(test_id))
        })
        .cloned()
        .collect();

    ValidationInput {
        fixtures,
        peers: Vec::new(),
        topologies,
        test_cases,
        reports,
    }
}

fn scope_for_test_case(root: &Path, input: &ValidationInput, path: &Path) -> ValidationInput {
    let test_cases = filter_by_path(&input.test_cases, path);
    let test_ids: HashSet<_> = test_cases
        .iter()
        .map(|test_case| test_case.value.test_id.as_str())
        .collect();
    let topology_refs: HashSet<_> = test_cases
        .iter()
        .map(|test_case| test_case.value.topology.as_str())
        .collect();
    let fixture_ids: HashSet<_> = test_cases
        .iter()
        .map(|test_case| fixture_dir_name(&test_case.value.fixture_set))
        .collect();

    let topologies: Vec<_> = input
        .topologies
        .iter()
        .filter(|topology| {
            topology_refs.contains(topology.value.topology_id.as_str())
                || relative_display(root, &topology.path)
                    .is_some_and(|path| topology_refs.contains(path.as_str()))
        })
        .cloned()
        .collect();
    let topology_ids: HashSet<_> = topologies
        .iter()
        .map(|topology| topology.value.topology_id.as_str())
        .collect();
    let fixtures: Vec<_> = input
        .fixtures
        .iter()
        .filter(|fixture| fixture_ids.contains(&fixture.value.fixture_id))
        .cloned()
        .collect();
    let reports: Vec<_> = input
        .reports
        .iter()
        .filter(|report| {
            report
                .value
                .test_id
                .as_deref()
                .is_some_and(|test_id| test_ids.contains(test_id))
                || topology_ids.contains(report.value.topology_id.as_str())
        })
        .cloned()
        .collect();

    ValidationInput {
        fixtures,
        peers: Vec::new(),
        topologies,
        test_cases,
        reports,
    }
}

fn scope_for_report(input: &ValidationInput, path: &Path) -> ValidationInput {
    let reports = filter_by_path(&input.reports, path);
    let fixture_ids: HashSet<_> = reports
        .iter()
        .map(|report| report.value.fixture_id.as_str())
        .collect();
    let topology_ids: HashSet<_> = reports
        .iter()
        .map(|report| report.value.topology_id.as_str())
        .collect();
    let test_ids: HashSet<_> = reports
        .iter()
        .filter_map(|report| report.value.test_id.as_deref())
        .collect();

    let fixtures: Vec<_> = input
        .fixtures
        .iter()
        .filter(|fixture| fixture_ids.contains(fixture.value.fixture_id.as_str()))
        .cloned()
        .collect();
    let topologies: Vec<_> = input
        .topologies
        .iter()
        .filter(|topology| topology_ids.contains(topology.value.topology_id.as_str()))
        .cloned()
        .collect();
    let test_cases: Vec<_> = input
        .test_cases
        .iter()
        .filter(|test_case| test_ids.contains(test_case.value.test_id.as_str()))
        .cloned()
        .collect();

    ValidationInput {
        fixtures,
        peers: Vec::new(),
        topologies,
        test_cases,
        reports,
    }
}

fn scope_for_fixtures_dir(root: &Path, input: &ValidationInput, path: &Path) -> ValidationInput {
    if is_fixture_scenario_dir(path) {
        return scope_for_fixture(root, input, &path.join("fixture.json"));
    }

    let fixtures = filter_by_dir(&input.fixtures, path);
    if fixtures.is_empty() {
        return ValidationInput {
            fixtures,
            peers: Vec::new(),
            topologies: Vec::new(),
            test_cases: Vec::new(),
            reports: Vec::new(),
        };
    }

    let fixture_ids: HashSet<_> = fixtures
        .iter()
        .map(|fixture| fixture.value.fixture_id.as_str())
        .collect();
    let topologies: Vec<_> = input
        .topologies
        .iter()
        .filter(|topology| {
            fixture_ids.contains(fixture_dir_name(&topology.value.fixture_set).as_str())
        })
        .cloned()
        .collect();
    let topology_ids: HashSet<_> = topologies
        .iter()
        .map(|topology| topology.value.topology_id.as_str())
        .collect();
    let topology_paths: HashSet<_> = topologies
        .iter()
        .filter_map(|topology| relative_display(root, &topology.path))
        .collect();
    let test_cases: Vec<_> = input
        .test_cases
        .iter()
        .filter(|test_case| {
            fixture_ids.contains(fixture_dir_name(&test_case.value.fixture_set).as_str())
                || topology_ids.contains(test_case.value.topology.as_str())
                || topology_paths.contains(test_case.value.topology.as_str())
        })
        .cloned()
        .collect();
    let test_ids: HashSet<_> = test_cases
        .iter()
        .map(|test_case| test_case.value.test_id.as_str())
        .collect();
    let reports: Vec<_> = input
        .reports
        .iter()
        .filter(|report| {
            fixture_ids.contains(report.value.fixture_id.as_str())
                || topology_ids.contains(report.value.topology_id.as_str())
                || report
                    .value
                    .test_id
                    .as_deref()
                    .is_some_and(|test_id| test_ids.contains(test_id))
        })
        .cloned()
        .collect();

    ValidationInput {
        fixtures,
        peers: Vec::new(),
        topologies,
        test_cases,
        reports,
    }
}

fn scope_for_topologies_dir(root: &Path, input: &ValidationInput, path: &Path) -> ValidationInput {
    let topologies = filter_by_dir(&input.topologies, path);
    let topology_ids: HashSet<_> = topologies
        .iter()
        .map(|topology| topology.value.topology_id.as_str())
        .collect();
    let topology_paths: HashSet<_> = topologies
        .iter()
        .filter_map(|topology| relative_display(root, &topology.path))
        .collect();
    let fixture_ids: HashSet<_> = topologies
        .iter()
        .map(|topology| fixture_dir_name(&topology.value.fixture_set))
        .collect();
    let fixtures: Vec<_> = input
        .fixtures
        .iter()
        .filter(|fixture| fixture_ids.contains(&fixture.value.fixture_id))
        .cloned()
        .collect();
    let test_cases: Vec<_> = input
        .test_cases
        .iter()
        .filter(|test_case| {
            topology_ids.contains(test_case.value.topology.as_str())
                || topology_paths.contains(test_case.value.topology.as_str())
        })
        .cloned()
        .collect();
    let test_ids: HashSet<_> = test_cases
        .iter()
        .map(|test_case| test_case.value.test_id.as_str())
        .collect();
    let reports: Vec<_> = input
        .reports
        .iter()
        .filter(|report| {
            topology_ids.contains(report.value.topology_id.as_str())
                || report
                    .value
                    .test_id
                    .as_deref()
                    .is_some_and(|test_id| test_ids.contains(test_id))
        })
        .cloned()
        .collect();

    ValidationInput {
        fixtures,
        peers: Vec::new(),
        topologies,
        test_cases,
        reports,
    }
}

fn scope_for_tests_dir(root: &Path, input: &ValidationInput, path: &Path) -> ValidationInput {
    let test_cases = filter_by_dir(&input.test_cases, path);
    let test_ids: HashSet<_> = test_cases
        .iter()
        .map(|test_case| test_case.value.test_id.as_str())
        .collect();
    let topology_refs: HashSet<_> = test_cases
        .iter()
        .map(|test_case| test_case.value.topology.as_str())
        .collect();
    let fixture_ids: HashSet<_> = test_cases
        .iter()
        .map(|test_case| fixture_dir_name(&test_case.value.fixture_set))
        .collect();
    let topologies: Vec<_> = input
        .topologies
        .iter()
        .filter(|topology| {
            topology_refs.contains(topology.value.topology_id.as_str())
                || relative_display(root, &topology.path)
                    .is_some_and(|path| topology_refs.contains(path.as_str()))
        })
        .cloned()
        .collect();
    let topology_ids: HashSet<_> = topologies
        .iter()
        .map(|topology| topology.value.topology_id.as_str())
        .collect();
    let fixtures: Vec<_> = input
        .fixtures
        .iter()
        .filter(|fixture| fixture_ids.contains(&fixture.value.fixture_id))
        .cloned()
        .collect();
    let reports: Vec<_> = input
        .reports
        .iter()
        .filter(|report| {
            report
                .value
                .test_id
                .as_deref()
                .is_some_and(|test_id| test_ids.contains(test_id))
                || topology_ids.contains(report.value.topology_id.as_str())
        })
        .cloned()
        .collect();

    ValidationInput {
        fixtures,
        peers: Vec::new(),
        topologies,
        test_cases,
        reports,
    }
}

fn scope_for_reports_dir(input: &ValidationInput, path: &Path) -> ValidationInput {
    let reports = filter_by_dir(&input.reports, path);
    let fixture_ids: HashSet<_> = reports
        .iter()
        .map(|report| report.value.fixture_id.as_str())
        .collect();
    let topology_ids: HashSet<_> = reports
        .iter()
        .map(|report| report.value.topology_id.as_str())
        .collect();
    let test_ids: HashSet<_> = reports
        .iter()
        .filter_map(|report| report.value.test_id.as_deref())
        .collect();
    let fixtures: Vec<_> = input
        .fixtures
        .iter()
        .filter(|fixture| fixture_ids.contains(fixture.value.fixture_id.as_str()))
        .cloned()
        .collect();
    let topologies: Vec<_> = input
        .topologies
        .iter()
        .filter(|topology| topology_ids.contains(topology.value.topology_id.as_str()))
        .cloned()
        .collect();
    let test_cases: Vec<_> = input
        .test_cases
        .iter()
        .filter(|test_case| test_ids.contains(test_case.value.test_id.as_str()))
        .cloned()
        .collect();

    ValidationInput {
        fixtures,
        peers: Vec::new(),
        topologies,
        test_cases,
        reports,
    }
}

fn filter_by_path<T: Clone + HasPath>(items: &[T], path: &Path) -> Vec<T> {
    items
        .iter()
        .filter(|item| item.path() == path)
        .cloned()
        .collect()
}

fn filter_by_dir<T: Clone + HasPath>(items: &[T], path: &Path) -> Vec<T> {
    items
        .iter()
        .filter(|item| item.path().parent().is_some_and(|parent| parent == path))
        .cloned()
        .collect()
}

trait HasPath {
    fn path(&self) -> &Path;
}

impl HasPath for NamedFixture {
    fn path(&self) -> &Path {
        &self.path
    }
}

impl HasPath for NamedPeer {
    fn path(&self) -> &Path {
        &self.path
    }
}

impl HasPath for NamedTopology {
    fn path(&self) -> &Path {
        &self.path
    }
}

impl HasPath for NamedTestCase {
    fn path(&self) -> &Path {
        &self.path
    }
}

impl HasPath for NamedReport {
    fn path(&self) -> &Path {
        &self.path
    }
}

fn find_repo_root(start: &Path) -> Option<PathBuf> {
    let absolute = normalize_input_path(start);
    let mut current = if absolute.is_file() {
        absolute.parent()?.to_path_buf()
    } else {
        absolute
    };

    loop {
        if current.join("Cargo.toml").exists()
            && current.join("fixtures").is_dir()
            && current.join("sim").is_dir()
        {
            return Some(current);
        }

        if !current.pop() {
            return None;
        }
    }
}

fn normalize_input_path(path: &Path) -> PathBuf {
    if path.is_absolute() {
        path.to_path_buf()
    } else {
        std::env::current_dir()
            .unwrap_or_else(|_| PathBuf::from("."))
            .join(path)
    }
}

fn detect_target(root: &Path, input: &Path) -> Result<ValidationTarget, String> {
    let path = if input.is_absolute() {
        input.to_path_buf()
    } else {
        root.join(input)
    };

    if !path.exists() {
        return Err(format!("path does not exist: {}", path.display()));
    }

    let relative = path
        .strip_prefix(root)
        .map_err(|_| format!("path is outside repository root: {}", path.display()))?;

    if relative.as_os_str().is_empty() || relative == Path::new(".") {
        return Ok(ValidationTarget::Repo);
    }

    if path.is_dir() {
        if relative.starts_with("fixtures/object-sets") {
            return Ok(ValidationTarget::FixturesDir(path));
        }
        if relative == Path::new("sim/peers") {
            return Ok(ValidationTarget::PeersDir(path));
        }
        if relative == Path::new("sim/topologies") {
            return Ok(ValidationTarget::TopologiesDir(path));
        }
        if relative == Path::new("sim/tests") {
            return Ok(ValidationTarget::TestsDir(path));
        }
        if relative == Path::new("sim/reports") || relative == Path::new("sim/reports/out") {
            return Ok(ValidationTarget::ReportsDir(path));
        }

        return Ok(ValidationTarget::Repo);
    }

    let Some(name) = path.file_name().and_then(|value| value.to_str()) else {
        return Err(format!("unsupported target path: {}", path.display()));
    };

    if name.ends_with(".schema.json") {
        return Err(format!(
            "schema files are not validate targets: {}",
            path.display()
        ));
    }

    if relative.starts_with("fixtures/object-sets") && name == "fixture.json" {
        return Ok(ValidationTarget::Fixture(path));
    }
    if relative.starts_with("sim/peers") && name.ends_with(".json") {
        return Ok(ValidationTarget::Peer(path));
    }
    if relative.starts_with("sim/topologies") && name.ends_with(".json") {
        return Ok(ValidationTarget::Topology(path));
    }
    if relative.starts_with("sim/tests") && name.ends_with(".json") {
        return Ok(ValidationTarget::TestCase(path));
    }
    if relative.starts_with("sim/reports") && name.ends_with(".json") {
        return Ok(ValidationTarget::Report(path));
    }

    Err(format!("unsupported validate target: {}", path.display()))
}

fn is_fixture_scenario_dir(path: &Path) -> bool {
    path.join("fixture.json").exists()
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
    load_json_files_recursive::<Report>(&base, summary)
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

fn load_json_files_recursive<T: serde::de::DeserializeOwned>(
    base: &Path,
    summary: &mut ValidationSummary,
) -> Vec<(PathBuf, T)> {
    let mut items = Vec::new();

    for path in read_dir_paths_recursive(base, summary) {
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

fn read_dir_paths_recursive(base: &Path, summary: &mut ValidationSummary) -> Vec<PathBuf> {
    let mut paths = Vec::new();

    for path in read_dir_paths(base, summary) {
        paths.push(path.clone());
        if path.is_dir() {
            paths.extend(read_dir_paths_recursive(&path, summary));
        }
    }

    paths
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

fn validate_peer_topology_usage(
    peers: &[NamedPeer],
    topologies: &[NamedTopology],
    summary: &mut ValidationSummary,
) {
    let referenced_node_ids: HashSet<_> = topologies
        .iter()
        .flat_map(|topology| {
            topology
                .value
                .peers
                .iter()
                .map(|peer| peer.node_id.as_str())
        })
        .collect();

    for peer in peers {
        if !referenced_node_ids.contains(peer.value.node_id.as_str()) {
            push_warning(
                summary,
                &peer.path,
                format!(
                    "standalone peer '{}' is not referenced by any loaded topology",
                    peer.value.node_id
                ),
            );
        }
    }
}

fn validate_quality_hints(
    peers: &[NamedPeer],
    test_cases: &[NamedTestCase],
    reports: &[NamedReport],
    summary: &mut ValidationSummary,
) {
    for peer in peers {
        if peer.value.capabilities.is_empty() {
            push_warning(
                summary,
                &peer.path,
                format!(
                    "peer '{}' does not declare any capabilities",
                    peer.value.node_id
                ),
            );
        }
    }

    for test_case in test_cases {
        if test_case.value.assertions.is_empty() {
            push_warning(
                summary,
                &test_case.path,
                format!(
                    "test-case '{}' does not declare any assertions",
                    test_case.value.test_id
                ),
            );
        }
    }

    for report in reports {
        if report.value.test_id.is_none() {
            push_warning(
                summary,
                &report.path,
                format!("report '{}' does not declare test_id", report.value.run_id),
            );
        }

        if report.value.summary.is_none() {
            push_warning(
                summary,
                &report.path,
                format!(
                    "report '{}' does not include a summary block",
                    report.value.run_id
                ),
            );
        }

        if report.value.events.is_empty() {
            push_warning(
                summary,
                &report.path,
                format!(
                    "report '{}' does not include any event trace entries",
                    report.value.run_id
                ),
            );
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
    summary.errors.push(ValidationMessage {
        path: path.display().to_string(),
        message,
    });
}

fn push_warning(summary: &mut ValidationSummary, path: &Path, message: String) {
    summary.warnings.push(ValidationMessage {
        path: path.display().to_string(),
        message,
    });
}
