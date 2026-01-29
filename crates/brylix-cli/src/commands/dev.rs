//! `brylix dev` command implementation.

use console::{style, Emoji};
use std::process::Command;

static ROCKET: Emoji<'_, '_> = Emoji("🚀 ", "");

/// Run the development server with hot reload.
pub fn run(port: u16) {
    println!(
        "{} Starting development server on port {}...",
        ROCKET,
        style(port).cyan().bold()
    );

    // Check if cargo-lambda is installed
    let lambda_check = Command::new("cargo")
        .args(["lambda", "--version"])
        .output();

    if lambda_check.is_err() {
        eprintln!(
            "{} cargo-lambda is not installed. Install it with:",
            style("Error:").red().bold()
        );
        eprintln!("  cargo install cargo-lambda");
        std::process::exit(1);
    }

    // Run cargo lambda watch
    let status = Command::new("cargo")
        .args([
            "lambda",
            "watch",
            "--env-file",
            ".env",
            "--invoke-address",
            &format!("127.0.0.1:{}", port),
        ])
        .status()
        .expect("Failed to execute cargo lambda watch");

    if !status.success() {
        std::process::exit(status.code().unwrap_or(1));
    }
}
