use wasmer_runtime::{func, imports, instantiate, Instance, Ctx};
use wasmer_wasi::{
    generate_import_object_for_version, WasiVersion
};
use super::{Config, U8WasmPtr};
use crossbeam::channel;

pub struct WasmInstance {
	instance: Instance
}


impl WasmInstance {
	/// spawns WASM module in separate thread
	/// TODO: This function is panicing on any exception
	pub fn spawn<'a>(wasm_bytes: Vec<u8>, config: &Config) 
		-> (tokio::task::JoinHandle<()>, channel::Sender<Vec<u8>>, channel::Receiver<Vec<u8>>) 
	{
		// TODO: move base_imports to global cache to avoid loading bytes multiple times?
		// WASI imports
		let mut base_imports = generate_import_object_for_version(WasiVersion::Snapshot0, config.args_as_bytes(), vec![], vec![], vec![(".".to_owned(), ".".into())]);
		// create communication channels from WASM runner to host app
		let (from_wasm_s, from_wasm_r) = channel::bounded::<Vec<u8>>(5);
		let (to_wasm_s, to_wasm_r) = channel::bounded::<Vec<u8>>(5);
		// prepare custom imports for wasm
		let send_message = move |ctx: &mut Ctx, message_ptr: U8WasmPtr, len: u32| {
			let memory = ctx.memory(0);
			let message = message_ptr.get_slice(memory, len)
				.expect("websocket_send_message: failed to deref message");
			from_wasm_s.send(message.to_vec())
				.expect("send message");
		};
		let custom_imports = imports! {
			"websocket" => {
				"send_message" => func!(send_message),
			},
		};
		base_imports.extend(custom_imports);
		// TODO: when panic is hapenning in the thread it hangs the process
		let handle = tokio::task::spawn_blocking(move || {
			// TODO: add use of WASM compiler cache
			let instance = WasmInstance {
				instance: instantiate(&wasm_bytes[..], &base_imports)
					.expect("failed to instantiate module")
			};
			instance.start();
			for msg in to_wasm_r.iter() {
				instance.on_message(&msg[..])
			}
		});

		(handle, to_wasm_s, from_wasm_r)
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
		let memory = self.instance.context().memory(0);
		let buffer_pointer = self.instance.func::<(), U8WasmPtr>("buffer_pointer")
			.expect("failed to find buffer_pointer in wasm module");
		let buffer = buffer_pointer.call()
			.expect("failed to acquire memory buffer from wasm module");
		if msg.len() > 1024*1024 {
			panic!(format!("Received message {} does not fit into buffer", msg.len()))
		}
		let output = buffer.get_mut_slice(memory, msg.len() as u32)
			.expect("failed to deref buffer as mutable u8 buffer");
		output.copy_from_slice(msg);

		let on_message = self.instance.func::<(U8WasmPtr, i32), ()>("on_websocket_message")
			.expect("failed to find on_websocket_message in wasm module");
		// call the "entry_point" function in WebAssembly
		on_message.call(buffer, msg.len() as i32)
			.expect("failed to call module's on_websocket_message")
	}
}