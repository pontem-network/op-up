use bollard::image::{ListImagesOptions, RemoveImageOptions};
use bollard::volume::{ListVolumesOptions, RemoveVolumeOptions};
use bollard::Docker;
use clap::Args;
use eyre::Result;
use std::collections::HashMap;
use std::env;
use std::path::{Path, PathBuf};
use tokio::process::Command;

use op_config::Config;

/// The Down CLI Subcommand.
#[derive(Debug, Args)]
pub struct CleanCommand {
    /// An optional path to a stack config file.
    #[arg(long, short)]
    pub config: Option<PathBuf>,
}

impl CleanCommand {
    /// Create a new Clean CLI Subcommand.
    pub fn new(config: Option<PathBuf>) -> Self {
        Self { config }
    }

    /// Run the Clean CLI Subcommand.
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

            tracing::info!(target: "cli", "Loading op-stack config from {:?}", config_dir);
            let stack = Config::load_with_root(config_dir);

            let current_dir = env::current_dir()?;

            tracing::info!(target: "cli", "Stack: {:#?}", stack);

            // Removing artifacts directory
            let status = Command::new("rm")
                .args(["-rf", stack.artifacts.to_str().unwrap()])
                .current_dir(&current_dir)
                .status()
                .await?;

            if status.success() {
                tracing::info!(target: "cli", "{} deleted", stack.artifacts.to_str().unwrap());
            } else {
                tracing::error!(target: "cli", "Failed to delete dir {}", stack.artifacts.to_str().unwrap());
            }

            // Removing .devnet dir
            let status = Command::new("rm")
                .args(["-rf", config_dir.join(".devnet").to_str().unwrap()])
                .current_dir(&current_dir)
                .status()
                .await?;

            tracing::info!(target: "cli", "config_dir: {:?}", config_dir.parent());
            if status.success() {
                tracing::info!(target: "cli", ".devnet directory deleted");
            } else {
                tracing::error!(target: "cli", "Failed to delete .devnet dir");
            }

            // Stop docker containers
            let status = Command::new("sh")
                .arg("-c")
                .arg("cd ./docker && docker compose down")
                .status()
                .await?;

            if status.success() {
                tracing::info!(target: "cli", "Docker containers are down");
            } else {
                tracing::error!(target: "cli", "Failed to stop docker containers");
            }

            // TODO: rename docker images & volumes here
            let mut list_images_options = HashMap::new();
            list_images_options.insert(String::from("reference"), vec![String::from("docker_*")]);
            let images = docker
                .list_images(Some(ListImagesOptions {
                    filters: list_images_options,
                    ..Default::default()
                }))
                .await?;

            for image in images {
                docker
                    .remove_image(&image.id, None::<RemoveImageOptions>, None)
                    .await?;
            }

            let mut list_volumes_options = HashMap::new();
            list_volumes_options.insert(String::from("name"), vec![String::from("docker_*")]);
            let volumes = docker
                .list_volumes(Some(ListVolumesOptions {
                    filters: list_volumes_options,
                }))
                .await?;

            if let Some(vols) = volumes.volumes {
                for vol in vols {
                    docker
                        .remove_volume(&vol.name, None::<RemoveVolumeOptions>)
                        .await?;
                }
            }

            // TODO: rm -rf ./packages/contracts-bedrock/deployments/devnetL1

            Ok(())
        })
    }
}
