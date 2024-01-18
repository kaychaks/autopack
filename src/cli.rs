use self::init::Init;
use crate::{
    autopack::AutoPack,
    error::AppError,
    log::{banner, error},
};
use clap::{self, Parser, Subcommand};
use std::path::PathBuf;

pub(crate) mod init;

#[derive(Parser)]
#[clap(version, about)]
#[clap(propagate_version = true)]
/// Auto Pack CLI
pub struct Cli {
    #[clap(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    /// Initializes auto-pack
    Init {
        /// Path to the project (default: present working directory)
        #[clap(short = 'p', long = "project")]
        client_project_path: Option<PathBuf>,
        /// Enable live reload (default: false)
        #[clap(short = 'l', long = "live-reload", action)]
        live_reload: bool,
        /// Don't build after init (default: false)
        #[clap(long = "no-build", action)]
        no_build: bool,
        /// Create runtime directory anyway (default: false)
        #[clap(short = 'f', long = "force-create-runtime", action)]
        force_create_runtime: bool,
    },

    /// Build auto-pack
    Build {
        /// Clears cache when building
        #[clap(long = "clear-cache", action)]
        clear_cache: bool,
    },

    /// Runs auto-pack
    Run {
        /// Re-build auto-pack
        #[clap(short = 'b', long = "build", action)]
        build: bool,

        /// Clears cache when re-building
        #[clap(long = "clear-cache", action)]
        clear_cache: bool,

        /// Port to run the application on
        #[clap(long = "port", default_value_t = 8080)]
        port: usize,
    },
}

impl Cli {
    pub fn new() -> Cli {
        Cli::parse()
    }

    fn build(&self, clear_cache: bool) -> Result<(), AppError> {
        banner("Building project using autopack");
        let autopack = AutoPack::load_validate(None).map_err(|e| {
            error("Failed validating autopack. Please run `auto-pack init` again.");
            e
        })?;

        autopack.build(clear_cache).map_err(|e| {
            error("Autopack build failure. Exiting.");
            AppError::BuildError("Build failure", e)
        })?;
        Ok(())
    }
    pub async fn run(self) -> Result<(), AppError> {
        // let cli = Cli::parse();
        match self.command {
            Commands::Init {
                ref client_project_path,
                live_reload,
                no_build,
                force_create_runtime,
            } => {
                banner("Initializing autopack");
                let ap = Init::pre_configure(client_project_path.clone())?
                    .configure(live_reload)?
                    .post_configure(force_create_runtime, live_reload)
                    .await?
                    .install();

                ap.save(None).map_err(|e| {
                    AppError::PostConfigureError("Failed to serialize autopack state", e)
                })?;

                banner("Initialized autopack");

                if !no_build {
                    self.build(false)?;
                }

                Ok(())
            }

            Commands::Build { clear_cache } => self.build(clear_cache),

            Commands::Run {
                build,
                clear_cache,
                port,
            } => {
                let autopack = AutoPack::load_validate(None).map_err(|e| {
                    error("Failed validating autopack. Please run `auto-pack init` again.");
                    e
                })?;

                if build {
                    self.build(clear_cache)?;
                }

                autopack.run(port).await.map_err(|e| {
                    AppError::RunError("Failed running autopack project", anyhow::anyhow!(e))
                })?;

                Ok(())
            }
        }
    }
}

impl Default for Cli {
    fn default() -> Self {
        Self::new()
    }
}
