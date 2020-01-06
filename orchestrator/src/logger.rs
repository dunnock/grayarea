use tokio::io::{BufReader, AsyncBufReadExt};
use tokio::process::ChildStdout;
use futures::future::{TryFuture};
use log::{info};
use anyhow::anyhow;


/// Creates DefaultLogHandler
pub fn default_log_handler() -> impl LogHandler {
	DefaultLogHandler::from_handler(log_handler)
}

/// Given stdout log AsyncRead stream handler is processing stream until the end.
/// See `impl DefaultLogHandler` for hints
pub trait LogHandler: Sync+Send {
	type Logger: TryFuture<Ok=(),Error=anyhow::Error>;
	fn handler(&self, reader: ChildStdout, name: String) -> Self::Logger;
}

/// Default log handler will read lines from process stdout 
/// and log them with info level adding process name
pub struct DefaultLogHandler<L> {
	log_handler: fn(ChildStdout, String) -> L
}

impl<L> DefaultLogHandler<L> {
	pub fn from_handler(handler: fn(ChildStdout, String) -> L) -> Self 
	{
		Self {
			log_handler: handler
		}
	}
}

impl<L> LogHandler for DefaultLogHandler<L> where L: TryFuture<Ok=(),Error=anyhow::Error> {
	type Logger = L;
	fn handler(&self, reader: ChildStdout, name: String) -> Self::Logger {
		(self.log_handler)(reader, name)
	}
}

async fn log_handler(reader: ChildStdout, name: String) -> anyhow::Result<()> {
	let mut reader = BufReader::new(reader).lines();
	while let Some(line) = reader.next_line().await? {
		info!(target: &name, "{}", line);
	};
	Err(anyhow!("runtime `{}` closed its output", name))
}	
