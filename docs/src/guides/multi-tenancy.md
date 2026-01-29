# Multi-Tenancy

Brylix supports multi-tenant architecture where each tenant has isolated data in separate databases.

## Enabling Multi-Tenant Mode

```toml
# Cargo.toml
[dependencies]
brylix = { version = "0.1", features = ["mysql", "multi-tenant"] }
```

```env
MULTI_TENANT_MODE=true
BRYLIX_DB_PASSWORD=tenant_db_password
REQUIRED_DB_VERSION=1
```

## URL Structure

| Endpoint | Description |
|----------|-------------|
| `/api/{tenant}` | GraphQL API for specific tenant |
| `/playground/{tenant}` | GraphQL Playground for tenant |

## Architecture

### Pool-per-Droplet

Brylix uses an efficient connection pooling strategy:

- One connection pool per database server (droplet)
- Multiple tenants share a pool on the same server
- `USE {tenant_db}` switches database context per request

**Benefits:**
- Memory: O(servers) instead of O(tenants)
- 5,000 tenants on 250 servers = 250 pools (not 5,000)

### Tenant Registry

Master database tracks tenants:

```sql
CREATE TABLE tenants (
    id BIGINT PRIMARY KEY AUTO_INCREMENT,
    name VARCHAR(255) NOT NULL UNIQUE,
    droplet_id BIGINT,           -- Which server hosts this tenant
    customer_id VARCHAR(255),    -- External billing ID
    active BOOLEAN DEFAULT true,
    db_version INT DEFAULT 0,
    created_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP
);
```

## Setup

### 1. Handler Configuration

```rust
use brylix::prelude::*;
use std::sync::Arc;

#[tokio::main]
async fn main() -> Result<(), Error> {
    let config = BrylixConfig::from_env()?;

    // Master database (tenant registry)
    let master_db = brylix::db::init_database(&config).await?;

    // Tenant manager for connection pooling
    let tenant_manager = Arc::new(
        brylix::tenant::TenantManager::new(config.clone())?
    );

    let schema = build_schema();

    // Multi-tenant handler
    let handler = brylix::handler::create_multi_tenant_handler(
        schema,
        master_db,
        tenant_manager,
        &config,
    );

    run(service_fn(|req| handler(req))).await
}
```

### 2. Create Tenant Database

```sql
-- In master database
INSERT INTO tenants (name, active) VALUES ('acme', true);

-- Create tenant database
CREATE DATABASE acme;
```

### 3. First Login

On first login to `/api/acme`, migrations run automatically.

## Authentication Flow

1. User calls `login` on `/api/{tenant}`
2. Server validates tenant exists in master DB
3. Authenticates user against tenant's database
4. Returns JWT with tenant claim: `{ sub: user_id, tenant: "acme" }`
5. Subsequent requests validate token tenant matches URL tenant

```rust
// Login mutation
async fn login(&self, ctx: &Context<'_>, email: String, password: String) -> Result<LoginResponse> {
    let data = ctx.data_unchecked::<ContextData>();
    let config = ctx.data_unchecked::<BrylixConfig>();

    // Get tenant from context (extracted from URL by handler)
    let tenant = data.tenant.as_ref()
        .ok_or(gql_bad_request("Tenant required"))?;

    // Generate token with tenant claim
    let token = brylix::auth::generate_token(config, user.id, Some(tenant.clone()))?;
    // ...
}
```

## Tenant Context

Access tenant in resolvers:

```rust
let data = ctx.data_unchecked::<ContextData>();

// Tenant name (from URL path)
let tenant: Option<&String> = data.tenant.as_ref();

// Database connection (connected to tenant's database)
let db: &DatabaseConnection = &data.db;
```

## Database Operations

The handler automatically connects to the correct tenant database:

```rust
// This queries the tenant's database, not master
let users = UserService::list(&data.db).await?;
```

## Version Management

Track schema versions per tenant:

```env
REQUIRED_DB_VERSION=2
```

Tenants with outdated schemas:
```rust
// Response extension shows update available
{
  "extensions": {
    "update_available": true
  }
}
```

Trigger migration:
```graphql
mutation {
  triggerMigration {
    success
    newVersion
  }
}
```

## Tenant Management

### List Tenants (Master DB)

```graphql
query {
  listTenants {
    id
    name
    active
    dbVersion
  }
}
```

### Create Tenant

```graphql
mutation {
  createTenant(name: "newclient", customerId: "cust_123") {
    id
    name
  }
}
```

### Deactivate Tenant

```graphql
mutation {
  updateTenant(id: 1, active: false) {
    id
    active
  }
}
```

## Best Practices

1. **Use master DB for registry only** - Never store tenant data in master
2. **Validate tenant on every request** - Handler does this automatically
3. **Isolate completely** - No cross-tenant queries
4. **Monitor pool usage** - Watch for connection exhaustion
5. **Backup per-tenant** - Individual tenant restoration capability
