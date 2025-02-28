use crate::clean::CleanCommand;
use clap::{ArgAction, Parser, Subcommand};
use eyre::Result;

use crate::down::DownCommand;
use crate::up::UpCommand;

/// Command line arguments
#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
pub struct Args {
    /// Verbosity level (0-4)
    #[arg(long, short, action = ArgAction::Count, default_value = "2")]
    v: u8,

    /// The subcommand to run
    #[clap(subcommand)]
    pub command: Option<Command>,
}

/// Possible CLI subcommands
#[derive(Debug, Subcommand)]
pub enum Command {
    /// Build and run the devnet stack
    Up(UpCommand),
    /// Bring the devnet stack down
    Down(DownCommand),
    /// Nuke the devnet stack
    Nuke,
    /// Clean all stack artifacts
    Clean(CleanCommand),
}

pub fn run() -> Result<()> {
    let Args { v, command } = Args::parse();

    crate::telemetry::init_tracing_subscriber(v)?;

    crate::banners::banner()?;

    match command {
        // If no subcommand is provided, run the Up command with default config.
        None => UpCommand::new(None, false).run()?,

        Some(command) => match command {
            Command::Up(up_command) => up_command.run()?,
            Command::Down(down_command) => down_command.run()?,
            Command::Nuke => unimplemented!("nuke command not yet implemented"),
            Command::Clean(clean_command) => clean_command.run()?,
        },
    }

    Ok(())
}
