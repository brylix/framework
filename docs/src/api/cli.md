# brylix-cli

Command-line tool for creating and managing Brylix projects.

## Installation

```bash
cargo install brylix-cli
```

## Commands

### `brylix new`

Create a new Brylix project.

```bash
brylix new <project-name>
```

**Arguments:**
- `<project-name>` - Name of the project directory

**Creates:**
- Complete project structure
- Cargo.toml with dependencies
- GraphQL schema boilerplate
- Environment example file
- Git repository (optional)

**Example:**
```bash
brylix new my-api
cd my-api
```

### `brylix generate`

Generate code from templates.

```bash
brylix generate <type> <name>
```

**Types:**

| Type | Description | Output |
|------|-------------|--------|
| `entity` | SeaORM entity | `src/model/{name}.rs` |
| `service` | Business logic | `src/service/{name}_service.rs` |
| `repository` | Database access | `src/repository/{name}_repository.rs` |
| `resolver` | GraphQL resolver | `src/graphql/{name}.rs` |
| `migration` | Database migration | `migration/src/m{timestamp}_{name}.rs` |
| `all` | All of the above | Multiple files |

**Arguments:**
- `<type>` - Type of code to generate
- `<name>` - Entity name in PascalCase (e.g., `User`, `BlogPost`)

**Examples:**
```bash
# Generate entity
brylix generate entity User

# Generate service
brylix generate service User

# Generate all for an entity
brylix generate all Post
```

**Name Conversion:**
- `User` → `src/model/user.rs`, `users` table
- `BlogPost` → `src/model/blog_post.rs`, `blog_posts` table

### `brylix dev`

Run local development server.

```bash
brylix dev [OPTIONS]
```

**Options:**
- `--port <PORT>` - Server port (default: 9000)
- `--env-file <FILE>` - Environment file (default: .env)

**Example:**
```bash
brylix dev
brylix dev --port 8080
```

Starts cargo-lambda in watch mode at `http://localhost:9000`.

### `brylix build`

Build for Lambda deployment.

```bash
brylix build [OPTIONS]
```

**Options:**
- `--release` - Release build (default: true)
- `--arm64` - Build for ARM64/Graviton (default: true)

**Example:**
```bash
brylix build
```

Output: `target/lambda/bootstrap`

### `brylix deploy`

Deploy to AWS Lambda.

```bash
brylix deploy [OPTIONS]
```

**Options:**
- `--function-name <NAME>` - Lambda function name
- `--profile <PROFILE>` - AWS profile

**Example:**
```bash
brylix deploy
brylix deploy --profile production
```

Requires AWS credentials configured.

### `brylix migrate`

Manage database migrations.

```bash
brylix migrate <subcommand>
```

**Subcommands:**

| Subcommand | Description |
|------------|-------------|
| `up` | Run pending migrations |
| `down` | Rollback last migration |
| `status` | Show migration status |
| `generate <name>` | Create new migration |

**Examples:**
```bash
brylix migrate up
brylix migrate down
brylix migrate status
brylix migrate generate add_posts_table
```

## Global Options

| Option | Description |
|--------|-------------|
| `-h, --help` | Show help |
| `-V, --version` | Show version |
| `-v, --verbose` | Verbose output |

## Configuration

The CLI reads configuration from:

1. `.env` file in current directory
2. Environment variables
3. Command-line arguments (highest priority)

## Examples

### Full Workflow

```bash
# Create project
brylix new blog-api
cd blog-api

# Configure
cp .env.example .env
# Edit .env with your database credentials

# Run locally
brylix dev

# Generate new entity
brylix generate all Post

# Run migrations
brylix migrate up

# Build and deploy
brylix build
brylix deploy
```

### Development Cycle

```bash
# Terminal 1: Run dev server
brylix dev

# Terminal 2: Generate code as needed
brylix generate service Comment
brylix generate resolver Comment

# Terminal 2: Run migrations
brylix migrate up
```
