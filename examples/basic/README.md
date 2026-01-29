# Basic Blog API Example

A minimal single-tenant GraphQL API using the Brylix framework.

## Features

- CRUD operations for blog posts
- GraphQL Playground UI
- SeaORM with MySQL
- AWS Lambda deployment ready

## Project Structure

```
src/
├── main.rs           # Entry point with migrations
├── model/            # SeaORM entities
│   └── post.rs
├── repository/       # Database operations
│   └── post_repository.rs
├── service/          # Business logic
│   └── post_service.rs
└── graphql/          # GraphQL schema
    ├── schema.rs     # Query & Mutation resolvers
    └── types.rs      # DTOs
```

## Quick Start

1. **Setup database:**
   ```bash
   mysql -u root -p -e "CREATE DATABASE brylix_blog;"
   ```

2. **Configure environment:**
   ```bash
   cp .env.example .env
   # Edit .env with your database credentials
   ```

3. **Run locally:**
   ```bash
   cargo lambda watch --env-file .env
   ```

4. **Open playground:**
   http://localhost:9000/playground

## Example Queries

### Create a post
```graphql
mutation {
  createPost(title: "Hello World", content: "My first post!") {
    id
    title
    createdAt
  }
}
```

### List all posts
```graphql
query {
  posts {
    id
    title
    published
    createdAt
  }
}
```

### Publish a post
```graphql
mutation {
  publishPost(id: 1) {
    id
    published
  }
}
```

## Deploy to AWS Lambda

```bash
cargo lambda build --release --arm64
cargo lambda deploy
```
