# Security Policy

## Supported Versions

We take security seriously. The following versions of the Semantic Browser are currently supported with security updates:

| Version | Supported          |
| ------- | ------------------ |
| 0.1.x   | :white_check_mark: |
| < 0.1   | :x:                |

## Reporting a Vulnerability

If you discover a security vulnerability in the Semantic Browser, please help us by reporting it responsibly.

**Please do not report security vulnerabilities through public GitHub issues.**

Instead, please report security vulnerabilities by creating a private security advisory on GitHub or contacting the maintainers directly.

### What to Include

When reporting a vulnerability, please include:

- A clear description of the vulnerability
- Steps to reproduce the issue
- Potential impact and severity
- Any suggested fixes or mitigations (optional)

### Our Response Process

1. **Acknowledgment**: We will acknowledge receipt of your report within 48 hours
2. **Investigation**: We will investigate the issue and determine its validity and severity
3. **Updates**: We will provide regular updates on our progress (at least weekly)
4. **Fix**: Once a fix is developed, we will coordinate disclosure with you
5. **Public Disclosure**: We will publicly disclose the vulnerability after the fix is released

### Disclosure Policy

- We follow a coordinated disclosure process
- We will credit researchers who responsibly report vulnerabilities
- We will not disclose vulnerability details until a fix is available
- We aim to release fixes within 90 days of initial report (or sooner for critical issues)

## Security Best Practices

### For Users

#### Deployment Security

- **Network Security**: Deploy behind reverse proxy (nginx, traefik) with TLS
- **Firewall Configuration**: Restrict API access to trusted networks/IPs
- **Container Security**: Use non-root user, read-only filesystems, resource limits
- **Secret Management**: Use external secret stores (HashiCorp Vault, AWS Secrets Manager)

#### Authentication & Authorization

- **JWT Secrets**: Rotate `JWT_SECRET` quarterly, use 48+ character random strings
- **Token Expiration**: Set short token lifetimes (1-24 hours) based on use case
- **Role-Based Access**: Use roles to limit API access (admin, user, readonly)
- **Token Revocation**: Enable Redis integration for immediate token invalidation

#### Operational Security

- **Log Monitoring**: Set up alerts for authentication failures, rate limit hits
- **Regular Backups**: Backup KG data and configuration regularly
- **Update Management**: Apply security patches within 30 days
- **Access Auditing**: Log all API access with user context

#### Input Validation

- **HTML Size Limits**: Configure `MAX_HTML_SIZE` based on expected content
- **SPARQL Complexity**: Set `MAX_QUERY_LENGTH` to prevent expensive queries
- **URL Validation**: Use allowlists for browsing operations
- **Content Filtering**: Enable `SECURITY_STRICT_MODE` for production

### For Contributors

#### Secure Development

- **Input Validation**: Validate all inputs at API boundaries
- **Output Encoding**: Sanitize HTML/SPARQL outputs to prevent injection
- **Error Handling**: Don't leak sensitive information in error messages
- **Dependency Scanning**: Use tools like `cargo audit`, `safety` for vulnerabilities

#### Code Review Checklist

- [ ] Authentication required for all endpoints
- [ ] Input validation on all user inputs
- [ ] Proper error handling without information leakage
- [ ] No hardcoded secrets or credentials
- [ ] Resource limits on expensive operations
- [ ] Logging of security-relevant events

#### Testing Security

- **Unit Tests**: Test input validation and authentication logic
- **Integration Tests**: Test rate limiting and authorization
- **Fuzz Testing**: Use fuzzing for input parsing functions
- **Security Scanning**: Run SAST/DAST tools in CI/CD

#### Infrastructure Security

- **CI/CD Security**: Secure build pipelines, signed releases
- **Container Images**: Scan for vulnerabilities, use minimal base images
- **Infrastructure as Code**: Version control infrastructure configurations
- **Monitoring**: Implement comprehensive logging and alerting

## Security Features

The Semantic Browser includes several security features:

- **Input Validation**: All HTML and SPARQL inputs are validated
- **Rate Limiting**: API endpoints are rate-limited to prevent abuse
- **Authentication**: Bearer token authentication for API access
- **Sandboxing**: Optional seccomp sandboxing on Linux
- **Logging**: Comprehensive security event logging

## Contact

For security-related questions or concerns, please contact [INSERT CONTACT INFORMATION].

Thank you for helping keep the Semantic Browser and its users secure!