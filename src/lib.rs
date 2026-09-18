//! lib.rs — Vigil: supply-chain dormancy scanner.
//! Written in pure std:: Rust (Edition 2021). Zero external dependencies.

pub mod manifest;
pub mod policy;
pub mod probe;
pub mod report;
pub mod score;

pub use manifest::{Dependency, Manifest, ManifestKind};
pub use policy::{Policy, PolicyVerdict};
pub use probe::{ProbeResult, HeuristicProbe, ActivityProbe};
pub use report::{emit_badge_svg, emit_json, emit_markdown};
pub use score::{RiskLevel, ScoreError, ScoreReport, ScoredDependency};

/// Scans a manifest against the given policy and produces a score report.
pub fn scan(manifest: &Manifest, _policy: &Policy) -> Result<ScoreReport, score::ScoreError> {
    score::compute(manifest)
}
