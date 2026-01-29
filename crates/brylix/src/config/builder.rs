//! Builder pattern for creating Config instances.

use super::{Config, DatabaseConfig, JwtConfig, MultiTenantConfig};

/// Builder for creating Config instances programmatically.
///
/// # Example
///
/// ```rust
/// use brylix::config::ConfigBuilder;
///
/// let config = ConfigBuilder::new()
///     .database_host("localhost")
///     .database_user("root")
///     .database_password("secret")
///     .database_name("mydb")
///     .jwt_secret("my-secret-key")
///     .jwt_exp_days(7)
///     .multi_tenant(true)
///     .required_db_version(2)
///     .build()?;
/// ```
#[derive(Debug, Default)]
pub struct ConfigBuilder {
    database: DatabaseConfig,
    jwt: JwtConfig,
    multi_tenant: MultiTenantConfig,
    log_level: String,
}

impl ConfigBuilder {
    /// Create a new ConfigBuilder with default values
    pub fn new() -> Self {
        Self {
            database: DatabaseConfig::default(),
            jwt: JwtConfig::default(),
            multi_tenant: MultiTenantConfig::default(),
            log_level: "info".to_string(),
        }
    }

    // =========================================================================
    // Database configuration
    // =========================================================================

    /// Set the database host
    pub fn database_host(mut self, host: impl Into<String>) -> Self {
        self.database.host = host.into();
        self
    }

    /// Set the database user
    pub fn database_user(mut self, user: impl Into<String>) -> Self {
        self.database.user = user.into();
        self
    }

    /// Set the database password
    pub fn database_password(mut self, password: impl Into<String>) -> Self {
        self.database.password = password.into();
        self
    }

    /// Set the database name
    pub fn database_name(mut self, name: impl Into<String>) -> Self {
        self.database.name = name.into();
        self
    }

    /// Set the database port
    pub fn database_port(mut self, port: u16) -> Self {
        self.database.port = port;
        self
    }

    /// Set the full database URL (parses host, user, password, name, port)
    pub fn database_url(mut self, url: &str) -> Self {
        // Simple URL parsing for mysql://user:pass@host:port/dbname
        if let Some(rest) = url.strip_prefix("mysql://").or(url.strip_prefix("postgres://")) {
            if let Some((auth, host_db)) = rest.split_once('@') {
                // Parse user:password
                if let Some((user, password)) = auth.split_once(':') {
                    self.database.user = user.to_string();
                    self.database.password = password.to_string();
                }

                // Parse host:port/dbname
                if let Some((host_port, dbname)) = host_db.split_once('/') {
                    self.database.name = dbname.to_string();
                    if let Some((host, port)) = host_port.split_once(':') {
                        self.database.host = host.to_string();
                        if let Ok(p) = port.parse() {
                            self.database.port = p;
                        }
                    } else {
                        self.database.host = host_port.to_string();
                    }
                }
            }
        }
        self
    }

    // =========================================================================
    // JWT configuration
    // =========================================================================

    /// Set the JWT secret
    pub fn jwt_secret(mut self, secret: impl Into<String>) -> Self {
        self.jwt.secret = secret.into();
        self
    }

    /// Set the JWT expiry in days
    pub fn jwt_exp_days(mut self, days: i64) -> Self {
        self.jwt.exp_days = days;
        self
    }

    // =========================================================================
    // Multi-tenant configuration
    // =========================================================================

    /// Enable or disable multi-tenant mode
    pub fn multi_tenant(mut self, enabled: bool) -> Self {
        self.multi_tenant.enabled = enabled;
        self
    }

    /// Set the required database version
    pub fn required_db_version(mut self, version: i32) -> Self {
        self.multi_tenant.required_db_version = version;
        self
    }

    /// Set the tenant database password
    pub fn tenant_db_password(mut self, password: impl Into<String>) -> Self {
        self.multi_tenant.db_password = Some(password.into());
        self
    }

    // =========================================================================
    // Logging configuration
    // =========================================================================

    /// Set the log level
    pub fn log_level(mut self, level: impl Into<String>) -> Self {
        self.log_level = level.into();
        self
    }

    // =========================================================================
    // Build
    // =========================================================================

    /// Build the Config instance
    ///
    /// # Errors
    /// Returns an error if required fields are missing
    pub fn build(self) -> Result<Config, String> {
        // Validate required fields
        if self.jwt.secret.is_empty() {
            return Err("JWT secret is required".to_string());
        }

        if self.database.host.is_empty() {
            return Err("Database host is required".to_string());
        }

        Ok(Config {
            database: self.database,
            jwt: self.jwt,
            multi_tenant: self.multi_tenant,
            log_level: self.log_level,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_builder() {
        let config = ConfigBuilder::new()
            .database_host("localhost")
            .database_user("root")
            .database_password("secret")
            .database_name("testdb")
            .jwt_secret("test-secret")
            .jwt_exp_days(14)
            .multi_tenant(true)
            .required_db_version(2)
            .build()
            .unwrap();

        assert_eq!(config.database.host, "localhost");
        assert_eq!(config.database.name, "testdb");
        assert_eq!(config.jwt.exp_days, 14);
        assert!(config.multi_tenant.enabled);
        assert_eq!(config.multi_tenant.required_db_version, 2);
    }

    #[test]
    fn test_database_url_parsing() {
        let config = ConfigBuilder::new()
            .database_url("mysql://myuser:mypass@myhost:3307/mydb")
            .jwt_secret("secret")
            .build()
            .unwrap();

        assert_eq!(config.database.user, "myuser");
        assert_eq!(config.database.password, "mypass");
        assert_eq!(config.database.host, "myhost");
        assert_eq!(config.database.port, 3307);
        assert_eq!(config.database.name, "mydb");
    }

    #[test]
    fn test_missing_jwt_secret() {
        let result = ConfigBuilder::new()
            .database_host("localhost")
            .build();

        assert!(result.is_err());
        assert!(result.unwrap_err().contains("JWT secret"));
    }
}
