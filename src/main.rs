use grayarea::{WasmInstance, Opt, WebSocket};
use structopt::StructOpt;
use tungstenite::protocol::Message;
use futures::StreamExt;
use anyhow::anyhow;
use tokio::sync::Mutex;
use std::sync::Arc;
use crossbeam::channel;

async fn send_from_wasm(r: channel::Receiver<Vec<u8>>, ws: Arc<Mutex<WebSocket>>) {
//    for msg in r.iter() {
    let msg = r.iter().next().unwrap();
        dbg!(&msg);
        let mut ws_g = ws.lock().await;
        ws_g.send_message(msg).await;
//    }
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let opt = Opt::from_args();
    // Load the plugin data
    let wasm_bytes = opt.load_wasm_bytes();
    let config = opt.load_config();

    // Spawn wasm module in separate thread
    // also receive msgs bridge to wasm module
    let (handle, s, r) = WasmInstance::spawn(wasm_bytes, &config);
    let ws = Arc::new(Mutex::new(WebSocket::connect(config.websocket.url.clone()).await?));
    println!("Connected to {}", &config.websocket.url);
    tokio::spawn(send_from_wasm(r, ws.clone()));
    while let Some(msg) = ws.lock().await.stream.next().await {
        match msg {
            Ok(Message::Text(t)) => s.send(t.into_bytes())?,
            _ => ()
        }
    };
    handle.join().map_err(|err| anyhow!("WASM module failure: {:?}", err))
}
