use std::env;
use std::path::PathBuf;

use mycel_core::workspace_banner;
use mycel_sim::manifest::SimulatorPaths;
use mycel_sim::simulator_banner;
use mycel_sim::validate::validate_path;

fn print_usage() {
    println!("mycel <command> [path]");
    println!();
    println!("Commands:");
    println!("  info       Show workspace and simulator scaffold information");
    println!("  validate   Validate the repo root, one file, or one supported directory");
    println!("  help       Show this message");
    println!();
    println!("Planned next commands:");
    println!("  sim        Run a simulator test case");
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

fn validate(target: PathBuf) -> i32 {
    let summary = validate_path(&target);

    if let Some(root) = &summary.root {
        println!("repo root: {}", root.display());
    }
    if let Some(target) = &summary.target {
        println!("validated target: {}", target.display());
    }
    println!("fixtures: {}", summary.fixture_count);
    println!("peers: {}", summary.peer_count);
    println!("topologies: {}", summary.topology_count);
    println!("tests: {}", summary.test_case_count);
    println!("reports: {}", summary.report_count);

    if summary.is_ok() {
        println!("validation: ok");
        0
    } else {
        println!("validation: failed");
        for error in &summary.errors {
            eprintln!("error: {}: {}", error.path, error.message);
        }
        1
    }
}

fn main() {
    let mut args = env::args().skip(1);

    match args.next().as_deref() {
        Some("info") => print_info(),
        Some("validate") => {
            let target = args
                .next()
                .map(PathBuf::from)
                .unwrap_or_else(|| PathBuf::from("."));
            std::process::exit(validate(target));
        }
        Some("help") | None => print_usage(),
        Some(other) => {
            eprintln!("unknown command: {other}");
            eprintln!();
            print_usage();
            std::process::exit(2);
        }
    }
}
