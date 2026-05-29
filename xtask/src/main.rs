mod commands;
mod utils;

use clap::{Parser, Subcommand};
use color_eyre::Result;
use semver::Version;
use std::str::FromStr;

#[derive(Parser)]
#[command(name = "xtask")]
#[command(about = "RustyWind automation tasks", long_about = None)]
struct Cli {
    #[command(subcommand)]
    command: Command,
}

#[derive(Subcommand)]
enum Command {
    /// Fuzz testing commands
    Fuzz {
        #[command(subcommand)]
        subcommand: FuzzCommand,
    },
    /// NPM package management commands
    Npm {
        #[command(subcommand)]
        subcommand: NpmCommand,
    },
}

#[derive(Subcommand)]
enum FuzzCommand {
    /// Set up fuzz test environment (build release binary + install npm deps)
    Setup,

    /// Run fuzz tests with automatic failure analysis
    Run {
        /// Number of rounds to run
        #[arg(default_value = "25")]
        rounds: usize,

        /// Number of parallel workers (auto-detected if not specified)
        #[arg(short, long)]
        workers: Option<usize>,

        /// Base seed for deterministic testing (generates seed0, seed1, etc.)
        #[arg(long)]
        seed: Option<String>,
    },
}

/// Version bump specification: either a bump type or an explicit version
#[derive(Clone, Debug)]
pub enum BumpSpec {
    Major,
    Minor,
    Patch,
    Version(String),
}

impl FromStr for BumpSpec {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s.to_lowercase().as_str() {
            "major" => Ok(BumpSpec::Major),
            "minor" => Ok(BumpSpec::Minor),
            "patch" => Ok(BumpSpec::Patch),
            _ => {
                let version = s.strip_prefix('v').unwrap_or(s);
                Version::parse(version).map_err(|_| {
                    format!(
                        "invalid value '{s}': expected major, minor, patch, or semver version (e.g., v0.25.0 or v0.25.0-alpha.1)"
                    )
                })?;
                Ok(BumpSpec::Version(s.to_string()))
            }
        }
    }
}

#[derive(Subcommand)]
enum NpmCommand {
    /// Bump version and release npm packages
    Bump {
        /// Version bump: major, minor, patch, or explicit semver version
        spec: BumpSpec,
        /// GitHub token for API access
        #[arg(long, env = "GITHUB_TOKEN")]
        token: Option<String>,
        /// Dry run - don't actually publish
        #[arg(long)]
        dry_run: bool,
    },
    /// Update version across all npm packages (without releasing)
    UpdateVersion {
        /// The version to set (e.g., 0.25.0)
        version: String,
    },
    /// Download binaries from GitHub release and prepare packages
    PrepareBinaries {
        /// The version/tag to download (e.g., v0.25.0)
        version: String,
        /// GitHub token for API access (optional, uses GITHUB_TOKEN env var)
        #[arg(long, env = "GITHUB_TOKEN")]
        token: Option<String>,
    },
    /// Publish all npm packages (without downloading binaries)
    Publish {
        /// Dry run - don't actually publish
        #[arg(long)]
        dry_run: bool,
    },
}

fn main() -> Result<()> {
    color_eyre::install()?;
    let cli = Cli::parse();

    match cli.command {
        Command::Fuzz { subcommand } => match subcommand {
            FuzzCommand::Setup => commands::setup::run(),
            FuzzCommand::Run {
                rounds,
                workers,
                seed,
            } => commands::run::run(rounds, workers, seed),
        },
        Command::Npm { subcommand } => match subcommand {
            NpmCommand::Bump {
                spec,
                token,
                dry_run,
            } => commands::npm::bump(spec, token.as_deref(), dry_run),
            NpmCommand::UpdateVersion { version } => commands::npm::update_version(&version),
            NpmCommand::PrepareBinaries { version, token } => {
                commands::npm::prepare_binaries(&version, token.as_deref())
            }
            NpmCommand::Publish { dry_run } => commands::npm::publish(dry_run),
        },
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn accepts_prerelease_version_specs() {
        let spec: BumpSpec = "v0.25.0-alpha.1".parse().unwrap();

        match spec {
            BumpSpec::Version(version) => assert_eq!(version, "v0.25.0-alpha.1"),
            _ => panic!("expected explicit version bump spec"),
        }
    }

    #[test]
    fn rejects_invalid_version_specs() {
        assert!("v0.25".parse::<BumpSpec>().is_err());
        assert!("v0.25.0-alpha..1".parse::<BumpSpec>().is_err());
    }
}
