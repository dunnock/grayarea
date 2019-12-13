use wasmer_runtime::{func, imports, instantiate, Instance};
use wasmer_wasi::{
    generate_import_object_for_version, WasiVersion
};
use super::{websocket_send_message, Config};
use crossbeam::{channel};

pub struct WasmInstance {
	instance: Instance
}

impl WasmInstance {
	pub fn spawn<'a>(wasm_bytes: Vec<u8>, config: &Config) -> (std::thread::JoinHandle<()>, channel::Sender<Vec<u8>>) {
		// TODO: move base_imports to global cache to avoid loading bytes multiple times?
		// WASI imports
		let mut base_imports = generate_import_object_for_version(WasiVersion::Snapshot0, config.args_as_bytes(), vec![], vec![], vec![(".".to_owned(), ".".into())]);
		// env is the default namespace for extern functions
		let custom_imports = imports! {
			"websocket" => {
				"send_message" => func!(websocket_send_message),
			},
		};
		// The WASI imports object contains all required import functions for a WASI module to run.
		// Extend this imports with our custom imports containing "it_works" function so that our custom wasm code may run.
		base_imports.extend(custom_imports);
		let (sender, r) = channel::bounded::<Vec<u8>>(5);
		let handle = std::thread::spawn(move || {
			// TODO: add use of WASM compiler cache
			let instance = WasmInstance {
				instance: instantiate(&wasm_bytes[..], &base_imports)
					.expect("failed to instantiate module")
			};
			instance.start();
			for msg in r.iter() {
				instance.on_message(&msg[..])
			}
		});

		(handle, sender)
	}

	/// panics - it is run from within thread
	pub fn start(&self) -> () {
		// get a reference to the function "plugin_entrypoint"
		let entry_point = self.instance.func::<(), ()>("_start")
			.expect("failed to find entry point in wasm module");
		// call the "entry_point" function in WebAssembly
	    entry_point.call().expect("failed to execute module")
	}

	/// panics - it is run from within thread
	pub fn on_message(&self, msg: &[u8]) -> () {
		// get a reference to the function "plugin_entrypoint"
		let on_message = self.instance.func::<(i32, i32), ()>("on_message")
			.expect("failed to find entry point in wasm module");
		// call the "entry_point" function in WebAssembly
		on_message.call(msg.as_ptr() as i32, msg.len() as i32)
			.expect("failed to call module's on_message")
	}
}