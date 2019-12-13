use structopt::StructOpt;
use std::path::PathBuf;
use std::fs::read;
use super::Config;

#[derive(StructOpt)] 
#[structopt(name = "grayarea", about = "Serverless WASM runner with WebSocket, HTTP Fetch, Nats and Click House store")]
pub struct Opt {
    /// WASM function code
    #[structopt(parse(from_os_str))]
    wasm: PathBuf,
    /// Function config
    #[structopt(parse(from_os_str), short="c", long="config")]
    config: PathBuf,
}

impl Opt {
    pub fn load_config(&self) -> Config {
        let buf = read(self.config.clone()).expect(
            &format!(
                "Could not read config at {:?}",
                self.config
        ));
        serde_yaml::from_slice(buf.as_slice()).expect("Malformed config")
    }
    pub fn load_wasm_bytes(&self) -> Vec<u8> {
        read(self.wasm.clone()).expect(
            &format!(
                "Could not read WASM plugin at {:?}",
                self.wasm
        ))
    }
}
