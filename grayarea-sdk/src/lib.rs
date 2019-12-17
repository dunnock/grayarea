pub mod memory;
pub use anyhow::Result;
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
    fn on_message(&mut self, message: &[u8]) -> Result<()>;
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
}


/// Message handler is required to process incoming messages.
/// Please note, there is no queue therefore messages received 
/// before handler initialized will be lost.
//#[must_use]
pub fn set_message_handler(new_handler: Box<dyn MessageHandler>) {
    HANDLER.with(|handler| handler.replace(Some(new_handler)));
}

// We store reference to handler as a static variable
fn on_message_slice(message: &[u8]) -> Result<()>  {
    HANDLER.with(|handler| {
        if let Some(handler) = &mut *handler.borrow_mut() {
            handler.on_message(message)
        } else {
            Err(Box::new(std::io::Error::new(
                std::io::ErrorKind::NotConnected,
                "Message handler was not initialized",
            )).into())
        }
    })
}

/// This method is exposed to WASM runtime and invoked on incoming message 
/// Current implementation is panicing on failures which will shutdown host runner
/// TODO: rethink error handling
#[no_mangle]
fn on_message(ptr: *const u8, len: i32) {
    if ptr.is_null() {
        panic!("null pointer passed to on_message");
    }
    let msg = unsafe { std::slice::from_raw_parts(ptr, len as usize) };
    match on_message_slice(msg) {
        Err(err) => panic!(format!("Failed to process message: {}", err)),
        _ => ()
    };
}


#[cfg(test)]
mod tests {
    use super::MessageHandler;
    use super::{set_message_handler, on_message};
    use std::sync::{Arc, RwLock};

    struct State(usize);

    struct Processor(Arc<RwLock<State>>);

    impl MessageHandler for Processor {
        fn on_message(&mut self, _: &[u8]) ->  anyhow::Result<()>{
            self.0.write().unwrap().0 += 1;
            Ok(())
        }
    }

    impl Processor {
        pub fn count(&self) -> usize {
            self.0.read().unwrap().0
        }
    }

    #[test]
    fn message_handler() {
        let state = Arc::new(RwLock::new(State(0)));
        let processor = Box::new(Processor(state.clone()));
        set_message_handler(processor);
        on_message(b"message".as_ptr(), 7);
        assert_eq!(state.read().unwrap(), State(1))
    }
}
