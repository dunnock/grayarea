use grayarea::{WasmInstance, WebSocket};
use grayarea::channel::{Sender, Receiver};
use grayarea_runtime::{ Opt, config };
use structopt::StructOpt;
use tungstenite::protocol::Message;
use futures::future::try_join_all;
use anyhow::anyhow;
use tokio::sync::Mutex;
use tokio::task::spawn_blocking;
use std::sync::Arc;
use crossbeam::channel;

async fn message_from_wasm(rx: channel::Receiver<Vec<u8>>, ws: Arc<Mutex<WebSocket>>) -> anyhow::Result<()> {
    loop {
        let rx = rx.clone();
        // Some workaround to wait on sync message from crossbeam while not blocking Tokio
        // TODO: probably whole WASM <-> Tokio communication shall be rethought!
        if let Some(msg) = spawn_blocking(move || rx.iter().next()).await? {
            let mut ws_g = ws.lock().await;
            ws_g.send_message(msg).await?;    
        }
    };
}

async fn ws_processor(tx: Sender, ws: Arc<Mutex<WebSocket>>) -> anyhow::Result<()> {
    while let Some(msg) = ws.lock().await.read().await {
        match msg {
            // Send message as &[u8] to wasm module
            Ok(Message::Text(t)) => tx.send(t.into_bytes())?, // this might block - think again if we shall block here
            Ok(Message::Binary(t)) => tx.send(t)?, // this might block - think again if we shall block here
            // Reply on ping from ws server
            Ok(Message::Ping(v)) => ws.lock().await.pong(v).await?,
            Ok(Message::Pong(_)) => (),
            // Following is most likely websocket connection error
            // TODO: shall we restart connection on error? 
            //   If that is the case perhaps processor should be part of websocket logic?
            Ok(Message::Close(_)) => { ws.lock().await.stream = None; return Err(anyhow!("Connection closed")) },
            Err(err) => return Err(err.into())
        }
    };
    Ok(())
}

async fn msg_processor(tx: channel::Sender<Vec<u8>>, rx: Receiver) -> anyhow::Result<()> {
    spawn_blocking(move || 
        loop {
            let msg = rx.recv()?;
            tx.send(msg)?;
        }
    ).await?
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let opt = Opt::from_args();
    // Load the plugin data
    let config = opt.load_config().await?;
    let wasm_bytes = config.load_wasm_bytes().await?;
    let mut processors = Vec::new();


    // Spawn wasm module in separate thread
    // also receive msgs bridge to wasm module
    let (wasm_handle, tx, rx) = WasmInstance::spawn(wasm_bytes, config.args_as_bytes());
    processors.push(wasm_handle);

    // Connect to the IPC server
    if opt.has_ipc() {        
        let (stx, srx) = opt.ipc_channel().await?.split();
        let stx = stx.ok_or_else(|| anyhow!("failed to sending create channel"))?;
        let srx = srx.ok_or_else(|| anyhow!("failed to sending create channel"))?;

        // spawn IPC messages processor
        let ws_handle = tokio::spawn(msg_processor(tx, srx));
        processors.push(ws_handle);

        // TODO: add websocket inactivity timeout
        // Connect to websocket server
        let ws = Arc::new(Mutex::new(WebSocket::default()));
        if let Some(config::StreamOneOf::WebSocket(config::WebSocketConfig{ url })) = config.stream {
            ws.lock().await.connect(url.clone()).await?;
            println!("Connected to {}", &url);
            // Spawn websocket messages processor
            let ws_handle = tokio::spawn(ws_processor(stx, ws.clone()));
            processors.push(ws_handle);
        }

        // Spawn wasm message processor
        let wasm_msgs_handle = tokio::spawn(message_from_wasm(rx, ws.clone()));
        processors.push(wasm_msgs_handle);
    }

    // Await them all in parallel
    // TODO: Implement graceful cancellation and restart on Err
    // until then following code will just force exit killing all running futures
    // https://github.com/Matthias247/futures-intrusive/blob/master/examples/cancellation.rs

    let res = try_join_all(processors).await;
    dbg!(&res);
    // TODO -- rustc bug??: unoptimized build is exiting, release is hanging if without following check:
    match res {
        Err(err) => { dbg!(err); std::process::exit(1) },
        _ => Ok(())
    }
}
