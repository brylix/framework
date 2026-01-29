# Project Structure

A Brylix project follows a layered architecture that separates concerns and promotes maintainability.

## Directory Layout

```
my-api/
├── Cargo.toml           # Project dependencies
├── .env                  # Environment variables
├── src/
│   ├── main.rs          # Lambda entry point
│   ├── model/           # SeaORM entities
│   │   ├── mod.rs
│   │   └── user.rs
│   ├── repository/      # Database operations
│   │   ├── mod.rs
│   │   └── user_repository.rs
│   ├── service/         # Business logic
│   │   ├── mod.rs
│   │   └── user_service.rs
│   └── graphql/         # GraphQL schema
│       ├── mod.rs
│       ├── schema.rs    # Query & Mutation
│       └── types.rs     # DTOs
└── migration/           # Database migrations
    └── src/
        ├── lib.rs
        └── m20240101_000001_create_users_table.rs
```

## Layer Responsibilities

### Model Layer (`src/model/`)

SeaORM entities that map to database tables:

```rust
#[derive(Clone, Debug, DeriveEntityModel)]
#[sea_orm(table_name = "users")]
pub struct Model {
    #[sea_orm(primary_key)]
    pub id: i64,
    pub email: String,
    pub password_hash: String,
    pub created_at: NaiveDateTime,
}
```

### Repository Layer (`src/repository/`)

Database operations (CRUD). Pure data access, no business logic:

```rust
pub struct UserRepository;

impl UserRepository {
    pub async fn find_by_id(db: &DatabaseConnection, id: i64) -> DomainResult<Option<user::Model>> {
        Entity::find_by_id(id)
            .one(db)
            .await
            .map_err(|e| DomainError::DatabaseError(e.to_string()))
    }
}
```

### Service Layer (`src/service/`)

Business logic. Validation, authorization, orchestration:

```rust
pub struct UserService;

impl UserService {
    pub async fn get_by_id(db: &DatabaseConnection, id: i64) -> DomainResult<user::Model> {
        UserRepository::find_by_id(db, id)
            .await?
            .ok_or(DomainError::NotFound("User not found".into()))
    }
}
```

### GraphQL Layer (`src/graphql/`)

Schema definition, resolvers, and DTOs:

```rust
#[Object]
impl QueryRoot {
    async fn user(&self, ctx: &Context<'_>, id: i64) -> Result<UserDto> {
        let data = ctx.data_unchecked::<ContextData>();
        let user = UserService::get_by_id(&data.db, id)
            .await
            .map_err(gql_from_domain)?;
        Ok(UserDto::from(user))
    }
}
```

## Data Flow

```
Request → Handler → GraphQL Resolver → Service → Repository → Database
                           ↑
                         Model
```

1. **Handler** receives HTTP request, extracts JWT, sets up context
2. **Resolver** handles GraphQL operation
3. **Service** applies business rules and validation
4. **Repository** performs database operation
5. **Model** represents database entity

## Naming Conventions

| Layer | Pattern | Example |
|-------|---------|---------|
| Model | `snake_case` fields | `user_id`, `created_at` |
| Repository | `find_*`, `create_*`, `update_*` | `find_by_id()` |
| Service | `get_*`, `create_*`, `list_*` | `get_by_id()` |
| GraphQL DTO | `PascalCase` + `Dto` | `UserDto` |
| GraphQL field | `camelCase` | `createdAt` |

## Feature Flags

Enable features in `Cargo.toml`:

```toml
[dependencies]
brylix = { version = "0.1", features = ["mysql", "playground", "multi-tenant"] }
```

Available features:
- `mysql` - MySQL database support
- `postgres` - PostgreSQL database support
- `playground` - GraphQL Playground UI
- `multi-tenant` - Multi-tenant mode
