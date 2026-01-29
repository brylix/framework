//! `brylix migrate` command implementation.

use console::{style, Emoji};
use std::process::Command;

static DATABASE: Emoji<'_, '_> = Emoji("🗄️ ", "");

/// Run database migrations.
pub fn run(generate: Option<&str>, down: bool) {
    if let Some(name) = generate {
        println!("{} Generating migration: {}", DATABASE, style(name).cyan());

        let status = Command::new("sea-orm-cli")
            .args(["migrate", "generate", name, "-d", "migration"])
            .status()
            .expect("Failed to execute sea-orm-cli");

        if !status.success() {
            eprintln!(
                "{} Failed to generate migration. Make sure sea-orm-cli is installed:",
                style("Error:").red().bold()
            );
            eprintln!("  cargo install sea-orm-cli");
            std::process::exit(status.code().unwrap_or(1));
        }
    } else if down {
        println!("{} Rolling back last migration...", DATABASE);

        let status = Command::new("sea-orm-cli")
            .args(["migrate", "down", "-d", "migration"])
            .status()
            .expect("Failed to execute sea-orm-cli");

        if !status.success() {
            std::process::exit(status.code().unwrap_or(1));
        }
    } else {
        println!("{} Running migrations...", DATABASE);

        let status = Command::new("sea-orm-cli")
            .args(["migrate", "up", "-d", "migration"])
            .status()
            .expect("Failed to execute sea-orm-cli");

        if status.success() {
            println!(
                "{} Migrations completed successfully!",
                style("✅").green().bold()
            );
        } else {
            std::process::exit(status.code().unwrap_or(1));
        }
    }
}
