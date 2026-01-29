//! Basic Blog API Example
//!
//! A minimal single-tenant GraphQL API using the Brylix framework.
//! Demonstrates entity, service, repository, and GraphQL resolver patterns.

use brylix::prelude::*;
use lambda_http::{run, service_fn, Error};
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

    // Initialize database connection
    let db = brylix::db::connect(&config.database.url())
        .await
        .expect("Failed to connect to database");

    // Run migrations
    migration::Migrator::up(&db, None)
        .await
        .expect("Failed to run migrations");

    // Build GraphQL schema with database connection
    let schema = build_schema(db);

    // Start Lambda (simplified for example)
    println!("Starting server...");
    // Note: Full handler implementation would use brylix::handler utilities
    Ok(())
}

/// Migration module - re-export from migration crate
mod migration {
    use sea_orm_migration::prelude::*;

    pub struct Migrator;

    #[async_trait::async_trait]
    impl MigratorTrait for Migrator {
        fn migrations() -> Vec<Box<dyn MigrationTrait>> {
            vec![Box::new(super::m20240101_000001_create_posts_table::Migration)]
        }
    }
}

mod m20240101_000001_create_posts_table {
    use sea_orm_migration::prelude::*;

    #[derive(DeriveMigrationName)]
    pub struct Migration;

    #[async_trait::async_trait]
    impl MigrationTrait for Migration {
        async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
            manager
                .create_table(
                    Table::create()
                        .table(Post::Table)
                        .if_not_exists()
                        .col(
                            ColumnDef::new(Post::Id)
                                .big_integer()
                                .not_null()
                                .auto_increment()
                                .primary_key(),
                        )
                        .col(ColumnDef::new(Post::Title).string_len(255).not_null())
                        .col(ColumnDef::new(Post::Content).text().not_null())
                        .col(ColumnDef::new(Post::Published).boolean().not_null().default(false))
                        .col(
                            ColumnDef::new(Post::CreatedAt)
                                .timestamp()
                                .not_null()
                                .default(Expr::current_timestamp()),
                        )
                        .col(ColumnDef::new(Post::UpdatedAt).timestamp().null())
                        .to_owned(),
                )
                .await
        }

        async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
            manager
                .drop_table(Table::drop().table(Post::Table).to_owned())
                .await
        }
    }

    #[derive(Iden)]
    enum Post {
        Table,
        Id,
        Title,
        Content,
        Published,
        CreatedAt,
        UpdatedAt,
    }
}
