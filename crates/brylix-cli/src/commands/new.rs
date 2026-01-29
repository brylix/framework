//! `brylix new` command implementation.

use console::{style, Emoji};
use std::fs;
use std::path::Path;

static SPARKLE: Emoji<'_, '_> = Emoji("✨ ", "");
static FOLDER: Emoji<'_, '_> = Emoji("📁 ", "");
static CHECK: Emoji<'_, '_> = Emoji("✅ ", "");

/// Run the `new` command to create a new Brylix project.
pub fn run(name: &str, multi_tenant: bool, database: &str) {
    println!(
        "{} Creating new Brylix project: {}",
        SPARKLE,
        style(name).cyan().bold()
    );

    let project_path = Path::new(name);

    if project_path.exists() {
        eprintln!(
            "{} Directory '{}' already exists",
            style("Error:").red().bold(),
            name
        );
        std::process::exit(1);
    }

    // Create directory structure
    create_project_structure(project_path, name, multi_tenant, database);

    println!();
    println!("{} Project created successfully!", CHECK);
    println!();
    println!("Next steps:");
    println!("  cd {}", name);
    println!("  # Update .env with your database credentials");
    println!("  brylix dev");
    println!();
}

fn create_project_structure(path: &Path, name: &str, multi_tenant: bool, database: &str) {
    // Create directories
    let dirs = [
        "",
        "src",
        "src/graphql",
        "src/service",
        "src/repository",
        "src/model",
        "migration",
        "migration/src",
    ];

    for dir in dirs {
        let dir_path = path.join(dir);
        fs::create_dir_all(&dir_path).expect("Failed to create directory");
        if !dir.is_empty() {
            println!("  {} Created {}/", FOLDER, dir);
        }
    }

    // Create Cargo.toml
    let cargo_toml = generate_cargo_toml(name, multi_tenant, database);
    fs::write(path.join("Cargo.toml"), cargo_toml).expect("Failed to write Cargo.toml");
    println!("  {} Created Cargo.toml", CHECK);

    // Create .env.example
    let env_content = generate_env_example(multi_tenant);
    fs::write(path.join(".env.example"), &env_content).expect("Failed to write .env.example");
    fs::write(path.join(".env"), &env_content).expect("Failed to write .env");
    println!("  {} Created .env.example", CHECK);

    // Create main.rs
    let main_rs = generate_main_rs(multi_tenant);
    fs::write(path.join("src/main.rs"), main_rs).expect("Failed to write main.rs");
    println!("  {} Created src/main.rs", CHECK);

    // Create graphql modules
    create_graphql_modules(path);

    // Create migration
    create_migration_files(path);
}

fn generate_cargo_toml(name: &str, multi_tenant: bool, database: &str) -> String {
    let db_feature = if database == "postgres" {
        r#"brylix = { version = "0.1", default-features = false, features = ["postgres", "playground"] }"#
    } else if multi_tenant {
        r#"brylix = { version = "0.1", features = ["multi-tenant"] }"#
    } else {
        r#"brylix = "0.1""#
    };

    format!(
        r#"[package]
name = "{name}"
version = "0.1.0"
edition = "2021"

[dependencies]
{db_feature}
tokio = {{ version = "1", features = ["rt-multi-thread", "macros"] }}
async-graphql = {{ version = "7", features = ["chrono"] }}
sea-orm = {{ version = "1.1", features = ["sqlx-mysql", "runtime-tokio-rustls", "macros"] }}
serde = {{ version = "1", features = ["derive"] }}
chrono = {{ version = "0.4", features = ["serde"] }}
tracing = "0.1"
tracing-subscriber = {{ version = "0.3", features = ["env-filter"] }}

[dependencies.migration]
path = "./migration"
"#
    )
}

fn generate_env_example(multi_tenant: bool) -> String {
    let mut content = r#"# Database configuration
DB_HOST=localhost
DB_USER=root
DB_PASSWORD=
DB_NAME=my_database
DB_PORT=3306

# JWT configuration
JWT_SECRET=change-this-to-a-secure-secret-key
JWT_EXP_DAYS=7

# Logging
RUST_LOG=info
"#
    .to_string();

    if multi_tenant {
        content.push_str(
            r#"
# Multi-tenant configuration
MULTI_TENANT_MODE=true
REQUIRED_DB_VERSION=1
TENANT_DB_PASSWORD=tenant-db-password
"#,
        );
    }

    content
}

fn generate_main_rs(_multi_tenant: bool) -> String {
    r##"use brylix::prelude::*;
use async_graphql::{EmptySubscription, Schema};

mod graphql;
mod model;
mod repository;
mod service;

#[tokio::main]
async fn main() -> Result<(), brylix::Error> {
    // Initialize tracing
    tracing_subscriber::fmt()
        .with_env_filter(tracing_subscriber::EnvFilter::from_default_env())
        .init();

    // Build and run the application
    let config = Config::init().expect("Failed to load configuration");

    let db = brylix::db::init_db::<migration::Migrator>(&config.database.url())
        .await
        .expect("Failed to initialize database");

    // For now, run a simple Lambda handler
    // In production, use Brylix::builder() pattern
    tracing::info!("Starting Brylix application");

    lambda_http::run(lambda_http::service_fn(|_req| async {
        brylix::handler::error_response(
            http::StatusCode::OK,
            r#"{"message":"Hello from Brylix!"}"#.to_string(),
        )
    }))
    .await
}
"##.to_string()
}

