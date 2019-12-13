use grayarea::{WasmInstance, Opt};
use structopt::StructOpt;

fn main() -> anyhow::Result<()> {
    let opt = Opt::from_args();
    // Load the plugin data
    let wasm_bytes = opt.load_wasm_bytes();
    let config = opt.load_config();

    let instance = WasmInstance::init(&wasm_bytes[..], &config)?;
    instance.start()
}
