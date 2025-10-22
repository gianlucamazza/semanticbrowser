# Contributing to Semantic Browser

Thank you for your interest in contributing to Semantic Browser! This document provides guidelines and instructions for contributing to the project.

## Table of Contents

1. [Getting Started](#getting-started)
2. [Development Workflow](#development-workflow)
3. [Code Style & Quality](#code-style--quality)
4. [Testing](#testing)
5. [Commit Messages](#commit-messages)
6. [Pull Request Process](#pull-request-process)
7. [Code of Conduct](#code-of-conduct)

---

## Getting Started

### Prerequisites

- **Rust**: Install from [rustup.rs](https://www.rustup.rs/)
- **Git**: Version control system
- **Cargo**: Rust package manager (comes with Rust)

### Development Setup (5 minutes)

```bash
# 1. Clone the repository
git clone https://github.com/gianlucamazza/semanticbrowser.git
cd semanticbrowser

# 2. Copy environment template
cp .env.example .env

# 3. Install Rust toolchain (if not already done)
rustup update stable
rustup component add rustfmt clippy

# 4. Build the project
cargo build

# 5. Run tests to verify setup
cargo test --lib

# 6. Run an example
cargo run --example agent_simple_task
```

### Install Pre-commit Hooks (Recommended)

Pre-commit hooks automatically check formatting and run clippy before each commit:

```bash
# Install pre-commit framework
pip install pre-commit

# Install hooks from configuration
pre-commit install

# Test hooks are working
pre-commit run --all-files
```

---

## Development Workflow

### 1. Create a Feature Branch

```bash
# Create and checkout new branch
git checkout -b feature/my-feature-name

# Or for bug fixes
git checkout -b fix/bug-description

# Or for documentation
git checkout -b docs/doc-title
```

### 2. Make Changes

- Keep changes focused and atomic
- One feature/fix per branch
- Write tests for new functionality

### 3. Verify Your Changes

```bash
# Format code
cargo fmt

# Run linting with strict warnings
cargo clippy --all-features -- -D warnings

# Run tests
cargo test --all-features

# Build in release mode
cargo build --release
```

### 4. Commit Your Changes

Follow conventional commits format:

```bash
# Format: <type>(<scope>): <subject>
git commit -m "feat(llm): add streaming support for OpenAI"
```

**Types:**
- `feat`: New feature
- `fix`: Bug fix
- `docs`: Documentation
- `test`: Adding or updating tests
- `refactor`: Code restructuring
- `perf`: Performance improvement
- `chore`: Build system, dependencies, etc.

### 5. Push and Create Pull Request

```bash
# Push feature branch to remote
git push origin feature/my-feature-name

# Create PR on GitHub
# Set description, link related issues, request reviewers
```

---

## Code Style & Quality

### Style Guide

Follow the [BEST_PRACTICES.md](BEST_PRACTICES.md) document for:
- Module organization
- Naming conventions
- Documentation requirements
- Error handling patterns

### Required Checks Before Submission

All of these must pass before submitting a PR:

```bash
# 1. Code formatting
cargo fmt --check

# 2. Linting (with strict warnings)
cargo clippy --all-features -- -D warnings

# 3. All tests pass
cargo test --all-features

# 4. Build succeeds
cargo build --all-features

# 5. Documentation compiles
cargo doc --no-deps
```

### Pre-commit Hook Checklist

The `.pre-commit-config.yaml` automatically checks:
- ✅ Code formatting with `cargo fmt`
- ✅ Linting with `cargo clippy`
- ✅ Unit tests pass
- ✅ No compiler warnings

---

## Testing

### Test Organization

```
src/module/
├── lib.rs
└── tests.rs        # Unit tests

tests/
├── integration_tests.rs  # Integration tests
└── fixtures/             # Test data
```

### Test Naming Convention

```rust
#[test]
fn test_happy_path() { }           // Success case

#[test]
fn test_error_handling() { }       // Error case

#[test]
fn test_edge_case_empty_input() { } // Edge case

#[tokio::test]
async fn test_async_operation() { } // Async test
```

### Running Tests

```bash
# All tests
cargo test --all-features

# Specific module
cargo test --lib module_name

# With output
cargo test --lib -- --nocapture

# Integration tests only
cargo test --test integration_tests

# Single test
cargo test test_name -- --exact
```

### Coverage Requirements

- **Core logic**: 90%+
- **Public API**: 85%+
- **Overall**: 80%+

Check coverage locally:

```bash
cargo tarpaulin --all-features --timeout 300
```

---

## Commit Messages

### Format

Use conventional commits:

```
<type>(<scope>): <subject>

<body>

<footer>
```

### Examples

**Feature:**
```
feat(llm): add streaming support for OpenAI

Implement server-sent events handling for real-time token streaming.
Includes proper error handling and reconnection logic.

Closes #123
```

**Bug Fix:**
```
fix(workflow): resolve tool_call variable reference

The tool_call parameter was prefixed with underscore but still referenced.
Updated parameter naming to match usage.

Fixes #456
```

**Documentation:**
```
docs(setup): add ML integration guide

Created comprehensive guide covering Ollama, OpenAI, and Anthropic setup.
Includes troubleshooting and configuration examples.
```

---

## Pull Request Process

### Before Submitting

- [ ] Tests pass: `cargo test --all-features`
- [ ] Clippy clean: `cargo clippy --all-features -- -D warnings`
- [ ] Code formatted: `cargo fmt`
- [ ] Documentation updated (if applicable)
- [ ] CHANGELOG.md updated (for features/fixes)
- [ ] Commit messages follow conventions
- [ ] No merge conflicts
- [ ] Branch is up-to-date with main

### PR Description Template

```markdown
## Description
Brief description of changes

## Type of Change
- [ ] New feature
- [ ] Bug fix
- [ ] Documentation
- [ ] Refactoring
- [ ] Other: ...

## Testing
- [ ] Added new tests
- [ ] All tests passing
- [ ] Manual testing done

## Documentation
- [ ] Updated README if needed
- [ ] Added code comments
- [ ] Updated CHANGELOG

## Code Quality
- [ ] Ran `cargo fmt`
- [ ] Ran `cargo clippy`
- [ ] No compiler warnings
- [ ] Code follows style guide
```

### Review Requirements

- **2 approvals** from team members
- **All CI checks** passing
- **Code coverage** maintained or improved
- **No merge conflicts**

### After Approval

1. Ensure all comments are addressed
2. Squash commits if requested
3. Merge to main branch
4. Delete feature branch

---

## Code of Conduct

### Our Pledge

We are committed to providing a welcoming and inclusive environment for all contributors.

### Expected Behavior

- Be respectful and constructive
- Welcome different perspectives
- Provide helpful feedback
- Focus on ideas, not people

### Unacceptable Behavior

- Harassment or discrimination
- Offensive language
- Personal attacks
- Trolling or spam

### Reporting

Report violations to: [security@example.com](mailto:security@example.com)

---

## Questions?

- **GitHub Issues**: For bugs and feature requests
- **GitHub Discussions**: For questions and ideas
- **Discord**: For real-time chat (if available)
- **Email**: For sensitive matters

---

## Resources

- [BEST_PRACTICES.md](BEST_PRACTICES.md) - Code quality standards
- [DEVELOPMENT.md](docs/DEVELOPMENT.md) - Architecture details
- [README.md](README.md) - Project overview
- [Rust Book](https://doc.rust-lang.org/book/) - Learn Rust
- [Conventional Commits](https://www.conventionalcommits.org/) - Commit format

---

## Additional Notes

### When to Ask for Help

- Unsure about approach? Ask!
- Need clarification? Ask!
- Want feedback? Ask!

### License

By contributing, you agree that your contributions will be licensed under the same license as the project (see LICENSE file).

---

**Thank you for contributing to Semantic Browser! 🎉**

Last updated: 2025-10-22
