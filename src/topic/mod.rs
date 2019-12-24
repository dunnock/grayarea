use wasmer_runtime::{func, imports, Ctx};
use crate::{U8WasmPtr, WasmHandler, wasm::WasmHandle};
use crossbeam::channel;
use crate::channel::Message;

type Receiver = channel::Receiver<Message>;

pub struct WasmTopicInstance {
	inner: WasmHandler,
	rx: channel::Receiver<Message>
}

impl WasmTopicInstance {
	/// spawns WASM module in separate thread
	/// TODO: This function is panicing on any exception
	pub fn spawn(wasm_bytes: Vec<u8>, args: Vec<Vec<u8>>, topics: Vec<String>) ->  Self {
		let (tx, rx) = channel::bounded::<Message>(1);

		// prepare custom imports for wasm
		let send_topic_message = move |ctx: &mut Ctx, topic: u32, message_ptr: U8WasmPtr, len: u32| {
			let memory = ctx.memory(0);
			let message = message_ptr.get_slice(memory, len)
				.expect("send_topic_message: failed to deref message");
			let topic = topics[topic as usize].clone();
			let msg = Message { topic, data: message.to_vec() };
			tx.send(msg)
				.expect("send_topic_message: failed to send message");
		};

		let custom_imports = imports! {
			"io" => {
				"send_topic_message" => func!(send_topic_message),
			},
		};

		let inner = WasmHandler::spawn(wasm_bytes, args, Some(custom_imports), false);

		WasmTopicInstance { inner, rx }
	}

	pub fn clone_receiver(&self) -> Receiver {
		self.rx.clone()
	}
}

impl Into<WasmHandle> for WasmTopicInstance {
	fn into(self) -> WasmHandle {
		self.inner.into()
	}
}