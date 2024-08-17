use std::{
    path::{Path, PathBuf},
    time::Duration,
};

use clap::Parser;
use tokio::process::{Child, Command};

#[derive(clap::Parser)]
struct Cli {
    config: Option<PathBuf>,
    #[command(flatten)]
    other: Config,
}

#[derive(clap::Args, serde::Serialize, serde::Deserialize)]
struct Config {
    #[arg(short = 's', long = "stardust")]
    stardust_server: Option<PathBuf>,
    #[arg(short = 'e', long = "stardust_env")]
    stardust_env_file: Option<PathBuf>,
    #[arg(short = 'd', long)]
    xwayland_display: Option<String>,
}

impl Config {
    async fn extend_with_file(&mut self, path: impl AsRef<Path>) {
        let Ok(config_file) = tokio::fs::read_to_string(path).await else {
            return;
        };
        let config = match toml::from_str::<Config>(&config_file) {
            Ok(v) => v,
            Err(err) => {
                println!("ERROR: unable to parse config file: {err}");
                return;
            }
        };
        self.stardust_server = self.stardust_server.take().or(config.stardust_server);
        self.stardust_env_file = self.stardust_env_file.take().or(config.stardust_env_file);
        self.xwayland_display = self.xwayland_display.take().or(config.xwayland_display);
    }
}

#[allow(dead_code)]
struct Handles {
    stardust_server: Child,
}

async fn start_server_and_xwayland(config: &Config) -> color_eyre::Result<Handles> {
    let mut cmd = Command::new(
        config
            .stardust_server
            .as_ref()
            .expect("No Stardust Server Path"),
    );
    cmd.args(["-o", "1"]);
    if let Some(env_file) = config.stardust_env_file.as_ref() {
        cmd.arg("-e").arg(env_file);
    }
    if let Some(xway_display) = config.xwayland_display.as_ref() {
        cmd.env("DISPLAY", xway_display);
    }
    let server = cmd.spawn()?;

    Ok(Handles {
        stardust_server: server,
    })
}

#[tokio::main(flavor = "current_thread")]
async fn main() -> color_eyre::Result<()> {
    let args = Cli::parse();
    let config_path = args
        .config
        .or(dirs::config_dir().map(|v| v.join("stardust_rocket").join("config.toml")))
        .expect("Unable to find config directory");
    println!("{}", config_path.to_str().unwrap());
    let mut config = args.other;
    config.extend_with_file(config_path).await;
    let _handles = start_server_and_xwayland(&config).await?;
    let mut _seat = libseat::Seat::open(|seat_ref, event| match event {
        libseat::SeatEvent::Enable => {}
        libseat::SeatEvent::Disable => seat_ref.disable().unwrap(),
    })?;
    tokio::time::sleep(Duration::MAX).await;
    Ok(())
}
