use log::{info, error};
use tokio::task::JoinHandle;
use futures::future::{try_join_all, FutureExt, FusedFuture};
use futures::{select, pin_mut};
use anyhow::{anyhow};
use std::pin::Pin;
use crate::{Receiver, Bridge, Sender};
use crate::message;
use crate::should_not_complete;

type TryAllPin = Pin<Box<dyn FusedFuture<Output=anyhow::Result<Vec<()>>>>>;

/// Orchestrator with successfully started processes connected via IPC
pub struct ConnectedOrchestrator<LF: FusedFuture> {
	pub bridges: Vec<Bridge>,
	pipes: Vec<JoinHandle<anyhow::Result<()>>>,
	loggers: Pin<Box<LF>>,
	processes: TryAllPin
}

impl<LF> ConnectedOrchestrator<LF>
where LF: FusedFuture<Output=anyhow::Result<Vec<()>>> {
	pub(crate) fn new(bridges: Vec<Bridge>, processes: TryAllPin, loggers: Pin<Box<LF>>) -> Self {
		ConnectedOrchestrator{
			bridges,
			processes,
			loggers,
			pipes: Vec::new()
		}
	}

	/// Build a pipe from bridge b_in to b_out
	/// Spawns pipe handler in a tokio blocking task thread
	/// - b_in index of incoming bridge from Self::bridges
	/// - b_out index of outgoing bridge from Self::bridges
	pub fn pipe_bridges(&mut self, b_in: usize, b_out: usize) -> anyhow::Result<()> {
        let name_1 = self.bridges[b_in].name.clone();
        let name_2 = self.bridges[b_out].name.clone();
        info!("setting communication {} -> {}", name_1, name_2);
        let rx: Receiver = self.bridges[b_in].channel.rx_take()
            .ok_or_else(|| anyhow!("Failed to get receiver from {}", name_1))?;
        let tx: Sender = self.bridges[b_out].channel.tx_take()
            .ok_or_else(|| anyhow!("Failed to get sender from {}", name_2))?;
        let handle = tokio::task::spawn_blocking(move || {
            loop {
                let buf: message::Message = rx.recv()
                    .unwrap_or_else(|err| todo!("receiving message from {} failed: {}", name_1, err));
                tx.send(buf)
                    .unwrap_or_else(|err| todo!("sending message to {} failed: {}", name_2, err));
            }
        });
		self.pipes.push(handle);
		Ok(())
	}

	/// Run processes to completion
	pub async fn run(self) -> anyhow::Result<()> {
		let pipes = try_join_all(self.pipes).fuse();
		pin_mut!(pipes);
		select!(
			res = pipes => should_not_complete!("channels", res) as anyhow::Result<()>,
			res = self.processes => should_not_complete!("processes", res),
			res = self.loggers => should_not_complete!("logs", res),
		)
	}
}
