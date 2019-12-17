use structopt::StructOpt;
use std::path::PathBuf;
use tokio::fs::read;
use crate::config::ModuleConfig;
use anyhow::{Context};

#[derive(StructOpt)] 
#[structopt(name = "grayarea", about = "Serverless WASM runner with WebSocket subscription support")]
pub struct Opt {
    /// Path to Yaml config for wasm module
    #[structopt(parse(from_os_str))]
    config: PathBuf,
    /// IPC channel name for WASM module output messages
    #[structopt(short="o", long="ipc-output", default_value="")]
    ipc_output: String,
}

impl Opt {
    pub async fn load_config(&self) -> anyhow::Result<ModuleConfig> {
        let buf = read(self.config.clone()).await
            .with_context(|| 
                format!("Could not read config at {:?}", self.config))?;
        serde_yaml::from_slice(buf.as_slice())
            .with_context(|| 
                format!("Malformed module config {:?}", self.config))
    }
}
