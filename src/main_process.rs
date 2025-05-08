pub struct Test;
use core::panic;
use std::{collections::HashMap, sync::Arc};

use tokio::{
    process::Command,
    sync::{mpsc, Notify},
    task::JoinSet,
};
use zbus::interface;

use crate::non_main_process::IgniterProxy;

pub async fn run_main_process(server_command: Vec<String>) -> color_eyre::Result<()> {
    if server_command.is_empty() {
        panic!("Provide a valid StardustXR server launch command");
    }
    let runtime_notifier = Arc::new(Notify::new());
    let (env_channel_tx, mut env_channel_rx) = mpsc::channel(1);
    let interface = Launchpad {
        runtime_ready_notifier: runtime_notifier.clone(),
        stardust_server_started: env_channel_tx,
    };
    let conn = zbus::connection::Builder::session()?
        .name("org.stardustxr.Launchpad")?
        .serve_at("/org/stardustxr/Launchpad", interface)?
        .build()
        .await?;
    let mut instatly_start = false;
    if let Ok(proxy) = IgniterProxy::new(&conn).await {
        instatly_start = proxy.instant_ignite().await.unwrap_or(false);
    }
    if !instatly_start {
        runtime_notifier.notified().await;
    }
    let mut command_iter = server_command.into_iter();
    let command = tokio::spawn(
        Command::new(command_iter.next().unwrap())
            .args(command_iter)
            .status(),
    );

    let env_vars = env_channel_rx.recv().await.unwrap();
    let env_vars = filter_env_vars(env_vars);
    let mut join_set = JoinSet::new();
    for (var, value) in env_vars.into_iter() {
        join_set.spawn(
            Command::new("systemctl")
                .args(["--user", "set-environment", &format!("{var}={value}")])
                .status(),
        );
    }

    join_set.join_all().await;

    _ = Command::new("systemctl")
        .args(["--user", "start", "--no-block", "stardust-xr-session.target"])
        .status()
        .await;

    _ = command.await;

    _ = Command::new("systemctl")
        .args(["--user", "stop", "--no-block", "stardust-xr-session.target"])
        .status()
        .await;

    Ok(())
}

fn filter_env_vars(env_vars: HashMap<String, String>) -> HashMap<String, String> {
    env_vars
        .into_iter()
        // this filter might have to be improved
        .filter(|(var, _value)| (!matches!(var.as_str(), "XAUTHORITY" | "_" | "SHELL" | "SHLVL")))
        .collect()
}

struct Launchpad {
    runtime_ready_notifier: Arc<Notify>,
    stardust_server_started: mpsc::Sender<HashMap<String, String>>,
}

#[interface(
    name = "org.stardustxr.launchpad.Launchpad",
    proxy(
        gen_blocking = false,
        default_path = "/org/stardustxr/Launchpad",
        default_service = "org.stardustxr.Launchpad",
    )
)]
impl Launchpad {
    async fn xr_runtime_ready(&self) {
        self.runtime_ready_notifier.notify_waiters();
    }

    async fn stardust_server_started(&self, env_vars: HashMap<String, String>) {
        _ = self.stardust_server_started.try_send(env_vars);
    }
}
