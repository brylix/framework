//! Multi-Tenant Task SaaS Example
//!
//! A multi-tenant GraphQL API using the Brylix framework.
//! Demonstrates tenant isolation, authentication, and connection pooling.

use brylix::prelude::*;
use lambda_http::Error;
use sea_orm_migration::MigratorTrait;

mod graphql;
mod model;
mod repository;
mod service;

use graphql::schema::build_schema;

#[tokio::main]
async fn main() -> Result<(), Error> {
    // Initialize configuration from environment
    let config = brylix::config::Config::from_env()
        .expect("Failed to load configuration");

    // Initialize master database connection (for tenant registry)
    let master_db = brylix::db::connect(&config.database.url())
        .await
        .expect("Failed to connect to master database");

    // Build GraphQL schema
    let _schema = build_schema();

    // Note: Full multi-tenant handler implementation would:
    // 1. Parse tenant from URL path (/api/{tenant})
    // 2. Validate tenant exists in master database
    // 3. Get/create connection pool for tenant's database host
    // 4. Run migrations on first access
    // 5. Set up ContextData with tenant info

    println!("Multi-tenant server starting...");
    println!("Master DB connected: {}", config.database.host);

    Ok(())
}

/// Migration module for tenant databases
pub mod migration {
    use sea_orm_migration::prelude::*;

    pub struct Migrator;

    #[async_trait::async_trait]
    impl MigratorTrait for Migrator {
        fn migrations() -> Vec<Box<dyn MigrationTrait>> {
            vec![
                Box::new(m20240101_000001_create_users_table::Migration),
                Box::new(m20240101_000002_create_tasks_table::Migration),
            ]
        }
    }

    mod m20240101_000001_create_users_table {
        use sea_orm_migration::prelude::*;

        #[derive(DeriveMigrationName)]
        pub struct Migration;

        #[async_trait::async_trait]
        impl MigrationTrait for Migration {
            async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
                manager
                    .create_table(
                        Table::create()
                            .table(User::Table)
                            .if_not_exists()
                            .col(
                                ColumnDef::new(User::Id)
                                    .big_integer()
                                    .not_null()
                                    .auto_increment()
                                    .primary_key(),
                            )
                            .col(ColumnDef::new(User::Email).string_len(255).not_null().unique_key())
                            .col(ColumnDef::new(User::PasswordHash).string_len(255).not_null())
                            .col(ColumnDef::new(User::Name).string_len(255).not_null())
                            .col(
                                ColumnDef::new(User::CreatedAt)
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
                    .drop_table(Table::drop().table(User::Table).to_owned())
                    .await
            }
        }

        #[derive(Iden)]
        enum User {
            Table,
            Id,
            Email,
            PasswordHash,
            Name,
            CreatedAt,
        }
    }

    mod m20240101_000002_create_tasks_table {
        use sea_orm_migration::prelude::*;

        #[derive(DeriveMigrationName)]
        pub struct Migration;

        #[async_trait::async_trait]
        impl MigrationTrait for Migration {
            async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
                manager
                    .create_table(
                        Table::create()
                            .table(Task::Table)
                            .if_not_exists()
                            .col(
                                ColumnDef::new(Task::Id)
                                    .big_integer()
                                    .not_null()
                                    .auto_increment()
                                    .primary_key(),
                            )
                            .col(ColumnDef::new(Task::Title).string_len(255).not_null())
                            .col(ColumnDef::new(Task::Description).text().null())
                            .col(ColumnDef::new(Task::Status).string_len(50).not_null().default("pending"))
                            .col(ColumnDef::new(Task::AssigneeId).big_integer().null())
                            .col(ColumnDef::new(Task::DueDate).date().null())
                            .col(
                                ColumnDef::new(Task::CreatedAt)
                                    .timestamp()
                                    .not_null()
                                    .default(Expr::current_timestamp()),
                            )
                            .col(ColumnDef::new(Task::UpdatedAt).timestamp().null())
                            .to_owned(),
                    )
                    .await
            }

            async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
                manager
                    .drop_table(Table::drop().table(Task::Table).to_owned())
                    .await
            }
        }

        #[derive(Iden)]
        enum Task {
            Table,
            Id,
            Title,
            Description,
            Status,
            AssigneeId,
            DueDate,
            CreatedAt,
            UpdatedAt,
        }
    }
}
