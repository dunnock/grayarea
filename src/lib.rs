mod output;
pub use output::Output;

pub mod channel;

#[cfg(feature="wasm")]
mod wasm;
#[cfg(feature="wasm")]
mod ptr;
#[cfg(feature="wasm")]
pub use wasm::WasmHandler;
#[cfg(feature="wasm")]
pub use ptr::U8WasmPtr;

// WebSocket module support
#[cfg(feature="ws")]
mod websocket;
#[cfg(feature="ws")]
pub use websocket::{WebSocket, wasm::WasmWSInstance};
