//! Tailored message structure which provides ultra fast serialization/deserialization
//! Tailored to be used with IpcSender / IpcReceiver
//! 
//! # Usage
//! ```
//! use grayarea::Message;
//! let msg = Message { topic: 0u32, data: vec![1,2,3,4] };
//! ```

use serde::{Serialize, Deserialize};

#[derive(Serialize, Deserialize, Debug)]
pub struct Message {
	//pub topic: String,
	pub topic: u32,
	#[serde(with = "serde_bytes")]
	pub data: Vec<u8>
}
