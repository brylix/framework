//! Provider trait for external service integrations.
//!
//! This module defines the `Provider` trait that applications can implement
//! to integrate with external services like DigitalOcean, AWS, etc.
//!
//! # Usage
//!
//! ```rust
//! use brylix::provider::Provider;
//! use async_trait::async_trait;
//!
//! struct MyProvider {
//!     api_key: String,
//! }
//!
//! #[async_trait]
//! impl Provider for MyProvider {
//!     type Resource = MyResource;
//!     type Health = MyHealth;
//!
//!     async fn sync(&self, db: &DatabaseConnection) -> Result<Vec<Self::Resource>> {
//!         // Sync resources from external service
//!     }
//!
//!     async fn fetch_health(&self, resource_id: i64) -> Result<Self::Health> {
//!         // Fetch health metrics for a resource
//!     }
//! }
//! ```

use anyhow::Result;
use sea_orm::DatabaseConnection;
use serde::{Deserialize, Serialize};

/// Generic health metrics for a resource.
///
/// Applications can extend this with their own health data.
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct HealthMetrics {
    /// CPU usage percentage (0-100)
    pub cpu_percent: Option<f32>,

    /// System load average
    pub load: Option<f32>,

    /// Memory usage percentage (0-100)
    pub memory_percent: Option<f32>,

    /// Disk read speed in MB/s
    pub disk_read_mb_s: Option<f32>,

    /// Disk write speed in MB/s
    pub disk_write_mb_s: Option<f32>,

    /// Disk usage percentage (0-100)
    pub disk_usage_percent: Option<f32>,

    /// Network receive speed in MB/s
    pub net_recv_mb_s: Option<f32>,

    /// Network transmit speed in MB/s
    pub net_trans_mb_s: Option<f32>,

    /// Total bandwidth in Mbps
    pub bandwidth_mbps: Option<f32>,
}

/// Generic provider resource info.
#[derive(Debug, Clone)]
pub struct ResourceInfo {
    /// Provider's resource ID
    pub id: i64,

    /// Resource name
    pub name: String,

    /// Current status (e.g., "running", "stopped")
    pub status: String,

    /// Region/location
    pub region: Option<String>,

    /// Public IP address
    pub public_ip: Option<String>,

    /// Private IP address
    pub private_ip: Option<String>,
}

/// Trait for external service providers.
///
/// Implement this trait to integrate with cloud providers, monitoring services,
/// or other external APIs.
#[async_trait::async_trait]
pub trait Provider: Send + Sync {
    /// Sync resources from the provider and update the database.
    ///
    /// # Arguments
    ///
    /// * `db` - Database connection for storing synced data
    ///
    /// # Returns
    ///
    /// List of synced resources
    async fn sync(&self, db: &DatabaseConnection) -> Result<Vec<ResourceInfo>>;

    /// Fetch health metrics for a resource.
    ///
    /// # Arguments
    ///
    /// * `resource_id` - The provider's resource ID
    ///
    /// # Returns
    ///
    /// Health metrics for the resource
    async fn fetch_health(&self, resource_id: i64) -> Result<HealthMetrics>;

    /// Check if the provider is configured and ready to use.
    fn is_configured(&self) -> bool {
        true
    }
}

/// A no-op provider for testing or when no provider is configured.
pub struct NoOpProvider;

#[async_trait::async_trait]
impl Provider for NoOpProvider {
    async fn sync(&self, _db: &DatabaseConnection) -> Result<Vec<ResourceInfo>> {
        Ok(vec![])
    }

    async fn fetch_health(&self, _resource_id: i64) -> Result<HealthMetrics> {
        Ok(HealthMetrics::default())
    }

    fn is_configured(&self) -> bool {
        false
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_health_metrics_default() {
        let metrics = HealthMetrics::default();
        assert!(metrics.cpu_percent.is_none());
        assert!(metrics.memory_percent.is_none());
    }

    #[test]
    fn test_resource_info() {
        let info = ResourceInfo {
            id: 123,
            name: "web-server".to_string(),
            status: "running".to_string(),
            region: Some("nyc1".to_string()),
            public_ip: Some("1.2.3.4".to_string()),
            private_ip: Some("10.0.0.1".to_string()),
        };
        assert_eq!(info.id, 123);
        assert_eq!(info.name, "web-server");
    }
}
