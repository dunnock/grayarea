use serde::{Serialize, Deserialize};
use ipc_channel::ipc::{self, IpcSender, IpcReceiver};

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
	pub fn tx_ref(&self) -> &IpcSender<Vec<u8>> {
		&self.0
	}
	pub fn rx_ref(&self) -> &IpcReceiver<Vec<u8>> {
		&self.1
	}
	pub fn tx_copy(&self) -> anyhow::Result<IpcSender<Vec<u8>>> {
		let ser: Vec<u8> = bincode::serialize(&self.0)?;
		bincode::deserialize(&ser[..]).map_err(|err| err.into())
	}
	pub fn rx_copy(&self) -> anyhow::Result<IpcReceiver<Vec<u8>>> {
		let ser: Vec<u8> = bincode::serialize(&self.1)?;
		bincode::deserialize(&ser[..]).map_err(|err| err.into())
	}
}

unsafe impl Send for Channel {}
unsafe impl Sync for Channel {}