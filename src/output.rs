use tokio::io;
use tokio::io::AsyncWriteExt;

pub struct Output {
	pub out: io::Stdout
}

impl Output {
	pub fn new() -> Self {
		Output {
			out: io::stdout()
		}
	}

	pub async fn write_message(&mut self, msg: Vec<u8>) -> std::io::Result<()>  {
		self.out.write_all(msg.as_slice()).await
	}	
}


