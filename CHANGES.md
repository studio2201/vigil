# Changelog — vigil

All notable changes to this project are documented here.
Format: [Keep a Changelog](https://keepachangelog.com/) 1.1.0.
This project adheres to [Semantic Versioning](https://semver.org/).

## [Unreleased]

### Added
- (placeholder — next iteration's changes land here)

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
