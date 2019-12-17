use serde::{Deserialize};
use tokio::fs::read;
use anyhow::{Context};

#[derive(Deserialize)]
pub struct Config {
	pub args: Vec<String>,
	pub module: Module,
	pub websocket: WebSocket
}

#[derive(Deserialize)]
pub struct WebSocket {
	pub url: url::Url
}

#[derive(Deserialize)]
pub enum Module {
	#[serde(alias = "path")]
	Path(std::path::PathBuf)
}


impl Config {
	pub fn args_as_bytes(&self) -> Vec<Vec<u8>> {
		self.args.iter().map(|a| a.as_bytes().to_vec()).collect()
	}

	pub async fn load_wasm_bytes(&self) -> anyhow::Result<Vec<u8>> {
		match &self.module {
			Module::Path(path) => read(path.clone()).await
				.with_context(|| 
					format!("Could not read WASM plugin at {:?}", path))
		}
    }
}
