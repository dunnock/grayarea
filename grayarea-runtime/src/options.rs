use anyhow::{anyhow, Context, Result};
use grayarea::config::ModuleConfig;
use ipc_channel::ipc::IpcSender;
use ipc_orchestrator::{Channel, connect_ipc_server};
use std::path::PathBuf;
use structopt::StructOpt;
use tokio::fs::read;
use tokio::task::spawn_blocking;

#[derive(StructOpt)]
#[structopt(
    name = "grayarea",
    about = "Serverless WASM runner with WebSocket subscription support"
)]
pub struct Opt {
    /// Path to Yaml config for wasm module
    #[structopt(parse(from_os_str))]
    config: PathBuf,
}

impl Opt {
    pub async fn load_config(&self) -> Result<ModuleConfig> {
        let buf = read(self.config.clone())
            .await
            .with_context(|| format!("Could not read config at {:?}", self.config))?;
        let config: ModuleConfig = serde_yaml::from_slice(buf.as_slice())
            .with_context(|| format!("Malformed module config {:?}", self.config))?;
        // Validation
        if config.stream.is_some() && !self.has_ipc() {
            Err(anyhow!(
                "stream in config requires {} env var",
                ipc_orchestrator::IPC_SERVER_ENV_VAR
            ))
        } else {
            Ok(config)
        }
    }
    pub fn has_ipc(&self) -> bool {
        std::env::var(ipc_orchestrator::IPC_SERVER_ENV_VAR).is_ok()
    }
    pub async fn ipc_channel(&self) -> Result<Channel> {
        if self.has_ipc() {
            spawn_blocking(|| connect_ipc_server()).await?
        } else {
            Err(anyhow!("--orchestrator-ch option was not set"))
        }
    }
}
