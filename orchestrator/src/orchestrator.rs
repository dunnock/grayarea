use crate::connected::ConnectedOrchestrator;
use crate::logger::default_log_handler;
use crate::should_not_complete;
use crate::{Bridge, Channel, Process};
use anyhow::{anyhow, Context};
use futures::future::{try_join_all, Fuse, Future, FutureExt, TryFuture, TryJoinAll};
use futures::{pin_mut, select};
use ipc_channel::ipc::IpcOneShotServer;
use log::{debug, error, info, warn};
use std::collections::HashMap;
use std::pin::Pin;
use std::process::Stdio;
use tokio::process::ChildStdout;
use tokio::process::Command;

type BFR<R> = Pin<Box<dyn Future<Output = anyhow::Result<R>>>>;

/// Create default orchestrator
///
/// Default orchestrator comes with `default_log_handler`
///
/// Default log handler will read lines from process stdout
/// and log them with info level adding process name
pub fn orchestrator() -> Orchestrator<impl Future<Output = anyhow::Result<()>>> {
    Orchestrator::from_handlers(default_log_handler)
}

/// Orchestrator which is in progress of starting up
pub struct Orchestrator<LF: TryFuture> {
    pub processes: HashMap<String, Process>,
    loggers: Vec<LF>,
    bridges: Vec<BFR<Bridge>>,
    ipc: bool,
    rust_backtrace: bool,
    logger: fn(ChildStdout, String) -> LF,
}

impl<LF: TryFuture> Orchestrator<LF> {
    /// Create orchestrator with provided log handler
    ///
    /// Log handler is a function: `fn(ChildStdout, String) -> impl TryFuture`
    /// Provided future should process ChildStdout until eof,
    /// returning with anyhow::Result<()>
    pub fn from_handlers(logger: fn(ChildStdout, String) -> LF) -> Self {
        Self {
            processes: HashMap::new(),
            loggers: Vec::new(),
            bridges: Vec::new(),
            ipc: false,
            rust_backtrace: false,
            logger,
        }
    }
}

impl<LF> Orchestrator<LF>
where
    LF: Future<Output = anyhow::Result<()>>,
{
    /// Start provided command with communication channel
    /// As opinionated executor for all the processes Orchestrator provides following setup:
    /// 1. Start IpcOneShotServer and provide server name to process via
    /// commandline argument `--orchestrator-ch`
    /// 2. cmd.kill_on_drop(true) - process will exit if orchestrator's handle is dropped
    /// 3. cmd.stdout(Stdio::piped()) - stdout will be logged as info!(target: &name, ...)
    pub fn start(&mut self, name: &str, cmd: &mut Command) -> anyhow::Result<()> {
        if self.processes.contains_key(name) {
            return Err(anyhow::anyhow!("process named `{}` already started", name));
        }

        let (server, server_name) =
            IpcOneShotServer::new().context("Failed to start IpcOneShotServer")?;

        cmd.kill_on_drop(true).stdout(Stdio::piped());
        if self.ipc {
            cmd.arg(format!("--orchestrator-ch={}", server_name));
        }
        if self.rust_backtrace {
            cmd.env("RUST_BACKTRACE", "1");
        }

        debug!(target: "orchestrator", "Starting {} {:?}", name, cmd);

        let mut child = cmd.spawn()?;

        // Redirect command output to stdout - quick and dirty logging
        let stdout = child
            .stdout
            .take()
            .ok_or_else(|| anyhow!("child did not provide a handle to stdout"))?;
        self.loggers.push((self.logger)(stdout, name.to_owned()));

        self.processes.insert(
            name.to_owned(),
            Process {
                name: name.to_owned(),
                child,
            },
        );

        // Spawning Ipc Server to accept incoming channel from child process
        if self.ipc {
            self.bridges
                .push(Box::pin(ipc_handler(server, name.to_owned())));
        }

        Ok(())
    }

    /// Connect to processes IPC channels
    /// Resulting ConnectedOrchestrator can be used to further setup handlers
    /// over processes bridges
    pub async fn connect(self) -> anyhow::Result<ConnectedOrchestrator<Fuse<TryJoinAll<LF>>>> {
        let Orchestrator {
            mut processes,
            bridges,
            loggers,
            ..
        } = self;
        let processes: Vec<BFR<()>> = processes
            .drain()
            .map(|(_k, v)| v)
            .map(never_exit_process_handler)
            .collect();

        // Main future executor, had to implement due to customized pipeline
        // Wait for all bridges to connect to server and pass ipc handles
        let bridges = try_join_all(bridges).fuse();
        // Wait for all logs to complete or any of them to fail
        let mut loggers = Box::pin(try_join_all(loggers).fuse());
        //let i: u32 = loggers;
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
            Ok(channels) => Ok(ConnectedOrchestrator::new(channels, processes, loggers)),
            Err(err) => {
                error!(target: "orchestrator", "{}", &err);
                Err(err)
            }
        }
    }
}

impl<LF: TryFuture> Orchestrator<LF> {
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

async fn ipc_handler(server: IpcOneShotServer<Channel>, name: String) -> anyhow::Result<Bridge> {
    let name1 = name.clone();
    let server = tokio::task::spawn_blocking(move || {
        server
            .accept()
            .unwrap_or_else(|err| todo!("failed to establish connection from {}: {}", name1, err))
    });
    let name = name.clone();
    server
        .map(|res| match res {
            Ok((_, channel)) => Ok(Bridge { channel, name }),
            Err(err) => Err(err.into()),
        })
        .await
}

fn never_exit_process_handler(p: Process) -> BFR<()> {
    let Process { child, name } = p;
    let name1 = name.clone();
    Box::pin(
        child
            .inspect(move |status| warn!(target: &name1, "exiting {:?}", status))
            .map(move |status| match status {
                Ok(n) => Err(anyhow!(
                    "process `{}` finish with {}, closing pipeline",
                    name,
                    n
                )),
                Err(err) => Err(err.into()),
            }),
    )
}
