use grayarea::{WasmInstance, Opt, WebSocket};
use structopt::StructOpt;
use tungstenite::protocol::Message;
use futures::StreamExt;
use anyhow::anyhow;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let opt = Opt::from_args();
    // Load the plugin data
    let wasm_bytes = opt.load_wasm_bytes();
    let config = opt.load_config();
    let (handle, s) = WasmInstance::spawn(wasm_bytes, &config);
    let mut ws = WebSocket::connect(config.websocket.url.clone()).await?;
    println!("Connected to {}", &config.websocket.url);
    while let Some(msg) = ws.stream.next().await {
        match msg {
            Ok(Message::Text(t)) => s.send(t.into_bytes())?,
            _ => ()
        }
    };
    handle.join().map_err(|err| anyhow!("WASM module failure: {:?}", err))
}
