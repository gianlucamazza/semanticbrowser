// Security module

use tracing::info;

/// Validate HTML input to prevent malicious content
pub fn validate_html_input(html: &str) -> Result<(), &'static str> {
    if html.len() > 10_000_000 {
        return Err("HTML too large");
    }
    let html_lower = html.to_lowercase();
    // Block embed-style tags outright.
    if html_lower.contains("<iframe")
        || html_lower.contains("<object")
        || html_lower.contains("<embed")
    {
        return Err("Potentially malicious HTML: script or embed tags detected");
    }
    // Allow <script> ONLY when *every* script tag is a JSON-LD data block. A single
    // non-JSON-LD <script> (even alongside a valid JSON-LD one) is rejected.
    if !all_scripts_are_json_ld(&html_lower) {
        return Err("Potentially malicious HTML: script or embed tags detected");
    }
    // Check for other malicious patterns
    if html_lower.contains("javascript:")
        || html_lower.contains("onload=")
        || html_lower.contains("onerror=")
        || html_lower.contains("eval(")
    {
        return Err("Potentially malicious HTML: dangerous attributes or functions detected");
    }
    Ok(())
}

/// Return true if the (already lowercased) HTML contains no `<script>` tags, or if
/// every opening `<script` tag declares `type="application/ld+json"`. Quote style and
/// surrounding whitespace are tolerated; any other (or missing) type fails the check.
fn all_scripts_are_json_ld(html_lower: &str) -> bool {
    let mut rest = html_lower;
    while let Some(idx) = rest.find("<script") {
        // Slice the opening tag (up to the first '>').
        let after = &rest[idx + "<script".len()..];
        let tag = match after.find('>') {
            Some(end) => &after[..end],
            None => return false, // malformed/unterminated script tag
        };
        // Normalize the tag attributes and require the JSON-LD content type.
        let compact: String = tag.chars().filter(|c| !c.is_whitespace()).collect();
        let is_json_ld = compact.contains("type=\"application/ld+json\"")
            || compact.contains("type='application/ld+json'")
            || compact.contains("type=application/ld+json");
        if !is_json_ld {
            return false;
        }
        rest = after;
    }
    true
}

/// Validate SPARQL query to prevent injection or expensive queries
pub fn validate_sparql_query(query: &str) -> Result<(), &'static str> {
    if query.len() > 10_000 {
        return Err("Query too long");
    }

    // Strip `#` line comments and quoted string literals before keyword scanning so
    // that dangerous keywords cannot hide in comments/strings and, conversely, legit
    // keywords appearing inside data do not trigger false positives.
    let sanitized = strip_sparql_noise(query).to_uppercase();
    let trimmed = sanitized.trim();

    // Allow SELECT, INSERT, DELETE, CONSTRUCT, ASK, DESCRIBE (after any PREFIX/BASE prologue)
    let allowed_operations = ["SELECT", "INSERT", "DELETE", "CONSTRUCT", "ASK", "DESCRIBE"];

    // Skip the PREFIX/BASE prologue when checking the first real operation.
    let first_op = trimmed
        .split_whitespace()
        .find(|w| !matches!(*w, "PREFIX" | "BASE") && !w.starts_with('<') && !w.starts_with('@'))
        .unwrap_or("");
    let is_valid = allowed_operations.iter().any(|op| trimmed.starts_with(op) || first_op == *op);

    if !is_valid {
        return Err("Unsupported SPARQL operation. Allowed: SELECT, INSERT, DELETE, CONSTRUCT, ASK, DESCRIBE");
    }

    // Block destructive graph-management keywords as whole words (case-insensitive,
    // comments/strings already removed above).
    for danger in ["DROP", "CLEAR", "LOAD"] {
        if contains_word(trimmed, danger) {
            return Err("Potentially dangerous SPARQL operation detected");
        }
    }

    Ok(())
}

