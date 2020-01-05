#![allow(clippy::unnecessary_mut_passed)]
//! Opinionated orchestrator for services which communicate via IPC and are not expected to exit
//! It allows to start and control processes, handling all the necessary boilerplate:
//! - Uses tokio::process::Command with predefined params 
//!   to execute commands
//! - Uses log with info+ levels to 
//! - Uses ipc-channel to establish communication from and to processes
//! ```no_run
//! use tokio::process::{Command};
//! use orchestrator::Orchestrator;
//! let orchestrator = Orchestrator::default();
//! orchestrator.start("start".to_owned(), &mut Command::new("set"));
//! orchestrator.connect();
//! ```

mod channel;
pub mod message;

use log::{debug, info, warn, error};
use std::collections::HashMap;
use std::process::Stdio;
use tokio::process::{Command, Child};
use tokio::io::{AsyncRead, BufReader, AsyncBufReadExt};
use tokio::task::JoinHandle;
use futures::future::{try_join_all, FutureExt, Future, FusedFuture};
use futures::{select, pin_mut};
use anyhow::{anyhow, Context};
use ipc_channel::ipc::{IpcOneShotServer, IpcSender, IpcReceiver};
use std::pin::Pin;

/// Channel for duplex communication via IPC
pub type Channel = channel::Channel<message::Message>;
/// IPC Sender for Message
pub type Sender = IpcSender<message::Message>;
/// IPC Receiver for Message
pub type Receiver = IpcReceiver<message::Message>;

// reusable handler of result which should never be given for select!
macro_rules! should_not_complete {
    ( $text:expr, $res:expr ) => {
        match $res {
            Ok(_) => { info!("All the {} completed", $text); Err(anyhow!("All the {} exit", $text)) },
            Err(err) => { error!("{} failure: {}", $text, err); Err(anyhow::Error::from(err)) }
        }
    };
}

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

type BFR<R> = Pin<Box<dyn Future<Output=anyhow::Result<R>>>>;
type TryAllPin = Pin<Box<dyn FusedFuture<Output=anyhow::Result<Vec<()>>>>>;

/// Orchestrator which is in progress of starting up
pub struct Orchestrator {
	pub processes: HashMap<String, Process>,
	loggers: Vec<BFR<()>>,
	bridges: Vec<BFR<Bridge>>,
	ipc: bool,
	rust_backtrace: bool
}

/// Orchestrator with successfully started processes connected via IPC
pub struct ConnectedOrchestrator {
	pub bridges: Vec<Bridge>,
	pipes: Vec<JoinHandle<anyhow::Result<()>>>,
	loggers: TryAllPin,
	processes: TryAllPin
}


impl Default for Orchestrator {
	fn default() -> Self {
		Self {
			processes: HashMap::new(),
			loggers: Vec::new(),
			bridges: Vec::new(),
			ipc: false,
			rust_backtrace: false
		}
	}
}

