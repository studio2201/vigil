//! manifest.rs — Manifest detection and extraction in pure std.
//! Supports package.json, Cargo.lock, Cargo.toml, requirements.txt, and go.mod.

use std::fmt;
use std::fs;
use std::path::Path;

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ManifestKind {
    PackageJson,
    CargoLock,
    CargoToml,
    RequirementsTxt,
    GoMod,
    Unknown,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Dependency {
    pub name: String,
    pub version: String,
    pub is_dev: bool,
    pub days_inactive: Option<u32>,
}

#[derive(Debug, Clone)]
pub struct Manifest {
    pub kind: ManifestKind,
    pub path: String,
    pub dependencies: Vec<Dependency>,
}

#[derive(Debug)]
pub enum ManifestError {
    Io(std::io::Error),
    ParseError(String),
}

impl fmt::Display for ManifestError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            ManifestError::Io(e) => write!(f, "I/O error: {}", e),
            ManifestError::ParseError(msg) => write!(f, "Parse error: {}", msg),
        }
    }
}

impl std::error::Error for ManifestError {}

impl From<std::io::Error> for ManifestError {
    fn from(e: std::io::Error) -> Self {
        ManifestError::Io(e)
    }
}

pub fn detect_kind(path: &Path) -> ManifestKind {
    match path.file_name().and_then(|n| n.to_str()).unwrap_or("") {
        "package.json" => ManifestKind::PackageJson,
        "Cargo.lock" => ManifestKind::CargoLock,
        "Cargo.toml" => ManifestKind::CargoToml,
        "requirements.txt" => ManifestKind::RequirementsTxt,
        "go.mod" => ManifestKind::GoMod,
        _ => ManifestKind::Unknown,
    }
}

pub fn parse_file(path: &Path) -> Result<Manifest, ManifestError> {
    let kind = detect_kind(path);
    let content = fs::read_to_string(path)?;
    let dependencies = parse_content(kind.clone(), &content)?;
    Ok(Manifest {
        kind,
        path: path.to_string_lossy().into_owned(),
        dependencies,
    })
}

pub fn parse_content(kind: ManifestKind, content: &str) -> Result<Vec<Dependency>, ManifestError> {
    match kind {
        ManifestKind::PackageJson => parse_package_json(content),
        ManifestKind::CargoLock => parse_cargo_lock(content),
        ManifestKind::CargoToml => parse_cargo_toml(content),
        ManifestKind::RequirementsTxt => parse_requirements_txt(content),
        ManifestKind::GoMod => parse_go_mod(content),
        ManifestKind::Unknown => Ok(Vec::new()),
    }
}

fn parse_package_json(content: &str) -> Result<Vec<Dependency>, ManifestError> {
    let mut deps = Vec::new();
    let (mut in_deps, mut in_dev) = (false, false);
    for line in content.lines() {
        let t = line.trim();
        if t.starts_with("\"dependencies\"") {
            in_deps = true; in_dev = false; continue;
        } else if t.starts_with("\"devDependencies\"") {
            in_deps = false; in_dev = true; continue;
        }
        if (in_deps || in_dev) && t.starts_with('}') {
            in_deps = false; in_dev = false; continue;
        }
        if (in_deps || in_dev) && t.contains(':') {
            let p: Vec<&str> = t.splitn(2, ':').collect();
            let k = p[0].trim().trim_matches('"');
            let v = p[1].trim().trim_matches(',').trim().trim_matches('"');
            if !k.is_empty() {
                deps.push(Dependency {
                    name: k.to_string(), version: v.to_string(), is_dev: in_dev, days_inactive: None,
                });
            }
        }
    }
    Ok(deps)
}

