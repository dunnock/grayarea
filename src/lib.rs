mod output;
pub use output::Output;
pub mod config;

#[cfg(feature = "wasm")]
mod ptr;
#[cfg(feature = "wasm")]
mod wasm;
#[cfg(feature = "wasm")]
pub use ptr::U8WasmPtr;
#[cfg(feature = "wasm")]
pub use wasm::WasmHandler;

// WebSocket module support
#[cfg(all(feature = "ws", feature = "wasm"))]
mod websocket;
#[cfg(all(feature = "ws", feature = "wasm"))]
pub use websocket::{wasm::WasmWSInstance, WebSocket};

#[cfg(feature = "wasm")]
mod topic;
#[cfg(feature = "wasm")]
pub use topic::WasmTopicInstance;

pub const CHANNEL_SIZE: usize = 1000;
