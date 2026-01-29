//! `brylix build` command implementation.

use console::{style, Emoji};
use std::process::Command;

static HAMMER: Emoji<'_, '_> = Emoji("🔨 ", "");

/// Build the project for AWS Lambda deployment.
pub fn run(release: bool, arm64: bool) {
    println!("{} Building for AWS Lambda...", HAMMER);

    let mut args = vec!["lambda", "build"];

    if release {
        args.push("--release");
    }

    if arm64 {
        args.push("--arm64");
        println!("  Building for ARM64 (Graviton)");
    }

    let status = Command::new("cargo")
        .args(&args)
        .status()
        .expect("Failed to execute cargo lambda build");

    if status.success() {
        println!(
            "{} Build completed successfully!",
            style("✅").green().bold()
        );
    } else {
        eprintln!("{} Build failed", style("❌").red().bold());
        std::process::exit(status.code().unwrap_or(1));
    }
}
