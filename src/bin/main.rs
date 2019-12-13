use grayarea::{WasmInstance, Opt, WebSocket};
use structopt::StructOpt;
use tungstenite::protocol::Message;
use futures::StreamExt;
use anyhow::anyhow;
use tokio::sync::Mutex;
use std::sync::Arc;
use crossbeam::channel;

async fn send_from_wasm(r: channel::Receiver<Vec<u8>>, ws: Arc<Mutex<WebSocket>>) -> anyhow::Result<()> {
    loop {
        let r = r.clone();
        // Some workaround to wait on sync message from crossbeam
        // TODO: probably whole WASM <-> Tokio communication shall be rethought!
        if let Some(msg) = tokio::task::spawn_blocking(move || r.iter().next()).await? {
            let mut ws_g = ws.lock().await;
            ws_g.send_message(msg).await?;    
        }
    };
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let opt = Opt::from_args();
    // Load the plugin data
    let wasm_bytes = opt.load_wasm_bytes();
    let config = opt.load_config();

    // Spawn wasm module in separate thread
    // also receive msgs bridge to wasm module
    let (handle, tx, rx) = WasmInstance::spawn(wasm_bytes, &config);
    let ws = Arc::new(Mutex::new(WebSocket::connect(config.websocket.url.clone()).await?));
    println!("Connected to {}", &config.websocket.url);
    let wasm_msgs_handle = tokio::spawn(send_from_wasm(rx, ws.clone()));
    while let Some(msg) = ws.lock().await.stream.next().await {
        match msg {
            Ok(Message::Text(t)) => tx.send(t.into_bytes())?, // this might block - think again if we shall block here
            Ok(Message::Binary(t)) => tx.send(t)?, // this might block - think again if we shall block here
            Ok(Message::Ping(v)) => ws.lock().await.pong(v).await?,
            _ => panic!()
        }
    };
    wasm_msgs_handle.await?;
    handle.join().map_err(|err| anyhow!("WASM module failure: {:?}", err))
}
