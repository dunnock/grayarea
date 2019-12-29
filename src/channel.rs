//! Channel has helper methods and abstraction for creation of Sender and Receiver. 
//! Purpose is that in the future it will be easier to change transport.
//! Implementation uses IpcBytesSender / IpcBytesReceiver as they seem order of magnitude faster
//! See https://github.com/dunnock/ipc-bench

use serde::{Serialize, Deserialize};
use ipc_channel::ipc::{self, IpcSender, IpcReceiver};
use crate::Message;

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
	pub fn split(self) -> anyhow::Result<(Sender, Receiver)> {
		let Channel(txo, rxo) = self;
        let tx = txo.ok_or_else(|| anyhow::anyhow!("failed to obtain sending channel"))?;
        let rx = rxo.ok_or_else(|| anyhow::anyhow!("failed to obtain receiving channel"))?;
		Ok((tx, rx))
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