// For compiling with wasm32-wasi target
#[link(wasm_import_module = "io")]
extern {
    fn send_websocket_message(msg: u32, len: u32);
}

/// WebSocket connector for grayarea
/// 
/// ```ignore
/// WebSocket::send_message(b"hello world!");
/// ```
pub struct WebSocket;

impl WebSocket {
    /// Sends provided bytes slice via websocket
    /// Please note, current implementation might panic on issue with websocket
    /// TODO: rethink error handling
    pub fn send_message(message: &[u8]) {
        unsafe { send_websocket_message(message.as_ptr() as u32, message.len() as u32); }
    }
}
