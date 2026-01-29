# Multi-Tenant Task SaaS Example

A multi-tenant GraphQL API using the Brylix framework with tenant isolation and JWT authentication.

## Features

- Multi-tenant architecture with isolated databases
- JWT authentication with tenant claims
- Task management with assignments
- User registration and login per tenant
- Pool-per-droplet connection optimization

## Project Structure

```
src/
├── main.rs           # Entry point with migrations
├── model/            # SeaORM entities
│   ├── user.rs
│   └── task.rs
├── repository/       # Database operations
│   ├── user_repository.rs
│   └── task_repository.rs
├── service/          # Business logic
│   ├── auth_service.rs
│   └── task_service.rs
└── graphql/          # GraphQL schema
    ├── schema.rs     # Query & Mutation resolvers
    └── types.rs      # DTOs
```

## URL Structure

| Endpoint | Description |
|----------|-------------|
| `/api/{tenant}` | GraphQL API for specific tenant |
| `/playground/{tenant}` | GraphQL Playground for tenant |

## Quick Start

1. **Setup master database:**
   ```bash
   mysql -u root -p -e "CREATE DATABASE brylix_master;"
   ```

2. **Create tenant databases:**
   ```bash
   mysql -u root -p -e "CREATE DATABASE tenant_acme;"
   mysql -u root -p -e "CREATE DATABASE tenant_globex;"
   ```

3. **Register tenants in master database:**
   ```sql
   INSERT INTO tenants (name, active, db_version)
   VALUES ('tenant_acme', true, 0);
   INSERT INTO tenants (name, active, db_version)
   VALUES ('tenant_globex', true, 0);
   ```

4. **Configure environment:**
   ```bash
   cp .env.example .env
   # Edit .env with your credentials
   ```

5. **Run locally:**
   ```bash
   cargo lambda watch --env-file .env
   ```

6. **Open playground:**
   http://localhost:9000/playground/tenant_acme

## Example Operations

### Register a user (no auth required)
```graphql
mutation {
  register(email: "user@example.com", password: "SecurePass123!", name: "John Doe") {
    id
    email
    name
  }
}
```

### Login
```graphql
mutation {
  login(email: "user@example.com", password: "SecurePass123!") {
    token
    user {
      id
      name
    }
  }
}
```

### Create a task (requires auth)
```graphql
mutation {
  createTask(title: "Review PR", description: "Review feature branch", dueDate: "2024-12-31") {
    id
    title
    status
  }
}
```

### Get my tasks
```graphql
query {
  myTasks {
    id
    title
    status
    dueDate
  }
}
```

### Complete a task
```graphql
mutation {
  completeTask(id: 1) {
    id
    status
  }
}
```

## Authentication Flow

1. User calls `login` mutation on `/api/{tenant}` endpoint
2. Server validates credentials against tenant's database
3. JWT token is returned with tenant claim: `{ sub: user_id, tenant: "tenant_name" }`
4. User includes token in `Authorization: Bearer {token}` header
5. Server verifies JWT tenant matches URL tenant for all subsequent requests

## Connection Pooling

This example uses the pool-per-droplet architecture:
- One connection pool per database host (droplet)
- `USE {tenant_db}` switches database context per request
- Efficient memory usage: O(database_hosts) instead of O(tenants)

## Deploy to AWS Lambda

```bash
cargo lambda build --release --arm64
cargo lambda deploy
```
