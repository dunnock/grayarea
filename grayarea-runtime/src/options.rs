use structopt::StructOpt;
use std::path::PathBuf;
use tokio::fs::read;
use crate::config::ModuleConfig;
use anyhow::{Context, anyhow, Result};
use grayarea::channel::{Channel};
use ipc_channel::ipc::{IpcSender};
use tokio::task::spawn_blocking;

#[derive(StructOpt)] 
#[structopt(name = "grayarea", about = "Serverless WASM runner with WebSocket subscription support")]
pub struct Opt {
    /// Path to Yaml config for wasm module
    #[structopt(parse(from_os_str))]
    config: PathBuf,
    /// IPC channel name for WASM module output messages
    #[structopt(short="o", long="ipc-output")]
    ipc_output: Option<String>,
}

impl Opt {
    pub async fn load_config(&self) -> Result<ModuleConfig> {
        let buf = read(self.config.clone()).await
            .with_context(|| 
                format!("Could not read config at {:?}", self.config))?;
        let config: ModuleConfig = serde_yaml::from_slice(buf.as_slice())
            .with_context(|| 
                format!("Malformed module config {:?}", self.config))?;
        // Validation
        if config.websocket.is_some() && self.ipc_output.is_none() {
            Err(anyhow!("WebSocket in config requires --ipc-output channel option"))
        } else {
            Ok(config)
        }
    }
    pub fn has_ipc(&self) -> bool {
        self.ipc_output.is_some()
    }
    pub async fn ipc_channel(&self) -> Result<Channel> {
        if let Some(name) = &self.ipc_output {
            let name = name.clone();
            spawn_blocking( || {
                let name_1 = name.clone();
                println!("Connecting to server: {}", &name_1);
                let tx = IpcSender::connect(name)?;
                let (ch1, ch2) = Channel::duplex()?;
                println!("Connected, sending Channel to server: {}", &name_1);
                tx.send(ch1)?;
                Ok(ch2)
            }).await?
        } else {
            Err(anyhow!("--ipc-output option was not set"))
        }
    }
}
