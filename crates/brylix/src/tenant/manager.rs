//! TenantManager implementation for multi-tenant connection management.

use super::pool::{TenantError, TenantInfo};
use crate::config::Config;
use crate::db::PoolConfig;
use sea_orm::{ConnectOptions, ConnectionTrait, Database, DatabaseConnection, DbErr, Statement};
use std::collections::HashMap;
use std::sync::Arc;
use std::time::{Duration, Instant};
use tokio::sync::RwLock;

/// Maximum number of cached droplet pools
const MAX_CACHED_POOLS: usize = 50;

/// Tenant info cache TTL (5 minutes)
const TENANT_CACHE_TTL: Duration = Duration::from_secs(300);

/// Cache entry for TenantInfo with TTL tracking
struct TenantCacheEntry {
    info: TenantInfo,
    cached_at: Instant,
}

impl TenantCacheEntry {
    fn new(info: TenantInfo) -> Self {
        Self {
            info,
            cached_at: Instant::now(),
        }
    }

    fn is_expired(&self) -> bool {
        self.cached_at.elapsed() > TENANT_CACHE_TTL
    }
}

/// Droplet pool entry with LRU tracking
struct PoolEntry {
    pool: DatabaseConnection,
    last_used: Instant,
}

/// Manages tenant database connections with pool-per-droplet caching.
///
/// # Architecture
///
/// - Pools are cached by droplet_id, not by tenant name
/// - `USE {database}` switches database context per request
/// - LRU eviction when pool count exceeds `MAX_CACHED_POOLS`
pub struct TenantManager {
    /// Pools cached by droplet_id
    pools: RwLock<HashMap<i64, PoolEntry>>,

    /// Master database connection
    master_connection: RwLock<Option<DatabaseConnection>>,

    /// Tenant info cache with TTL
    tenant_cache: RwLock<HashMap<String, TenantCacheEntry>>,

    /// Pool configuration
    pool_config: PoolConfig,

    /// Callback to fetch tenant info from master database
    /// Applications must provide this during initialization
    tenant_fetcher: Option<Arc<dyn TenantFetcher>>,
}

/// Trait for fetching tenant info from the master database.
///
/// Applications must implement this to provide tenant lookup logic.
#[async_trait::async_trait]
pub trait TenantFetcher: Send + Sync {
    /// Fetch tenant info by name from the master database.
    async fn fetch_tenant(
        &self,
        master_db: &DatabaseConnection,
        tenant_name: &str,
    ) -> Result<Option<TenantInfo>, TenantError>;

    /// Build connection URL for a droplet.
    async fn build_connection_url(
        &self,
        master_db: &DatabaseConnection,
        droplet_id: i64,
    ) -> Result<String, TenantError>;
}

impl TenantManager {
    /// Create a new TenantManager
    pub fn new() -> Self {
        Self {
            pools: RwLock::new(HashMap::new()),
            master_connection: RwLock::new(None),
            tenant_cache: RwLock::new(HashMap::new()),
            pool_config: PoolConfig::default(),
            tenant_fetcher: None,
        }
    }

    /// Create with custom pool configuration
    pub fn with_pool_config(mut self, config: PoolConfig) -> Self {
        self.pool_config = config;
        self
    }

    /// Set the tenant fetcher implementation
    pub fn with_tenant_fetcher(mut self, fetcher: Arc<dyn TenantFetcher>) -> Self {
        self.tenant_fetcher = Some(fetcher);
        self
    }

    /// Get or initialize the master database connection
    pub async fn get_master_connection(&self) -> Result<DatabaseConnection, TenantError> {
        // Fast path: check if already initialized
        {
            let guard = self.master_connection.read().await;
            if let Some(conn) = guard.as_ref() {
                return Ok(conn.clone());
            }
        }

        // Slow path: initialize
        let mut guard = self.master_connection.write().await;

        // Double-check after acquiring write lock
        if let Some(conn) = guard.as_ref() {
            return Ok(conn.clone());
        }

        let config = Config::try_get()
            .ok_or_else(|| TenantError::Internal("Config not initialized".to_string()))?;

        tracing::info!("Initializing master database connection");
        let opt = self.pool_config.to_connect_options(&config.database.url());
        let db = Database::connect(opt).await.map_err(TenantError::Database)?;

        *guard = Some(db.clone());
        Ok(db)
    }

