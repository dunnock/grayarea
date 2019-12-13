use wasmer_runtime::{func, imports, instantiate, Instance};
use wasmer_wasi::{
    generate_import_object_for_version, WasiVersion
};
use super::{websocket_send_message, Config};
use anyhow::{anyhow};

pub struct WasmInstance {
	instance: Instance
}

impl WasmInstance {
	pub fn init(wasm_bytes: &[u8], config: &Config) -> anyhow::Result<Self> {
		// WASI imports
		let mut base_imports = generate_import_object_for_version(WasiVersion::Snapshot0, config.args_as_bytes(), vec![], vec![], vec![(".".to_owned(), ".".into())]);
		// env is the default namespace for extern functions
		let custom_imports = imports! {
			"env" => {
				"websocket_send_message" => func!(websocket_send_message),
			},
		};
		// The WASI imports object contains all required import functions for a WASI module to run.
		// Extend this imports with our custom imports containing "it_works" function so that our custom wasm code may run.
		base_imports.extend(custom_imports);

		Ok(WasmInstance {
			instance: instantiate(wasm_bytes, &base_imports)
				.map_err(|err| anyhow!("failed to instantiate module: {}", err))?
		})
	}

	pub fn start(&self) -> anyhow::Result<()> {
		// get a reference to the function "plugin_entrypoint"
		let entry_point = self.instance.func::<(), ()>("_start").unwrap();
		// call the "entry_point" function in WebAssembly
	    entry_point.call().map_err(|err| anyhow!("failed to execute module: {}", err))
	}
}