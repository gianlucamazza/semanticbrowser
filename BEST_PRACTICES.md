# 🎯 Development Best Practices

**Version**: 1.0
**Last Updated**: 2025-10-22
**Status**: Active Guidelines
**Maintainer**: Technical Lead

---

## 📋 Table of Contents
1. Code Quality
2. Testing Strategy
3. Git Workflow
4. Documentation
5. Performance
6. Security
7. Team Communication
8. Release Process

---

## 1️⃣ Code Quality Standards

### 1.1 Compilation & Linting

**Zero Tolerance Policy**:
```bash
# Must pass ALL of these before PR submission
cargo fmt --check          # Code formatting
cargo clippy --all-features -- -D warnings  # Linting with warnings as errors
cargo test --all-features  # All tests must pass
cargo build --all-features # Must compile successfully
cargo doc --no-deps        # Documentation must generate
```

**Pre-commit Hook**:
```bash
#!/bin/bash
cargo fmt --check && \
cargo clippy --all-features -- -D warnings && \
cargo test --lib --all-features

exit $?
```

### 1.2 Code Style

**Naming Conventions**:
```rust
// Constants: UPPER_SNAKE_CASE
const MAX_RETRIES: u32 = 3;

// Types: PascalCase
struct BrowserConfig;
enum ProviderType { Ollama, OpenAI, Anthropic }

// Functions/methods: snake_case
fn execute_workflow() {}

// Private helpers: prefixed with underscore if unused
fn _internal_helper() {}
```

**Module Organization**:
```
src/
├── lib.rs              # Public API exports
├── error.rs            # Error types
├── config.rs           # Configuration
├── llm/
│   ├── mod.rs         # Module exports
│   ├── provider.rs    # Trait definition
│   ├── ollama.rs      # Provider implementation
│   └── tools.rs       # Tool definitions
├── ml/
│   ├── mod.rs
│   ├── embeddings.rs
│   └── inference.rs
└── tests/
    └── integration_tests.rs
```

**Documentation**:
```rust
/// Brief one-liner.
///
/// Longer explanation with context about what this does,
/// why it exists, and when to use it.
///
/// # Arguments
/// * `param1` - Description
/// * `param2` - Description
///
/// # Returns
/// Description of return value
///
/// # Examples
/// ```
/// let result = my_function(1, 2)?;
/// assert_eq!(result, 3);
/// ```
///
/// # Errors
/// Returns error if ...
///
/// # Panics
/// Panics if invariant is violated
pub fn my_function(param1: u32, param2: u32) -> Result<u32> {
    // Implementation
}
```

### 1.3 Error Handling

**Pattern: Never unwrap in production code**:
```rust
// ❌ WRONG - Can panic
let data = file.read().unwrap();

// ✅ CORRECT - Proper error handling
let data = file.read()?;

// ✅ CORRECT - With context
let data = file.read()
    .context("Failed to read config file")?;
```

**Error Types**:
```rust
use thiserror::Error;

#[derive(Error, Debug)]
pub enum BrowserError {
    #[error("Navigation failed: {0}")]
    NavigationFailed(String),

    #[error("Timeout waiting for element: {selector}")]
    Timeout { selector: String },

    #[error("Invalid selector: {0}")]
    InvalidSelector(String),
}
```

**Result Type Convention**:
```rust
// Define at crate level
pub type Result<T> = std::result::Result<T, YourError>;

// Use throughout
pub fn my_function() -> Result<String> { ... }
```

---

## 2️⃣ Testing Strategy

### 2.1 Test Pyramid

```
         △ E2E Tests (10%)
        △△ Integration Tests (30%)
       △△△ Unit Tests (60%)
```

**Guidelines**:
- **Unit Tests**: Fast, isolated, test single functions
- **Integration Tests**: Test component interactions
- **E2E Tests**: Real workflows with actual systems

### 2.2 Test Organization

**Location**:
```
src/module/
├── lib.rs
└── tests.rs        # Unit tests

tests/
├── integration_tests.rs
├── fixtures/
│   └── mock_data.rs
└── utils/
    └── test_helpers.rs
```

**Naming**:
```rust
#[test]
fn test_happy_path() { }

#[test]
fn test_error_handling() { }

#[test]
fn test_edge_case_empty_input() { }

