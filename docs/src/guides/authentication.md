# Authentication

Brylix provides JWT-based authentication with optional multi-tenant support.

## Overview

1. User calls `login` mutation with credentials
2. Server validates and returns JWT token
3. Client includes token in `Authorization` header
4. Server validates token on each request

## Login Flow

### Mutation

```rust
#[Object]
impl MutationRoot {
    async fn login(
        &self,
        ctx: &Context<'_>,
        email: String,
        password: String,
    ) -> Result<LoginResponse> {
        let data = ctx.data_unchecked::<ContextData>();
        let config = ctx.data_unchecked::<BrylixConfig>();

        // Find user
        let user = UserRepository::find_by_email(&data.db, &email)
            .await?
            .ok_or(DomainError::InvalidCredentials)?;

        // Verify password
        if !brylix::auth::verify_password(&password, &user.password_hash)? {
            return Err(gql_from_domain(DomainError::InvalidCredentials));
        }

        // Generate token
        let token = brylix::auth::generate_token(config, user.id, None)?;

        Ok(LoginResponse {
            token,
            user: UserDto::from(user),
        })
    }
}
```

### Client Usage

```graphql
mutation {
  login(email: "user@example.com", password: "secret") {
    token
    user { id name }
  }
}
```

Include token in subsequent requests:

```
Authorization: Bearer eyJhbGciOiJIUzI1NiIs...
```

## Password Hashing

Use Argon2 for secure password storage:

```rust
use brylix::auth::{hash_password, verify_password};

// Hash password for storage
let hash = hash_password("user_password")?;

// Verify password
let valid = verify_password("user_password", &hash)?;
```

## Token Generation

```rust
use brylix::auth::generate_token;

// Single-tenant
let token = generate_token(&config, user_id, None)?;

// Multi-tenant (includes tenant claim)
let token = generate_token(&config, user_id, Some("tenant_name".to_string()))?;
```

## Token Validation

The handler automatically validates JWT tokens. In resolvers:

```rust
use brylix::prelude::*;

// Require authentication (errors if not authenticated)
let user_id = require_auth(ctx)?;

// Optional: check if authenticated
let data = ctx.data_unchecked::<ContextData>();
if let Some(user_id) = data.user_id {
    // Authenticated
}
```

## JWT Claims

Token payload:

```json
{
  "sub": 123,           // User ID
  "tenant": "acme",     // Tenant name (multi-tenant only)
  "exp": 1704067200,    // Expiration timestamp
  "iat": 1703980800     // Issued at timestamp
}
```

## Configuration

```env
# Secret key (use strong random value, min 256 bits)
JWT_SECRET=your-super-secret-key-change-in-production

# Token expiry in hours
JWT_EXPIRY_HOURS=24
```

## Multi-Tenant Authentication

In multi-tenant mode, tokens include a `tenant` claim:

```rust
// Login with tenant
async fn login(&self, ctx: &Context<'_>, email: String, password: String) -> Result<LoginResponse> {
    let data = ctx.data_unchecked::<ContextData>();
    let tenant = data.tenant.as_ref().ok_or(gql_bad_request("Tenant required"))?;

    // Generate token with tenant claim
    let token = generate_token(&config, user.id, Some(tenant.clone()))?;
    // ...
}
```

The handler validates that the token's tenant matches the URL tenant.

## Protected Routes

```rust
#[Object]
impl QueryRoot {
    // Public - no auth required
    async fn public_data(&self, ctx: &Context<'_>) -> Result<String> {
        Ok("Hello World".to_string())
    }

    // Protected - requires authentication
    async fn private_data(&self, ctx: &Context<'_>) -> Result<UserDto> {
        let user_id = require_auth(ctx)?;  // Throws if not authenticated
        // ...
    }
}
```

## Registration

```rust
async fn register(
    &self,
    ctx: &Context<'_>,
    email: String,
    password: String,
    name: String,
) -> Result<UserDto> {
    // Validate email
    brylix::validation::validate_email(&email)?;

    // Validate password
    brylix::validation::validate_password(&password)?;

    // Hash password
    let password_hash = brylix::auth::hash_password(&password)?;

    // Create user
    let user = UserRepository::create(&data.db, email, password_hash, name).await?;

    Ok(UserDto::from(user))
}
```

