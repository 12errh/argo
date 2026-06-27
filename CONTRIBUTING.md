# Contributing to Argo

Thank you for your interest in contributing to Argo! This document provides guidelines and instructions for contributing.

## Code of Conduct

Be respectful, inclusive, and constructive. We are building something important together.

## How to Contribute

### Reporting Bugs

1. Check existing issues to avoid duplicates
2. Open a new issue using the Bug Report template
3. Include: description, reproduction steps, environment, expected vs actual behavior

### Suggesting Features

1. Check existing issues and RFCs
2. Open a new issue using the Feature Request template
3. For major features, consider writing an RFC first

### Submitting Code

1. Fork the repository
2. Create a feature branch from `main`
3. Make your changes following the coding standards
4. Add or update tests
5. Ensure all tests pass
6. Submit a pull request

## Development Setup

### Prerequisites

- Rust 1.75+ (stable)
- Docker and Docker Compose
- Git

### Setup

```bash
# Clone the repository
git clone https://github.com/argo-agents/argo.git
cd argo

# Start development services (Redis, SurrealDB, Qdrant)
docker compose up -d

# Build the project
cargo build

# Run tests
cargo test

# Check formatting and linting
cargo fmt --all -- --check
cargo clippy --all-targets --all-features
```

## Coding Standards

### Rust

- Follow `rustfmt` formatting (run `cargo fmt`)
- No `clippy` warnings (run `cargo clippy`)
- Use `thiserror` for error types, `anyhow` for error propagation
- Use `async-trait` for async trait definitions
- Prefer `serde` for serialization
- Use `uuid::Uuid` for identifiers
- Use `chrono` for timestamps

### Documentation

- All public APIs must have doc comments
- RFCs must follow the template in `docs/rfcs/`
- Use clear, concise language
- Include examples where helpful

### Testing

- Unit tests in the same file or `tests/` module
- Integration tests in `tests/` directory
- Use `tokio-test` for async tests
- Mock external services (LLM providers, databases) in unit tests

## Commit Messages

Follow Conventional Commits:

```
type(scope): description

[optional body]

[optional footer]
```

Types:
- `feat`: New feature
- `fix`: Bug fix
- `docs`: Documentation changes
- `style`: Formatting changes
- `refactor`: Code restructuring
- `test`: Adding or updating tests
- `chore`: Maintenance tasks

Examples:
```
feat(core): add agent actor implementation
fix(memory): handle Redis connection timeout
docs(rfc): add A-01 actor engine design
```

## Pull Request Process

1. Fill out the PR template completely
2. Link related issues
3. Ensure CI passes
4. Request review from relevant CODEOWNERS
5. Address review feedback
6. Squash and merge when approved

## RFC Process

For major changes:

1. Copy the RFC template
2. Fill out all sections
3. Submit as a draft PR
4. Discuss with the team
5. Iterate until approved
6. Merge the RFC
7. Implement the approved design

## Questions?

Open a discussion in GitHub Discussions or ask in Discord.
