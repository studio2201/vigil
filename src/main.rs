//! main.rs — Vigil CLI entry point.
//! Standard CLI flags: -h/--help, -V/--version, -f/--format, -o/--output, -q/--quiet, -v/--verbose.

mod cli;
mod doctor;
mod update;
mod xdg;

use cli::{parse_args, print_help, CliConfig, CliError, OutputFormat, Subcommand, VERSION};
use std::env;
use std::fs;
use std::path::{Path, PathBuf};
use std::process;
use vigil::{manifest, policy, report, score, Policy};

fn locate_manifest(base: &Path) -> Result<PathBuf, String> {
    if base.is_file() {
        return Ok(base.to_path_buf());
    }
    let candidates = [
        "package.json",
        "Cargo.lock",
        "Cargo.toml",
        "requirements.txt",
        "go.mod",
    ];
    for c in &candidates {
        let p = base.join(c);
        if p.is_file() {
            return Ok(p);
        }
    }
    Err(format!("No supported manifest found in {}", base.display()))
}

fn write_output(content: &str, target: Option<&PathBuf>) -> Result<(), std::io::Error> {
    if let Some(path) = target {
        fs::write(path, content)
    } else {
        print!("{}", content);
        Ok(())
    }
}

fn run_scan(config: &CliConfig) -> Result<i32, CliError> {
    let manifest_path = locate_manifest(&config.target_path)?;
    if config.verbose {
        eprintln!("vigil: discovered manifest at {}", manifest_path.display());
    }

    let manifest = manifest::parse_file(&manifest_path)
        .map_err(|e| format!("Failed to parse manifest: {}", e))?;
    let score_report = score::compute(&manifest)
        .map_err(|e| format!("Scoring failed: {}", e))?;

    match config.subcommand {
        Subcommand::Badge => {
            let svg = report::emit_badge_svg(score_report.average_score);
            write_output(&svg, config.output_file.as_ref())?;
            Ok(0)
        }
        Subcommand::PolicyCheck => {
            let mut pol = Policy::default();
            pol.max_dormancy_days = config.max_dormancy;
            let verdict = policy::evaluate(&score_report, &pol);
            if !config.quiet {
                if verdict.passed {
                    println!("vigil policy check: PASSED (all dependencies within threshold)");
                } else {
                    eprintln!("vigil policy check: FAILED");
                    for v in &verdict.violations {
                        eprintln!("  - {}", v);
                    }
                }
            }
            Ok(if verdict.passed { 0 } else { 1 })
        }
        _ => {
            let formatted = match config.format {
                OutputFormat::Json => report::emit_json(&score_report),
                OutputFormat::Markdown => report::emit_markdown(&score_report),
                OutputFormat::Text => {
                    format!(
                        "vigil: scanned {} dependencies in {}\n\
                         Risk score: {:.1}/100 | Critical: {} | High: {} | Medium: {} | Low: {}\n",
                        score_report.total_deps,
                        score_report.manifest_path,
                        score_report.average_score,
                        score_report.critical_count,
                        score_report.high_count,
                        score_report.medium_count,
                        score_report.low_count,
                    )
                }
            };
            write_output(&formatted, config.output_file.as_ref())?;
            Ok(if score_report.critical_count > 0 { 1 } else { 0 })
        }
    }
}

fn run() -> Result<i32, CliError> {
    let args: Vec<String> = env::args().collect();
    let config = parse_args(&args)?;

    match config.subcommand {
        Subcommand::Help => {
            print_help();
            Ok(0)
        }
        Subcommand::Version => {
            println!("vigil {}", VERSION);
            Ok(0)
        }
        Subcommand::Doctor => Ok(doctor::run_doctor("vigil", VERSION, config.format)),
        Subcommand::Update => update::run_update("vigil", VERSION).map_err(CliError::Runtime),
        Subcommand::Scan | Subcommand::PolicyCheck | Subcommand::Badge => run_scan(&config),
    }
}

fn main() {
    match run() {
        Ok(code) => process::exit(code),
        Err(CliError::Parse(err)) => {
            eprintln!("error: {}", err);
            process::exit(2);
        }
        Err(CliError::Runtime(err)) => {
            eprintln!("error: {}", err);
            process::exit(1);
        }
    }
}
