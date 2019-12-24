pub mod wasm;
use wasm::WasmWSInstance;

pub use tungstenite::protocol::Message;
use tungstenite::error::Error;
use futures::{SinkExt, StreamExt};
use tokio_tungstenite::{connect_async, WebSocketStream, MaybeTlsStream};
use url;
use anyhow::anyhow;
use tokio::sync::{Mutex};
use std::sync::Arc;

type WS = WebSocketStream<MaybeTlsStream<tokio::net::TcpStream>>;

#[derive(Clone)]
pub struct WebSocket {
	pub stream: Arc<Mutex<Option<WS>>>
}

impl Default for WebSocket {
	fn default() -> Self {
		WebSocket{ stream: Arc::new(Mutex::new(None)) }
	}
}

use std::ops::DerefMut;

impl WebSocket {
	pub async fn connect(&self, addr: url::Url) -> Result<(), Error> {
		let stream = connect_async(addr).await?.0;
		self.stream.lock().await.replace(stream);
		Ok(())
	}

	pub async fn send_message(&self, msg: Vec<u8>) -> anyhow::Result<()>  {
		println!("{}", std::str::from_utf8(msg.as_slice())?);
		match self.stream.lock().await.deref_mut() {
			Some(stream) => Ok(stream.send(Message::Binary(msg)).await?),
			None => Err(anyhow!("tried to send message to disconnected WebSocket"))
		}
	}

	pub async fn pong(&self, msg: Vec<u8>) -> anyhow::Result<()>  {
		match self.stream.lock().await.deref_mut() {
			Some(stream) => Ok(stream.send(Message::Pong(msg)).await?),
			None => Err(anyhow!("tried to send pong to disconnected WebSocket"))
		}
	}

	#[inline]
	pub async fn read(&self) -> Option<Result<Message, Error>>  {
		match self.stream.lock().await.deref_mut() {
			Some(stream) => stream.next().await,
			None => Some(Err(Error::AlreadyClosed))
		}
	}

	pub async fn clean(&self) {
		self.stream.lock().await.take();
	}

	pub async fn set_handshaker(&self, wasm: &WasmWSInstance) -> anyhow::Result<()> {
		let rx = wasm.clone_receiver();
		loop {
			let rx = rx.clone();
			// Some workaround to wait on sync message from crossbeam while not blocking Tokio
			// TODO: probably whole WASM <-> Tokio communication shall be rethought!
			let msg = tokio::task::spawn_blocking(move || rx.recv()).await??;
			self.send_message(msg).await?;
		};
	}
}


