# Database & SeaORM

Brylix uses [SeaORM](https://www.sea-ql.org/SeaORM/) for type-safe database operations with MySQL and PostgreSQL.

## Configuration

### MySQL

```toml
# Cargo.toml
[dependencies]
brylix = { version = "0.1", features = ["mysql"] }
```

```env
DB_HOST=localhost
DB_PORT=3306
DB_USER=root
DB_PASSWORD=password
DB_NAME=myapp
```

### PostgreSQL

```toml
[dependencies]
brylix = { version = "0.1", features = ["postgres"] }
```

```env
DB_HOST=localhost
DB_PORT=5432
DB_USER=postgres
DB_PASSWORD=password
DB_NAME=myapp
```

## Initialization

```rust
use brylix::prelude::*;

let config = BrylixConfig::from_env()?;
let db = brylix::db::init_database(&config).await?;
```

## Entities

Define database models in `src/model/`:

```rust
// src/model/user.rs
use sea_orm::entity::prelude::*;
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, PartialEq, Eq, DeriveEntityModel, Serialize, Deserialize)]
#[sea_orm(table_name = "users")]
pub struct Model {
    #[sea_orm(primary_key)]
    pub id: i64,

    #[sea_orm(unique)]
    pub email: String,

    pub password_hash: String,

    pub name: String,

    pub created_at: chrono::NaiveDateTime,

    pub updated_at: Option<chrono::NaiveDateTime>,
}

#[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
pub enum Relation {}

impl ActiveModelBehavior for ActiveModel {}
```

## Repository Pattern

```rust
// src/repository/user_repository.rs
use brylix::prelude::*;
use sea_orm::{ActiveModelTrait, DatabaseConnection, EntityTrait, Set};
use crate::model::user::{self, ActiveModel, Entity};

pub struct UserRepository;

impl UserRepository {
    pub async fn find_by_id(
        db: &DatabaseConnection,
        id: i64,
    ) -> DomainResult<Option<user::Model>> {
        Entity::find_by_id(id)
            .one(db)
            .await
            .map_err(|e| DomainError::DatabaseError(e.to_string()))
    }

    pub async fn create(
        db: &DatabaseConnection,
        email: String,
        password_hash: String,
        name: String,
    ) -> DomainResult<user::Model> {
        let model = ActiveModel {
            email: Set(email),
            password_hash: Set(password_hash),
            name: Set(name),
            created_at: Set(chrono::Utc::now().naive_utc()),
            ..Default::default()
        };

        model
            .insert(db)
            .await
            .map_err(|e| DomainError::DatabaseError(e.to_string()))
    }

    pub async fn update(
        db: &DatabaseConnection,
        id: i64,
        name: Option<String>,
    ) -> DomainResult<user::Model> {
        let mut model = ActiveModel {
            id: Set(id),
            updated_at: Set(Some(chrono::Utc::now().naive_utc())),
            ..Default::default()
        };

        if let Some(n) = name {
            model.name = Set(n);
        }

        model
            .update(db)
            .await
            .map_err(|e| DomainError::DatabaseError(e.to_string()))
    }

    pub async fn delete(db: &DatabaseConnection, id: i64) -> DomainResult<()> {
        Entity::delete_by_id(id)
            .exec(db)
            .await
            .map_err(|e| DomainError::DatabaseError(e.to_string()))?;
        Ok(())
    }
}
```

## Migrations

Create migrations with sea-orm-cli:

```bash
# Install CLI
cargo install sea-orm-cli

# Generate migration
sea-orm-cli migrate generate create_users_table

# Run migrations
sea-orm-cli migrate up

# Rollback
sea-orm-cli migrate down
```

Migration example:

```rust
// migration/src/m20240101_000001_create_users_table.rs
use sea_orm_migration::prelude::*;

#[derive(DeriveMigrationName)]
pub struct Migration;

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .create_table(
                Table::create()
                    .table(User::Table)
                    .if_not_exists()
                    .col(
                        ColumnDef::new(User::Id)
                            .big_integer()
                            .not_null()
                            .auto_increment()
                            .primary_key(),
                    )
                    .col(ColumnDef::new(User::Email).string_len(255).not_null().unique_key())
                    .col(ColumnDef::new(User::PasswordHash).string_len(255).not_null())
                    .col(ColumnDef::new(User::Name).string_len(255).not_null())
                    .col(
                        ColumnDef::new(User::CreatedAt)
                            .timestamp()
                            .not_null()
                            .default(Expr::current_timestamp()),
                    )
                    .col(ColumnDef::new(User::UpdatedAt).timestamp().null())
                    .to_owned(),
            )
            .await
    }

    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .drop_table(Table::drop().table(User::Table).to_owned())
            .await
    }
}

#[derive(Iden)]
enum User {
    Table,
    Id,
    Email,
    PasswordHash,
    Name,
    CreatedAt,
    UpdatedAt,
}
```

## Queries

```rust
use sea_orm::{ColumnTrait, EntityTrait, QueryFilter, QueryOrder};

// Find with filter
let users = Entity::find()
    .filter(Column::Email.contains("@example.com"))
    .all(db)
    .await?;

// Order and limit
let recent = Entity::find()
    .order_by_desc(Column::CreatedAt)
    .limit(10)
    .all(db)
    .await?;

// Count
let count = Entity::find()
    .filter(Column::Active.eq(true))
    .count(db)
    .await?;
```

## Transactions

```rust
use sea_orm::TransactionTrait;

db.transaction::<_, (), DbErr>(|txn| {
    Box::pin(async move {
        // All operations use txn
        UserRepository::create(txn, email, hash, name).await?;
        ProfileRepository::create(txn, user_id, bio).await?;
        Ok(())
    })
}).await?;
```

## Best Practices

1. **Use repositories** - Don't put SeaORM code in services
2. **Handle errors** - Convert DbErr to DomainError
3. **Use transactions** - For multi-step operations
4. **Index wisely** - Add indexes for frequently queried columns
5. **Use migrations** - Don't modify schema manually
