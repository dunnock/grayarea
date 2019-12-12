use wasmer_runtime::{WasmPtr, Array, Ctx};

pub fn websocket_send_message(ctx: &mut Ctx, message_ptr: WasmPtr<u8, Array>, len: u32) {
	let memory = ctx.memory(0);
	let message = message_ptr.get_utf8_string(memory, len);
    println!("{:?}", message);
}

