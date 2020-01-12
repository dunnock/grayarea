use wasmer_runtime::{instantiate, Instance, ImportObject};
use wasmer_wasi::{
    generate_import_object_for_version, WasiVersion
};
use super::U8WasmPtr;
use crossbeam::channel;
use anyhow::Result;
use tokio::task::{JoinHandle, spawn_blocking};

pub type Sender = channel::Sender<Vec<u8>>;
pub type WasmHandle = JoinHandle<Result<()>>;

pub struct WasmHandler {
	pub handle: WasmHandle,
	txo: Option<Sender>
}

pub struct WasmInstance {
	instance: Instance,
}

impl WasmHandler {
	/// spawns WASM module in separate thread
	/// TODO: This function is panicing on any exception
	pub fn spawn(wasm_bytes: Vec<u8>, args:  Vec<Vec<u8>>, custom_imports: Option<ImportObject>, message_handler: bool) 
		-> WasmHandler
	{

		// TODO: add structured logging / standard loggin to wasm
		// TODO: add WasiFs, handle stdin/stdout
		// TODO: move base_imports to global cache to avoid loading bytes multiple times?
		// WASI imports
		let mut base_imports = generate_import_object_for_version(WasiVersion::Snapshot1, args, vec![], vec![], vec![]);
		if let Some(imports) = custom_imports {
			base_imports.extend(imports);
		}

		// create communication channels from WASM runner to host app
		let (mut txo, mut rxo) = (None, None);
		if message_handler {
			let (tx, rx) = channel::bounded::<Vec<u8>>(crate::CHANNEL_SIZE);
			txo.replace(tx);
			rxo.replace(rx);
		}

		// TODO: when panic is hapenning in the thread it hangs the process
		let handle = spawn_blocking(move || {
			// TODO: add use of WASM compiler cache
			let instance = WasmInstance {
				instance: instantiate(&wasm_bytes[..], &base_imports)
					.expect("failed to instantiate module")
			};
			instance.start();
			if let Some(rx) = rxo {
				for msg in rx.iter() {
					instance.on_message(&msg[..])
				};
			}
			Ok(())
		});

		WasmHandler { handle, txo }
	}

	pub fn clone_sender(&self) -> Option<Sender> {
		self.txo.clone()
	}
}

impl Into<WasmHandle> for WasmHandler {
	fn into(self) -> WasmHandle {
		self.handle
	}
}

impl WasmInstance {
	/// Start WASM process panics on any exceptions. 
	/// It is run in a WASM thread
	pub fn start(&self) {
		// get a reference to the function "plugin_entrypoint"
		let entry_point = self.instance.func::<(), ()>("_start")
			.expect("failed to find entry point in wasm module");
		// call the "entry_point" function in WebAssembly
	    entry_point.call().expect("failed to execute module")
	}

	/// Message handler for messages sent from a WASM
	/// Panics on exceptions
	/// It runs in a WASM thread
	pub fn on_message(&self, msg: &[u8]) {
		// get a reference to the function "plugin_entrypoint"
		let memory = self.instance.context().memory(0);
		let buffer_pointer = self.instance.func::<(), U8WasmPtr>("buffer_pointer")
			.expect("failed to find buffer_pointer in wasm module");
		let buffer = buffer_pointer.call()
			.expect("failed to acquire memory buffer from wasm module");
		if msg.len() > 1024*1024 {
			panic!(format!("Received message {} does not fit into buffer", msg.len()))
		}
		// Should be safe as it works in the same thread with WASM and 
		// does not give control back to WASM module which manages this memory
		unsafe { 
			let output = buffer.get_mut_slice(memory, msg.len() as u32)
				.expect("failed to deref buffer as mutable u8 buffer");
			output.copy_from_slice(msg);
		}

		let on_message = self.instance.func::<(U8WasmPtr, i32), ()>("on_message")
			.expect("failed to find on_message in wasm module");
		// call the "entry_point" function in WebAssembly
		on_message.call(buffer, msg.len() as i32)
			.expect("failed to call module's on_message")
	}
}