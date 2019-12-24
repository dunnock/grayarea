use grayarea::{WasmHandler, WasmWSInstance, WebSocket};
use grayarea::channel::{Sender, Receiver, Message};
use grayarea_runtime::{ Opt, config };
use structopt::StructOpt;
use tungstenite::protocol::Message as WSMessage;
use futures::future::try_join_all;
use anyhow::{anyhow, Result};
use tokio::task::spawn_blocking;
use crossbeam::channel;

type Handle = tokio::task::JoinHandle<Result<()>>;

async fn ws_processor(tx: Sender, ws: WebSocket, topic: String) -> anyhow::Result<()> {
    while let Some(msg) = ws.read().await {
        let topic = topic.clone();
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
    spawn_blocking(move || 
        loop {
            let msg = rx.recv()?;
            tx.send(msg.data)?;
        }
    ).await?
}


// spawns worker of type input stream
async fn spawn_input(opt: Opt, config: config::ModuleConfig) -> anyhow::Result<Vec<Handle>> {
    let mut handles = Vec::new();
    let wasm_bytes = config.load_wasm_bytes().await?;
    match &config.stream {
        Some(config::StreamOneOf::WebSocket(config::WebSocketConfig{ url })) =>  {
            let wasm_handler = WasmWSInstance::spawn(wasm_bytes, config.args_as_bytes());

            // Connect to pipeline via IPC
            let (stx, _) = opt.ipc_channel().await?.split();
            let stx = stx.ok_or_else(|| anyhow!("failed to create sending channel"))?;

            let ws = WebSocket::default();
            ws.connect(url.clone()).await?;
            // TODO - structured logging to stderr 
            println!("Connected to {}", &url); 
            // Spawn websocket messages processor
            if let Some(config::Output { topics }) = config.output {
                let ws_handle = tokio::spawn(ws_processor(stx, ws.clone(), topics[0].clone()));
                handles.push(ws_handle);
            }

            // Spawn wasm message processor
            let wasm_msgs_handle = tokio::spawn(async move { ws.set_handshaker(&wasm_handler).await });
            handles.push(wasm_msgs_handle);
        },
        None => panic!("Stream configuration was not provided, it's required for *input* type of instance!")
    };
    Ok(handles)
}

// spawns worker of type processor
async fn spawn_processor(opt: Opt, config: config::ModuleConfig) -> anyhow::Result<Vec<Handle>> {
    let mut handles = Vec::new();
    let wasm_bytes = config.load_wasm_bytes().await?;
    let mut wasm_handler = WasmHandler::spawn(wasm_bytes, config.args_as_bytes(), None, true);

    if opt.has_ipc() {
        let (stx, srx) = opt.ipc_channel().await?.split();
        let _stx = stx.ok_or_else(|| anyhow!("failed to create sending channel"))?;
        let srx = srx.ok_or_else(|| anyhow!("failed to create receiving channel"))?;

        // spawn IPC messages processor
        if let Some(tx) = wasm_handler.clone_sender() {
            let ws_handle = tokio::spawn(msg_processor(tx, srx));
            handles.push(ws_handle);
        }
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
        config::ModuleKind::Processor => spawn_processor(opt, config).await?
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
