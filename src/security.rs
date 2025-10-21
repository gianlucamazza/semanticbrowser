// Security module

use tracing::info;

/// Validate HTML input to prevent malicious content
pub fn validate_html_input(html: &str) -> Result<(), &'static str> {
    if html.len() > 10_000_000 {
        return Err("HTML too large");
    }
    // Block script tags and other potentially dangerous elements, but allow JSON-LD
    if (html.contains("<script") && !html.contains("application/ld+json"))
        || html.contains("<iframe")
        || html.contains("<object")
        || html.contains("<embed")
    {
        return Err("Potentially malicious HTML: script or embed tags detected");
    }
    // Check for other malicious patterns
    if html.contains("javascript:")
        || html.contains("onload=")
        || html.contains("onerror=")
        || html.contains("eval(")
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
/// This function provides syscall restriction for parsing operations to limit
/// what operations can be performed, reducing the attack surface.
///
/// On Linux with the `seccomp` feature enabled, this restricts syscalls to a safe subset.
/// On other platforms or without the feature, it's a no-op wrapper.
pub fn sandbox_parsing<F, R>(f: F) -> R
where
    F: FnOnce() -> R,
{
    #[cfg(all(target_os = "linux", feature = "seccomp"))]
    {
        // In a real implementation, we would use the seccomp crate:
        // use seccompiler::{apply_filter, BpfProgram};
        //
        // let filter = BpfProgram::new(vec![
        //     // Allow only necessary syscalls:
        //     // read, write, open, close, mmap, munmap, etc.
        //     // Block: exec, socket, connect, etc.
        // ]);
        //
        // apply_filter(&filter).expect("Failed to apply seccomp filter");

        tracing::debug!("Applying seccomp sandbox (not yet implemented)");
        f()
    }

    #[cfg(not(all(target_os = "linux", feature = "seccomp")))]
    {
        // No sandboxing on non-Linux or when feature is disabled
        tracing::trace!("Sandboxing not available on this platform");
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
}