## Admin Override (POS/Kiosk Pattern)

> Requires the `admin-override` feature flag.

In POS/supermarket scenarios, a cashier (User role) is logged in but sometimes needs temporary admin authorization for privileged actions like deleting an invoice. Instead of logging out and back in as admin, the admin "taps in" their password to authorize a single action.

### How It Works

```
1. Cashier is logged in         -> Authorization: Bearer <cashier_token>
2. Admin enters password         -> App verifies, calls issue_admin_override_token()
3. Frontend sends both headers   -> Authorization + X-Admin-Override
4. require_admin(ctx) succeeds   -> Returns admin_id from override
5. require_auth_user_id(ctx)     -> Still returns cashier_id
```

### Configuration

```env
ADMIN_JWT_SECRET=your-admin-secret        # Same secret as admin role JWT
ADMIN_OVERRIDE_EXPIRY_SECS=60             # Optional, default 60 seconds
```

### Issuing Override Tokens

After verifying the admin's credentials (e.g. password), issue a short-lived token:

```rust
use brylix::prelude::*;

async fn admin_tap_in(
    ctx: &Context<'_>,
    admin_email: String,
    admin_password: String,
    action: Option<String>,
) -> Result<String> {
    let data = ctx.data_unchecked::<ContextData>();

    // Verify admin credentials
    let admin = AdminRepository::find_by_email(&data.db, &admin_email)
        .await?
        .ok_or(gql_from_domain(DomainError::InvalidCredentials))?;

    if !verify_password(&admin_password, &admin.password_hash)? {
        return Err(gql_from_domain(DomainError::InvalidCredentials));
    }

    // Issue short-lived override token
    let config = AdminOverrideConfig::new(std::env::var("ADMIN_JWT_SECRET").unwrap());
    let token = issue_admin_override_token(
        &config,
        admin.id,
        &admin.name,
        action.as_deref(),
    ).map_err(|e| gql_error("INTERNAL_ERROR", &e))?;

    Ok(token)
}
```

### Using Guards with Override

The `require_admin()` guard works for **both** scenarios automatically:

```rust
// Scenario 1: Admin logged in directly     -> returns admin_id from role
// Scenario 2: Cashier + admin override     -> returns admin_id from override
let admin_id = require_admin(ctx)?;
```

For audit trails, check who performed the action vs who authorized it:

```rust
async fn delete_invoice(ctx: &Context<'_>, id: i64) -> Result<bool> {
    let admin_id = require_admin(ctx)?;
    let user_id = require_auth_user_id(ctx)?;

    // Build audit trail
    if let Some(ao) = get_admin_override(ctx) {
        let audit = AdminOverrideAudit {
            actor_user_id: user_id,
            authorizer_admin_id: ao.admin_id,
            authorizer_name: ao.admin_name.clone(),
            action: ao.action.clone(),
        };
        audit.log();
    }

    InvoiceService::delete(&data.db, id).await.map_err(gql_from_domain)?;
    Ok(true)
}
```

### Requiring Both User + Override

To explicitly require both an authenticated user AND an admin override:

```rust
let (cashier_id, admin_override) = require_auth_with_admin_override(ctx)?;
// cashier_id: the logged-in user
// admin_override.admin_id: the admin who authorized
```

### Security Notes

- Override tokens are **short-lived** (default 60 seconds)
- Tokens include a `token_type: "admin_override"` marker, so regular admin JWTs cannot be used as overrides
- The `X-Admin-Override` header is included in CORS allowed headers
- Override tokens use the same secret as the admin role JWT (`ADMIN_JWT_SECRET`)

## Best Practices

1. **Use strong JWT secrets** - Minimum 256 bits of entropy
2. **Short token expiry** - 24 hours or less
3. **HTTPS only** - Never transmit tokens over HTTP
4. **Validate on every request** - Don't cache authentication status
5. **Rotate secrets** - Regularly in production
