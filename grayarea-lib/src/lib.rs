pub mod memory;
use std::cell::RefCell;


// TODO: move all below to websocket.rs

// For compiling with wasm32-wasi target
#[link(wasm_import_module = "websocket")]
extern {
    fn send_message(msg: u32, len: u32);
}


thread_local! {
    static HANDLER: RefCell<Option<Box<dyn MessageHandler>>> = RefCell::new(None);
}

pub trait MessageHandler {
    fn on_message(&mut self, message: &[u8]) -> std::io::Result<()>;
}

/// WebSocket connector for grayarea
/// ```
/// WebSocket::send_message(b"hello world!");
/// ```
pub struct WebSocket;

impl WebSocket {
    /// Sends provided bytes slice via websocket
    /// Please note, current implementation might panic on issue with websocket
    /// TODO: rethink error handling
    pub fn send_message(message: &[u8]) {
        unsafe { send_message(message.as_ptr() as u32, message.len() as u32); }
    }
    /// Message handler is required to process incoming messages.
    /// Please note, there is no queue therefore messages received 
    /// before handler initialized will be lost.
    pub fn set_message_handler(new_handler: Box<dyn MessageHandler>) {
        HANDLER.with(|handler| handler.replace(Some(new_handler)));
    }

    // We store reference to handler as a static variable
    fn on_message(message: &[u8]) -> std::io::Result<()>  {
        HANDLER.with(|handler| {
            if let Some(handler) = &mut *handler.borrow_mut() {
                handler.on_message(message)
            } else {
                Err(std::io::Error::new(
                    std::io::ErrorKind::NotConnected,
                    "Message handler was not initialized",
                ))
            }
        })
    }
}

/// This method is exposed to WASM runtime and invoked on incoming message 
/// Current implementation is panicing on failures which will shutdown host runner
/// TODO: rethink error handling
#[no_mangle]
fn on_websocket_message(ptr: *const u8, len: i32) {
    if ptr.is_null() {
        panic!("null pointer passed to on_message");
    }
    let msg = unsafe { std::slice::from_raw_parts(ptr, len as usize) };
    match WebSocket::on_message(msg) {
        Err(err) => panic!(format!("Failed to process message: {}", err)),
        _ => ()
    };
}


#[cfg(test)]
mod tests {
    #[test]
    fn it_works() {
        assert_eq!(2 + 2, 4);
    }
}
