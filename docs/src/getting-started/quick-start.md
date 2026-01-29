# Quick Start

Create your first Brylix project in under 5 minutes.

## Create a New Project

```bash
brylix new my-api
cd my-api
```

This scaffolds a complete project with:
- GraphQL schema
- Entity, service, repository layers
- Authentication setup
- Database migrations

## Configure Environment

```bash
cp .env.example .env
```

Edit `.env` with your database credentials:

```env
DB_HOST=localhost
DB_PORT=3306
DB_USER=root
DB_PASSWORD=your_password
DB_NAME=myapp
JWT_SECRET=change-this-in-production
```

## Run Locally

```bash
brylix dev
```

This starts a local Lambda emulator at `http://localhost:9000`.

## Open GraphQL Playground

Visit [http://localhost:9000/playground](http://localhost:9000/playground) to explore your API.

## Try Your First Query

```graphql
query {
  users {
    id
    email
    name
  }
}
```

## Generate Code

Add a new entity with full CRUD:

```bash
# Generate entity
brylix generate entity Post

# Generate service layer
brylix generate service Post

# Generate repository
brylix generate repository Post

# Generate GraphQL resolver
brylix generate resolver Post

# Or generate all at once
brylix generate all Post
```

## Run Migrations

```bash
brylix migrate up
```

## Build for Production

```bash
brylix build
```

## Deploy to AWS Lambda

```bash
brylix deploy
```

## Next Steps

- Learn about [Project Structure](./project-structure.md)
- Configure your app in [Configuration Guide](../guides/configuration.md)
- Build your [GraphQL Schema](../guides/graphql.md)
