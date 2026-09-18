//! policy.rs — CI gating against dormancy and risk thresholds.
//! Evaluates score reports against compliance rules.

use crate::score::ScoreReport;

#[derive(Debug, Clone)]
pub struct Policy {
    pub max_average_score: f32,
    pub max_critical_deps: usize,
    pub max_high_deps: usize,
    pub max_dormancy_days: u32,
    pub fail_on_critical: bool,
}

impl Default for Policy {
    fn default() -> Self {
        Policy {
            max_average_score: 45.0,
            max_critical_deps: 0,
            max_high_deps: 10,
            max_dormancy_days: 365,
            fail_on_critical: true,
        }
    }
}

#[derive(Debug, Clone)]
pub struct PolicyVerdict {
    pub passed: bool,
    pub violations: Vec<String>,
}

pub fn evaluate(report: &ScoreReport, policy: &Policy) -> PolicyVerdict {
    let mut violations = Vec::new();

    if policy.fail_on_critical && report.critical_count > policy.max_critical_deps {
        violations.push(format!(
            "Critical dependencies count {} exceeds threshold {}",
            report.critical_count, policy.max_critical_deps
        ));
    }

    if report.high_count > policy.max_high_deps {
        violations.push(format!(
            "High risk dependencies count {} exceeds threshold {}",
            report.high_count, policy.max_high_deps
        ));
    }

    if report.average_score > policy.max_average_score {
        violations.push(format!(
            "Average risk score {:.1} exceeds threshold {:.1}",
            report.average_score, policy.max_average_score
        ));
    }

    for dep in &report.entries {
        if dep.days_dormant > policy.max_dormancy_days {
            violations.push(format!(
                "Dependency '{}' ({} days dormant) exceeds max dormancy of {} days",
                dep.name, dep.days_dormant, policy.max_dormancy_days
            ));
        }
    }

    let passed = violations.is_empty();
    PolicyVerdict { passed, violations }
}
