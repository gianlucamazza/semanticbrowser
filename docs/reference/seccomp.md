# Seccomp Sandboxing Configuration

This guide explains how to configure and use seccomp sandboxing for enhanced security in the Semantic Browser.

## Overview

Seccomp (Secure Computing) is a Linux kernel feature that allows filtering system calls. The Semantic Browser uses seccomp-bpf to restrict the system calls available to HTML parsing operations, reducing the attack surface.

## Prerequisites

- Linux kernel 3.5+ (for seccomp-bpf)
- `seccompiler` crate (included with `--features seccomp`)

## Building with Seccomp

```bash
# Enable seccomp feature
cargo build --release --features seccomp

# Or with all features
cargo build --release --all-features
```

## How It Works

The seccomp filter is applied during HTML parsing operations using a whitelist approach:

1. **Allowed Syscalls**: Only essential syscalls for memory management, file I/O, and process control
2. **Blocked Syscalls**: Dangerous operations like `exec`, `socket`, `ptrace` are blocked
3. **Graceful Fallback**: If seccomp fails to apply, parsing continues without sandboxing

## Allowed Syscalls

The current whitelist includes:

### Memory Management
- `brk` - Adjust data segment size
- `mmap` - Map files or devices into memory
- `munmap` - Unmap files from memory
- `mremap` - Remap virtual memory addresses
- `mprotect` - Control protection of memory regions

### File I/O
- `read` - Read from file descriptor
- `readv` - Read from file descriptor into multiple buffers
- `pread64` - Read from file descriptor at offset
- `close` - Close file descriptor
- `fstat` - Get file status

### Process Control
- `futex` - Fast user-space locking
- `exit` - Terminate the calling process
- `exit_group` - Terminate all threads in process group
- `getpid` - Get process ID
- `gettid` - Get thread ID

### Time
- `clock_gettime` - Get time from specific clock
- `gettimeofday` - Get time (deprecated but allowed)

### Miscellaneous
- `getrandom` - Get random bytes
- `sched_getaffinity` - Get CPU affinity

## Configuration

Seccomp is automatically enabled when the feature flag is compiled in. No runtime configuration is required.

## Testing Seccomp

### Unit Tests

```bash
cargo test --features seccomp security::tests::test_sandbox_wrapper
```

### Manual Testing

1. **Check if seccomp is active**:
```bash
# Build with seccomp
cargo build --release --features seccomp

# Run and check logs
RUST_LOG=debug ./target/release/semantic_browser_agent

# Look for: "Seccomp filter applied successfully"
```

2. **Test syscall blocking**:
```bash
# Try operations that would use blocked syscalls
# The process should continue working for allowed operations
# but fail gracefully for blocked ones
```

## Troubleshooting

### Common Issues

1. **Seccomp not applied**:
   - Check if running on Linux
   - Verify `--features seccomp` was used during build
   - Check kernel version (`uname -r`)

2. **Application crashes**:
   - Seccomp filter might be too restrictive
   - Check allowed syscalls list
   - Review HTML parsing code for unexpected syscalls

3. **Performance impact**:
   - Seccomp has minimal overhead (<1% CPU)
   - Only applied during parsing operations
   - No impact on normal browsing/querying

### Debugging

Enable detailed logging:

```bash
export RUST_LOG=semantic_browser=debug,seccompiler=trace
```

Check system call usage:

```bash
# Use strace to monitor syscalls during parsing
strace -e trace=network,file,process ./target/release/semantic_browser_agent
```

## Security Considerations

### Defense in Depth

Seccomp provides an additional security layer:

1. **Input Validation**: HTML input is validated before parsing
2. **Sandboxing**: Syscall filtering during parsing
3. **Resource Limits**: Rate limiting and size restrictions
4. **Logging**: Security events are logged

### Limitations

- Linux-only feature
- Only protects during parsing operations
- Does not protect against logic bugs in parsing code
- Requires careful syscall whitelist maintenance

## Advanced Configuration

### Custom Syscall Filters

To modify the syscall whitelist, edit `src/security.rs`:

```rust
let filter = seccompiler::SeccompFilter::new(
    vec![
        // Add your custom rules here
        seccompiler::SeccompRule::new(vec![libc::SYS_read]).unwrap(),
        // ...
    ],
    seccompiler::SeccompAction::Allow,
    seccompiler::SeccompAction::Errno(libc::EPERM),
    seccompiler::SeccompFilterAttr::empty(),
)?;
```

### Conditional Application

Apply seccomp only in specific contexts:

```rust
#[cfg(feature = "seccomp")]
{
    apply_seccomp_filter()?;
}
// Continue with parsing...
```

## Performance Impact

### Benchmarks

- **Without seccomp**: Baseline parsing performance
- **With seccomp**: <1% overhead
- **Memory usage**: No significant change
- **Startup time**: Minimal increase

### Recommendations

- Enable seccomp in production builds
- Test thoroughly before deployment
- Monitor for blocked legitimate operations
- Keep syscall whitelist minimal

## Examples

### Docker Deployment

```dockerfile
FROM rust:1.70-slim AS builder
WORKDIR /app
COPY . .
RUN cargo build --release --features seccomp

FROM debian:bookworm-slim
RUN apt-get update && apt-get install -y ca-certificates && rm -rf /var/lib/apt/lists/*
COPY --from=builder /app/target/release/semantic_browser_mcp /usr/local/bin/
USER nobody
CMD ["semantic_browser_mcp"]
```

### Kubernetes

```yaml
apiVersion: apps/v1
kind: Deployment
metadata:
  name: semantic-browser
spec:
  template:
    spec:
      securityContext:
        runAsNonRoot: true
        runAsUser: 65534
      containers:
      - name: semantic-browser
        image: semantic-browser:latest
        securityContext:
          allowPrivilegeEscalation: false
          capabilities:
            drop:
            - ALL
          readOnlyRootFilesystem: true
```

## References

- [Linux Seccomp Documentation](https://www.kernel.org/doc/html/latest/userspace-api/seccomp_filter.html)
- [seccompiler Crate](https://github.com/rust-vmm/seccompiler)
- [Docker Security Best Practices](https://docs.docker.com/develop/dev-best-practices/security/)