/// Remove `#` line comments and single/double quoted string literals from a SPARQL
/// query, replacing them with spaces so keyword scanning sees only structural tokens.
fn strip_sparql_noise(query: &str) -> String {
    let mut out = String::with_capacity(query.len());
    let mut chars = query.chars().peekable();
    while let Some(c) = chars.next() {
        match c {
            '#' => {
                // Comment: skip to end of line.
                for next in chars.by_ref() {
                    if next == '\n' {
                        out.push('\n');
                        break;
                    }
                }
            }
            '"' | '\'' => {
                // String literal: skip until matching unescaped quote.
                let quote = c;
                out.push(' ');
                while let Some(next) = chars.next() {
                    if next == '\\' {
                        chars.next(); // skip escaped char
                    } else if next == quote {
                        break;
                    }
                }
                out.push(' ');
            }
            '<' => {
                // IRI reference `<...>` (no internal whitespace) — blank it so path
                // segments like `.../load` are not mistaken for the LOAD keyword.
                // A `<` followed by whitespace is a comparison operator: keep it.
                let span: String = chars.clone().take_while(|c| *c != '>' && *c != '<').collect();
                if !span.is_empty() && !span.contains(char::is_whitespace) {
                    for _ in 0..=span.chars().count() {
                        chars.next(); // consume span chars + closing '>'
                    }
                    out.push(' ');
                } else {
                    out.push('<');
                }
            }
            _ => out.push(c),
        }
    }
    out
}

/// Whole-word, case-insensitive containment check (`needle` must already be uppercase
/// and `haystack` must be the uppercased query).
fn contains_word(haystack: &str, needle: &str) -> bool {
    haystack.split(|c: char| !c.is_ascii_alphanumeric()).any(|w| w == needle)
}

/// Log agent actions
pub fn log_action(action: &str, details: &str) {
    info!("Agent action: {} - {}", action, details);
}

