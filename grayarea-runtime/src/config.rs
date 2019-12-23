use serde::{Deserialize};
use tokio::fs::read;
use anyhow::{Context};

#[derive(Deserialize)]
pub struct ModuleConfig {
	#[serde(default = "empty_args")]
	pub args: Vec<String>,
	pub module: Module,
	pub stream: Option<StreamOneOf>
}

#[derive(Deserialize)]
pub enum StreamOneOf {
	WebSocket(WebSocketConfig)
}

#[derive(Deserialize)]
pub struct WebSocketConfig {
	pub url: url::Url
}

#[derive(Deserialize)]
pub enum Module {
	#[serde(alias = "path")]
	Path(std::path::PathBuf)
}

fn empty_args() -> Vec<String> {
	vec![]
}

impl ModuleConfig {
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
