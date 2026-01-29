//! Configuration management for Brylix applications.
//!
//! Provides a builder pattern for configuring the framework and loading
//! configuration from environment variables.
//!
//! # Usage
//!
//! ```rust
//! use brylix::config::Config;
//!
//! // Initialize from environment variables
//! let config = Config::init()?;
//!
//! // Or use the builder pattern
//! let config = ConfigBuilder::new()
//!     .database_host("localhost")
//!     .database_user("root")
//!     .database_password("secret")
//!     .database_name("mydb")
//!     .jwt_secret("my-secret")
//!     .jwt_exp_days(7)
//!     .build()?;
//! ```

mod builder;

pub use builder::ConfigBuilder;

use std::env;
use std::sync::OnceLock;

/// Global configuration singleton
static CONFIG: OnceLock<Config> = OnceLock::new();

/// Database configuration
#[derive(Debug, Clone)]
pub struct DatabaseConfig {
    pub host: String,
    pub user: String,
    pub password: String,
    pub name: String,
    pub port: u16,
}

impl DatabaseConfig {
    /// Build MySQL connection URL
    pub fn url(&self) -> String {
        format!(
            "mysql://{}:{}@{}:{}/{}",
            self.user, self.password, self.host, self.port, self.name
        )
    }

    /// Build PostgreSQL connection URL
    pub fn postgres_url(&self) -> String {
        format!(
            "postgres://{}:{}@{}:{}/{}",
            self.user, self.password, self.host, self.port, self.name
        )
    }

    /// Build connection URL for a specific tenant database
    pub fn url_for_tenant(&self, tenant: &str) -> String {
        format!(
            "mysql://{}:{}@{}:{}/{}",
            self.user, self.password, self.host, self.port, tenant
        )
    }
}

impl Default for DatabaseConfig {
    fn default() -> Self {
        Self {
            host: "localhost".to_string(),
            user: "root".to_string(),
            password: String::new(),
            name: "brylix".to_string(),
            port: 3306,
        }
    }
}

/// Multi-tenant configuration
#[derive(Debug, Clone)]
pub struct MultiTenantConfig {
    /// Whether multi-tenant mode is enabled
    pub enabled: bool,
    /// Required database schema version (increment on breaking changes)
    pub required_db_version: i32,
    /// Password for connecting to tenant databases
    pub db_password: Option<String>,
}

impl Default for MultiTenantConfig {
    fn default() -> Self {
        Self {
            enabled: false,
            required_db_version: 1,
            db_password: None,
        }
    }
}

/// JWT authentication configuration
#[derive(Debug, Clone)]
pub struct JwtConfig {
    pub secret: String,
    pub exp_days: i64,
}

impl Default for JwtConfig {
    fn default() -> Self {
        Self {
            secret: String::new(),
            exp_days: 7,
        }
    }
}

/// Main application configuration
#[derive(Debug, Clone)]
pub struct Config {
    pub database: DatabaseConfig,
    pub jwt: JwtConfig,
    pub multi_tenant: MultiTenantConfig,
    pub log_level: String,
}

impl Config {
    /// Load configuration from environment variables
    pub fn from_env() -> Result<Self, String> {
        // Database config (required)
        let database = DatabaseConfig {
            host: env::var("DB_HOST").map_err(|_| "DB_HOST must be set")?,
            user: env::var("DB_USER").map_err(|_| "DB_USER must be set")?,
            password: env::var("DB_PASSWORD").map_err(|_| "DB_PASSWORD must be set")?,
            name: env::var("DB_NAME").map_err(|_| "DB_NAME must be set")?,
            port: env::var("DB_PORT")
                .ok()
                .and_then(|s| s.parse().ok())
                .unwrap_or(3306),
        };

        // JWT config (required)
        let jwt = JwtConfig {
            secret: env::var("JWT_SECRET").map_err(|_| "JWT_SECRET must be set")?,
            exp_days: env::var("JWT_EXP_DAYS")
                .map_err(|_| "JWT_EXP_DAYS must be set")?
                .parse()
                .map_err(|_| "JWT_EXP_DAYS must be a valid integer")?,
        };

        // Multi-tenant config
        let multi_tenant = MultiTenantConfig {
            enabled: env::var("MULTI_TENANT_MODE")
                .map(|v| v == "true")
                .unwrap_or(false),
            required_db_version: env::var("REQUIRED_DB_VERSION")
                .ok()
                .and_then(|v| v.parse().ok())
                .unwrap_or(1),
            db_password: env::var("TENANT_DB_PASSWORD").ok(),
        };

        // Logging
        let log_level = env::var("RUST_LOG").unwrap_or_else(|_| "info".to_string());

        Ok(Config {
            database,
            jwt,
            multi_tenant,
            log_level,
        })
    }

    /// Initialize the global config singleton
    pub fn init() -> Result<&'static Config, String> {
        // Check if already initialized
        if let Some(config) = CONFIG.get() {
            return Ok(config);
        }

        // Try to initialize
        tracing::info!("Loading application configuration");
        let config = Config::from_env()?;

        // Try to set it (may fail if another thread beat us, which is fine)
        if CONFIG.set(config).is_err() {
            tracing::debug!("Config already initialized by another thread (race condition)");
        }

        // Return the value (either ours or the one another thread set)
        CONFIG
            .get()
            .ok_or_else(|| "Failed to initialize config".to_string())
    }

    /// Initialize with a custom config (useful for testing)
    pub fn init_with(config: Config) -> Result<&'static Config, String> {
        if CONFIG.set(config).is_err() {
            tracing::debug!("Config already initialized");
        }
        CONFIG
            .get()
            .ok_or_else(|| "Failed to initialize config".to_string())
    }

    /// Get the global config.
    ///
    /// # Panics
    /// Panics if `Config::init()` has not been called.
    pub fn get() -> &'static Config {
        CONFIG
            .get()
            .expect("Config not initialized. Call Config::init() first.")
    }

    /// Try to get the global config (returns None if not initialized)
    pub fn try_get() -> Option<&'static Config> {
        CONFIG.get()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_database_url() {
        let db = DatabaseConfig {
            host: "localhost".to_string(),
            user: "root".to_string(),
            password: "secret".to_string(),
            name: "testdb".to_string(),
            port: 3306,
        };
        assert_eq!(db.url(), "mysql://root:secret@localhost:3306/testdb");
    }

    #[test]
    fn test_postgres_url() {
        let db = DatabaseConfig {
            host: "localhost".to_string(),
            user: "root".to_string(),
            password: "secret".to_string(),
            name: "testdb".to_string(),
            port: 5432,
        };
        assert_eq!(
            db.postgres_url(),
            "postgres://root:secret@localhost:5432/testdb"
        );
    }
}