/// Sandbox a function execution (seccomp-based on Linux)
///
/// Best practices 2025:
/// - Use seccompiler for easy seccomp-bpf filter creation
/// - Allow minimal syscalls needed for safe operation
/// - Block dangerous syscalls (exec, socket, ptrace, etc.)
/// - Apply filters before untrusted code execution
///
/// On Linux with the `seccomp` feature enabled, this restricts syscalls to a safe subset.
/// On other platforms or without the feature, it's a no-op wrapper.
pub fn sandbox_parsing<F, R>(f: F) -> R
where
    F: FnOnce() -> R,
{
    #[cfg(all(target_os = "linux", feature = "seccomp"))]
    {
        use seccompiler::{apply_filter, BpfProgram, SeccompAction, SeccompFilter, SeccompRule};
        use std::collections::HashMap;

        // Define allowed syscalls for HTML parsing
        // Best practice: whitelist approach - only allow what's needed
        let allowed_syscalls = vec![
            // Memory operations
            libc::SYS_brk,
            libc::SYS_mmap,
            libc::SYS_munmap,
            libc::SYS_mremap,
            libc::SYS_mprotect,
            // File operations (read-only)
            libc::SYS_read,
            libc::SYS_readv,
            libc::SYS_pread64,
            libc::SYS_close,
            libc::SYS_fstat,
            libc::SYS_lseek,
            // Thread/process management (minimal)
            libc::SYS_futex,
            libc::SYS_rt_sigreturn,
            libc::SYS_exit,
            libc::SYS_exit_group,
            libc::SYS_getpid,
            libc::SYS_gettid,
            // Time operations
            libc::SYS_clock_gettime,
            libc::SYS_gettimeofday,
            // Misc required
            libc::SYS_getrandom,
            libc::SYS_sched_getaffinity,
        ];

        // Create filter rules
        let mut rules = HashMap::new();
        for &syscall in &allowed_syscalls {
            rules.insert(syscall as i64, vec![SeccompRule::new(vec![], SeccompAction::Allow)]);
        }

        // Create the seccomp filter
        let filter = SeccompFilter::new(
            rules,
            SeccompAction::Trap, // Trap on disallowed syscalls
            SeccompAction::Allow,
            std::arch::consts::ARCH.try_into().unwrap(),
        )
        .expect("Failed to create seccomp filter");

        // Compile to BPF program
        let bpf_filter: BpfProgram = filter.try_into().expect("Failed to compile BPF filter");

        // Apply the filter
        match apply_filter(&bpf_filter) {
            Ok(_) => {
                tracing::debug!("Seccomp sandbox applied successfully");
                let result = f();
                tracing::debug!("Sandboxed operation completed");
                result
            }
            Err(e) => {
                tracing::error!("Failed to apply seccomp filter: {}", e);
                // Fall back to executing without sandbox
                tracing::warn!("Executing without sandbox due to filter application failure");
                f()
            }
        }
    }

    #[cfg(not(all(target_os = "linux", feature = "seccomp")))]
    {
        // No sandboxing on non-Linux or when feature is disabled
        tracing::trace!("Sandboxing not available on this platform or feature not enabled");
        f()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_sandbox_wrapper() {
        let result = sandbox_parsing(|| {
            // This should execute normally
            42
        });
        assert_eq!(result, 42);
    }

    #[test]
    fn test_html_validation() {
        assert!(validate_html_input("<html><body>Test</body></html>").is_ok());
        assert!(
            validate_html_input("<html><body><iframe src='evil'></iframe></body></html>").is_err()
        );
    }

    #[test]
    fn test_sparql_validation() {
        assert!(validate_sparql_query("SELECT * WHERE { ?s ?p ?o }").is_ok());
        assert!(validate_sparql_query("DROP ALL").is_err());
    }

    #[test]
    fn test_html_allows_json_ld_script() {
        let html = r#"<html><head><script type="application/ld+json">{}</script></head></html>"#;
        assert!(
            validate_html_input(html).is_ok(),
            "JSON-LD scripts should be allowed by the validator"
        );
    }

    #[test]
    fn test_html_rejects_inline_event_handler() {
        let html = r#"<img src="test.png" onerror="alert('xss')" />"#;
        assert!(validate_html_input(html).is_err(), "Inline event handlers must be rejected");
    }

    #[test]
    fn test_html_case_insensitive_checks() {
        assert!(
            validate_html_input("<SCRIPT>alert(1)</SCRIPT>").is_err(),
            "Uppercase script tag should be rejected"
        );
        let json_ld_upper = r#"<script TYPE="APPLICATION/LD+JSON">{}</script>"#;
        assert!(
            validate_html_input(json_ld_upper).is_ok(),
            "JSON-LD script type should be treated case-insensitively"
        );
    }

    #[test]
    fn test_sparql_validation_handles_whitespace_and_case() {
        assert!(
            validate_sparql_query("   select * WHERE { ?s ?p ?o }").is_ok(),
            "Leading whitespace with lowercase operation should still be accepted"
        );
        assert!(
            validate_sparql_query("ask WHERE { ?s ?p ?o }").is_ok(),
            "Lowercase ASK should be normalized"
        );
        assert!(
            validate_sparql_query("/* comment */ drop DATASET").is_err(),
            "Dangerous keywords inside comments should still trigger rejection"
        );
    }

    #[test]
    fn test_html_rejects_mixed_script_with_json_ld() {
        // Regression (A4): a malicious <script> alongside a valid JSON-LD block must be rejected.
        let html = r#"<script type="application/ld+json">{}</script><script>alert(1)</script>"#;
        assert!(
            validate_html_input(html).is_err(),
            "A non-JSON-LD script must be rejected even when a JSON-LD script is present"
        );
    }

    #[test]
    fn test_sparql_keyword_in_literal_is_allowed() {
        // Regression (A3): a danger keyword inside a string literal is not an operation.
        assert!(
            validate_sparql_query(r#"SELECT * WHERE { ?s ?p "please DROP this" }"#).is_ok(),
            "DROP inside a string literal should not be flagged"
        );
    }

    #[test]
    fn test_sparql_keyword_in_iri_is_allowed() {
        // Regression (A3): a path segment named like a keyword inside an IRI is safe.
        assert!(
            validate_sparql_query("SELECT * WHERE { ?a <http://example.org/load> ?c }").is_ok(),
            "LOAD as an IRI path segment should not be flagged"
        );
    }

    #[test]
    fn test_sparql_hash_comment_keyword_is_blocked() {
        // A real DROP after a hash comment must still be caught.
        assert!(
            validate_sparql_query("# harmless\nDROP GRAPH <http://x>").is_err(),
            "An actual DROP operation must be rejected"
        );
    }
}
