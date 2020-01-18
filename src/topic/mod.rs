use crate::{wasm, wasm::WasmHandle, U8WasmPtr, WasmHandler};
use crossbeam::channel;
use ipc_orchestrator::message::Message;
use wasmer_runtime::{func, imports, Ctx};

type Receiver = channel::Receiver<Message>;

pub struct WasmTopicInstance {
    inner: WasmHandler,
    rx: channel::Receiver<Message>,
}

impl WasmTopicInstance {
    /// spawns WASM module in separate thread
    /// TODO: This function is panicing on any exception
    pub fn spawn(wasm_bytes: Vec<u8>, args: Vec<Vec<u8>>, topics: Vec<String>) -> Self {
        let (tx, rx) = channel::bounded::<Message>(crate::CHANNEL_SIZE);
        let topics_len = topics.len() as u32;

        // prepare custom imports for wasm
        let send_topic_message =
            move |ctx: &mut Ctx, topic: u32, message_ptr: U8WasmPtr, len: u32| {
                assert!(
                    topic < topics_len,
                    "send_topic_message: provided topic index {} out of bounds {}",
                    topic,
                    topics_len
                );
                let memory = ctx.memory(0);
                let data = message_ptr
                    .to_vec(memory, len)
                    .expect("send_topic_message: failed to deref message");
                let topic = topics[topic as usize].clone();
                let msg = Message { topic, data };
                tx.send(msg)
                    .expect("send_topic_message: failed to send message");
            };

        let custom_imports = imports! {
            "io" => {
                "send_message_to_topic_idx" => func!(send_topic_message),
            },
        };

        let inner = WasmHandler::spawn(wasm_bytes, args, Some(custom_imports), true);

        WasmTopicInstance { inner, rx }
    }

    pub fn clone_receiver(&self) -> Receiver {
        self.rx.clone()
    }

    pub fn clone_sender(&self) -> Option<wasm::Sender> {
        self.inner.clone_sender()
    }
}

impl Into<WasmHandle> for WasmTopicInstance {
    fn into(self) -> WasmHandle {
        self.inner.into()
    }
}
