// Security module

use tracing::info;

/// Validate HTML input to prevent malicious content
pub fn validate_html_input(html: &str) -> Result<(), &'static str> {
    if html.len() > 10_000_000 {
        return Err("HTML too large");
    }
    let html_lower = html.to_lowercase();
    // Block script tags and other potentially dangerous elements, but allow JSON-LD
    let has_script = html_lower.contains("<script");
    let has_json_ld = html_lower.contains("application/ld+json");
    if (has_script && !has_json_ld)
        || html_lower.contains("<iframe")
        || html_lower.contains("<object")
        || html_lower.contains("<embed")
    {
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

/// Validate SPARQL query to prevent injection or expensive queries
pub fn validate_sparql_query(query: &str) -> Result<(), &'static str> {
    if query.len() > 10_000 {
        return Err("Query too long");
    }

    let trimmed = query.trim().to_uppercase();

    // Allow SELECT, INSERT, DELETE, CONSTRUCT, ASK, DESCRIBE
    let allowed_operations = ["SELECT", "INSERT", "DELETE", "CONSTRUCT", "ASK", "DESCRIBE"];

    let is_valid = allowed_operations.iter().any(|op| trimmed.starts_with(op));

    if !is_valid {
        return Err("Unsupported SPARQL operation. Allowed: SELECT, INSERT, DELETE, CONSTRUCT, ASK, DESCRIBE");
    }

    // Basic security checks
    if trimmed.contains("DROP") || trimmed.contains("CLEAR") || trimmed.contains("LOAD") {
        return Err("Potentially dangerous SPARQL operation detected");
    }

    Ok(())
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
}
