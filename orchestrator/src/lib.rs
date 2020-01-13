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
mod connected;
mod logger;
mod macros;
pub mod message;
mod orchestrator;

use ipc_channel::ipc::{IpcReceiver, IpcSender};
use tokio::process::Child;

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

/// Communication channel for module `name`
#[derive(Debug)]
pub struct Bridge {
    pub channel: Channel,
    pub name: String,
}
