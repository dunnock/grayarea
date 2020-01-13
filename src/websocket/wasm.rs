use crate::{wasm::WasmHandle, U8WasmPtr, WasmHandler};
use crossbeam::channel;
use wasmer_runtime::{func, imports, Ctx};

type Receiver = channel::Receiver<Vec<u8>>;

pub struct WasmWSInstance {
    inner: WasmHandler,
    rx: Receiver,
}

impl WasmWSInstance {
    /// spawns WASM module in separate thread
    /// TODO: This function is panicing on any exception
    pub fn spawn(wasm_bytes: Vec<u8>, args: Vec<Vec<u8>>) -> Self {
        let (tx, rx) = channel::bounded::<Vec<u8>>(crate::CHANNEL_SIZE);

        // prepare custom imports for wasm
        let send_websocket_message = move |ctx: &mut Ctx, message_ptr: U8WasmPtr, len: u32| {
            let memory = ctx.memory(0);
            let message = message_ptr
                .to_vec(memory, len)
                .expect("send_websocket_message: failed to get message");
            tx.send(message)
                .expect("send_websocket_message: failed to send message");
        };

        let custom_imports = imports! {
            "io" => {
                "send_websocket_message" => func!(send_websocket_message),
            },
        };

        let inner = WasmHandler::spawn(wasm_bytes, args, Some(custom_imports), false);

        WasmWSInstance { inner, rx }
    }

    pub fn clone_receiver(&self) -> Receiver {
        self.rx.clone()
    }
}

impl Into<WasmHandle> for WasmWSInstance {
    fn into(self) -> WasmHandle {
        self.inner.into()
    }
}
