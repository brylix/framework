//! Database initialization and connection management.
//!
//! Provides utilities for initializing database connections and running migrations.
//!
//! # Usage
//!
//! ```rust
//! use brylix::db::init_db;
//! use sea_orm_migration::MigratorTrait;
//!
//! // Initialize database and run migrations
//! let db = init_db::<migration::Migrator>("mysql://...").await?;
//! ```

use sea_orm::{ConnectOptions, Database, DatabaseConnection, DbErr};
use sea_orm_migration::MigratorTrait;
use std::time::Duration;

/// Default connection pool settings
const DEFAULT_MIN_CONNECTIONS: u32 = 2;
const DEFAULT_MAX_CONNECTIONS: u32 = 10;
const DEFAULT_IDLE_TIMEOUT: Duration = Duration::from_secs(60);
const DEFAULT_MAX_LIFETIME: Duration = Duration::from_secs(300);

/// Initialize a database connection and run migrations.
///
/// # Arguments
///
/// * `database_url` - The database connection URL
///
/// # Type Parameters
///
/// * `M` - The migration trait implementation
///
/// # Returns
///
/// The initialized database connection
///
/// # Errors
///
/// Returns an error if the connection or migration fails
pub async fn init_db<M: MigratorTrait>(database_url: &str) -> Result<DatabaseConnection, DbErr> {
    // Connect to database
    let db = Database::connect(database_url).await?;

    // Run migrations
    M::up(&db, None).await?;

    tracing::info!("Database migrations completed");

    Ok(db)
}

/// Initialize a database connection without running migrations.
///
/// # Arguments
///
/// * `database_url` - The database connection URL
///
/// # Returns
///
/// The database connection
pub async fn connect(database_url: &str) -> Result<DatabaseConnection, DbErr> {
    Database::connect(database_url).await
}

/// Initialize a database connection with custom pool options.
///
/// # Arguments
///
/// * `database_url` - The database connection URL
/// * `min_connections` - Minimum connections in the pool
/// * `max_connections` - Maximum connections in the pool
///
/// # Returns
///
/// The database connection with the configured pool
pub async fn connect_with_pool(
    database_url: &str,
    min_connections: u32,
    max_connections: u32,
) -> Result<DatabaseConnection, DbErr> {
    let mut opt = ConnectOptions::new(database_url);
    opt.min_connections(min_connections)
        .max_connections(max_connections)
        .idle_timeout(DEFAULT_IDLE_TIMEOUT)
        .max_lifetime(DEFAULT_MAX_LIFETIME)
        .sqlx_logging(false);

    Database::connect(opt).await
}

/// Create connection options with default pool settings.
///
/// Tuned for AWS Lambda with moderate concurrency:
/// - MIN: 2 connections for reduced cold start latency
/// - MAX: 10 connections for concurrent request handling
/// - IDLE_TIMEOUT: 60s to conserve resources
/// - MAX_LIFETIME: 5 min to prevent stale connections
pub fn default_connect_options(database_url: &str) -> ConnectOptions {
    let mut opt = ConnectOptions::new(database_url);
    opt.min_connections(DEFAULT_MIN_CONNECTIONS)
        .max_connections(DEFAULT_MAX_CONNECTIONS)
        .idle_timeout(DEFAULT_IDLE_TIMEOUT)
        .max_lifetime(DEFAULT_MAX_LIFETIME)
        .sqlx_logging(false);
    opt
}

/// Configuration for database connection pools.
#[derive(Debug, Clone)]
pub struct PoolConfig {
    pub min_connections: u32,
    pub max_connections: u32,
    pub idle_timeout: Duration,
    pub max_lifetime: Duration,
    pub sqlx_logging: bool,
}

impl Default for PoolConfig {
    fn default() -> Self {
        Self {
            min_connections: DEFAULT_MIN_CONNECTIONS,
            max_connections: DEFAULT_MAX_CONNECTIONS,
            idle_timeout: DEFAULT_IDLE_TIMEOUT,
            max_lifetime: DEFAULT_MAX_LIFETIME,
            sqlx_logging: false,
        }
    }
}

impl PoolConfig {
    /// Create a new pool config with custom settings
    pub fn new(min: u32, max: u32) -> Self {
        Self {
            min_connections: min,
            max_connections: max,
            ..Default::default()
        }
    }

    /// Convert to SeaORM connect options
    pub fn to_connect_options(&self, database_url: &str) -> ConnectOptions {
        let mut opt = ConnectOptions::new(database_url);
        opt.min_connections(self.min_connections)
            .max_connections(self.max_connections)
            .idle_timeout(self.idle_timeout)
            .max_lifetime(self.max_lifetime)
            .sqlx_logging(self.sqlx_logging);
        opt
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_pool_config_default() {
        let config = PoolConfig::default();
        assert_eq!(config.min_connections, 2);
        assert_eq!(config.max_connections, 10);
    }

    #[test]
    fn test_pool_config_custom() {
        let config = PoolConfig::new(5, 20);
        assert_eq!(config.min_connections, 5);
        assert_eq!(config.max_connections, 20);
    }
}
