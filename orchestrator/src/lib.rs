#![allow(clippy::unnecessary_mut_passed)]
//! Opinionated orchestrator for services which communicate via IPC and are not expected to exit
//! It allows to start and control processes, handling all the necessary boilerplate:
//! - Uses tokio::process::Command with predefined params 
//!   to execute commands
//! - Uses log with info+ levels to 
//! - Uses ipc-channel to establish communication from and to processes
//! ```
//! use tokio::process::{Command};
//! use orchestrator::orchestrator;
//! let mut orchestrator = orchestrator().ipc(false);
//! orchestrator.start("start", &mut Command::new("set"));
//! orchestrator.connect();
//! ```

mod channel;
pub mod message;
mod macros;
mod logger;
mod orchestrator;

use log::{info, error};
use tokio::process::{Child};
use tokio::task::JoinHandle;
use futures::future::{try_join_all, FutureExt, FusedFuture};
use futures::{select, pin_mut};
use anyhow::{anyhow};
use ipc_channel::ipc::{IpcSender, IpcReceiver};
use std::pin::Pin;

pub use orchestrator::orchestrator;

/// Channel for duplex communication via IPC
pub type Channel = channel::Channel<message::Message>;
/// IPC Sender for Message
pub type Sender = IpcSender<message::Message>;
/// IPC Receiver for Message
pub type Receiver = IpcReceiver<message::Message>;

pub struct Process {
	name: String,
	child: Child,
}
impl std::fmt::Debug for Process {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "Process {{ {} }}", self.name)
    }
}

#[derive(Debug)]
pub struct Bridge {
    pub channel: Channel,
    pub name: String
}

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
	fn new(bridges: Vec<Bridge>, processes: TryAllPin, loggers: Pin<Box<LF>>) -> Self {
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
