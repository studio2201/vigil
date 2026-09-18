# Vigil

[![CI](https://github.com/studio2201/vigil/actions/workflows/ci.yml/badge.svg?branch=master)](https://github.com/studio2201/vigil/actions/workflows/ci.yml)
[![Release](https://img.shields.io/badge/version-v0.2.4-blue.svg)](https://github.com/studio2201/vigil/releases)
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

## Quick Start

```bash
# Install via studio2201 installer
curl -fsSL https://studio2201.com/install.sh | sh -s vigil

# Scan dependencies in current directory
vigil scan

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

## What it does

1. Detect package manager from `package.json` / `Cargo.lock` / `pyproject.toml` / `go.mod` / `pom.xml` / `composer.json` / `pubspec.yaml` / `Package.swift`.
2. Look up each dependency's last-commit timestamp and rank it against a dormancy heuristic.
3. Emit `SUPPLY-CHAIN.md` ranking deps by their dormancy index.
4. Emit `vigil.svg` — a badge you can drop in your README.

## CLI Commands

- `vigil scan [path]` — Scan dependency manifests
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

## Why

- One input, one data source, one output. The viral unit is the smallest.
- Same procurement story as Sigstore / Socket / Snyk — Vigil owns the "supply-chain dormancy" slice they don't.
- Pure Rust, `std::` only. Zero crates.io dependencies. Strictly <= 256 LOC per source file.

## License

Apache-2.0.
