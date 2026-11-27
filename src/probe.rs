//! probe.rs — Upstream dependency activity and dormancy probe.
//! Queries live package registries (npm, crates.io, PyPI, Go proxy) in pure std::.
use crate::manifest::{Dependency, ManifestKind};
use std::process::Command;
use std::time::{SystemTime, UNIX_EPOCH};

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

pub struct HeuristicProbe;

impl ActivityProbe for HeuristicProbe {
    fn probe(&self, dep: &Dependency) -> ProbeResult {
        result_from_days(dep, dep.days_inactive.unwrap_or(0))
    }
}

pub fn probe_dependency(dep: &Dependency) -> ProbeResult {
    HeuristicProbe.probe(dep)
}

pub fn probe_for_kind(dep: &Dependency, kind: &ManifestKind) -> ProbeResult {
    if let Some(days) = dep.days_inactive {
        return result_from_days(dep, days);
    }
    match query_registry_days(&dep.name, kind) {
        Some(days) => result_from_days(dep, days),
        None => {
            eprintln!("vigil: warning: could not query registry for '{}'", dep.name);
            ProbeResult {
                name: dep.name.clone(),
                version: dep.version.clone(),
                days_dormant: 0,
                commit_velocity_annual: 1.0,
                is_unmaintained: false,
            }
        }
    }
}

fn result_from_days(dep: &Dependency, days: u32) -> ProbeResult {
    let velocity = if days > 365 {
        0.1
    } else {
        12.0 * (1.0 - (days as f32 / 365.0))
    };
    ProbeResult {
        name: dep.name.clone(),
        version: dep.version.clone(),
        days_dormant: days,
        commit_velocity_annual: velocity.max(0.0),
        is_unmaintained: days >= 365,
    }
}

fn is_safe_pkg_name(name: &str) -> bool {
    !name.is_empty()
        && name.bytes().all(|b| {
            b.is_ascii_alphanumeric() || matches!(b, b'@' | b'/' | b'.' | b'-' | b'_')
        })
}

fn escape_go_module(name: &str) -> String {
    let mut out = String::with_capacity(name.len());
    for c in name.chars() {
        if c.is_ascii_uppercase() {
            out.push('!');
            out.push(c.to_ascii_lowercase());
        } else {
            out.push(c);
        }
    }
    out
}

fn query_registry_days(name: &str, kind: &ManifestKind) -> Option<u32> {
    if !is_safe_pkg_name(name) {
        return None;
    }
    let now_days = SystemTime::now().duration_since(UNIX_EPOCH).ok()?.as_secs() / 86_400;
    let date = match kind {
        ManifestKind::PackageJson => {
            let enc = name.replace('/', "%2F");
            let url = format!("https://registry.npmjs.org/{}", enc);
            let body = curl_get(&url, &["-H", "Accept: application/vnd.npm.install-v1+json"])?;
            extract_date_for_key(&body, "modified")?.to_string()
        }
        ManifestKind::CargoLock | ManifestKind::CargoToml => {
            let url = format!("https://crates.io/api/v1/crates/{}", name);
            let body = curl_get(&url, &["-A", "studio2201-vigil/0.2"])?;
            extract_date_for_key(&body, "updated_at")?.to_string()
        }
        ManifestKind::RequirementsTxt => {
            let url = format!("https://pypi.org/pypi/{}/json", name);
            let body = curl_get(&url, &[])?;
            extract_max_date_for_key(&body, "upload_time_iso_8601")?.to_string()
        }
        ManifestKind::GoMod => {
            let esc = escape_go_module(name);
            let url = format!("https://proxy.golang.org/{}/@latest", esc);
            let body = curl_get(&url, &[])?;
            extract_date_for_key(&body, "Time")?.to_string()
        }
        ManifestKind::Unknown => return None,
    };
    days_since_iso_date(&date, now_days as i64)
}

fn curl_get(url: &str, extra_args: &[&str]) -> Option<String> {
    let out = Command::new("curl")
        .args(["-fsSL", "--max-time", "6"])
        .args(extra_args)
        .arg(url)
        .output()
        .ok()?;
    if !out.status.success() {
        return None;
    }
    Some(String::from_utf8_lossy(&out.stdout).into_owned())
}

