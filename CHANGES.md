# Changelog — vigil

All notable changes to this project are documented here.
Format: [Keep a Changelog](https://keepachangelog.com/) 1.1.0.
This project adheres to [Semantic Versioning](https://semver.org/).

## [Unreleased]

## [0.2.7] — 2026-09-19

### Changed
- Updated README header to unified Option 1 Single Suite Badge.
- Bumped version to 0.2.7.

## [0.2.6] — 2026-09-19

### Added
- Native composite GitHub Action (`action.yml`) with sub-2s fast bootstrap via `install.sh`.
- Native `$GITHUB_STEP_SUMMARY` Markdown scorecard and audit reporting.
- Human-first 'Why This Action Is Needed' rationale with CISA/OpenSSF citations.
- Agent-first 'Prompt for your AI Agent' blocks and drop-in CI workflow YAML.

## [0.2.5] — 2026-09-18

### Added
- Expanded documentation in README with authoritative problem descriptions and citations:
  - CISA Open Source Software Security Roadmap prioritizing unmaintained dependencies.
  - Harvard / Linux Foundation Census III research on open-source supply chain fragility.
  - OpenSSF Scorecards evaluation of dependency maintenance latency and risks.
- Added comprehensive "How It Works Under the Hood" architectural breakdown.
- Upgraded release metadata and diagnostic baseline.

## [0.2.4] — 2026-09-18

### Added
- Tool-specific badges on README: Dormancy Index, Supply-Chain Health, Manifest Support, and Policy Gate.
- Detailed README section on embedding native `vigil.svg` and shields.io dormancy badges.
- Upgraded release metadata and diagnostic baseline.

## [0.2.0] — 2026-09-18

### Added
- Working pure `std::` Rust implementation of Vigil supply-chain dormancy scanner.
- Pure `std::` multi-manifest detection and extraction for `package.json`, `Cargo.lock`, `Cargo.toml`, `requirements.txt`, and `go.mod`.
- Upstream activity heuristic probe evaluating commit velocity and release latency.
- Composite risk scoring with strict inequality gating and `SUPPLY-CHAIN.md`, JSON, and SVG badge generation.
- Standardized CLI flags: `-h/--help`, `-V/--version`, `--format`, `-o/--output`, `-q/--quiet`, `-v/--verbose`.
- Performance test verifying 10,000 synthetic dependencies scanned in < 1ms (budget 800ms).

## [0.1.2] — 2026-09-17

### Changed
- `README.md` rewritten to drop openOODA substrate references
  (seance/opm/bb/ML-DSA-65 mentions removed where they described the
  implementation path; dormancy-via-last-commit heuristic kept as the
  *behavior*).

## [0.1.1] — 2026-09-17

### Added
- §15 threat model: `docs/threat-model.md` (vigil-specific adversary:
  malicious or unmaintained registry plus lockfile attacker)
- §16 reproducible builds: `tools/dev/repro.sh` with per-host baselines
- §17 security disclosure: `SECURITY.md` pointing at GHSA tab
- §18 performance budgets: `tools/perf/budget.md` and
  `tests/integration.rs::perf_vigil_scan_within_budget` (std::time, median-of-5)

### Notes
- Pre-1.0.0: GHSA-only security advisories; CVEs reserved for 1.0.0+
- Budget defaults are first-cut placeholders, not aspirational

## [0.1.0] — 2026-09-17

### Added
- Initial scaffold: Apache-2.0 LICENSE, README, .gitignore
- One question (§0): "How much of my project's risk comes from dead libraries?"
