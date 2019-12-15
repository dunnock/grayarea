pub use tungstenite::protocol::Message;
use tungstenite::error::Error;
use futures::{SinkExt, StreamExt};
use tokio_tungstenite::{connect_async, WebSocketStream, MaybeTlsStream};
use url;
use anyhow::anyhow;

type WS = WebSocketStream<MaybeTlsStream<tokio::net::TcpStream>>;

pub struct WebSocket {
	pub stream: Option<WS>
}

impl WebSocket {
	pub fn new() -> Self {
		WebSocket{ stream: None }
	}

	pub async fn connect(&mut self, addr: url::Url) -> Result<(), Error> 
	{
		self.stream = Some(connect_async(addr).await?.0);
		Ok(())
	}

	pub async fn send_message(&mut self, msg: Vec<u8>) -> anyhow::Result<()>  {
		println!("{}", std::str::from_utf8(msg.as_slice())?);
		if let Some(stream) = &mut self.stream {
			stream.send(Message::Binary(msg)).await?;
			Ok(())
		} else {
			Err(anyhow!("tried to send message to disconnected WebSocket"))
		}
	}	

	pub async fn pong(&mut self, msg: Vec<u8>) -> anyhow::Result<()>  {
		if let Some(stream) = &mut self.stream {
			stream.send(Message::Pong(msg)).await?;
			Ok(())
		} else {
			Err(anyhow::anyhow!("tried to send message to disconnected WebSocket"))
		}
	}	

	#[inline]
	pub async fn next(&mut self) -> Option<Result<Message, tungstenite::error::Error>>  {
		if let Some(stream) = &mut self.stream {
			stream.next().await
		} else {
			Some(Err(tungstenite::error::Error::AlreadyClosed))
		}
	}	
}


