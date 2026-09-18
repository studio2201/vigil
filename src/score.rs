//! score.rs — Composite risk calculation per dependency.
//! Scores dormancy from 0.0 (active) to 100.0 (abandoned / high risk).

use crate::manifest::Manifest;
use crate::probe::{probe_dependency, ProbeResult};
use std::fmt;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum RiskLevel {
    Low,
    Medium,
    High,
    Critical,
}

impl fmt::Display for RiskLevel {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            RiskLevel::Low => write!(f, "LOW"),
            RiskLevel::Medium => write!(f, "MEDIUM"),
            RiskLevel::High => write!(f, "HIGH"),
            RiskLevel::Critical => write!(f, "CRITICAL"),
        }
    }
}

#[derive(Debug, Clone)]
pub struct ScoredDependency {
    pub name: String,
    pub version: String,
    pub is_dev: bool,
    pub days_dormant: u32,
    pub commit_velocity: f32,
    pub risk_score: f32,
    pub risk_level: RiskLevel,
}

#[derive(Debug, Clone)]
pub struct ScoreReport {
    pub manifest_path: String,
    pub total_deps: usize,
    pub average_score: f32,
    pub critical_count: usize,
    pub high_count: usize,
    pub medium_count: usize,
    pub low_count: usize,
    pub entries: Vec<ScoredDependency>,
}

#[derive(Debug)]
pub struct ScoreError(pub String);

impl fmt::Display for ScoreError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "Score calculation error: {}", self.0)
    }
}

impl std::error::Error for ScoreError {}

pub fn calculate_dep_score(probe: &ProbeResult, is_dev: bool) -> (f32, RiskLevel) {
    let dormancy_factor = (probe.days_dormant as f32 / 365.0).min(2.0);
    let velocity_penalty = if probe.commit_velocity_annual < 1.0 {
        25.0
    } else if probe.commit_velocity_annual < 5.0 {
        10.0
    } else {
        0.0
    };

    let mut raw_score = (dormancy_factor * 40.0) + velocity_penalty;
    if probe.is_unmaintained {
        raw_score += 20.0;
    }
    if is_dev {
        raw_score *= 0.75; // Dev dependencies present lower operational supply-chain risk
    }

    let score = raw_score.clamp(0.0, 100.0);
    let level = if score >= 75.0 {
        RiskLevel::Critical
    } else if score >= 50.0 {
        RiskLevel::High
    } else if score >= 25.0 {
        RiskLevel::Medium
    } else {
        RiskLevel::Low
    };

    (score, level)
}

pub fn compute(manifest: &Manifest) -> Result<ScoreReport, ScoreError> {
    let mut entries = Vec::with_capacity(manifest.dependencies.len());
    let mut total_score = 0.0;
    let mut critical_count = 0;
    let mut high_count = 0;
    let mut medium_count = 0;
    let mut low_count = 0;

    for dep in &manifest.dependencies {
        let probe = probe_dependency(dep);
        let (score, level) = calculate_dep_score(&probe, dep.is_dev);

        match level {
            RiskLevel::Critical => critical_count += 1,
            RiskLevel::High => high_count += 1,
            RiskLevel::Medium => medium_count += 1,
            RiskLevel::Low => low_count += 1,
        }
        total_score += score;

        entries.push(ScoredDependency {
            name: dep.name.clone(),
            version: dep.version.clone(),
            is_dev: dep.is_dev,
            days_dormant: probe.days_dormant,
            commit_velocity: probe.commit_velocity_annual,
            risk_score: score,
            risk_level: level,
        });
    }

    let total = entries.len();
    let average_score = if total > 0 {
        total_score / (total as f32)
    } else {
        0.0
    };

    Ok(ScoreReport {
        manifest_path: manifest.path.clone(),
        total_deps: total,
        average_score,
        critical_count,
        high_count,
        medium_count,
        low_count,
        entries,
    })
}
