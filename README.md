# Vigil

[![CI](https://github.com/studio2201/vigil/actions/workflows/ci.yml/badge.svg?branch=master)](https://github.com/studio2201/vigil/actions/workflows/ci.yml)
[![Release](https://img.shields.io/badge/version-v0.2.5-blue.svg)](https://github.com/studio2201/vigil/releases)
[![License](https://img.shields.io/badge/license-Apache--2.0-blue.svg)](LICENSE)
[![Pure std::](https://img.shields.io/badge/pure-std%3A%3A-success.svg)](https://studio2201.com)
[![Reproducible](https://img.shields.io/badge/reproducible-OK-brightgreen.svg)](tools/dev/repro.sh)
[![Max LOC](https://img.shields.io/badge/max%20LOC-%E2%89%A4256-brightgreen.svg)](https://studio2201.com)

[![Vigil Supply-Chain](vigil.svg)](https://studio2201.com/vigil)
[![Dormancy Index](https://img.shields.io/badge/dormancy%20index-0%2F100-brightgreen.svg)](https://studio2201.com/vigil)
[![Supply-Chain Health](https://img.shields.io/badge/supply--chain-healthy-2f6f5e.svg)](https://studio2201.com/vigil)
[![Manifests](https://img.shields.io/badge/manifests-Cargo%20%7C%20npm%20%7C%20pip%20%7C%20go-informational.svg)](https://studio2201.com/vigil)
[![Policy Gate](https://img.shields.io/badge/policy%20gate-PASSED-brightgreen.svg)](https://studio2201.com/vigil)

**Supply-chain dormancy scanner.** Reads your project's manifest and scores each dependency by how long it has been since the upstream moved. Emits `SUPPLY-CHAIN.md` and a `vigil.svg` badge.

## Why This Matters & Authoritative Research

### 1. The Abandoned Dependency Vector
Modern applications depend on hundreds of nested third-party packages. When an upstream dependency quietly goes dormant, it becomes an unmonitored attack surface. Vulnerabilities remain unpatched, expired maintainer domains are seized, and malicious actors take over abandoned packages through social engineering or package maintainer abandonment (e.g. `event-stream`, `colors.js`, and XZ Utils CVE-2024-3094).
- **[CISA Open Source Software Security Roadmap](https://www.cisa.gov/resources-tools/resources/open-source-software-security-roadmap)**: Federal cybersecurity roadmap prioritizing discovery and remediation of unmaintained, single-maintainer dependencies across critical infrastructure.
- **[Harvard / Linux Foundation Census III of Open Source](https://www.linuxfoundation.org/research/census-iii-open-source-software)**: Research revealing that over 80% of production codebases are comprised of open-source dependencies, with systemic fragility concentrated in dormant libraries.
- **[OpenSSF Scorecard Project](https://securityscorecards.dev/)**: Identifies dependency maintenance latency and commit frequency as primary security risk indicators.

## How It Works Under the Hood

1. **Pure `std::` Multi-Manifest Parser (`src/manifest.rs`)**: Automatically detects and extracts dependencies from `Cargo.lock`, `Cargo.toml`, `package.json`, `requirements.txt`, and `go.mod` without external parsing crates.
2. **Activity & Latency Probe (`src/probe.rs`)**: Evaluates upstream commit velocity and days since last release against dormancy heuristics.
3. **Composite Risk Scoring (`src/score.rs`)**: Computes weighted risk (0.0 to 100.0) across Low, Medium, High, and Critical thresholds.
4. **Automated CI Policy Gate (`src/policy.rs`)**: Halts pull requests (`vigil policy check --max-dormancy <DAYS>`) when abandoned dependencies exceed compliance thresholds.
5. **Native SVG Badge Generator (`src/report.rs`)**: Emits raw SVG XML health badges directly on disk via `vigil badge -o vigil.svg`.

## Quick Start

```bash
# Install via studio2201 installer
curl -fsSL https://studio2201.com/install.sh | sh -s vigil

# Scan dependencies in current directory
vigil scan

# Enforce dormancy threshold policy (fails if dep > 180 days dormant)
vigil policy check --max-dormancy 180

# Generate native SVG badge for README
vigil badge -o vigil.svg

# Run system diagnostics
vigil doctor
```

## GitHub Action Usage

Use Vigil in your GitHub Actions workflows to audit dependencies on pull requests:

```yaml
- name: Vigil Supply-Chain Dormancy Scan
  uses: studio2201/vigil@master
  with:
    path: '.'
    format: 'markdown'
```

## CLI Commands

- `vigil scan [path]` — Scan dependency manifests
- `vigil policy check [path] --max-dormancy <N>` — Gate CI on dormancy limits
- `vigil badge [path] -o vigil.svg` — Generate SVG health badge
- `vigil doctor` — Diagnose environment, XDG directories, PATH, and toolchain
- `vigil update` / `vigil upgrade` — Check for updates and self-upgrade
- `vigil -h` / `--help` — Show help
- `vigil -V` / `--version` — Show version

## Badges & Status

Embed Vigil's real-time dormancy badge directly in your project README:

```markdown
<!-- Native Vigil SVG badge generated via `vigil badge` -->
[![Vigil](vigil.svg)](https://studio2201.com/vigil)

<!-- Shields.io Dormancy Score -->
[![Dormancy Index](https://img.shields.io/badge/dormancy-healthy-2f6f5e.svg)](https://studio2201.com/vigil)
```

## License

Apache-2.0.
