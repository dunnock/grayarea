use structopt::StructOpt;
use grayarea_desktop::Opt;
use tokio::process::Command;
use tokio::io::{BufReader, AsyncBufReadExt};
use ipc_channel::ipc::{IpcOneShotServer};
use grayarea::channel::{Channel};
use futures::future::{try_join_all, FutureExt};
use futures::{select, pin_mut};
use ipc_channel::ipc::{IpcSender, IpcReceiver};
use std::process::Stdio;
use anyhow::anyhow;

struct Bridge {
    channel: Channel,
    name: String
}

async fn pipe_all(channels: Vec<Bridge>) -> anyhow::Result<()> {
    let mut futures = Vec::new();
    // Starting communication pipeline
    for pair in channels.windows(2) {
        let name_1 = pair[0].name.clone();
        let name_2 = pair[1].name.clone();
        println!("engine: setting communication {} -> {}", name_1, name_2);
        let rx: IpcReceiver<Vec<u8>> = pair[0].channel.rx_copy()?;
        let tx: IpcSender<Vec<u8>> = pair[1].channel.tx_copy()?;
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

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let opt = Opt::from_args();
    let config = opt.load_config().await?;    
    
    // Start out commands
    let mut bridges = Vec::new();
    let mut processes = Vec::new();
    for stage in config.stages.iter() {
        let (server, server_name) = IpcOneShotServer::new().unwrap();
        
        // Preparing to spawn a child runtime process
        let mut command = Command::new("grayarea-runtime");
        command.arg(&stage.config).arg(format!("-o={}", server_name))
            .kill_on_drop(true)
            .stdout(Stdio::piped());
        println!("{}: Starting {:?}", stage.name, command);
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
        processes.push(log);

        // Start command with a handle managed by tokio runtime
        let name = stage.name.clone();
        let cmd = tokio::spawn(async move { 
            let status = child.await
                .expect("child process encountered an error");
            println!("{}: exiting with {}", name, status);
        });
        processes.push(cmd);

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

//        let (_, channel): (_, Channel) = server.accept().unwrap().into();
//    pipeline.push(Process { name: name.clone(), channel });

    // Main future executor, had to implement due to customized pipeline
    // Wait for all processes to complete or any of them to fail
    let bridges_jh = try_join_all(bridges).fuse();
    let processes_jh = try_join_all(processes).fuse();
    pin_mut!(bridges_jh, processes_jh);

    let res = select!(
        res = bridges_jh => match res {
            Ok(channels) => { Ok(tokio::spawn(pipe_all(channels))) },
            Err(err) => { eprintln!("failed to establish connection: {}", err); Err(err.into()) }
        },
        res = processes_jh => match res {
            Ok(_) => { println!("All the processes completed"); Err(anyhow!("All the processes exit")) },
            Err(err) => { eprintln!("process failure: {}", err); Err(err.into()) }
        },
    );
    match res {
        Ok(jh) => jh.await??,
        Err(err) => { dbg!(err); std::process::exit(1); }
    };
    Ok(())
}
