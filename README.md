# Vigil

**Supply-chain dormancy scanner.** Reads your project's manifest and scores each dependency by how long it has been since the upstream moved. Emits `SUPPLY-CHAIN.md` and a `vigil.svg` badge.

```
vigil scan
```

**Status:** pre-release scaffold (2026-09-17). No source code yet.

## What it does

1. Detect package manager from `package.json` / `Cargo.lock` / `pyproject.toml` / `go.mod` / `pom.xml` / `composer.json` / `pubspec.yaml` / `Package.swift`.
2. Look up each dependency's last-commit timestamp and rank it against a dormancy heuristic.
3. Emit `SUPPLY-CHAIN.md` ranking deps by their dormancy index.
4. Emit `vigil.svg` — a badge you can drop in your README.

## Why

- One input, one data source, one output. The viral unit is the smallest.
- Same procurement story as Sigstore / Socket / Snyk — Vigil owns the "supply-chain dormancy" slice they don't.
- Pure Rust, `std::` only. Zero crates.io.

## License

Apache-2.0.