#[tokio::test]
async fn test_async_operation() { }
```

### 2.3 Mocking Strategy

**Use Mock When**:
- ✅ Testing external API calls
- ✅ Simulating error conditions
- ✅ Avoiding slow operations
- ✅ Testing timing-sensitive code

**Tools**:
```toml
[dev-dependencies]
mockito = "1.2"      # HTTP mocking
mockall = "0.12"     # Trait mocking
proptest = "1.4"     # Property-based testing
```

**Example**:
```rust
#[tokio::test]
async fn test_with_mock_provider() {
    let mock_provider = MockLLMProvider::new();
    mock_provider.expect_chat_completion()
        .returning(|_| Ok(LLMResponse { ... }));

    let agent = AgentOrchestrator::new(
        Arc::new(mock_provider),
        config,
        tools,
    );

    // Test agent behavior
}
```

### 2.4 Test Coverage Goals

| Area | Target | Method |
|------|--------|--------|
| Core logic | 90%+ | Unit tests |
| Public API | 85%+ | Integration tests |
| Error paths | 80%+ | Both |
| Overall | 80%+ | Combined |

**Check Coverage**:
```bash
cargo tarpaulin --all-features --timeout 300
```

---

## 3️⃣ Git Workflow

### 3.1 Branch Strategy (Git Flow)

```
main (production)
  ↑
  ├─ release/v0.2.0 (release prep)
  │
develop (integration)
  ↑
  ├─ feature/streaming
  ├─ feature/vision-models
  ├─ feature/multi-tab
  ├─ bugfix/clippy-warnings
  └─ docs/api-documentation
```

### 3.2 Commit Messages

**Format** (Conventional Commits):
```
<type>(<scope>): <subject>

<body>

<footer>
```

**Types**:
- `feat`: New feature
- `fix`: Bug fix
- `docs`: Documentation
- `style`: Formatting, missing semicolons, etc.
- `refactor`: Code restructuring
- `perf`: Performance improvement
- `test`: Adding or updating tests
- `chore`: Dependency updates, build changes

**Examples**:
```
feat(llm): add streaming support for OpenAI

Implement server-sent events handling for real-time token streaming.
Includes proper error handling and reconnection logic.

Closes #123
Reviewed-by: @reviewer

---

fix(workflow): resolve tool_call variable reference

The tool_call parameter was prefixed with underscore but still referenced.
Updated parameter naming to match usage.

---

docs(setup): add ML integration guide

Created comprehensive guide covering Ollama, OpenAI, and Anthropic setup.
Includes troubleshooting and configuration examples.
```

### 3.3 Pull Request Process

**Checklist**:
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

## Deployment
- [ ] No breaking changes
- [ ] Database migrations (if applicable)
- [ ] Environment variables documented
```

**Review Requirements**:
- [ ] 2 approvals from team members
- [ ] All CI checks passing
- [ ] Code coverage maintained/improved
- [ ] No merge conflicts

### 3.4 Release Branch

```bash
# Create release branch
git checkout -b release/v0.2.0 develop

# Update version and changelog
cargo release version minor

# Create tag
git tag -a v0.2.0 -m "Release v0.2.0"

# Merge to main
git checkout main
git merge --no-ff release/v0.2.0

# Merge back to develop
git checkout develop
git merge --no-ff main

# Push everything
git push origin main develop v0.2.0
```

---

## 4️⃣ Documentation Standards

### 4.1 In-Code Documentation

**Rule**: "Write code so obvious it needs no comments, but document why"

```rust
// ❌ WRONG - Obvious what code does
// Increment counter
counter += 1;

// ✅ CORRECT - Explains why
// Reset counter for next batch to avoid overflow
counter = 0;
```

### 4.2 Module Documentation

```rust
//! Module for LLM provider abstraction.
//!
//! Provides unified interface to interact with different LLM providers
//! (OpenAI, Anthropic, Ollama) with streaming support and error handling.
//!
//! # Examples
//!
//! ```
//! use semantic_browser::llm::{OllamaProvider, LLMProvider};
//!
//! let provider = OllamaProvider::new(config);
//! let response = provider.chat_completion(messages, config).await?;
//! ```

pub mod anthropic;
pub mod ollama;
pub mod openai;
pub mod provider;
pub mod tools;

