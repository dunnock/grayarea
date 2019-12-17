mod wasm;
mod options;
mod config;
mod ptr;
mod output;
pub use wasm::WasmInstance;
pub use options::Opt;
pub use config::Config;
pub use ptr::U8WasmPtr;
pub use output::Output;

// WebSocket module support
#[cfg(feature="ws")]
mod websocket;
#[cfg(feature="ws")]
pub use websocket::WebSocket;
