use structopt::StructOpt;
use std::path::PathBuf;
use tokio::fs::read;
use super::Config;
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
    pub async fn load_config(&self) -> anyhow::Result<Config> {
        let buf = read(self.config.clone()).await
            .with_context(|| 
                format!("Could not read config at {:?}", self.config))?;
        Ok(serde_yaml::from_slice(buf.as_slice()).expect("Malformed config"))
    }
}
