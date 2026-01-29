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

## Best Practices

1. **Use strong JWT secrets** - Minimum 256 bits of entropy
2. **Short token expiry** - 24 hours or less
3. **HTTPS only** - Never transmit tokens over HTTP
4. **Validate on every request** - Don't cache authentication status
5. **Rotate secrets** - Regularly in production
