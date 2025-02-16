use std::{collections::HashMap, process::exit};

use tokio::process::Command;
use tracing::{error, info, warn};
use zbus::{interface, Connection};

use crate::main_process::LaunchpadProxy;

pub async fn server_ready(env_vars: Vec<String>) -> color_eyre::Result<()> {
    let vars: HashMap<String, String> = env_vars
        .into_iter()
        .filter_map(|var| match std::env::var(var.as_str()) {
            Ok(value) => Some((var, value)),
            Err(err) => {
                error!("invalid environment variable: \"{var}\": {err}");
                None
            }
        })
        .collect();

    let conn = Connection::session().await?;
    if let Ok(proxy) = LaunchpadProxy::new(&conn).await {
        if let Err(err) = proxy.stardust_server_started(vars).await {
            error!("{err}");
            exit(1);
        }
    } else {
        error!("Unable to create LaunchPad Proxy");
        exit(1);
    };
    Ok(())
}

pub async fn igniter(fallback_command: Option<Vec<String>>) -> color_eyre::Result<()> {
    let conn = match igniter_get_connection().await {
        Ok(c) => c,
        Err(err) => {
            start_fallback_command(fallback_command).await;
            return Err(err.into());
        }
    };
    match LaunchpadProxy::new(&conn).await {
        Ok(proxy) => {
            if let Err(err) = proxy.xr_runtime_ready().await {
                warn!("Unable to signal the main process: {err}");
                start_fallback_command(fallback_command).await;
            }
        }
        Err(err) => {
            warn!("Unable to create LaunchPad Proxy: {err}");
            start_fallback_command(fallback_command).await;
        }
    };
    Ok(())
}

async fn igniter_get_connection() -> zbus::Result<Connection> {
    zbus::connection::Builder::session()?
        .name("org.stardustxr.launchpad.Igniter")?
        .serve_at("/org/stardustxr/launchpad/Igniter", Igniter)?
        .build()
        .await
}

struct Igniter;

#[interface(
    name = "org.stardustxr.launchpad.Igniter",
    proxy(
        gen_blocking = false,
        default_path = "/org/stardustxr/launchpad/Igniter",
        default_service = "org.stardustxr.launchpad.Igniter",
    )
)]
impl Igniter {
    fn instant_ignite(&self) -> bool {
        true
    }
}

async fn start_fallback_command(fallback_command: Option<Vec<String>>) {
    if let Some(fallback_cmd) = fallback_command {
        info!(?fallback_cmd);
        if fallback_cmd.is_empty() {
            error!("fallback command is specified but empty");
        }
        let mut fallback_cmd_iter = fallback_cmd.into_iter();
        let _ = Command::new(fallback_cmd_iter.next().unwrap())
            .args(fallback_cmd_iter)
            .status()
            .await;
    }
}
