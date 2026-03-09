use std::env;
use std::path::PathBuf;

use mycel_core::workspace_banner;
use mycel_sim::manifest::SimulatorPaths;
use mycel_sim::run::run_test_case;
use mycel_sim::simulator_banner;
use mycel_sim::validate::validate_path;

fn print_usage() {
    println!("mycel <command> [path]");
    println!();
    println!("Commands:");
    println!("  info       Show workspace and simulator scaffold information");
    println!("  sim        Run a simulator test case");
    println!("  validate   Validate the repo root, one file, or one supported directory");
    println!("  help       Show this message");
    println!();
    println!("Sim options:");
    println!("  run <path> Run one test-case and write a report to sim/reports/out/");
    println!("  --json     Emit machine-readable run output");
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

fn sim_run(target: PathBuf, json: bool) -> i32 {
    match run_test_case(&target) {
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
        Some("info") => print_info(),
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

                for arg in args {
                    if arg == "--json" {
                        json = true;
                    } else if target.is_none() {
                        target = Some(PathBuf::from(arg));
                    } else {
                        eprintln!("unexpected sim run argument: {arg}");
                        eprintln!();
                        print_usage();
                        std::process::exit(2);
                    }
                }

                let Some(target) = target else {
                    eprintln!("missing sim run target");
                    eprintln!();
                    print_usage();
                    std::process::exit(2);
                };

                std::process::exit(sim_run(target, json));
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
