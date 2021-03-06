use anyhow::Context;
use grayarea::config::PipelineConfig;
use std::path::PathBuf;
use structopt::StructOpt;
use tokio::fs::read;

#[derive(StructOpt)]
#[structopt(
    name = "Grayarea Engine DE",
    about = "Engine for combining and running serverless pipelines written for grayarea runner"
)]
pub struct Opt {
    /// Path to Yaml config for wasm module
    #[structopt(parse(from_os_str))]
    config: PathBuf,
    #[structopt(short = "d", long = "debug")]
    pub debug: bool,
}

impl Opt {
    pub async fn load_config(&self) -> anyhow::Result<PipelineConfig> {
        let buf = read(self.config.clone())
            .await
            .with_context(|| format!("Could not read config at {:?}", self.config))?;
        Ok(serde_yaml::from_slice(buf.as_slice()).expect("Malformed config"))
    }
}
