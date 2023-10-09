use bollard::Docker;
use clap::Args;
use eyre::Result;
use std::path::{Path, PathBuf};
use tokio::process::Command;

use op_config::Config;

/// The Down CLI Subcommand.
#[derive(Debug, Args)]
pub struct DownCommand {
    /// An optional path to a stack config file.
    #[arg(long, short)]
    pub config: Option<PathBuf>,
}

impl DownCommand {
    /// Create a new Down CLI Subcommand.
    pub fn new(config: Option<PathBuf>) -> Self {
        Self { config }
    }

    /// Run the Down CLI Subcommand.
    pub fn run(&self) -> Result<()> {
        crate::runner::run_until_ctrl_c(async {
            tracing::info!(target: "cli", "bootstrapping op stack");

            // todo: remove this once we have a proper stage docker component
            //       for now, this placeholds use of [bollard].
            let docker = Docker::connect_with_local_defaults()?;
            let version = docker.version().await?;
            tracing::info!(target: "cli", "docker version: {:?}", version);

            // Get the directory of the config file if it exists.
            let config_dir = self.config.as_ref().and_then(|p| p.parent());
            let config_dir = config_dir.unwrap_or_else(|| Path::new("."));

            // TODO: do we need config here?
            tracing::info!(target: "cli", "Loading op-stack config from {:?}", config_dir);
            let _stack = Config::load_with_root(config_dir);

            tracing::info!(target: "cli", "Stack: {:#?}", _stack);

            // Get the current timestamp.
            let genesis_timestamp = format!(
                "{}",
                std::time::SystemTime::now()
                    .duration_since(std::time::UNIX_EPOCH)
                    .expect("Time went backwards")
                    .as_secs()
            );

            // Stop docker containers
            let status = Command::new("sh")
                .arg("-c")
                .arg(format!(
                    "cd ./docker && GENESIS_TIMESTAMP={} docker compose stop",
                    genesis_timestamp
                ))
                .status()
                .await?;

            if status.success() {
                tracing::info!(target: "cli", "Stopped docker containers");
            } else {
                tracing::error!(target: "cli", "Failed to stop docker containers");
            }
            Ok(())
        })
    }
}
