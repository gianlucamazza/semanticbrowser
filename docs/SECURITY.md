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

- Keep your dependencies updated
- Use strong, unique API secrets in production
- Regularly review and rotate authentication tokens
- Monitor logs for suspicious activity
- Use the latest stable version of the software

### For Contributors

- Follow secure coding practices
- Validate all inputs and sanitize outputs
- Use parameterized queries for database operations
- Implement proper authentication and authorization
- Regularly update dependencies
- Run security scans in CI/CD pipelines

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