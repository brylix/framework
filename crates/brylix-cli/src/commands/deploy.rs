//! `brylix deploy` command implementation.

use console::{style, Emoji};
use std::process::Command;

static ROCKET: Emoji<'_, '_> = Emoji("🚀 ", "");

/// Deploy to AWS Lambda.
pub fn run(profile: Option<&str>, function: Option<&str>) {
    println!("{} Deploying to AWS Lambda...", ROCKET);

    let mut args = vec!["lambda", "deploy"];

    if let Some(f) = function {
        args.push(f);
    }

    let mut cmd = Command::new("cargo");
    cmd.args(&args);

    if let Some(p) = profile {
        cmd.env("AWS_PROFILE", p);
        println!("  Using AWS profile: {}", style(p).cyan());
    }

    let status = cmd.status().expect("Failed to execute cargo lambda deploy");

    if status.success() {
        println!(
            "{} Deployment completed successfully!",
            style("✅").green().bold()
        );
    } else {
        eprintln!("{} Deployment failed", style("❌").red().bold());
        std::process::exit(status.code().unwrap_or(1));
    }
}
