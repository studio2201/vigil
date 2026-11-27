//! update.rs — Self-update mechanism for vigil.
use crate::cli::OutputFormat;
use crate::xdg;
use std::env;
use std::fs;
use std::process::Command;

fn parse_semver(v: &str) -> (u32, u32, u32) {
    let mut it = v.trim_start_matches('v').split('.');
    (
        it.next().and_then(|s| s.parse().ok()).unwrap_or(0),
        it.next().and_then(|s| s.parse().ok()).unwrap_or(0),
        it.next().and_then(|s| s.parse().ok()).unwrap_or(0),
    )
}

fn is_newer(latest: &str, current: &str) -> bool {
    parse_semver(latest) > parse_semver(current)
}

fn query_latest_version(app: &str) -> Result<String, String> {
    let gh_url = format!("https://github.com/studio2201/{}/releases/latest", app);
    if let Ok(out) = Command::new("curl").args(["-sI", &gh_url]).output() {
        let text = String::from_utf8_lossy(&out.stdout);
        for line in text.lines() {
            let lower = line.to_lowercase();
            if lower.starts_with("location:") {
                if let Some(pos) = line.rfind('/') {
                    let mut tag = line[pos + 1..].trim();
                    if let Some(stripped) = tag.strip_prefix('v') {
                        tag = stripped;
                    }
                    if !tag.is_empty() {
                        return Ok(tag.to_string());
                    }
                }
            }
        }
    }
    Err("Failed to query latest release version from remote".into())
}

fn verify_sha256(archive: &std::path::Path, sha_url: &str) -> Result<(), String> {
    let sha_file = archive.with_extension("sha256");
    let st = Command::new("curl")
        .args(["-fsSL", sha_url, "-o", sha_file.to_str().unwrap_or("")])
        .status()
        .map_err(|e| format!("Failed to fetch SHA-256 checksum: {}", e))?;
    if !st.success() || !sha_file.is_file() {
        let _ = fs::remove_file(&sha_file);
        return Err("Missing release .sha256 checksum; aborting fail-closed".into());
    }
    let content = fs::read_to_string(&sha_file).map_err(|e| e.to_string())?;
    let _ = fs::remove_file(&sha_file);
    let expected = content.split_whitespace().next().unwrap_or("");
    if expected.len() != 64 || !expected.bytes().all(|b| b.is_ascii_hexdigit()) {
        return Err("Malformed release .sha256 checksum".into());
    }
    let out = Command::new("sha256sum")
        .arg(archive)
        .output()
        .or_else(|_| Command::new("shasum").args(["-a", "256"]).arg(archive).output())
        .map_err(|e| format!("SHA-256 verification tool unavailable: {}", e))?;
    if !out.status.success() {
        return Err("SHA-256 computation failed".into());
    }
    let actual = String::from_utf8_lossy(&out.stdout)
        .split_whitespace()
        .next()
        .unwrap_or("")
        .to_string();
    if !actual.eq_ignore_ascii_case(expected) {
        return Err(format!("SHA-256 mismatch ({} != {})", actual, expected));
    }
    Ok(())
}

fn detect_target() -> Option<&'static str> {
    match (env::consts::OS, env::consts::ARCH) {
        ("linux", "x86_64") => Some("x86_64-unknown-linux-musl"),
        ("linux", "aarch64") => Some("aarch64-unknown-linux-musl"),
        ("macos", "x86_64") => Some("x86_64-apple-darwin"),
        ("macos", "aarch64") => Some("aarch64-apple-darwin"),
        _ => None,
    }
}

pub fn run_update(
    app: &str,
    current_ver: &str,
    fmt: OutputFormat,
) -> Result<(i32, String), String> {
    let latest = match query_latest_version(app) {
        Ok(v) => v,
        Err(e) => {
            if fmt == OutputFormat::Json {
                return Ok((
                    1,
                    format!(
                        "{{\"app\":\"{}\",\"current\":\"{}\",\"error\":\"{}\"}}\n",
                        app, current_ver, e
                    ),
                ));
            } else {
                return Err(e);
            }
        }
    };

    if !is_newer(&latest, current_ver) {
        let out = if fmt == OutputFormat::Json {
            format!(
                "{{\"app\":\"{}\",\"current\":\"{}\",\"latest\":\"{}\",\"status\":\"up_to_date\"}}\n",
                app, current_ver, latest
            )
        } else {
            format!("{} is already up to date ({})\n", app, current_ver)
        };
        return Ok((0, out));
    }

    let dest_dir = xdg::bin_dir();
    fs::create_dir_all(&dest_dir)
        .map_err(|e| format!("Failed to create destination dir: {}", e))?;
    let dest_bin = dest_dir.join(app);

    let mut upgraded = false;
    if let Some(target) = detect_target() {
        let asset_url = format!(
            "https://github.com/studio2201/{}/releases/download/v{}/{}-{}.tar.gz",
            app, latest, app, target
        );
        let tmp_tar = dest_dir.join(format!(".{}-update.tar.gz", app));
        let dl = Command::new("curl")
            .args(["-fsSL", &asset_url, "-o", tmp_tar.to_str().unwrap_or("")])
            .status();

        if let Ok(st) = dl {
            if st.success() && tmp_tar.is_file() {
                verify_sha256(&tmp_tar, &format!("{}.sha256", asset_url)).map_err(|e| {
                    let _ = fs::remove_file(&tmp_tar);
                    e
                })?;
                let untar = Command::new("tar")
                    .args([
                        "-xzf",
                        tmp_tar.to_str().unwrap_or(""),
                        "-C",
                        dest_dir.to_str().unwrap_or(""),
                    ])
                    .status();
                let _ = fs::remove_file(&tmp_tar);
                if let Ok(ust) = untar {
                    if ust.success() && dest_bin.is_file() {
                        upgraded = true;
                    }
                }
            }
        }
    }

    if !upgraded {
        return Err(format!(
            "Failed to upgrade {}: prebuilt release unavailable or unsupported platform",
            app
        ));
    }

    #[cfg(unix)]
    if dest_bin.exists() {
        use std::os::unix::fs::PermissionsExt;
        let _ = fs::set_permissions(&dest_bin, fs::Permissions::from_mode(0o755));
    }

    if upgraded || dest_bin.exists() {
        let out = if fmt == OutputFormat::Json {
            format!(
                "{{\"app\":\"{}\",\"current\":\"{}\",\"latest\":\"{}\",\"status\":\"upgraded\",\"path\":\"{}\"}}\n",
                app, current_ver, latest, dest_bin.display()
            )
        } else {
            format!(
                "Successfully upgraded {} to {} in {}\n",
                app, latest, dest_bin.display()
            )
        };
        Ok((0, out))
    } else {
        Err(format!("Failed to upgrade {}: download error", app))
    }
}
