# Security Policy

## Supported Versions
| Version | Supported          |
|---------|--------------------|
| 0.x.y   | :white_check_mark: |
| < 0.1   | :x:                |

## Reporting a Vulnerability
- Open a private security advisory at
  https://github.com/studio2201/vigil/security/advisories/new
- Response window: 14 days for triage, 90 days for coordinated disclosure
- We credit the reporter in the fix commit unless they request otherwise
- Severity is assessed via CVSS v3.1 base score; CVEs are issued via
  MITRE if score ≥ 4.0 (post-1.0.0 only)
- Pre-1.0.0 advisories use GHSA only (Severity: Low / Medium / High);
  CVEs are reserved for stable releases
- Disclosure scope is bounded by [docs/threat-model.md](docs/threat-model.md)
