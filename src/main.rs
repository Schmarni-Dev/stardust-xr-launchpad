pub mod main_process;
pub mod non_main_process;

use clap::Parser;
use main_process::run_main_process;
use non_main_process::{igniter, server_ready};

#[tokio::main]
async fn main() -> color_eyre::Result<()> {
    color_eyre::install()?;
    tracing_subscriber::fmt()
        .with_thread_names(true)
        .with_ansi(true)
        .with_line_number(true)
        .init();
    let args = Args::parse();
    match args {
        Args::Start {
            stardust_server_command,
        } => run_main_process(stardust_server_command).await?,
        Args::Igniter {
            fallback_command,
        } => igniter(fallback_command).await?,
        Args::ServerStarted { env_vars } => server_ready(env_vars).await?,
    };
    Ok(())
}

/// A Programm to manage a standalone StardustXR session
#[derive(Parser)]
#[command(version, about, long_about)]
enum Args {
    /// this starts the main "service" part, the stardust server is ran from this process
    Start {
        #[clap(action, last(true))]
        stardust_server_command: Vec<String>,
    },
    /// this signals the main process that the OpenXR runtime is ready to accept the StardustXR
    /// server
    Igniter {
        /// run this command if the main process doesn't exist or is unreachable
        #[clap(action, last(true))]
        fallback_command: Option<Vec<String>>,
    },
    /// this signals the main process that the StardustXR server has finished setup
    ServerStarted {
        env_vars: Vec<String>,
    },
}