fn parse_cargo_lock(content: &str) -> Result<Vec<Dependency>, ManifestError> {
    let mut deps = Vec::new();
    let (mut cur_name, mut cur_ver, mut has_source) = (None, None, false);
    for line in content.lines() {
        let t = line.trim();
        if t == "[[package]]" {
            if has_source {
                if let (Some(n), Some(v)) = (cur_name, cur_ver) {
                    deps.push(Dependency {
                        name: n, version: v, is_dev: false, days_inactive: None,
                    });
                }
            }
            cur_name = None;
            cur_ver = None;
            has_source = false;
        } else if let Some(n) = t.strip_prefix("name = ") {
            cur_name = Some(n.trim_matches('"').to_string());
        } else if let Some(v) = t.strip_prefix("version = ") {
            cur_ver = Some(v.trim_matches('"').to_string());
        } else if t.starts_with("source = ") {
            has_source = true;
        }
    }
    if has_source {
        if let (Some(n), Some(v)) = (cur_name, cur_ver) {
            deps.push(Dependency {
                name: n, version: v, is_dev: false, days_inactive: None,
            });
        }
    }
    Ok(deps)
}

fn parse_cargo_toml(content: &str) -> Result<Vec<Dependency>, ManifestError> {
    let mut deps = Vec::new();
    let (mut in_deps, mut in_dev) = (false, false);
    for line in content.lines() {
        let t = line.trim();
        if t == "[dependencies]" {
            in_deps = true; in_dev = false; continue;
        } else if t == "[dev-dependencies]" {
            in_deps = false; in_dev = true; continue;
        } else if t.starts_with('[') {
            in_deps = false; in_dev = false; continue;
        }
        if (in_deps || in_dev) && !t.is_empty() && !t.starts_with('#') {
            if let Some(pos) = t.find('=') {
                let name = t[..pos].trim().to_string();
                let ver = t[pos + 1..].trim().trim_matches('"').to_string();
                deps.push(Dependency { name, version: ver, is_dev: in_dev, days_inactive: None });
            }
        }
    }
    Ok(deps)
}

fn parse_requirements_txt(content: &str) -> Result<Vec<Dependency>, ManifestError> {
    let mut deps = Vec::new();
    for line in content.lines() {
        let t = line.trim();
        if t.is_empty() || t.starts_with('#') { continue; }
        let p: Vec<&str> = t.split(&['=', '>', '<', '~', '!'][..]).collect();
        let name = p[0].trim().to_string();
        let ver = if p.len() > 1 { t[name.len()..].trim().to_string() } else { "*".into() };
        deps.push(Dependency { name, version: ver, is_dev: false, days_inactive: None });
    }
    Ok(deps)
}

fn parse_go_mod(content: &str) -> Result<Vec<Dependency>, ManifestError> {
    let mut deps = Vec::new();
    let mut in_req = false;
    for line in content.lines() {
        let t = line.trim();
        if t.starts_with("require (") { in_req = true; continue; }
        if in_req && t == ")" { in_req = false; continue; }
        if in_req || t.starts_with("require ") {
            let s = if t.starts_with("require ") { t.strip_prefix("require ").unwrap_or("") } else { t };
            let p: Vec<&str> = s.split_whitespace().collect();
            if p.len() >= 2 {
                deps.push(Dependency {
                    name: p[0].into(), version: p[1].into(), is_dev: false, days_inactive: None,
                });
            }
        }
    }
    Ok(deps)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_cargo_lock_root_exclusion() {
        let lock = r#"
version = 4

[[package]]
name = "studio2201"
version = "0.1.7"

[[package]]
name = "serde"
version = "1.0.197"
source = "registry+https://github.com/rust-lang/crates.io-index"
"#;
        let deps = parse_cargo_lock(lock).unwrap();
        assert_eq!(deps.len(), 1);
        assert_eq!(deps[0].name, "serde");
        assert_eq!(deps[0].version, "1.0.197");
    }

    #[test]
    fn test_cargo_lock_pure_root_empty() {
        let lock = r#"
[[package]]
name = "studio2201"
version = "0.1.7"
"#;
        let deps = parse_cargo_lock(lock).unwrap();
        assert!(deps.is_empty());
    }

    #[test]
    fn test_cargo_lock_external_included() {
        let lock = r#"
[[package]]
name = "tokio"
version = "1.38.0"
source = "registry+https://github.com/rust-lang/crates.io-index"
"#;
        let deps = parse_cargo_lock(lock).unwrap();
        assert_eq!(deps.len(), 1);
        assert_eq!(deps[0].name, "tokio");
        assert_eq!(deps[0].version, "1.38.0");
    }
}