pub use provider::{LLMProvider, LLMResponse, Message};
```

### 4.3 README Format

```markdown
# Project Name

One-liner description

## Features
- Feature 1
- Feature 2

## Quick Start
```bash
# Get started in 5 minutes
```

## Documentation
- [API Reference](docs/api.md)
- [Deployment Guide](docs/deployment.md)
- [Contributing](CONTRIBUTING.md)

## Examples
- [Basic Usage](examples/basic.rs)
- [Advanced Workflow](examples/advanced.rs)

## License
MIT
```

### 4.4 API Documentation

**Generate with**:
```bash
cargo doc --no-deps --open
```

**Requirements**:
- ✅ All public items documented
- ✅ Examples in doc comments
- ✅ Error conditions documented
- ✅ Panics documented

---

## 5️⃣ Performance Guidelines

### 5.1 Benchmarking

```bash
cargo bench --all-features
```

**Benchmark Template**:
```rust
use criterion::{black_box, criterion_group, criterion_main, Criterion};

fn benchmark_agent_execution(c: &mut Criterion) {
    c.bench_function("agent_simple_task", |b| {
        b.to_async(tokio::runtime::Runtime::new().unwrap())
            .iter(|| {
                let agent = setup_agent();
                agent.execute(black_box(simple_task()))
            });
    });
}

criterion_group!(benches, benchmark_agent_execution);
criterion_main!(benches);
```

### 5.2 Performance Targets

| Operation | Target | Measurement |
|-----------|--------|-------------|
| Agent startup | < 1s | Time to first prompt |
| Simple navigation | < 5s | Navigate + wait for load |
| Form filling | < 2s | Fill single form |
| KG query | < 100ms | SPARQL query latency |
| Token generation | < 100ms | First token latency |

### 5.3 Profiling

```bash
# CPU profiling with flamegraph
cargo flamegraph --bin semantic_browser_agent

# Memory profiling
heaptrack ./target/release/semantic_browser_agent

# Async runtime profiling
TOKIO_CONSOLE=1 cargo run --bin semantic_browser_agent
```

---

## 6️⃣ Security Best Practices

### 6.1 Dependency Security

```bash
# Audit dependencies
cargo audit --deny warnings

# Check for unmaintained packages
cargo outdated

# Update dependencies safely
cargo update
```

**Policy**:
- ✅ Run audit on every PR
- ✅ Evaluate vulnerability severity
- ✅ Update within 7 days for critical issues
- ✅ Review changelog before major version updates

### 6.2 Secret Management

```rust
// ❌ WRONG - Hardcoded secrets
const API_KEY: &str = "sk-abc123...";

// ✅ CORRECT - From environment
let api_key = std::env::var("OPENAI_API_KEY")?;

// ✅ CORRECT - From secure config
let config = SecureConfig::load()?;
```

**.env Format**:
```bash
# .env.example (check in git, no secrets)
OPENAI_API_KEY=<your_key_here>
ANTHROPIC_API_KEY=<your_key_here>
JWT_SECRET=<your_secret_here>
```

### 6.3 Input Validation

```rust
// Always validate user input
pub fn execute_script(script: &str) -> Result<String> {
    // 1. Validate length
    if script.len() > 10_000 {
        return Err("Script too long".into());
    }

    // 2. Sanitize for security
    let safe_script = sanitize_js(script);

    // 3. Execute
    execute_js(&safe_script)
}
```

### 6.4 Error Information Disclosure

```rust
// ❌ WRONG - Leaks system information
println!("{:?}", database_error);  // Internal error details

