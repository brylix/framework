# GraphQL Schema

Brylix uses [async-graphql](https://async-graphql.github.io/async-graphql/) for building type-safe GraphQL APIs.

## Schema Structure

```rust
use async_graphql::{Schema, EmptySubscription};

pub type AppSchema = Schema<QueryRoot, MutationRoot, EmptySubscription>;

pub fn build_schema(db: DatabaseConnection) -> AppSchema {
    Schema::build(QueryRoot, MutationRoot, EmptySubscription)
        .data(ContextData { db, user_id: None, tenant: None })
        .finish()
}
```

## Query Resolvers

```rust
use async_graphql::{Context, Object, Result};
use brylix::prelude::*;

pub struct QueryRoot;

#[Object]
impl QueryRoot {
    /// Get user by ID
    async fn user(&self, ctx: &Context<'_>, id: i64) -> Result<UserDto> {
        let data = ctx.data_unchecked::<ContextData>();
        let user = UserService::get_by_id(&data.db, id)
            .await
            .map_err(gql_from_domain)?;
        Ok(UserDto::from(user))
    }

    /// List all users (requires authentication)
    async fn users(&self, ctx: &Context<'_>) -> Result<Vec<UserDto>> {
        let _ = require_auth(ctx)?;
        let data = ctx.data_unchecked::<ContextData>();

        let users = UserService::list(&data.db)
            .await
            .map_err(gql_from_domain)?;

        Ok(users.into_iter().map(UserDto::from).collect())
    }
}
```

## Mutation Resolvers

```rust
pub struct MutationRoot;

#[Object]
impl MutationRoot {
    /// Create a new user
    async fn create_user(
        &self,
        ctx: &Context<'_>,
        email: String,
        password: String,
        name: String,
    ) -> Result<UserDto> {
        let data = ctx.data_unchecked::<ContextData>();

        let user = UserService::create(&data.db, email, password, name)
            .await
            .map_err(gql_from_domain)?;

        Ok(UserDto::from(user))
    }
}
```

## Data Transfer Objects (DTOs)

DTOs transform database models to GraphQL types:

```rust
use async_graphql::SimpleObject;

#[derive(Debug, Clone, SimpleObject)]
#[graphql(name = "User")]
pub struct UserDto {
    pub id: i64,
    pub email: String,
    pub name: String,
    #[graphql(name = "createdAt")]
    pub created_at: NaiveDateTime,
}

impl From<user::Model> for UserDto {
    fn from(m: user::Model) -> Self {
        Self {
            id: m.id,
            email: m.email,
            name: m.name,
            created_at: m.created_at,
        }
    }
}
```

## Context Data

Access database and user context:

```rust
use brylix::prelude::*;

// Get database connection
let data = ctx.data_unchecked::<ContextData>();
let db = &data.db;

// Get authenticated user ID
let user_id = data.user_id; // Option<i64>

// Get tenant (in multi-tenant mode)
let tenant = &data.tenant; // Option<String>
```

## Authentication Guards

```rust
use brylix::prelude::*;

// Require authentication (returns user_id or error)
let user_id = require_auth(ctx)?;

// Check if authenticated (doesn't error)
if let Some(user_id) = data.user_id {
    // User is authenticated
}
```

## Error Handling

Convert domain errors to GraphQL errors:

```rust
// From DomainError
let user = UserService::get_by_id(&data.db, id)
    .await
    .map_err(gql_from_domain)?;

// Custom error
return Err(gql_bad_request("Invalid input"));

// Not found
return Err(gql_not_found("User not found"));

// Unauthorized
return Err(gql_unauthorized());
```

## Input Types

For complex mutations:

```rust
use async_graphql::InputObject;

#[derive(InputObject)]
pub struct CreateUserInput {
    pub email: String,
    pub password: String,
    pub name: String,
}

#[Object]
impl MutationRoot {
    async fn create_user(&self, ctx: &Context<'_>, input: CreateUserInput) -> Result<UserDto> {
        // ...
    }
}
```

## Pagination

```rust
#[derive(InputObject)]
pub struct PaginationInput {
    pub page: Option<i32>,
    pub per_page: Option<i32>,
}

#[derive(SimpleObject)]
pub struct UserConnection {
    pub nodes: Vec<UserDto>,
    pub total_count: i64,
    pub page_info: PageInfo,
}
```

## Field Naming

Use `#[graphql(name = "...")]` for camelCase:

```rust
#[derive(SimpleObject)]
pub struct UserDto {
    pub id: i64,

    #[graphql(name = "createdAt")]
    pub created_at: NaiveDateTime,

    #[graphql(name = "updatedAt")]
    pub updated_at: Option<NaiveDateTime>,
}
```

## Best Practices

1. **Keep resolvers thin** - Business logic belongs in services
2. **Use DTOs** - Don't expose database models directly
3. **Validate in services** - Not in resolvers
4. **Handle errors properly** - Use `gql_from_domain` for consistent errors
5. **Document with doc comments** - They appear in GraphQL schema
