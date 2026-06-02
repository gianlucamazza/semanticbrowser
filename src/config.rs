//! Centralized default constants.
//!
//! Values that were previously hard-coded across modules live here so they are easy
//! to find and change in one place. Runtime-overridable settings (API base URLs,
//! credentials) stay as struct fields / env vars; this module holds only the defaults.

/// Default TCP port the HTTP API server binds to.
pub const DEFAULT_PORT: u16 = 3000;

/// Default base URL for a local Ollama instance.
pub const OLLAMA_DEFAULT_BASE_URL: &str = "http://localhost:11434";

/// Default request timeout (seconds) for Ollama calls (local models can be slow).
pub const OLLAMA_DEFAULT_TIMEOUT_SECS: u64 = 120;
