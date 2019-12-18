use structopt::StructOpt;
use grayarea_desktop::Opt;
use tokio::process::Command;
use tokio::io::{BufReader, AsyncBufReadExt};
use ipc_channel::ipc::{IpcOneShotServer};
use grayarea::channel::{Channel};
use futures::future::{try_join_all, FutureExt};
use futures::try_join;
use futures::{select, pin_mut};
use ipc_channel::ipc::{IpcSender, IpcReceiver};
use std::process::Stdio;
use anyhow::anyhow;

struct Bridge {
    channel: Channel,
    name: String
}

// reusable handler of result which should never be given for select!
macro_rules! should_not_complete {
    ( $text:expr, $res:expr ) => {
        match $res {
            Ok(_) => { println!("All the {} completed", $text); Err(anyhow!("All the {} exit", $text)) },
            Err(err) => { eprintln!("{} failure: {}", $text, err); Err(err.into()) }
        }
    };
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let opt = Opt::from_args();
    let config = opt.load_config().await?;    
    
    // Start out commands
    let mut bridges = Vec::new();
    let mut processes = Vec::new();
    let mut logs = Vec::new();
    for stage in config.stages.iter() {
        let (server, server_name) = IpcOneShotServer::new().unwrap();
        
        // Preparing to spawn a child runtime process
        let mut command = Command::new("grayarea-runtime");
        command.arg(&stage.config).arg(format!("-o={}", server_name))
            .kill_on_drop(true)
            .stdout(Stdio::piped());
        println!("grayarea-desktop: Starting {} {:?}", stage.name, command);
        let mut child = command.spawn().expect("failed to spawn");

        // Redirect command output to stdout - quick and dirty logging
        // TODO: make better logging solution...
        let stdout = child.stdout().take()
            .expect("child did not have a handle to stdout");
        let mut reader = BufReader::new(stdout).lines();
        let name = stage.name.clone();
        let log = tokio::spawn(async move { 
            while let Ok(Some(line)) = reader.next_line().await {
                println!("{}: {}", name, line);
            }    
        });
        logs.push(log);

        // Start command with a handle managed by tokio runtime
        let name = stage.name.clone();
        processes.push(child.inspect(move |status| println!("{}: exiting {:?}", name, status)));

        // Spawning Ipc Server to accept incoming channel from child process
        let name = stage.name.clone();
        let server_res = tokio::task::spawn_blocking(move || 
            server.accept()
                .expect(format!("failed to establish connection from {}", name).as_str()));
        let name = stage.name.clone();
        let bridge = server_res
            .map(|res| match res {
                    Ok((_, channel)) => Ok(Bridge { channel, name }),
                    Err(err) => Err(err)
                });
        bridges.push(bridge);
    };


    // Main future executor, had to implement due to customized pipeline
    // Wait for all bridges to connect to server and pass ipc handles
    let bridges_jh = try_join_all(bridges).fuse();
    // Wait for all logs to complete or any of them to fail
    let logs_jh = try_join_all(logs).fuse();
    // Wait for all processes to complete or any of them to fail
    let processes_jh = try_join_all(processes).fuse();
    pin_mut!(bridges_jh, processes_jh, logs_jh);

    let res = select!(
        res = bridges_jh => match res {
            Ok(channels) => { Ok(pipe_all(channels)) },
            Err(err) => { eprintln!("failed to establish connection: {}", err); Err(err.into()) }
        },
        res = processes_jh => should_not_complete!("processes", res),
        res = logs_jh => should_not_complete!("logs", res),
    );

    // Following should run forever, except error
    // Wait for all connections to exchange messages in a loop
    // Wait for all processes to complete or any of them to fail
    let res = match res {
        Ok(channels) => {
            let channels = channels.fuse();
            pin_mut!(channels);
            select!(
                res = channels => match res {
                    // TODO: kill all child processes too
                    Ok(res) => { eprintln!("all channels closed"); Ok(()) },
                    Err(err) => { eprintln!("pipeline communication failure: {}", err); Err(err.into()) }
                },
                res = processes_jh => should_not_complete!("processes", res),
                res = logs_jh => should_not_complete!("logs", res),
            )},
        Err(err) => { dbg!(&err); Err(err.into()) }
    };
    dbg!(res);
    // Killing it hard since some spawned futures might still run
    std::process::exit(1);
}


async fn pipe_all(mut bridges: Vec<Bridge>) -> anyhow::Result<()> {
    let mut futures = Vec::new();
    // Starting communication pipeline
    for i in 0..bridges.len()-1 {
        let name_1 = bridges[i].name.clone();
        let name_2 = bridges[i+1].name.clone();
        println!("engine: setting communication {} -> {}", name_1, name_2);
        let rx: IpcReceiver<Vec<u8>> = bridges[i].channel.rx_take()
            .ok_or_else(|| anyhow!("Failed to get receiver from {}", name_1))?;
        let tx: IpcSender<Vec<u8>> = bridges[i+1].channel.tx_take()
            .ok_or_else(|| anyhow!("Failed to get sender from {}", name_2))?;
        let handle = tokio::task::spawn_blocking(move || {
            loop {
                let buf: Vec<u8> = rx.recv()
                    .expect(format!("receiving message from {} failed", name_1).as_str());
                tx.send(buf)
                    .expect(format!("receiving message to {} failed", name_2).as_str());
            }
        });
        futures.push(handle);
    };
    try_join_all(futures).await?;
    Ok(())
}
