//! doctor.rs — System & Environment Diagnostics for vigil.
use crate::cli::OutputFormat;
use crate::xdg;
use std::env;
use std::fs;
use std::path::{Path, PathBuf};
use std::process::Command;

#[derive(Debug)]
pub struct DoctorCheck {
    pub name: String,
    pub status: String,
    pub detail: String,
    pub remediation: Option<String>,
}

fn check_binary() -> DoctorCheck {
    let exe = env::current_exe().unwrap_or_else(|_| PathBuf::from("vigil"));
    if !exe.exists() {
        return DoctorCheck {
            name: "binary_integrity".into(),
            status: "error".into(),
            detail: format!("Binary not found at {}", exe.display()),
            remediation: Some("Reinstall vigil via install.sh".into()),
        };
    }
    let size = fs::metadata(&exe).map(|m| m.len()).unwrap_or(0);
    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        let mode = fs::metadata(&exe).map(|m| m.permissions().mode()).unwrap_or(0);
        if mode & 0o111 == 0 {
            return DoctorCheck {
                name: "binary_integrity".into(),
                status: "error".into(),
                detail: format!("Binary at {} is not executable (mode {:o})", exe.display(), mode),
                remediation: Some(format!("chmod 0755 {}", exe.display())),
            };
        }
    }
    DoctorCheck {
        name: "binary_integrity".into(),
        status: "ok".into(),
        detail: format!("{} ({} bytes, executable)", exe.display(), size),
        remediation: None,
    }
}

fn check_dir(name: &str, dir: &Path) -> DoctorCheck {
    if dir.exists() {
        DoctorCheck {
            name: name.into(),
            status: "ok".into(),
            detail: format!("{} (accessible)", dir.display()),
            remediation: None,
        }
    } else {
        match fs::create_dir_all(dir) {
            Ok(_) => DoctorCheck {
                name: name.into(),
                status: "ok".into(),
                detail: format!("{} (created successfully)", dir.display()),
                remediation: None,
            },
            Err(e) => DoctorCheck {
                name: name.into(),
                status: "warn".into(),
                detail: format!("{} (cannot create: {})", dir.display(), e),
                remediation: Some(format!("mkdir -p {}", dir.display())),
            },
        }
    }
}

fn check_toolchain() -> DoctorCheck {
    let git = Command::new("git").arg("--version").output().is_ok();
    let curl = Command::new("curl").arg("--version").output().is_ok();
    let wget = Command::new("wget").arg("--version").output().is_ok();
    if git && (curl || wget) {
        let net = if curl { "curl" } else { "wget" };
        DoctorCheck {
            name: "toolchain".into(),
            status: "ok".into(),
            detail: format!("git found, {} found", net),
            remediation: None,
        }
    } else {
        let mut missing = Vec::new();
        if !git { missing.push("git"); }
        if !curl && !wget { missing.push("curl/wget"); }
        DoctorCheck {
            name: "toolchain".into(),
            status: "warn".into(),
            detail: format!("Missing host tools: {}", missing.join(", ")),
            remediation: Some("Install git and curl using your package manager".into()),
        }
    }
}

fn check_path() -> DoctorCheck {
    let bin_dir = xdg::bin_dir();
    let path_var = env::var("PATH").unwrap_or_default();
    let in_path = env::split_paths(&path_var).any(|p| p == bin_dir);
    if in_path {
        DoctorCheck {
            name: "path_accessibility".into(),
            status: "ok".into(),
            detail: format!("{} is in PATH", bin_dir.display()),
            remediation: None,
        }
    } else {
        DoctorCheck {
            name: "path_accessibility".into(),
            status: "warn".into(),
            detail: format!("{} is not in PATH", bin_dir.display()),
            remediation: Some(format!("export PATH=\"{}:$PATH\"", bin_dir.display())),
        }
    }
}

pub fn run_doctor(app: &str, version: &str, fmt: OutputFormat) -> (i32, String) {
    let checks = vec![
        check_binary(),
        check_dir("config_directory", &xdg::config_dir(app)),
        check_dir("data_directory", &xdg::data_dir(app)),
        check_dir("state_directory", &xdg::state_dir(app)),
        check_dir("cache_directory", &xdg::cache_dir(app)),
        check_toolchain(),
        check_path(),
    ];
    let healthy = !checks.iter().any(|c| c.status == "error");

    let mut out = String::new();
    if fmt == OutputFormat::Json {
        let mut j = format!("{{\"app\":\"{}\",\"version\":\"{}\",\"healthy\":{},\"checks\":[",
            app, version, healthy);
        for (idx, c) in checks.iter().enumerate() {
            if idx > 0 { j.push(','); }
            let rem = match &c.remediation {
                Some(r) => format!("\"{}\"", r.replace('\"', "\\\"")),
                None => "null".into(),
            };
            j.push_str(&format!(
                "{{\"name\":\"{}\",\"status\":\"{}\",\"detail\":\"{}\",\"remediation\":{}}}",
                c.name, c.status, c.detail.replace('\"', "\\\""), rem
            ));
        }
        j.push_str("]}\n");
        out = j;
    } else {
        use std::fmt::Write;
        let _ = writeln!(out, "{} doctor — System & Environment Diagnostics", app);
        for c in &checks {
            let symbol = match c.status.as_str() {
                "ok" => "[✓]",
                "warn" => "[!]",
                _ => "[✗]",
            };
            let _ = writeln!(out, "{} {}: {}", symbol, c.name, c.detail);
            if let Some(ref r) = c.remediation {
                let _ = writeln!(out, "    Remediation: {}", r);
            }
        }
        let passed = checks.iter().filter(|c| c.status == "ok").count();
        let _ = writeln!(out, "\nStatus: {} ({}/{} checks ok)",
            if healthy { "HEALTHY" } else { "UNHEALTHY" }, passed, checks.len());
    }
    let code = if healthy { 0 } else { 1 };
    (code, out)
}
