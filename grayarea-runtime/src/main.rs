use grayarea::{WasmHandler, WasmWSInstance, WasmTopicInstance, WebSocket};
use orchestrator::{Sender, Receiver, message::Message};
use grayarea_runtime::{ Opt, config };
use structopt::StructOpt;
use tungstenite::protocol::Message as WSMessage;
use futures::future::{try_join_all, TryFutureExt};
use anyhow::{anyhow, Result};
use tokio::task::spawn_blocking;
use crossbeam::channel;

type Handle = tokio::task::JoinHandle<Result<()>>;

async fn ws_processor(tx: Sender, ws: WebSocket, topic: u32) -> anyhow::Result<()> {
    while let Some(msg) = ws.read().await {
        match msg {
            // Send message as &[u8] to wasm module
            Ok(WSMessage::Text(t)) => tx.send(Message { topic, data: t.into_bytes() })?, // this might block - think again if we shall block here
            Ok(WSMessage::Binary(data)) => tx.send(Message { topic, data })?, // this might block - think again if we shall block here
            // Reply on ping from ws server
            Ok(WSMessage::Ping(v)) => ws.pong(v).await?,
            Ok(WSMessage::Pong(_)) => (),
            // Following is most likely websocket connection error
            // TODO: shall we restart connection on error? 
            //   If that is the case perhaps processor should be part of websocket logic?
            Ok(WSMessage::Close(_)) => { ws.clean().await; return Err(anyhow!("Connection closed")) },
            Err(err) => return Err(err.into())
        }
    };
    Ok(())
}

async fn msg_processor(tx: channel::Sender<Vec<u8>>, rx: Receiver) -> anyhow::Result<()> {
    let res = 
        spawn_blocking(move || 
            loop {
                let msg = rx.recv()?;
                tx.send(msg.data)?;
            }
        ).await;
    dbg!(&res);
    res?
}

async fn out_msg_processor(tx: Sender, rx: channel::Receiver<Message>) -> anyhow::Result<()> {
    let res = 
        spawn_blocking(move || 
            loop {
                let msg = rx.recv()?;
                tx.send(msg)?;
            }
        ).await;
    dbg!(&res);
    res?
}


// spawns worker of type input stream
async fn spawn_input(opt: Opt, config: config::ModuleConfig) -> anyhow::Result<Vec<Handle>> {
    let mut handles = Vec::new();
    let wasm_bytes = config.load_wasm_bytes().await?;
    match &config.stream {
        Some(config::StreamOneOf::WebSocket(config::WebSocketConfig{ url })) =>  {
            let wasm_handler = WasmWSInstance::spawn(wasm_bytes, config.args_as_bytes());

            // Connect to pipeline via IPC
            let (stx, _) = opt.ipc_channel().await?.split()?;

            let ws = WebSocket::default();
            ws.connect(url.clone()).await?;
            // TODO - structured logging to stderr 
            println!("Connected to {}", &url); 
            // Spawn websocket messages processor
            //let topic = config.topics()?.remove(0);
            let topic = 0;
            let ws_handle = tokio::spawn(ws_processor(stx, ws.clone(), topic));
            handles.push(ws_handle);

            // Spawn wasm message processor
            let wasm_msgs_handle = tokio::spawn(async move { ws.set_handshaker(&wasm_handler).await });
            handles.push(wasm_msgs_handle);
        },
        None => panic!("Stream configuration was not provided, it's required for *input* type of instance!")
    };
    Ok(handles)
}

// spawns worker of type processor without specified outputs
async fn spawn_no_output(opt: Opt, config: config::ModuleConfig) -> anyhow::Result<Vec<Handle>> {
    let mut handles = Vec::new();
    let wasm_bytes = config.load_wasm_bytes().await?;
    let args = config.args_as_bytes();
    let wasm_handler = WasmHandler::spawn(wasm_bytes, args, None, true);

    if opt.has_ipc() {
        let (_, srx) = opt.ipc_channel().await?.split()?;

        // spawn IPC messages processor
        let tx = wasm_handler.clone_sender().expect("Receiver of messages not started");
        let ws_handle = tokio::spawn(msg_processor(tx, srx)
            .or_else(|err| async move { panic!("Communication failure: {}", err) }));
        handles.push(ws_handle);
    }

    handles.push(wasm_handler.into());

    Ok(handles)
}

// spawns worker of type processor with specifid outputs
async fn spawn_with_output(opt: Opt, config: config::ModuleConfig) -> anyhow::Result<Vec<Handle>> {
    let mut handles = Vec::new();
    let wasm_bytes = config.load_wasm_bytes().await?;
    let args = config.args_as_bytes();
    let topics = config.topics()?;
    let wasm_handler = WasmTopicInstance::spawn(wasm_bytes, args, topics);

    if opt.has_ipc() {
        let (stx, srx) = opt.ipc_channel().await?.split()?;

        // spawn IPC messages processor
        let tx = wasm_handler.clone_sender().expect("Receiver of messages not started");
        // TODO: is there a way to get rid of this spawn?
        let ws_handle = tokio::spawn(msg_processor(tx, srx)
            .or_else(|err| async move { panic!("Communication failure: {}", err) }));
        handles.push(ws_handle);

        let ws_handle = tokio::spawn(out_msg_processor(stx, wasm_handler.clone_receiver())
            .or_else(|err| async move { panic!("Communication failure: {}", err) }));
        handles.push(ws_handle);
    }

    handles.push(wasm_handler.into());

    Ok(handles)
}


#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let opt = Opt::from_args();
    // Load the plugin data
    let config = opt.load_config().await?;

    // Spawn wasm module in separate thread
    // also receive msgs bridge to wasm module
    let handles = match config.kind {
        config::ModuleKind::Input => spawn_input(opt, config).await?,
        config::ModuleKind::Processor if config.output.is_some()
            => spawn_with_output(opt, config).await?,
        config::ModuleKind::Processor
            => spawn_no_output(opt, config).await?,
    };

    // Await them all in parallel
    // TODO: Implement graceful cancellation and restart on Err
    // until then following code will just force exit killing all running futures
    // https://github.com/Matthias247/futures-intrusive/blob/master/examples/cancellation.rs

    let res = try_join_all(handles).await;
    dbg!(&res);
    // TODO -- rustc bug??: unoptimized build is exiting, release is hanging if without following check:
    match res {
        Err(err) => { dbg!(err); std::process::exit(1) },
        _ => Ok(())
    }
}
