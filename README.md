# Vigil

[![studio2201 Suite](https://img.shields.io/badge/studio2201-5%2F5%20Verified-2f6f5e?logo=shield)](https://studio2201.com/agents#badges)
[![Release](https://img.shields.io/badge/version-v0.2.11-blue.svg)](https://github.com/studio2201/vigil/releases)
[![License](https://img.shields.io/badge/license-Apache--2.0-blue.svg)](LICENSE)

**Supply-chain dormancy scanner.** Reads your project's manifest and scores each dependency
by how long it has been since the upstream moved. Emits `SUPPLY-CHAIN.md` and a `vigil.svg` badge.

## Why This Action Is Needed

### The Abandoned Dependency Vector & CVE Precedents
Modern software applications depend on hundreds of nested third-party packages. When an upstream
dependency quietly goes dormant, it becomes an unmonitored attack surface. Vulnerabilities remain
unpatched, expired maintainer domains are hijacked, and malicious actors take over abandoned
libraries through social engineering (e.g. `event-stream`, `colors.js`, and XZ Utils CVE-2024-3094).
- **[Harvard / Linux Foundation Census III](https://www.linuxfoundation.org/research/census-iii-open-source-software)**:
  Research proves over 80% of production code consists of open-source libraries, with systemic fragility concentrated
  in single-maintainer, dormant dependencies.
- **[CISA Open Source Security Roadmap](https://www.cisa.gov/resources-tools/resources/open-source-software-security-roadmap)**:
  Federal mandate prioritizing discovery and automated remediation of unmaintained dependencies across enterprise software.
- **Automated CI Gates vs Manual Compliance**: Manual auditing across deep dependency trees is humanly impossible
  to sustain. Developers under deadline pressure rarely check commit recency when adding dependencies.
  Vigil provides an automated, non-bypassable CI gate that fails closed when dormant packages breach policy limits.

## Autonomous Agent Integration

Integrate Vigil directly using your AI coding assistant or copy the workflow below.

### Prompt for your AI Agent

Copy and paste this prompt to Cursor, Claude Code, Copilot Workspace, or Devin:

```text
Add a GitHub Actions workflow to this repository at .github/workflows/studio2201.yml using studio2201/studio2201@master.
Trigger on pull_request and push to master/main, and workflow_dispatch.
Configure concurrency with cancel-in-progress on pull requests.
Set permissions to contents: read, checkout with fetch-depth: 0, run studio2201/studio2201@master with tools: 'all' and fail-on: 'block'.
Retain audit findings using actions/upload-artifact@v4 with if: always().
Finally, add the Option 1 Single Suite Badge or Option 2 Governance Scorecard to README.md.
```

### GitHub Actions Workflow

Commit this complete, production-ready workflow at `.github/workflows/studio2201.yml`:

```yaml
name: studio2201 Security Gate
on:
  push:
    branches: [ master, main ]
  pull_request:
    branches: [ master, main ]
  workflow_dispatch:

concurrency:
  group: ${{ github.workflow }}-${{ github.ref }}
  cancel-in-progress: ${{ github.event_name == 'pull_request' }}

permissions:
  contents: read

jobs:
  security-gate:
    name: studio2201 Security Gate
    runs-on: ubuntu-latest
    steps:
      - name: Checkout repository
        uses: actions/checkout@v4
        with:
          fetch-depth: 0

      - name: Run studio2201 Security Gate
        uses: studio2201/studio2201@master
        with:
          tools: 'all'
          fail-on: 'block'

      - name: Retain Audit Findings
        uses: actions/upload-artifact@v4
        if: always()
        with:
          name: studio2201-audit-findings
          path: |
            *_report.md
          if-no-files-found: ignore
```

## How It Works Under the Hood

1. **Pure `std::` Multi-Manifest Parser (`src/manifest.rs`)**: Automatically extracts dependencies from
   `Cargo.lock`, `Cargo.toml`, `package.json`, `requirements.txt`, and `go.mod` without external parsing crates.
2. **Activity & Latency Probe (`src/probe.rs`)**: Evaluates upstream commit velocity and days since last release
   against dormancy heuristics.
3. **Composite Risk Scoring (`src/score.rs`)**: Computes weighted risk (0.0 to 100.0) across Low, Medium, High,
   and Critical thresholds.
4. **Automated CI Policy Gate (`src/policy.rs`)**: Halts pull requests (`vigil policy check --max-dormancy <DAYS>`)
   when abandoned dependencies exceed compliance thresholds.
5. **Native SVG Badge Generator (`src/report.rs`)**: Emits raw SVG XML health badges directly on disk via
   `vigil badge -o vigil.svg`.

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
