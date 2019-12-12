// For compiling with wasm32-wasi target
extern "C" {
    fn websocket_send_message(msg: u32, len: u32);
}


/// WebSocket connector for grayarea
/// ```
/// WebSocket::send_message(b"hello world!");
/// ```
pub struct WebSocket;

impl WebSocket {
    pub fn send_message(message: &[u8]) {
        unsafe { websocket_send_message(message.as_ptr() as u32, message.len() as u32); }
    }
}

#[cfg(test)]
mod tests {
    #[test]
    fn it_works() {
        assert_eq!(2 + 2, 4);
    }
}
