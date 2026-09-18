//! main.rs — Vigil CLI entry point.
//! Standard CLI flags: -h/--help, -V/--version, --format, -o/--output, -q/--quiet, -v/--verbose.

use std::env;
use std::fs;
use std::path::{Path, PathBuf};
use std::process;
use vigil::{manifest, policy, report, score, Policy};

const VERSION: &str = "0.2.0";

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum OutputFormat {
    Text,
    Json,
    Markdown,
}

#[derive(Debug)]
struct CliConfig {
    subcommand: String,
    target_path: PathBuf,
    format: OutputFormat,
    output_file: Option<PathBuf>,
    quiet: bool,
    verbose: bool,
    max_dormancy: u32,
}

impl Default for CliConfig {
    fn default() -> Self {
        CliConfig {
            subcommand: "scan".to_string(),
            target_path: PathBuf::from("."),
            format: OutputFormat::Text,
            output_file: None,
            quiet: false,
            verbose: false,
            max_dormancy: 365,
        }
    }
}

fn print_help() {
    println!(
        "vigil {} — Supply-chain dormancy scanner\n\
        \n\
        USAGE:\n\
          vigil [SUBCOMMAND] [OPTIONS] [PATH]\n\
        \n\
        SUBCOMMANDS:\n\
          scan             Scan manifest and emit dormancy report (default)\n\
          policy check     Assert supply-chain policy compliance\n\
          badge            Generate SVG health badge\n\
        \n\
        OPTIONS:\n\
          -h, --help              Print help information\n\
          -V, --version           Print version information\n\
          --format <fmt>          Output format: text, json, markdown [default: text]\n\
          -o, --output <file>     Write report to file instead of stdout\n\
          -q, --quiet             Quiet mode; only emit errors and status codes\n\
          -v, --verbose           Verbose diagnostic logging\n\
          --max-dormancy <days>   Max allowable dependency dormancy [default: 365]\n\
        \n\
        EXAMPLES:\n\
          vigil scan\n\
          vigil scan --format json -o supply-chain.json\n\
          vigil policy check --max-dormancy 180\n",
        VERSION
    );
}

fn parse_args(args: &[String]) -> Result<Option<CliConfig>, String> {
    let mut config = CliConfig::default();
    let mut i = 1;
    let mut sub_set = false;

    while i < args.len() {
        match args[i].as_str() {
            "-h" | "--help" => {
                print_help();
                return Ok(None);
            }
            "-V" | "--version" => {
                println!("vigil {}", VERSION);
                return Ok(None);
            }
            "-q" | "--quiet" => config.quiet = true,
            "-v" | "--verbose" => config.verbose = true,
            "--format" => {
                i += 1;
                if i >= args.len() {
                    return Err("Missing argument for --format".to_string());
                }
                config.format = match args[i].to_lowercase().as_str() {
                    "json" => OutputFormat::Json,
                    "markdown" | "md" => OutputFormat::Markdown,
                    "text" => OutputFormat::Text,
                    other => return Err(format!("Unknown format: {}", other)),
                };
            }
            "-o" | "--output" => {
                i += 1;
                if i >= args.len() {
                    return Err("Missing argument for -o/--output".to_string());
                }
                config.output_file = Some(PathBuf::from(&args[i]));
            }
            "--max-dormancy" => {
                i += 1;
                if i >= args.len() {
                    return Err("Missing argument for --max-dormancy".to_string());
                }
                config.max_dormancy = args[i].parse::<u32>().map_err(|_| "Invalid days")?;
            }
            "policy" => {
                if i + 1 < args.len() && args[i + 1] == "check" {
                    i += 1;
                    config.subcommand = "policy_check".to_string();
                    sub_set = true;
                }
            }
            "scan" => {
                config.subcommand = "scan".to_string();
                sub_set = true;
            }
            "badge" => {
                config.subcommand = "badge".to_string();
                sub_set = true;
            }
            arg if !arg.starts_with('-') => {
                if !sub_set && (arg == "check") {
                    config.subcommand = "policy_check".to_string();
                } else {
                    config.target_path = PathBuf::from(arg);
                }
            }
            other => return Err(format!("Unknown option: {}", other)),
        }
        i += 1;
    }
    Ok(Some(config))
}

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

fn run() -> Result<i32, String> {
    let args: Vec<String> = env::args().collect();
    let config = match parse_args(&args)? {
        Some(c) => c,
        None => return Ok(0),
    };

    let manifest_path = locate_manifest(&config.target_path)?;
    if config.verbose {
        eprintln!("vigil: discovered manifest at {}", manifest_path.display());
    }

    let manifest = manifest::parse_file(&manifest_path)
        .map_err(|e| format!("Failed to parse manifest: {}", e))?;
    let score_report = score::compute(&manifest)
        .map_err(|e| format!("Scoring failed: {}", e))?;

    match config.subcommand.as_str() {
        "badge" => {
            let svg = report::emit_badge_svg(score_report.average_score);
            write_output(&svg, config.output_file.as_ref())
                .map_err(|e| format!("I/O write error: {}", e))?;
            Ok(0)
        }
        "policy_check" => {
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
            write_output(&formatted, config.output_file.as_ref())
                .map_err(|e| format!("I/O write error: {}", e))?;
            Ok(if score_report.critical_count > 0 { 1 } else { 0 })
        }
    }
}

fn main() {
    match run() {
        Ok(code) => process::exit(code),
        Err(err) => {
            eprintln!("error: {}", err);
            process::exit(2);
        }
    }
}
