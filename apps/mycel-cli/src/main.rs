use std::env;
use std::path::PathBuf;

use mycel_core::head::inspect_heads_from_path;
use mycel_core::head::HeadInspectSummary;
use mycel_core::verify::{verify_object_path, ObjectVerificationSummary};
use mycel_core::workspace_banner;
use mycel_sim::manifest::SimulatorPaths;
use mycel_sim::run::{run_test_case_with_options, RunOptions};
use mycel_sim::simulator_banner;
use mycel_sim::validate::validate_path;

fn print_usage() {
    println!("mycel <command> [path]");
    println!();
    println!("Commands:");
    println!("  head       Inspect accepted-head selection from a local input bundle");
    println!("  info       Show workspace and simulator scaffold information");
    println!("  object     Verify one Mycel object file");
    println!("  sim        Run a simulator test case");
    println!("  validate   Validate the repo root, one file, or one supported directory");
    println!("  help       Show this message");
    println!();
    println!("Head options:");
    println!("  inspect <doc_id> --input <path|fixture>  Inspect one document's accepted head");
    println!(
        "  --json                                   Emit machine-readable head inspection output"
    );
    println!();
    println!("Object options:");
    println!("  verify <path>  Verify one object file");
    println!("  --json         Emit machine-readable object verification output");
    println!();
    println!("Sim options:");
    println!("  run <path> Run one test-case and write a report to sim/reports/out/");
    println!("  --json     Emit machine-readable run output");
    println!("  --seed     Use a fixed seed, or 'random' / 'auto' to generate one");
    println!();
    println!("Validate options:");
    println!("  --json     Emit machine-readable validation output");
    println!("  --strict   Treat warnings as failures");
}

fn print_info() {
    let paths = SimulatorPaths::default();

    println!("{}", workspace_banner());
    println!("{}", simulator_banner());
    println!("fixtures: {}", paths.fixtures_root);
    println!("peers: {}", paths.peers_root);
    println!("topologies: {}", paths.topologies_root);
    println!("tests: {}", paths.tests_root);
    println!("reports: {}", paths.reports_root);
}

fn print_head_inspect_text(summary: &HeadInspectSummary) -> i32 {
    println!("input path: {}", summary.input_path.display());
    println!("doc id: {}", summary.doc_id);
    if let Some(profile_id) = &summary.profile_id {
        println!("profile id: {profile_id}");
    }
    if let Some(effective_selection_time) = summary.effective_selection_time {
        println!("effective selection time: {effective_selection_time}");
    }
    if let Some(selector_epoch) = summary.selector_epoch {
        println!("selector epoch: {selector_epoch}");
    }
    println!("verified revisions: {}", summary.verified_revision_count);
    println!("verified views: {}", summary.verified_view_count);
    println!("status: {}", summary.status);

    for head in &summary.eligible_heads {
        println!(
            "eligible head: {} timestamp={} score={} supporters={}",
            head.revision_id, head.revision_timestamp, head.selector_score, head.supporter_count
        );
    }

    if let Some(selected_head) = &summary.selected_head {
        println!("selected head: {selected_head}");
    }
    if let Some(tie_break_reason) = &summary.tie_break_reason {
        println!("tie break reason: {tie_break_reason}");
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
            eprintln!("error: {error}");
        }
        1
    }
}

fn print_head_inspect_json(summary: &HeadInspectSummary) -> i32 {
    match serde_json::to_string_pretty(summary) {
        Ok(json) => {
            println!("{json}");
            if summary.is_ok() {
                0
            } else {
                1
            }
        }
        Err(err) => {
            eprintln!("failed to serialize head inspection summary: {err}");
            2
        }
    }
}

fn head_inspect(doc_id: String, input_path: PathBuf, json: bool) -> i32 {
    let summary = inspect_heads_from_path(&input_path, &doc_id);
    if json {
        print_head_inspect_json(&summary)
    } else {
        print_head_inspect_text(&summary)
    }
}

fn print_object_verification_text(summary: &ObjectVerificationSummary) -> i32 {
    println!("object path: {}", summary.path.display());
    if let Some(object_type) = &summary.object_type {
        println!("object type: {object_type}");
    }
    if let Some(signature_rule) = &summary.signature_rule {
        println!("signature rule: {signature_rule}");
    }
    if let Some(signer_field) = &summary.signer_field {
        println!("signer field: {signer_field}");
    }
    if let Some(signer) = &summary.signer {
        println!("signer: {signer}");
    }
    if let Some(signature_verification) = &summary.signature_verification {
        println!("signature verification: {signature_verification}");
    }
    if let Some(declared_id) = &summary.declared_id {
        println!("declared id: {declared_id}");
    }
    if let Some(recomputed_id) = &summary.recomputed_id {
        println!("recomputed id: {recomputed_id}");
    }
    println!("status: {}", summary.status);

    for note in &summary.notes {
        println!("note: {note}");
    }

    if summary.is_ok() {
        println!("verification: ok");
        0
    } else {
        println!("verification: failed");
        for error in &summary.errors {
            eprintln!("error: {error}");
        }
        1
    }
}