// ✅ CORRECT - Generic error to user
error!("Database error: {:?}", database_error);  // Log internally
return Err("Database operation failed".into());  // Generic response
```

---

## 7️⃣ Team Communication

### 7.1 Code Review Culture

**Reviewer Responsibility**:
- ✅ Understand the change
- ✅ Verify tests cover new code
- ✅ Check performance impact
- ✅ Suggest improvements (constructively)
- ✅ Approve or request changes

**Author Responsibility**:
- ✅ Small, focused PRs (< 400 lines)
- ✅ Clear description of changes
- ✅ Link to related issues
- ✅ Respond to feedback promptly
- ✅ Resolve all comments before merge

**Review Checklist**:
```markdown
- [ ] Code compiles without warnings
- [ ] All tests passing
- [ ] New tests added if needed
- [ ] No clippy warnings
- [ ] Code follows style guide
- [ ] Documentation updated
- [ ] Performance impact acceptable
- [ ] No security issues
- [ ] Change log updated
```

### 7.2 Async Communication

**Synchronous** (Real-time):
- Slack for quick questions
- Discord for technical discussions
- Daily standup for blockers

**Asynchronous** (Documented):
- GitHub issues for tasks
- PR comments for code review
- Email for formal announcements
- Wiki for documentation

### 7.3 Decision Making

**For architecture decisions**:
1. Create GitHub issue with proposal
2. Discuss options (pros/cons)
3. RFC (Request for Comments) if complex
4. Document decision in ADR (Architecture Decision Record)
5. Implementation starts after consensus

---

## 8️⃣ Release Process

### 8.1 Pre-Release Checklist

**1 Week Before**:
- [ ] Create release branch
- [ ] Update CHANGELOG.md
- [ ] Bump version in Cargo.toml
- [ ] Verify all PRs merged
- [ ] Run full test suite

**2 Days Before**:
- [ ] Final code review
- [ ] Update documentation
- [ ] Create release notes
- [ ] Test on multiple platforms

**Release Day**:
- [ ] Create git tag
- [ ] Publish to crates.io
- [ ] Create GitHub release
- [ ] Update website
- [ ] Announce on social media

### 8.2 Versioning (Semantic Versioning)

```
MAJOR.MINOR.PATCH
  ↓      ↓      ↓
  0      2      0

MAJOR: Incompatible API changes
MINOR: New functionality (backward compatible)
PATCH: Bug fixes (backward compatible)

Examples:
0.1.0 → 0.1.1 (patch: bug fix)
0.1.1 → 0.2.0 (minor: new feature)
0.2.0 → 1.0.0 (major: breaking changes)
```

### 8.3 CHANGELOG Format

```markdown
## [0.2.0] - 2025-02-28

### Added
- Streaming support for OpenAI provider
- Vision models integration (GPT-4V)
- Multi-tab browser orchestration

### Changed
- Updated dependencies to latest versions
- Improved error messages for clarity

### Fixed
- Fixed clippy warnings in workflow module
- Fixed race condition in agent execution

### Security
- Added input validation for user scripts
- Updated vulnerable dependencies

### Deprecated
- Old config format (use new format, see docs)

### Breaking Changes
- Renamed `execute()` to `execute_task()`
- Removed deprecated `old_api()` function
```

### 8.4 Communication Plan

```
T-1 Week: Announce upcoming release
  ↓
T-2 Days: Release candidate, request testing
  ↓
T-1 Day: Final checks, lock changes
  ↓
T-0: Release published
  ↓
T+1 Day: Blog post, social media
  ↓
T+1 Week: Retrospective on release process
```

---

## 📚 Resources & References

### Internal
- [ROADMAP.md](ROADMAP.md) - Long-term planning
- [SPRINT_PLAN.md](SPRINT_PLAN.md) - Weekly planning
- [CONTRIBUTING.md](CONTRIBUTING.md) - Contribution guidelines

### External
- [Rust Book](https://doc.rust-lang.org/book/)
- [Rust API Guidelines](https://rust-lang.github.io/api-guidelines/)
- [Conventional Commits](https://www.conventionalcommits.org/)
- [Keep a Changelog](https://keepachangelog.com/)

---

## ✅ Checklist: Am I Ready to Submit a PR?

- [ ] Code compiles: `cargo build --all-features`
- [ ] Clippy clean: `cargo clippy --all-features -- -D warnings`
- [ ] Tests pass: `cargo test --all-features`
- [ ] Formatted: `cargo fmt`
- [ ] Documentation added/updated
- [ ] Tests added for new code
- [ ] Changelog updated
- [ ] Commit messages follow conventions
- [ ] No merge conflicts
- [ ] PR description is clear

**If all checked ✅, you're ready to submit!**

---

**Last Updated**: 2025-10-22
**Maintained By**: Technical Leadership
**Review Frequency**: Quarterly
**Feedback**: Open issues on GitHub