fn create_graphql_modules(path: &Path) {
    // mod.rs
    let mod_rs = r#"mod mutation;
mod query;
mod types;

pub use mutation::Mutation;
pub use query::Query;
pub use types::*;
"#;
    fs::write(path.join("src/graphql/mod.rs"), mod_rs).expect("Failed to write graphql/mod.rs");

    // query.rs
    let query_rs = r#"use async_graphql::{Context, Object, Result};
use brylix::prelude::*;

pub struct Query;

#[Object]
impl Query {
    /// Health check
    async fn health(&self) -> Result<String> {
        Ok("OK".to_string())
    }

    /// Get current authenticated user
    async fn me(&self, ctx: &Context<'_>) -> Result<String> {
        let user_id = require_auth_user_id(ctx)?;
        Ok(format!("User ID: {}", user_id))
    }
}
"#;
    fs::write(path.join("src/graphql/query.rs"), query_rs).expect("Failed to write graphql/query.rs");

    // mutation.rs
    let mutation_rs = r#"use async_graphql::{Context, Object, Result};
use brylix::prelude::*;

pub struct Mutation;

#[Object]
impl Mutation {
    /// Login and get JWT token
    async fn login(
        &self,
        ctx: &Context<'_>,
        email: String,
        password: String,
    ) -> Result<String> {
        // TODO: Implement actual login logic
        // For now, return a placeholder
        Err(gql_bad_request("Login not implemented yet"))
    }
}
"#;
    fs::write(path.join("src/graphql/mutation.rs"), mutation_rs)
        .expect("Failed to write graphql/mutation.rs");

    // types.rs
    let types_rs = r#"use async_graphql::SimpleObject;
use chrono::NaiveDateTime;

#[derive(Debug, Clone, SimpleObject)]
pub struct AuthPayload {
    pub token: String,
}
"#;
    fs::write(path.join("src/graphql/types.rs"), types_rs).expect("Failed to write graphql/types.rs");

    // Create placeholder modules
    fs::write(path.join("src/model/mod.rs"), "// SeaORM entities go here\n")
        .expect("Failed to write model/mod.rs");
    fs::write(path.join("src/service/mod.rs"), "// Business logic services go here\n")
        .expect("Failed to write service/mod.rs");
    fs::write(path.join("src/repository/mod.rs"), "// Database repositories go here\n")
        .expect("Failed to write repository/mod.rs");

    println!("  {} Created GraphQL modules", CHECK);
}

fn create_migration_files(path: &Path) {
    // migration/Cargo.toml
    let migration_cargo = r#"[package]
name = "migration"
version = "0.1.0"
edition = "2021"

[lib]
name = "migration"
path = "src/lib.rs"

[dependencies]
sea-orm-migration = { version = "1.1", features = ["runtime-tokio-rustls", "sqlx-mysql"] }
async-trait = "0.1"
"#;
    fs::write(path.join("migration/Cargo.toml"), migration_cargo)
        .expect("Failed to write migration/Cargo.toml");

    // migration/src/lib.rs
    let migration_lib = r#"pub use sea_orm_migration::prelude::*;

mod m20240101_000001_create_users_table;

pub struct Migrator;

#[async_trait::async_trait]
impl MigratorTrait for Migrator {
    fn migrations() -> Vec<Box<dyn MigrationTrait>> {
        vec![
            Box::new(m20240101_000001_create_users_table::Migration),
        ]
    }
}
"#;
    fs::write(path.join("migration/src/lib.rs"), migration_lib)
        .expect("Failed to write migration/src/lib.rs");

    // Initial migration
    let initial_migration = r#"use sea_orm_migration::prelude::*;

#[derive(DeriveMigrationName)]
pub struct Migration;

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .create_table(
                Table::create()
                    .table(Users::Table)
                    .if_not_exists()
                    .col(
                        ColumnDef::new(Users::Id)
                            .big_integer()
                            .not_null()
                            .auto_increment()
                            .primary_key(),
                    )
                    .col(
                        ColumnDef::new(Users::Email)
                            .string()
                            .not_null()
                            .unique_key(),
                    )
                    .col(ColumnDef::new(Users::Password).string().not_null())
                    .col(ColumnDef::new(Users::Name).string().not_null())
                    .col(
                        ColumnDef::new(Users::CreatedAt)
                            .timestamp()
                            .not_null()
                            .default(Expr::current_timestamp()),
                    )
                    .to_owned(),
            )
            .await
    }

    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .drop_table(Table::drop().table(Users::Table).to_owned())
            .await
    }
}

#[derive(Iden)]
enum Users {
    Table,
    Id,
    Email,
    Password,
    Name,
    CreatedAt,
}
"#;
    fs::write(
        path.join("migration/src/m20240101_000001_create_users_table.rs"),
        initial_migration,
    )
    .expect("Failed to write initial migration");

    println!("  {} Created migration files", CHECK);
}