fn is_yyyy_mm_dd(b: &[u8]) -> bool {
    b.len() >= 10
        && b[4] == b'-'
        && b[7] == b'-'
        && [0, 1, 2, 3, 5, 6, 8, 9].iter().all(|&i| b[i].is_ascii_digit())
}

pub fn extract_date_for_key<'a>(json: &'a str, key: &str) -> Option<&'a str> {
    let pat = format!("\"{}\"", key);
    let mut from = 0;
    while let Some(pos) = json[from..].find(&pat) {
        let after = &json[from + pos + pat.len()..];
        if let Some(rest) = after.trim_start().strip_prefix(':') {
            if let Some(val) = rest.trim_start().strip_prefix('"') {
                if is_yyyy_mm_dd(val.as_bytes()) {
                    return Some(&val[..10]);
                }
            }
        }
        from += pos + pat.len();
    }
    None
}

pub fn extract_max_date_for_key<'a>(json: &'a str, key: &str) -> Option<&'a str> {
    let pat = format!("\"{}\"", key);
    let mut from = 0;
    let mut best: Option<&'a str> = None;
    while let Some(pos) = json[from..].find(&pat) {
        let after = &json[from + pos + pat.len()..];
        if let Some(rest) = after.trim_start().strip_prefix(':') {
            if let Some(val) = rest.trim_start().strip_prefix('"') {
                if is_yyyy_mm_dd(val.as_bytes()) {
                    let candidate = &val[..10];
                    if best.map_or(true, |b| candidate > b) {
                        best = Some(candidate);
                    }
                }
            }
        }
        from += pos + pat.len();
    }
    best
}

pub fn days_from_civil(year: i64, month: u32, day: u32) -> Option<i64> {
    if !(1..=12).contains(&month) || !(1..=31).contains(&day) {
        return None;
    }
    let y = year - if month <= 2 { 1 } else { 0 };
    let era = if y >= 0 { y } else { y - 399 } / 400;
    let yoe = (y - era * 400) as u64;
    let m = month as u64;
    let m_adj = if m > 2 { m - 3 } else { m + 9 };
    let doy = (153 * m_adj + 2) / 5 + (day as u64) - 1;
    let doe = yoe * 365 + yoe / 4 - yoe / 100 + doy;
    Some(era * 146_097 + (doe as i64) - 719_468)
}

pub fn days_since_iso_date(date: &str, now_epoch_days: i64) -> Option<u32> {
    if !is_yyyy_mm_dd(date.as_bytes()) {
        return None;
    }
    let y: i64 = date[0..4].parse().ok()?;
    let m: u32 = date[5..7].parse().ok()?;
    let d: u32 = date[8..10].parse().ok()?;
    let then = days_from_civil(y, m, d)?;
    let diff = (now_epoch_days - then).max(0);
    Some(diff.min(u32::MAX as i64) as u32)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_civil_dates_and_delta() {
        assert_eq!(days_from_civil(1970, 1, 1), Some(0));
        assert_eq!(days_from_civil(2026, 11, 26), Some(20783));
        assert_eq!(days_since_iso_date("2026-11-24", 20783), Some(2));
        assert_eq!(days_since_iso_date("2025-11-26", 20783), Some(365));
    }

    #[test]
    fn test_json_date_and_go_escape() {
        assert_eq!(
            extract_date_for_key(r#"{"name":"cf","time":{"modified":"2026-11-25T16:40:39Z"}}"#, "modified"),
            Some("2026-11-25")
        );
        let pypi = r#"{"releases":{"0.1":[{"upload_time_iso_8601":"2020-01-02T00:00:00Z"}],"0.2":[{"upload_time_iso_8601":"2026-05-14T19:25:27Z"}]}}"#;
        assert_eq!(extract_max_date_for_key(pypi, "upload_time_iso_8601"), Some("2026-05-14"));
        assert_eq!(escape_go_module("github.com/BurntSushi/toml"), "github.com/!burnt!sushi/toml");
    }

    #[test]
    fn test_days_inactive_override_skips_network() {
        let dep = Dependency {
            name: "offline-pkg".into(), version: "1.0.0".into(), is_dev: false, days_inactive: Some(400),
        };
        let res = probe_for_kind(&dep, &ManifestKind::PackageJson);
        assert_eq!(res.days_dormant, 400);
        assert!(res.is_unmaintained);
    }
}