    /// Get tenant info with caching
    pub async fn get_tenant_info(&self, tenant: &str) -> Result<TenantInfo, TenantError> {
        // Fast path: check cache
        {
            let cache = self.tenant_cache.read().await;
            if let Some(entry) = cache.get(tenant) {
                if !entry.is_expired() {
                    return Ok(entry.info.clone());
                }
            }
        }

        // Slow path: fetch from database
        let master_db = self.get_master_connection().await?;

        let fetcher = self
            .tenant_fetcher
            .as_ref()
            .ok_or_else(|| TenantError::Internal("Tenant fetcher not configured".to_string()))?;

        let info = fetcher
            .fetch_tenant(&master_db, tenant)
            .await?
            .ok_or_else(|| TenantError::NotFound(tenant.to_string()))?;

        // Cache the result
        {
            let mut cache = self.tenant_cache.write().await;
            cache.insert(tenant.to_string(), TenantCacheEntry::new(info.clone()));
        }

        Ok(info)
    }

    /// Validate tenant exists in master database
    pub async fn validate_tenant_in_master(&self, tenant: &str) -> Result<(), TenantError> {
        self.get_tenant_info(tenant).await?;
        Ok(())
    }

    /// Get database connection for a tenant
    pub async fn get_connection(
        &self,
        tenant: &str,
    ) -> Result<(DatabaseConnection, TenantInfo), TenantError> {
        let tenant_info = self.get_tenant_info(tenant).await?;
        let droplet_id = tenant_info
            .droplet_id
            .ok_or(TenantError::NoDropletAssigned)?;

        let pool = self.get_droplet_pool(droplet_id).await?;
        self.switch_database(&pool, tenant).await?;

        Ok((pool, tenant_info))
    }

    /// Get or create a connection pool for a droplet
    async fn get_droplet_pool(&self, droplet_id: i64) -> Result<DatabaseConnection, TenantError> {
        // Fast path: check cache
        {
            let mut guard = self.pools.write().await;
            if let Some(entry) = guard.get_mut(&droplet_id) {
                entry.last_used = Instant::now();
                return Ok(entry.pool.clone());
            }
        }

        // Slow path: create new pool
        let master_db = self.get_master_connection().await?;

        let fetcher = self
            .tenant_fetcher
            .as_ref()
            .ok_or_else(|| TenantError::Internal("Tenant fetcher not configured".to_string()))?;

        let url = fetcher.build_connection_url(&master_db, droplet_id).await?;

        tracing::info!("Creating pool for droplet {}", droplet_id);
        let opt = self.pool_config.to_connect_options(&url);
        let pool = Database::connect(opt).await.map_err(TenantError::Database)?;

        // Add to cache
        let mut guard = self.pools.write().await;

        // Evict oldest if at capacity
        if guard.len() >= MAX_CACHED_POOLS {
            self.evict_oldest_pool(&mut guard);
        }

        guard.insert(
            droplet_id,
            PoolEntry {
                pool: pool.clone(),
                last_used: Instant::now(),
            },
        );

        Ok(pool)
    }

    /// Switch connection to use specific tenant database
    async fn switch_database(
        &self,
        pool: &DatabaseConnection,
        tenant: &str,
    ) -> Result<(), TenantError> {
        // Validate tenant name
        if crate::validation::validate_tenant_name(tenant).is_err() {
            return Err(TenantError::InvalidName(tenant.to_string()));
        }

        // Safe: tenant name validated
        let sql = format!("USE `{}`", tenant);
        pool.execute(Statement::from_string(
            sea_orm::DatabaseBackend::MySql,
            sql,
        ))
        .await
        .map_err(TenantError::Database)?;

        Ok(())
    }

    /// Evict the oldest pool from cache
    fn evict_oldest_pool(&self, cache: &mut HashMap<i64, PoolEntry>) {
        if let Some((oldest_id, _)) = cache
            .iter()
            .min_by_key(|(_, entry)| entry.last_used)
            .map(|(k, v)| (*k, v.last_used))
        {
            tracing::debug!("Evicting pool for droplet: {}", oldest_id);
            cache.remove(&oldest_id);
        }
    }

    /// Invalidate tenant cache for a specific tenant
    pub async fn invalidate_tenant_cache(&self, tenant: &str) {
        let mut cache = self.tenant_cache.write().await;
        cache.remove(tenant);
    }

    /// Clear all cached tenant info
    pub async fn clear_tenant_cache(&self) {
        let mut cache = self.tenant_cache.write().await;
        cache.clear();
    }
}

impl Default for TenantManager {
    fn default() -> Self {
        Self::new()
    }
}

/// Global tenant manager instance
static TENANT_MANAGER: tokio::sync::OnceCell<Arc<TenantManager>> = tokio::sync::OnceCell::const_new();

/// Get or initialize the global tenant manager
pub async fn get_tenant_manager() -> &'static Arc<TenantManager> {
    TENANT_MANAGER
        .get_or_init(|| async { Arc::new(TenantManager::new()) })
        .await
}

/// Initialize the global tenant manager with custom configuration
pub async fn init_tenant_manager(manager: TenantManager) -> &'static Arc<TenantManager> {
    TENANT_MANAGER
        .get_or_init(|| async { Arc::new(manager) })
        .await
}
