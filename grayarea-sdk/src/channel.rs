#[repr(C)]
pub struct Message {
    topic: String,
    data: Vec<u8>
}

// For compiling with wasm32-wasi target
#[link(wasm_import_module = "io")]
extern {
    fn send_topic_message(ch: u32, ch_len: u32, msg: u32, msg_len: u32);
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
    /// Sends provided bytes slice to topic
    /// Please note, current implementation might panic on issue with websocket
    /// TODO: rethink error handling
    pub fn send_message(message: &Message) {
        unsafe { 
			send_topic_message(
				message.topic.as_ptr() as u32,
				message.topic.len() as u32,
				message.data.as_ptr() as u32, 
				message.data.len() as u32
			); 
		}
    }
}
