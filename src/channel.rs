use serde::{Serialize, Deserialize};
use ipc_channel::ipc::{self, IpcOneShotServer, IpcSender, IpcReceiver};

pub type Sender = IpcSender<Vec<u8>>;
pub type Receiver = IpcReceiver<Vec<u8>>;

#[derive(Serialize, Deserialize)]
pub struct Channel(IpcSender<Vec<u8>>, IpcReceiver<Vec<u8>>);

impl Channel {
	pub fn simplex() -> anyhow::Result<Channel> {
		let (tx1, rx1) = ipc::channel()?;
		Ok(Channel(tx1, rx1))
	}
	pub fn duplex() -> anyhow::Result<(Channel, Channel)> {
		let (tx1, rx1) = ipc::channel()?;
		let (tx2, rx2) = ipc::channel()?;
		Ok((
			Channel(tx1, rx2),
			Channel(tx2, rx1)
		))
	}
	pub fn split(self) -> (IpcSender<Vec<u8>>, IpcReceiver<Vec<u8>>) {
		(self.0, self.1)
	}
}