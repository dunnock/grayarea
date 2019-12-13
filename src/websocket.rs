use wasmer_runtime::{Ctx};
use std::cell::RefCell;
use super::U8WasmPtr;
pub use tungstenite::protocol::Message;
use tungstenite::error::Error;
use futures::SinkExt;
use tokio_tungstenite::{connect_async, WebSocketStream, MaybeTlsStream};
use url;

pub struct WebSocket {
	stream: Option<WebSocketStream<MaybeTlsStream<tokio::net::TcpStream>>>
}

thread_local! {
    pub static WS: RefCell<WebSocket> = RefCell::new(WebSocket::new());
}

impl WebSocket {
	pub fn new() -> Self {
		WebSocket { stream: None }
	}
	pub async fn connect(&mut self, addr: url::Url) 
		-> Result<(), Error> 
	{
		let (mut ws_stream, _) = connect_async(addr).await?;
		WS.with(|ws| { ws.borrow_mut().stream = Some(ws_stream); });
		Ok(())
	}
	pub async fn send_message(message: &[u8]) {
		WS.with(|ws| { 
/*			if let Some(&mut stream) = ws.borrow().stream.as_mut() {
				stream.send(Message::binary(message));
			}; */
		});
		()	
	}
}

pub fn websocket_send_message(ctx: &mut Ctx, message_ptr: U8WasmPtr, len: u32) {
	let memory = ctx.memory(0);
	let message = message_ptr.get_slice(memory, len);
	println!("{}", std::str::from_utf8(message.unwrap()).unwrap());
	WebSocket::send_message(message.unwrap());
}

