//use wasmer_runtime::{Ctx};
//use std::cell::RefCell;
//use super::U8WasmPtr;
pub use tungstenite::protocol::Message;
use tungstenite::error::Error;
use futures::SinkExt;
use tokio_tungstenite::{connect_async, WebSocketStream, MaybeTlsStream};
use url;

pub struct WebSocket {
	pub stream: WebSocketStream<MaybeTlsStream<tokio::net::TcpStream>>
}

impl WebSocket {
	pub async fn connect(addr: url::Url) 
		-> Result<Self, Error> 
	{
		let (stream, _) = connect_async(addr).await?;
		Ok(WebSocket{ stream })
	}

	pub async fn send_message(&mut self, msg: Vec<u8>) {
		println!("{}", std::str::from_utf8(msg.as_slice()).expect("websocket_send_message: not utf8 message"));
		self.stream.send(Message::Binary(msg)).await
			.expect("websocket::send_message failed");
	}	
}


