//! Data model for the language-neutral simulator scaffold.

use serde::{Deserialize, Serialize};
use serde_json::Value;

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct FixtureDocumentRef {
    pub doc_id: String,
    #[serde(default)]
    pub head_ids: Vec<String>,
    #[serde(default)]
    pub notes: Option<String>,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct Fixture {
    #[serde(rename = "$schema", default)]
    pub schema: Option<String>,
    pub fixture_id: String,
    pub description: String,
    pub seed_peer: String,
    pub reader_peers: Vec<String>,
    #[serde(default)]
    pub documents: Vec<FixtureDocumentRef>,
    pub expected_outcomes: Vec<String>,
    #[serde(default)]
    pub fault_peer: Option<String>,
    #[serde(default)]
    pub notes: Vec<String>,
    #[serde(default)]
    pub metadata: Option<Value>,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct Peer {
    #[serde(rename = "$schema", default)]
    pub schema: Option<String>,
    pub node_id: String,
    pub role: String,
    pub bootstrap_peers: Vec<String>,
    #[serde(default)]
    pub endpoint: Option<String>,
    #[serde(default)]
    pub capabilities: Vec<String>,
    #[serde(default)]
    pub store_ref: Option<String>,
    #[serde(default)]
    pub fixture_policy: Option<Value>,
    #[serde(default)]
    pub notes: Vec<String>,
    #[serde(default)]
    pub metadata: Option<Value>,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct Topology {
    #[serde(rename = "$schema", default)]
    pub schema: Option<String>,
    pub topology_id: String,
    pub description: String,
    pub fixture_set: String,
    #[serde(default)]
    pub execution_mode: Option<String>,
    pub peers: Vec<Peer>,
    pub expected_outcomes: Vec<String>,
    #[serde(default)]
    pub notes: Vec<String>,
    #[serde(default)]
    pub metadata: Option<Value>,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct TestAssertion {
    pub assertion_id: String,
    pub description: String,
    #[serde(default)]
    pub severity: Option<String>,
    #[serde(default)]
    pub expected: Option<String>,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct TestCase {
    #[serde(rename = "$schema", default)]
    pub schema: Option<String>,
    pub test_id: String,
    pub description: String,
    pub category: String,
    pub topology: String,
    pub fixture_set: String,
    pub execution_mode: String,
    pub expected_result: String,
    #[serde(default)]
    pub expected_outcomes: Vec<String>,
    #[serde(default)]
    pub assertions: Vec<TestAssertion>,
    #[serde(default)]
    pub notes: Vec<String>,
    #[serde(default)]
    pub metadata: Option<Value>,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct ReportPeer {
    pub node_id: String,
    pub status: String,
    pub verified_object_ids: Vec<String>,
    #[serde(default)]
    pub rejected_object_ids: Vec<String>,
    #[serde(default)]
    pub notes: Vec<String>,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct ReportFailure {
    pub failure_id: String,
    #[serde(default)]
    pub node_id: Option<String>,
    pub description: String,
    #[serde(default)]
    pub severity: Option<String>,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct ReportEvent {
    pub step: u64,
    pub phase: String,
    pub action: String,
    pub outcome: String,
    #[serde(default)]
    pub node_id: Option<String>,
    #[serde(default)]
    pub object_ids: Vec<String>,
    #[serde(default)]
    pub detail: Option<String>,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct ReportSummary {
    #[serde(default)]
    pub verified_object_count: Option<u64>,
    #[serde(default)]
    pub rejected_object_count: Option<u64>,
    #[serde(default)]
    pub matched_expected_outcomes: Vec<String>,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct Report {
    #[serde(rename = "$schema", default)]
    pub schema: Option<String>,
    pub run_id: String,
    pub topology_id: String,
    pub fixture_id: String,
    #[serde(default)]
    pub test_id: Option<String>,
    #[serde(default)]
    pub execution_mode: Option<String>,
    #[serde(default)]
    pub started_at: Option<String>,
    #[serde(default)]
    pub finished_at: Option<String>,
    pub peers: Vec<ReportPeer>,
    pub result: String,
    #[serde(default)]
    pub events: Vec<ReportEvent>,
    #[serde(default)]
    pub failures: Vec<ReportFailure>,
    #[serde(default)]
    pub summary: Option<ReportSummary>,
    #[serde(default)]
    pub metadata: Option<Value>,
}
