use serde::{Deserialize};

#[derive(Deserialize)]
pub struct Config {
	pub args: Vec<String>,
	pub websocket: WebSocket
}

#[derive(Deserialize)]
pub struct WebSocket {
	pub url: url::Url
}


impl Config {
	pub fn args_as_bytes(&self) -> Vec<Vec<u8>> {
		self.args.iter().map(|a| a.as_bytes().to_vec()).collect()
	}
}
