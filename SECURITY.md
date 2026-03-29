# Security Policy

## Supported Versions

| Version | Supported |
|---------|-----------|
| 0.1.x   | Yes       |

## Scope

Vanaspati is a library crate performing mathematical computations. The primary security concerns are:

- **Denial of service** via malicious inputs causing excessive computation or memory use
- **Supply chain** — dependency vulnerabilities (monitored via `cargo audit` and `cargo deny`)
- **Numerical safety** — NaN/infinity propagation from edge-case inputs

## Reporting a Vulnerability

Please report security issues privately via GitHub Security Advisories:

1. Go to the [Security tab](https://github.com/MacCracken/vanaspati/security/advisories)
2. Click "Report a vulnerability"
3. Provide a description, reproduction steps, and impact assessment

You will receive a response within 7 days. Please do not open public issues for security vulnerabilities.