fn print_object_verification_json(summary: &ObjectVerificationSummary) -> i32 {
    match serde_json::to_string_pretty(summary) {
        Ok(json) => {
            println!("{json}");
            if summary.is_ok() {
                0
            } else {
                1
            }
        }
        Err(err) => {
            eprintln!("failed to serialize object verification summary: {err}");
            2
        }
    }
}

fn object_verify(target: PathBuf, json: bool) -> i32 {
    let summary = verify_object_path(&target);
    if json {
        print_object_verification_json(&summary)
    } else {
        print_object_verification_text(&summary)
    }
}

fn print_validation_text(summary: &mycel_sim::validate::ValidationSummary) -> i32 {
    if let Some(root) = &summary.root {
        println!("repo root: {}", root.display());
    }
    if let Some(target) = &summary.target {
        println!("validated target: {}", target.display());
    }
    println!("status: {}", summary.status);
    println!("fixtures: {}", summary.fixture_count);
    println!("peers: {}", summary.peer_count);
    println!("topologies: {}", summary.topology_count);
    println!("tests: {}", summary.test_case_count);
    println!("reports: {}", summary.report_count);

    if !summary.warnings.is_empty() {
        for warning in &summary.warnings {
            eprintln!("warning: {}: {}", warning.path, warning.message);
        }
    }

    if !summary.is_ok() {
        println!("validation: failed");
        for error in &summary.errors {
            eprintln!("error: {}: {}", error.path, error.message);
        }
        1
    } else if summary.has_warnings() {
        println!("validation: warning");
        0
    } else {
        println!("validation: ok");
        0
    }
}

fn print_validation_json(summary: &mycel_sim::validate::ValidationSummary) -> i32 {
    match serde_json::to_string_pretty(summary) {
        Ok(json) => {
            println!("{json}");
            if summary.is_ok() {
                0
            } else {
                1
            }
        }
        Err(err) => {
            eprintln!("failed to serialize validation summary: {err}");
            2
        }
    }
}

fn validate(target: PathBuf, json: bool, strict: bool) -> i32 {
    let summary = validate_path(&target);
    let exit_code = if !summary.is_ok() {
        1
    } else if strict && summary.has_warnings() {
        1
    } else {
        0
    };

    let print_code = if json {
        print_validation_json(&summary)
    } else {
        print_validation_text(&summary)
    };

    if print_code != 0 {
        print_code
    } else {
        exit_code
    }
}

fn print_run_text(summary: &mycel_sim::run::SimulationRunSummary) -> i32 {
    println!("repo root: {}", summary.root.display());
    println!("run target: {}", summary.target.display());
    println!("started at: {}", summary.started_at);
    println!("finished at: {}", summary.finished_at);
    println!("run duration ms: {}", summary.run_duration_ms);
    println!("deterministic seed: {}", summary.deterministic_seed);
    println!("seed source: {}", summary.seed_source);
    println!("events per second: {:.3}", summary.events_per_second);
    println!("ms per event: {:.3}", summary.ms_per_event);
    println!(
        "scheduled peer order: {}",
        summary.scheduled_peer_order.join(" -> ")
    );
    if summary.fault_plan.is_empty() {
        println!("fault plan: none");
    } else {
        println!(
            "fault plan: {}",
            summary
                .fault_plan
                .iter()
                .map(|entry| format!(
                    "#{}:{}:{}->{}",
                    entry.order,
                    entry.fault,
                    entry.source_node_id,
                    entry
                        .target_node_id
                        .as_deref()
                        .unwrap_or("unspecified-target")
                ))
                .collect::<Vec<_>>()
                .join(", ")
        );
    }
    println!("validation status: {}", summary.validation_status);
    println!("report path: {}", summary.report_path.display());
    println!("result: {}", summary.result);
    println!("peers: {}", summary.peer_count);
    println!("events: {}", summary.event_count);
    println!("verified objects: {}", summary.verified_object_count);
    println!("rejected objects: {}", summary.rejected_object_count);

    if !summary.matched_expected_outcomes.is_empty() {
        println!(
            "matched expected outcomes: {}",
            summary.matched_expected_outcomes.join(", ")
        );
    }

    for warning in &summary.validation_warnings {
        eprintln!("warning: {warning}");
    }

    0
}

fn print_run_json(summary: &mycel_sim::run::SimulationRunSummary) -> i32 {
    match serde_json::to_string_pretty(summary) {
        Ok(json) => {
            println!("{json}");
            0
        }
        Err(err) => {
            eprintln!("failed to serialize run summary: {err}");
            2
        }
    }
}

