//! Tailored message structure which provides ultra fast serialization/deserialization
//! Tailored to be used with IpcSender / IpcReceiver
//!
//! # Usage
//! ```
//! use orchestrator::message::Message;
//! let msg = Message { topic: 0u32, data: vec![1,2,3,4] };
//! ```

use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct Message {
    //pub topic: String,
    pub topic: String,
    #[serde(with = "serde_bytes")]
    pub data: Vec<u8>,
}
