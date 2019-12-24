use serde::{Serialize, Deserialize};
use ipc_channel::ipc::{self, IpcSender, IpcReceiver};

#[derive(Serialize, Deserialize)]
pub struct Message {
	pub topic: String,
	pub data: Vec<u8>
}

pub type Sender = IpcSender<Message>;
pub type Receiver = IpcReceiver<Message>;

#[derive(Serialize, Deserialize)]
pub struct Channel(Option<Sender>, Option<Receiver>);

impl Channel {
	pub fn simplex() -> anyhow::Result<Channel> {
		let (tx1, rx1) = ipc::channel()?;
		Ok(Channel(Some(tx1), Some(rx1)))
	}
	pub fn duplex() -> anyhow::Result<(Channel, Channel)> {
		let (tx1, rx1) = ipc::channel()?;
		let (tx2, rx2) = ipc::channel()?;
		Ok((
			Channel(Some(tx1), Some(rx2)),
			Channel(Some(tx2), Some(rx1))
		))
	}
	pub fn split(self) -> (Option<Sender>, Option<Receiver>) {
		(self.0, self.1)
	}
	pub fn tx_take(&mut self) -> Option<Sender> {
		self.0.take()
	}
	pub fn rx_take(&mut self) -> Option<Receiver> {
		self.1.take()
	}
}

unsafe impl Send for Channel {}
unsafe impl Sync for Channel {}