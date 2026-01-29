//! `brylix test` command implementation.

use console::{style, Emoji};
use std::process::Command;

static TEST_TUBE: Emoji<'_, '_> = Emoji("🧪 ", "");
static WATCH: Emoji<'_, '_> = Emoji("👀 ", "");

/// Run tests for the project.
pub fn run(
    unit: bool,
    integration: bool,
    watch: bool,
    release: bool,
    verbose: bool,
    filter: Option<&str>,
) {
    if watch {
        run_watch(unit, integration, release, verbose, filter);
    } else {
        run_tests(unit, integration, release, verbose, filter);
    }
}

fn run_tests(unit: bool, integration: bool, release: bool, verbose: bool, filter: Option<&str>) {
    let test_type = match (unit, integration) {
        (true, false) => "unit",
        (false, true) => "integration",
        _ => "all",
    };

    println!(
        "{} Running {} tests...",
        TEST_TUBE,
        style(test_type).cyan().bold()
    );

    let mut args = vec!["test"];

    // Test scope
    if unit && !integration {
        args.push("--lib");
    } else if integration && !unit {
        args.push("--test");
        args.push("*");
    }

    // Release mode
    if release {
        args.push("--release");
        println!("  Running in release mode");
    }

    // Verbose output
    if verbose {
        args.push("--");
        args.push("--nocapture");
    }

    // Test filter
    let filter_with_separator: String;
    if let Some(f) = filter {
        if !verbose {
            args.push("--");
        }
        filter_with_separator = f.to_string();
        args.push(&filter_with_separator);
        println!("  Filtering tests matching: {}", style(f).yellow());
    }

    let status = Command::new("cargo")
        .args(&args)
        .status()
        .expect("Failed to execute cargo test");

    if status.success() {
        println!(
            "\n{} All tests passed!",
            style("✅").green().bold()
        );
    } else {
        eprintln!(
            "\n{} Some tests failed",
            style("❌").red().bold()
        );
        std::process::exit(status.code().unwrap_or(1));
    }
}

fn run_watch(
    unit: bool,
    integration: bool,
    release: bool,
    verbose: bool,
    filter: Option<&str>,
) {
    println!(
        "{} Starting test watcher...",
        WATCH
    );
    println!("  Press Ctrl+C to stop\n");

    // Check if cargo-watch is installed
    let check = Command::new("cargo")
        .args(["watch", "--version"])
        .output();

    if check.is_err() || !check.unwrap().status.success() {
        eprintln!(
            "{} cargo-watch is not installed. Install it with:",
            style("⚠️").yellow().bold()
        );
        eprintln!("  cargo install cargo-watch");
        std::process::exit(1);
    }

    let mut test_args = String::from("test");

    // Test scope
    if unit && !integration {
        test_args.push_str(" --lib");
    } else if integration && !unit {
        test_args.push_str(" --test '*'");
    }

    // Release mode
    if release {
        test_args.push_str(" --release");
    }

    // Verbose and filter
    if verbose || filter.is_some() {
        test_args.push_str(" --");
    }

    if verbose {
        test_args.push_str(" --nocapture");
    }

    if let Some(f) = filter {
        test_args.push(' ');
        test_args.push_str(f);
    }

    let status = Command::new("cargo")
        .args(["watch", "-x", &test_args])
        .status()
        .expect("Failed to execute cargo watch");

    if !status.success() {
        std::process::exit(status.code().unwrap_or(1));
    }
}
