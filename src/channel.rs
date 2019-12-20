use serde::{Serialize, Deserialize};
use ipc_channel::ipc::{self, IpcSender, IpcReceiver};

pub type Sender = IpcSender<Vec<u8>>;
pub type Receiver = IpcReceiver<Vec<u8>>;

#[derive(Serialize, Deserialize)]
pub struct Channel(Option<IpcSender<Vec<u8>>>, Option<IpcReceiver<Vec<u8>>>);

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
	pub fn tx_take(&mut self) -> Option<IpcSender<Vec<u8>>> {
		self.0.take()
	}
	pub fn rx_take(&mut self) -> Option<IpcReceiver<Vec<u8>>> {
		self.1.take()
	}
}

unsafe impl Send for Channel {}
unsafe impl Sync for Channel {}