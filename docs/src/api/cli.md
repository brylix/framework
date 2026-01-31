# brylix-cli

Command-line tool for creating and managing Brylix projects.

## Installation

```bash
cargo install brylix-cli
```

## Getting Help

The CLI provides built-in help for all commands:

```bash
# Show all available commands and global options
brylix --help
brylix -h

# Show version
brylix --version
brylix -V

# Show help for a specific command
brylix new --help
brylix generate --help
brylix dev --help
brylix build --help
brylix deploy --help
brylix migrate --help
brylix test --help

# Show help for subcommands
brylix generate entity --help
brylix generate service --help
brylix generate all --help
```

### Example Output

```
$ brylix --help
CLI tool for the Brylix framework

Usage: brylix <COMMAND>

Commands:
  new       Create a new Brylix project
  generate  Generate code (entity, service, repository, resolver) [aliases: g]
  dev       Run development server with hot reload
  build     Build for AWS Lambda deployment
  deploy    Deploy to AWS Lambda
  migrate   Run database migrations
  test      Run tests
  help      Print this message or the help of the given subcommand(s)

Options:
  -h, --help     Print help
  -V, --version  Print version
```

## Commands

### `brylix new`

Create a new Brylix project.

```bash
brylix new <project-name> [OPTIONS]
```

**Arguments:**
- `<project-name>` - Name of the project directory

**Options:**
- `--multi-tenant` - Enable multi-tenant mode
- `--database <TYPE>` - Database type: `mysql` (default) or `postgres`

**Creates:**
- Complete project structure
- Cargo.toml with dependencies
- GraphQL schema boilerplate
- Environment example file
- Git repository (optional)

**Examples:**
```bash
# Basic project with MySQL
brylix new my-api
cd my-api

# With multi-tenant support
brylix new my-saas-api --multi-tenant

# With PostgreSQL
brylix new my-api --database postgres

# Multi-tenant with PostgreSQL
brylix new my-api --multi-tenant --database postgres
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

### `brylix test`

Run tests with various options.

```bash
brylix test [OPTIONS] [FILTER]
```

**Options:**
- `--unit` - Run only unit tests (--lib)
- `--integration` - Run only integration tests (--test)
- `-w, --watch` - Watch mode - re-run tests on file changes (requires cargo-watch)
- `--release` - Run tests in release mode
- `-v, --verbose` - Show output from passing tests (--nocapture)

**Arguments:**
- `[FILTER]` - Optional filter to run specific tests matching this string

**Examples:**
```bash
# Run all tests
brylix test

# Run only unit tests
brylix test --unit

# Run only integration tests
brylix test --integration

# Watch mode (re-runs on file changes)
brylix test --watch

# Run specific tests matching a pattern
brylix test user

# Verbose output
brylix test --verbose

# Combine options
brylix test --unit --watch --verbose
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
