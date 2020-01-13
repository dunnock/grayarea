use anyhow::anyhow;
use futures::future::Future;
use log::info;
use tokio::io::{AsyncBufReadExt, BufReader};
use tokio::process::ChildStdout;

/// Creates default log handler
/// Default log handler will read lines from process stdout
/// and log them with info level adding process name
pub fn default_log_handler(c: ChildStdout, s: String) -> impl Future<Output = anyhow::Result<()>> {
    log_handler(c, s)
}

async fn log_handler(reader: ChildStdout, name: String) -> anyhow::Result<()> {
    let mut reader = BufReader::new(reader).lines();
    while let Some(line) = reader.next_line().await? {
        info!(target: &name, "{}", line);
    }
    Err(anyhow!("runtime `{}` closed its output", name))
}
