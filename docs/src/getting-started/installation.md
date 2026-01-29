# Installation

## Prerequisites

- **Rust** (1.75 or later)
- **cargo-lambda** for local development and deployment
- **MySQL** or **PostgreSQL** database

## Install Rust

If you don't have Rust installed:

```bash
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
```

## Install cargo-lambda

cargo-lambda is required for local development and Lambda deployment:

```bash
# Using Homebrew (macOS)
brew tap cargo-lambda/cargo-lambda
brew install cargo-lambda

# Using pip
pip3 install cargo-lambda

# Using cargo
cargo install cargo-lambda
```

## Install Brylix CLI

```bash
cargo install brylix-cli
```

Or build from source:

```bash
git clone https://github.com/brylix/framework.git
cd framework
cargo install --path crates/brylix-cli
```

## Verify Installation

```bash
brylix --version
# brylix-cli 0.1.0

cargo lambda --version
# cargo-lambda 1.x.x
```

## Database Setup

### MySQL

```bash
# Install MySQL (macOS)
brew install mysql
brew services start mysql

# Create database
mysql -u root -e "CREATE DATABASE myapp;"
```

### PostgreSQL

```bash
# Install PostgreSQL (macOS)
brew install postgresql
brew services start postgresql

# Create database
createdb myapp
```

## Next Steps

Continue to [Quick Start](./quick-start.md) to create your first project.
