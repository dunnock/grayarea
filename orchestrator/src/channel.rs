//! Channel has helper methods and abstraction for creation of Sender and Receiver.
//! Purpose is that in the future it will be easier to change transport.
//! Implementation uses IpcBytesSender / IpcBytesReceiver as they seem order of magnitude faster
//! See https://github.com/dunnock/ipc-bench
//! 
//! TODO: can improve perf ~2 times converting to IpcChannel::bytes_channel()

use ipc_channel::ipc::{self, IpcReceiver, IpcSender};
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Debug)]
pub struct Channel<T>(Option<IpcSender<T>>, Option<IpcReceiver<T>>);

impl<T> Channel<T>
where
    T: for<'de> Deserialize<'de> + Serialize,
{
    pub fn simplex() -> anyhow::Result<Channel<T>> {
        let (tx1, rx1) = ipc::channel()?;
        Ok(Channel(Some(tx1), Some(rx1)))
    }
    pub fn duplex() -> anyhow::Result<(Channel<T>, Channel<T>)> {
        let (tx1, rx1) = ipc::channel()?;
        let (tx2, rx2) = ipc::channel()?;
        Ok((Channel(Some(tx1), Some(rx2)), Channel(Some(tx2), Some(rx1))))
    }
    pub fn split(self) -> anyhow::Result<(IpcSender<T>, IpcReceiver<T>)> {
        let Channel(txo, rxo) = self;
        let tx = txo.ok_or_else(|| anyhow::anyhow!("failed to obtain sending channel"))?;
        let rx = rxo.ok_or_else(|| anyhow::anyhow!("failed to obtain receiving channel"))?;
        Ok((tx, rx))
    }
    pub fn tx_take(&mut self) -> Option<IpcSender<T>> {
        self.0.take()
    }
    pub fn rx_take(&mut self) -> Option<IpcReceiver<T>> {
        self.1.take()
    }
}

unsafe impl<T> Send for Channel<T> where T: Send {}
unsafe impl<T> Sync for Channel<T> where T: Sync {}
