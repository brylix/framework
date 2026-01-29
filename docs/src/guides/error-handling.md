# Error Handling

Brylix provides a type-safe error system that maps domain errors to appropriate HTTP and GraphQL error codes.

## Error Types

### DomainError

Business logic errors:

```rust
use brylix::prelude::*;

pub enum DomainError {
    // Authentication
    InvalidCredentials,
    Unauthorized,
    Forbidden(String),

    // Not Found
    NotFound(String),
    UserNotFound,

    // Validation
    InvalidInput(String),
    DuplicateEntry(String),

    // Infrastructure
    DatabaseError(String),
    Internal(String),
}
```

### Using DomainError

```rust
pub async fn get_user(db: &DatabaseConnection, id: i64) -> DomainResult<user::Model> {
    UserRepository::find_by_id(db, id)
        .await?
        .ok_or(DomainError::NotFound("User not found".into()))
}

pub async fn create_user(db: &DatabaseConnection, email: String) -> DomainResult<user::Model> {
    // Check for duplicate
    if UserRepository::find_by_email(db, &email).await?.is_some() {
        return Err(DomainError::DuplicateEntry("Email already exists".into()));
    }
    // ...
}
```

## GraphQL Error Mapping

Domain errors automatically map to GraphQL error codes:

| DomainError | GraphQL Code |
|-------------|--------------|
| `InvalidCredentials` | `UNAUTHORIZED` |
| `Unauthorized` | `UNAUTHORIZED` |
| `Forbidden` | `FORBIDDEN` |
| `NotFound` | `NOT_FOUND` |
| `InvalidInput` | `BAD_REQUEST` |
| `DuplicateEntry` | `CONFLICT` |
| `DatabaseError` | `INTERNAL_SERVER_ERROR` |
| `Internal` | `INTERNAL_SERVER_ERROR` |

### In Resolvers

```rust
use brylix::prelude::*;

#[Object]
impl QueryRoot {
    async fn user(&self, ctx: &Context<'_>, id: i64) -> Result<UserDto> {
        let data = ctx.data_unchecked::<ContextData>();

        let user = UserService::get_by_id(&data.db, id)
            .await
            .map_err(gql_from_domain)?;  // Converts DomainError to GraphQL error

        Ok(UserDto::from(user))
    }
}
```

## GraphQL Error Helpers

```rust
use brylix::prelude::*;

// Custom error with code
return Err(gql_error("CUSTOM_CODE", "Error message"));

// Bad request (validation error)
return Err(gql_bad_request("Invalid email format"));

// Not found
return Err(gql_not_found("User not found"));

// Unauthorized
return Err(gql_unauthorized());
```

## Error Response Format

GraphQL errors include:
- `message` - Human-readable error message
- `extensions.code` - Machine-readable error code

```json
{
  "errors": [
    {
      "message": "User not found",
      "locations": [{"line": 2, "column": 3}],
      "path": ["user"],
      "extensions": {
        "code": "NOT_FOUND"
      }
    }
  ]
}
```

## Validation Errors

```rust
use brylix::validation;

pub async fn create_user(email: String, password: String) -> DomainResult<user::Model> {
    // These throw InvalidInput errors with descriptive messages
    validation::validate_email(&email)?;
    validation::validate_password(&password)?;
    // ...
}
```

## Database Error Handling

SeaORM errors auto-convert to DomainError:

```rust
pub async fn find_by_id(db: &DatabaseConnection, id: i64) -> DomainResult<Option<user::Model>> {
    Entity::find_by_id(id)
        .one(db)
        .await
        .map_err(|e| DomainError::DatabaseError(e.to_string()))
}
```

## Avoid Panics

**Never use `.unwrap()` or `.expect()` in production code.**

```rust
// BAD - Will panic
let user = find_user(id).await.unwrap();

// GOOD - Returns error
let user = find_user(id).await?.ok_or(DomainError::NotFound("User not found".into()))?;
```

## Custom Error Types

Extend for your domain:

```rust
#[derive(Debug, thiserror::Error)]
pub enum AppError {
    #[error(transparent)]
    Domain(#[from] DomainError),

    #[error("Payment failed: {0}")]
    PaymentFailed(String),

    #[error("Rate limited")]
    RateLimited,
}

impl From<AppError> for DomainError {
    fn from(e: AppError) -> Self {
        match e {
            AppError::Domain(d) => d,
            AppError::PaymentFailed(msg) => DomainError::Internal(msg),
            AppError::RateLimited => DomainError::Forbidden("Rate limited".into()),
        }
    }
}
```

## Best Practices

1. **Use DomainError** - Not anyhow or custom errors in services
2. **Be specific** - `UserNotFound` vs generic `NotFound`
3. **Include context** - `DomainError::NotFound(format!("User {} not found", id))`
4. **Don't expose internals** - Hide database error details from clients
5. **Log infrastructure errors** - For debugging, not user display