fn sim_run(target: PathBuf, json: bool, seed_override: Option<String>) -> i32 {
    let options = RunOptions { seed_override };
    match run_test_case_with_options(&target, &options) {
        Ok(summary) => {
            if json {
                print_run_json(&summary)
            } else {
                print_run_text(&summary)
            }
        }
        Err(message) => {
            eprintln!("sim run failed: {message}");
            1
        }
    }
}

fn main() {
    let mut args = env::args().skip(1);

    match args.next().as_deref() {
        Some("head") => match args.next().as_deref() {
            Some("inspect") => {
                let mut doc_id = None;
                let mut input_path = None;
                let mut json = false;
                let mut expect_input_path = false;

                for arg in args {
                    if expect_input_path {
                        input_path = Some(PathBuf::from(arg));
                        expect_input_path = false;
                    } else if arg == "--json" {
                        json = true;
                    } else if arg == "--input" {
                        expect_input_path = true;
                    } else if doc_id.is_none() {
                        doc_id = Some(arg);
                    } else {
                        eprintln!("unexpected head inspect argument: {arg}");
                        eprintln!();
                        print_usage();
                        std::process::exit(2);
                    }
                }

                if expect_input_path {
                    eprintln!("missing value for --input");
                    eprintln!();
                    print_usage();
                    std::process::exit(2);
                }

                let Some(doc_id) = doc_id else {
                    eprintln!("missing head inspect doc_id");
                    eprintln!();
                    print_usage();
                    std::process::exit(2);
                };
                let Some(input_path) = input_path else {
                    eprintln!("missing --input for head inspect");
                    eprintln!();
                    print_usage();
                    std::process::exit(2);
                };

                std::process::exit(head_inspect(doc_id, input_path, json));
            }
            Some(other) => {
                eprintln!("unknown head subcommand: {other}");
                eprintln!();
                print_usage();
                std::process::exit(2);
            }
            None => {
                eprintln!("missing head subcommand");
                eprintln!();
                print_usage();
                std::process::exit(2);
            }
        },
        Some("info") => print_info(),
        Some("object") => match args.next().as_deref() {
            Some("verify") => {
                let mut target = None;
                let mut json = false;

                for arg in args {
                    if arg == "--json" {
                        json = true;
                    } else if target.is_none() {
                        target = Some(PathBuf::from(arg));
                    } else {
                        eprintln!("unexpected object verify argument: {arg}");
                        eprintln!();
                        print_usage();
                        std::process::exit(2);
                    }
                }

                let Some(target) = target else {
                    eprintln!("missing object verify target");
                    eprintln!();
                    print_usage();
                    std::process::exit(2);
                };

                std::process::exit(object_verify(target, json));
            }
            Some(other) => {
                eprintln!("unknown object subcommand: {other}");
                eprintln!();
                print_usage();
                std::process::exit(2);
            }
            None => {
                eprintln!("missing object subcommand");
                eprintln!();
                print_usage();
                std::process::exit(2);
            }
        },
        Some("validate") => {
            let mut target = PathBuf::from(".");
            let mut json = false;
            let mut strict = false;

            for arg in args {
                if arg == "--json" {
                    json = true;
                } else if arg == "--strict" {
                    strict = true;
                } else if target == PathBuf::from(".") {
                    target = PathBuf::from(arg);
                } else {
                    eprintln!("unexpected validate argument: {arg}");
                    eprintln!();
                    print_usage();
                    std::process::exit(2);
                }
            }

            std::process::exit(validate(target, json, strict));
        }
        Some("sim") => match args.next().as_deref() {
            Some("run") => {
                let mut target = None;
                let mut json = false;
                let mut seed_override = None;
                let mut expect_seed_value = false;

                for arg in args {
                    if expect_seed_value {
                        seed_override = Some(arg);
                        expect_seed_value = false;
                    } else if arg == "--json" {
                        json = true;
                    } else if arg == "--seed" {
                        expect_seed_value = true;
                    } else if target.is_none() {
                        target = Some(PathBuf::from(arg));
                    } else {
                        eprintln!("unexpected sim run argument: {arg}");
                        eprintln!();
                        print_usage();
                        std::process::exit(2);
                    }
                }

                if expect_seed_value {
                    eprintln!("missing value for --seed");
                    eprintln!();
                    print_usage();
                    std::process::exit(2);
                }

                let Some(target) = target else {
                    eprintln!("missing sim run target");
                    eprintln!();
                    print_usage();
                    std::process::exit(2);
                };

                std::process::exit(sim_run(target, json, seed_override));
            }
            Some(other) => {
                eprintln!("unknown sim subcommand: {other}");
                eprintln!();
                print_usage();
                std::process::exit(2);
            }
            None => {
                eprintln!("missing sim subcommand");
                eprintln!();
                print_usage();
                std::process::exit(2);
            }
        },
        Some("help") | None => print_usage(),
        Some(other) => {
            eprintln!("unknown command: {other}");
            eprintln!();
            print_usage();
            std::process::exit(2);
        }
    }
}
