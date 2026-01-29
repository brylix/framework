# Contributing to Brylix

Thank you for your interest in contributing to Brylix! This document provides guidelines for contributing to the project.

## Code of Conduct

Please be respectful and constructive in all interactions. We welcome contributors of all skill levels.

## Getting Started

1. **Fork the repository** on GitHub
2. **Clone your fork:**
   ```bash
   git clone https://github.com/YOUR-USERNAME/brylix-framework.git
   cd brylix-framework
   ```
3. **Set up development environment:**
   ```bash
   # Install Rust (if not already installed)
   curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh

   # Install cargo-lambda for local testing
   cargo install cargo-lambda

   # Copy environment example
   cp .env.example .env
   ```

## Development Workflow

### Building

```bash
# Build all workspace members
cargo build

# Build specific crate
cargo build -p brylix
cargo build -p brylix-cli
```

### Testing

```bash
# Run all tests
cargo test

# Run tests for specific crate
cargo test -p brylix

# Run with output
cargo test -- --nocapture
```

### Linting

```bash
# Run clippy
cargo clippy --all-targets --all-features

# Format code
cargo fmt

# Check formatting
cargo fmt -- --check
```

## Project Structure

```
brylix-framework/
├── crates/
│   ├── brylix/          # Core framework library
│   └── brylix-cli/      # CLI tool
├── examples/
│   ├── basic/           # Single-tenant example
│   └── multi-tenant/    # Multi-tenant example
├── templates/           # Code generation templates
└── docs/                # Documentation
```

## Submitting Changes

### Pull Request Process

1. **Create a branch** for your feature or fix:
   ```bash
   git checkout -b feature/my-feature
   # or
   git checkout -b fix/my-fix
   ```

2. **Make your changes** and ensure:
   - Code compiles without errors
   - All tests pass
   - Code follows the existing style
   - No new clippy warnings

3. **Write meaningful commit messages:**
   ```
   feat: add support for PostgreSQL connection pooling

   - Add pg_pool module with connection manager
   - Update config to accept postgres feature flag
   - Add tests for pool lifecycle
   ```

4. **Push your branch** and create a Pull Request

5. **Respond to review feedback** promptly

### Commit Message Convention

We follow conventional commits:

- `feat:` - New feature
- `fix:` - Bug fix
- `docs:` - Documentation changes
- `refactor:` - Code refactoring
- `test:` - Test additions or fixes
- `chore:` - Build, CI, or tooling changes

### Code Style

- Follow Rust idioms and best practices
- Use `rustfmt` for formatting
- Address all clippy warnings
- Write documentation for public APIs
- Include tests for new functionality

## Areas for Contribution

### Good First Issues

Look for issues labeled `good first issue` for beginner-friendly tasks.

### Feature Requests

- GraphQL subscription support
- Additional database adapters
- CLI improvements
- Documentation improvements

### Bug Fixes

- Check existing issues for reported bugs
- Include reproduction steps in bug reports

## Questions?

- Open a GitHub issue for questions
- Tag with `question` label

## License

By contributing, you agree that your contributions will be licensed under the same license as the project (MIT OR Apache-2.0).
