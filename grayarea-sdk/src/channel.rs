pub struct Message {
    pub topic: u32,
    pub data: Vec<u8>,
}

// For compiling with wasm32-wasi target
#[link(wasm_import_module = "io")]
extern "C" {
    fn send_topic_message(topic: u32, msg: u32, msg_len: u32);
}

/// Output channel connector for grayarea
///
/// ```ignore
/// let message = Message {
/// 	channel: String { "" },
/// 	message: b"hello world!".to_vec();
/// };
/// Channel::send_message(&message);
/// ```
pub struct Channel;

impl Channel {
    /// Sends bytes to topic
    // TODO: rethink error handling
    pub fn send_message(message: &Message) {
        unsafe {
            send_topic_message(
                message.topic,
                message.data.as_ptr() as u32,
                message.data.len() as u32,
            );
        }
    }
}
