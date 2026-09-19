//! cli.rs — Standard CLI arguments and subcommand parser for vigil.
use std::fmt;
use std::path::PathBuf;

pub const VERSION: &str = "0.2.6";

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum OutputFormat {
    Text,
    Json,
    Markdown,
}

#[derive(Debug, PartialEq, Eq)]
pub enum Subcommand {
    Scan,
    PolicyCheck,
    Badge,
    Doctor,
    Update,
    Help,
    Version,
}

#[derive(Debug)]
pub struct CliConfig {
    pub subcommand: Subcommand,
    pub target_path: PathBuf,
    pub format: OutputFormat,
    pub output_file: Option<PathBuf>,
    pub quiet: bool,
    pub verbose: bool,
    pub max_dormancy: u32,
}

impl Default for CliConfig {
    fn default() -> Self {
        CliConfig {
            subcommand: Subcommand::Scan,
            target_path: PathBuf::from("."),
            format: OutputFormat::Text,
            output_file: None,
            quiet: false,
            verbose: false,
            max_dormancy: 365,
        }
    }
}

#[derive(Debug)]
pub enum CliError {
    Parse(String),
    Runtime(String),
}

impl fmt::Display for CliError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            CliError::Parse(s) | CliError::Runtime(s) => write!(f, "{}", s),
        }
    }
}

impl From<String> for CliError {
    fn from(s: String) -> Self {
        CliError::Runtime(s)
    }
}

impl From<&str> for CliError {
    fn from(s: &str) -> Self {
        CliError::Runtime(s.to_string())
    }
}

impl From<std::io::Error> for CliError {
    fn from(e: std::io::Error) -> Self {
        CliError::Runtime(e.to_string())
    }
}

pub fn print_help() {
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
          doctor           Inspect system health and environment\n\
          update, upgrade  Update binary to latest release\n\
          help             Print help information\n\
          version          Print version information\n\
        \n\
        OPTIONS:\n\
          -h, --help              Print help information\n\
          -V, --version           Print version information\n\
          -f, --format <fmt>      Output format: text, json, markdown [default: text]\n\
          -o, --output <file>     Write report to file instead of stdout\n\
          -q, --quiet             Quiet mode; only emit errors and status codes\n\
          -v, --verbose           Verbose diagnostic logging\n\
          --max-dormancy <days>   Max allowable dependency dormancy [default: 365]\n\
        \n\
        EXAMPLES:\n\
          vigil scan\n\
          vigil scan -f json -o supply-chain.json\n\
          vigil doctor\n\
          vigil update\n",
        VERSION
    );
}

pub fn parse_args(args: &[String]) -> Result<CliConfig, CliError> {
    let mut config = CliConfig::default();
    let mut i = 1;
    let mut sub_set = false;

    while i < args.len() {
        match args[i].as_str() {
            "-h" | "--help" | "help" => {
                config.subcommand = Subcommand::Help;
                return Ok(config);
            }
            "-V" | "--version" | "version" => {
                config.subcommand = Subcommand::Version;
                return Ok(config);
            }
            "-q" | "--quiet" => config.quiet = true,
            "-v" | "--verbose" => config.verbose = true,
            "-f" | "--format" => {
                i += 1;
                if i >= args.len() {
                    return Err(CliError::Parse("Missing argument for format option".into()));
                }
                config.format = match args[i].to_lowercase().as_str() {
                    "json" => OutputFormat::Json,
                    "markdown" | "md" => OutputFormat::Markdown,
                    "text" => OutputFormat::Text,
                    other => return Err(CliError::Parse(format!("Unknown format: {}", other))),
                };
            }
            "-o" | "--output" => {
                i += 1;
                if i >= args.len() {
                    return Err(CliError::Parse("Missing argument for output option".into()));
                }
                config.output_file = Some(PathBuf::from(&args[i]));
            }
            "--max-dormancy" => {
                i += 1;
                if i >= args.len() {
                    return Err(CliError::Parse("Missing argument for --max-dormancy".into()));
                }
                config.max_dormancy = args[i]
                    .parse::<u32>()
                    .map_err(|_| CliError::Parse("Invalid number for --max-dormancy".into()))?;
            }
            "scan" => {
                config.subcommand = Subcommand::Scan;
                sub_set = true;
            }
            "doctor" => {
                config.subcommand = Subcommand::Doctor;
                sub_set = true;
            }
            "update" | "upgrade" => {
                config.subcommand = Subcommand::Update;
                sub_set = true;
            }
            "badge" => {
                config.subcommand = Subcommand::Badge;
                sub_set = true;
            }
            "policy" => {
                if i + 1 < args.len() && args[i + 1] == "check" {
                    i += 1;
                    config.subcommand = Subcommand::PolicyCheck;
                    sub_set = true;
                } else {
                    return Err(CliError::Parse("Unknown policy subcommand: expected 'check'".into()));
                }
            }
            arg if !arg.starts_with('-') => {
                if !sub_set && (arg == "check") {
                    config.subcommand = Subcommand::PolicyCheck;
                    sub_set = true;
                } else {
                    config.target_path = PathBuf::from(arg);
                }
            }
            other => return Err(CliError::Parse(format!("Unknown option: {}", other))),
        }
        i += 1;
    }
    Ok(config)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_flags() {
        let args = vec!["vigil".into(), "-q".into(), "-v".into(), "-f".into(), "json".into()];
        let cfg = parse_args(&args).unwrap();
        assert!(cfg.quiet && cfg.verbose);
        assert_eq!(cfg.format, OutputFormat::Json);
    }

    #[test]
    fn test_subcommands() {
        let args = vec!["vigil".into(), "doctor".into()];
        assert_eq!(parse_args(&args).unwrap().subcommand, Subcommand::Doctor);
        let args = vec!["vigil".into(), "update".into()];
        assert_eq!(parse_args(&args).unwrap().subcommand, Subcommand::Update);
    }

    #[test]
    fn test_parse_errors() {
        let args = vec!["vigil".into(), "--unknown".into()];
        assert!(matches!(parse_args(&args), Err(CliError::Parse(_))));
        let args = vec!["vigil".into(), "-f".into()];
        assert!(matches!(parse_args(&args), Err(CliError::Parse(_))));
    }
}
