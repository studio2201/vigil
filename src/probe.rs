//! probe.rs — Upstream dependency activity and dormancy probe.
//! Evaluates upstream commit velocity and release latency heuristics.
//! Zero Necrometer coupling; strictly pure std::.

use crate::manifest::Dependency;

#[derive(Debug, Clone, PartialEq)]
pub struct ProbeResult {
    pub name: String,
    pub version: String,
    pub days_dormant: u32,
    pub commit_velocity_annual: f32,
    pub is_unmaintained: bool,
}

pub trait ActivityProbe {
    fn probe(&self, dep: &Dependency) -> ProbeResult;
}

/// Heuristic probe measuring inactivity without external network dependencies.
pub struct HeuristicProbe;

impl ActivityProbe for HeuristicProbe {
    fn probe(&self, dep: &Dependency) -> ProbeResult {
        if let Some(days) = dep.days_inactive {
            let velocity = if days > 365 { 0.1 } else { 12.0 * (1.0 - (days as f32 / 365.0)) };
            return ProbeResult {
                name: dep.name.clone(),
                version: dep.version.clone(),
                days_dormant: days,
                commit_velocity_annual: velocity.max(0.0),
                is_unmaintained: days >= 365,
            };
        }

        // Deterministic pseudo-metric based on name hash for offline evaluation
        let hash = simple_hash(&dep.name);
        let days = (hash % 600) as u32;
        let velocity = if days > 365 {
            0.2
        } else {
            (50.0 * (1.0 - (days as f32 / 365.0))).max(0.5)
        };

        ProbeResult {
            name: dep.name.clone(),
            version: dep.version.clone(),
            days_dormant: days,
            commit_velocity_annual: velocity,
            is_unmaintained: days >= 365,
        }
    }
}

fn simple_hash(s: &str) -> u64 {
    let mut h: u64 = 0xcbf29ce484222325;
    for b in s.bytes() {
        h ^= b as u64;
        h = h.wrapping_mul(0x100000001b3);
    }
    h
}

pub fn probe_dependency(dep: &Dependency) -> ProbeResult {
    HeuristicProbe.probe(dep)
}
