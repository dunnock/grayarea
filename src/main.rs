use grayarea::{WasmInstance, Opt, WebSocket};
use structopt::StructOpt;


#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let opt = Opt::from_args();
    // Load the plugin data
    let wasm_bytes = opt.load_wasm_bytes();
    let config = opt.load_config();
    let ws = WebSocket::connect(config.websocket.url.clone()).await?;
    println!("Connected to {}", &config.websocket.url);
    let instance = WasmInstance::init(wasm_bytes, &config)?;
    instance.start()
}
