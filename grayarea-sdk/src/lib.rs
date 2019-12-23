pub mod websocket;
pub mod channel;
pub mod memory;

pub use anyhow::Result;
use std::cell::RefCell;

thread_local! {
    static HANDLER: RefCell<Option<Box<dyn MessageHandler>>> = RefCell::new(None);
}

pub trait MessageHandler {
    fn on_message(&mut self, message: &[u8]) -> Result<()>;
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
    if let Err(err) =  on_message_slice(msg) {
        todo!("Failed to process message: {}", err);
    };
}


#[cfg(test)]
mod tests {
    use super::MessageHandler;
    use super::{set_message_handler, on_message};
    use std::sync::{Arc, RwLock};

    #[derive(Debug, PartialEq)]
    struct State(usize);

    struct Processor(Arc<RwLock<State>>);

    impl MessageHandler for Processor {
        fn on_message(&mut self, _: &[u8]) ->  anyhow::Result<()>{
            self.0.write().unwrap().0 += 1;
            Ok(())
        }
    }

    impl State {
        pub fn count(&self) -> usize {
            self.0
        }
    }

    #[test]
    fn message_handler() {
        let state = Arc::new(RwLock::new(State(0)));
        let processor = Box::new(Processor(state.clone()));
        set_message_handler(processor);
        on_message(b"message".as_ptr(), 7);
        assert_eq!(state.read().unwrap().count(), 1);
    }
}
