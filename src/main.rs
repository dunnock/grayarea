use grayarea::{WasmInstance, Opt};
use structopt::StructOpt;

fn main() -> anyhow::Result<()> {
    let opt = Opt::from_args();
    // Load the plugin data
    let wasm_bytes = opt.load_wasm_bytes();

    let instance = WasmInstance::load(&wasm_bytes[..])?;
    instance.start()
}