impl Orchestrator {
	/// Start provided command with communication channel
	/// As opinionated executor for all the processes it provides following setup:
	/// 1. Start IpcOneShotServer and provide server name to process via 
	/// commandline argument `--orchestrator-ch`
	/// 2. cmd.kill_on_drop(true) - process will exit if orchestrator's handle is dropped
	/// 3. cmd.stdout(Stdio::piped()) - stdout will be logged as info!(target: &name, ...)
	pub fn start(&mut self, name: &String, cmd: &mut Command) -> anyhow::Result<()> {
		if self.processes.contains_key(name) {
			return Err(anyhow::anyhow!("process named `{}` already started", name))
		}

		let (server, server_name) = IpcOneShotServer::new()
			.context("Failed to start IpcOneShotServer")?;

		cmd.kill_on_drop(true)
			.stdout(Stdio::piped());
		if self.ipc {
			cmd.arg(format!("--orchestrator-ch={}", server_name));
		}
		if self.rust_backtrace {
			cmd.env("RUST_BACKTRACE", "1");
		}

		debug!(target: "orchestrator", "Starting {} {:?}", name, cmd);

        let mut child = cmd.spawn()?;

        // Redirect command output to stdout - quick and dirty logging
		let stdout = child.stdout().take()
            .ok_or_else(|| anyhow!("child did not provide a handle to stdout"))?;
		self.loggers.push(Box::pin(log_handler(stdout, name.clone())));

		self.processes.insert(name.clone(), Process { name: name.clone(), child });

        // Spawning Ipc Server to accept incoming channel from child process
        let name1 = name.clone();
        let server_res = tokio::task::spawn_blocking(move || 
            server.accept()
                .unwrap_or_else(|err| todo!("failed to establish connection from {}: {}", name1, err)));
        let name = name.clone();
        let bridge = server_res
            .map(|res| match res {
                    Ok((_, channel)) => Ok(Bridge { channel, name }),
                    Err(err) => Err(err.into())
                });
        self.bridges.push(Box::pin(bridge));

		Ok(())
	}

	/// Connect to processes IPC channels
	/// Resulting ConnectedOrchestrator can be used to further setup handlers 
	/// over processes bridges
	pub async fn connect(self) -> anyhow::Result<ConnectedOrchestrator> {

		let Orchestrator { mut processes, bridges, loggers, .. } = self;
		let processes: Vec<BFR<()>> = processes.drain()
			.map(|(_k,v)| v)
			.map(never_exit_process_handler)
			.collect();

		// Main future executor, had to implement due to customized pipeline
		// Wait for all bridges to connect to server and pass ipc handles
		let bridges = try_join_all(bridges).fuse();
		// Wait for all logs to complete or any of them to fail
		let mut loggers = Box::pin(try_join_all(loggers).fuse());
		// Wait for all processes to complete or any of them to fail
		let mut processes = Box::pin(try_join_all(processes).fuse());
		pin_mut!(bridges);

		let res = select!(
			res = bridges => match res {
				Ok(channels) => { Ok(channels) },
				Err(err) => { error!("failed to establish connection: {}", err); Err(err.into()) }
			},
			res = processes => should_not_complete!("processes", res),
			res = loggers => should_not_complete!("logs", res),
		);

		match res {
			Ok(channels) => {
				Ok(ConnectedOrchestrator::new(
					channels,
					processes,
					loggers
				))
			},
			Err(err) => { 
				error!(target: "orchestrator", "{}", &err); 
				Err(err) 
			}
		}
	}

	/// Setup IPC channel
	/// Will pass IpcOneShotServer name via `--orchestrator-ch`
	pub fn ipc(mut self, ipc: bool) -> Self {
		self.ipc = ipc;
		self
	}

	/// Start child process with RUST_BACKTRACE=1 env option
	pub fn rust_backtrace(mut self, backtrace: bool) -> Self {
		self.rust_backtrace = backtrace;
		self
	}
}

impl ConnectedOrchestrator {
	fn new(bridges: Vec<Bridge>, processes: TryAllPin, loggers: TryAllPin) -> Self {
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

	/// Run processes and built pipes to completion
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

async fn log_handler(reader: impl AsyncRead+Unpin, name: String) -> anyhow::Result<()> {
    let mut reader = BufReader::new(reader).lines();
    while let Some(line) = reader.next_line().await? {
        info!(target: &name, "{}", line);
    };
    Err(anyhow!("runtime `{}` closed its output", name))
}

fn never_exit_process_handler(p: Process) -> BFR<()> {
	let Process { child, name } = p;
	let name1 = name.clone();
    Box::pin(child
        .inspect(move |status| warn!(target: &name1, "exiting {:?}", status))
        .map(move |status| match status { 
            Ok(n) => Err(anyhow!("process `{}` finish with {}, closing pipeline", name, n)),
            Err(err) => Err(err.into())
        }))
}


