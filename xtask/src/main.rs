mod commands;
mod utils;

use clap::{Parser, Subcommand};
use color_eyre::Result;

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
    }
}
