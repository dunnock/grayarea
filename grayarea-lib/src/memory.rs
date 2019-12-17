// When the `wee_alloc` feature is enabled, use `wee_alloc` as the global
// allocator.
#[cfg(feature = "wee_alloc")]
#[global_allocator]
static ALLOC: wee_alloc::WeeAlloc = wee_alloc::WeeAlloc::INIT;

// TODO: rewrite messages exchange to interface types rather than mem buffer

// Create a static mutable byte buffer.
// We will use for passing memory between our host and wasm.
// NOTE: global `static mut` means we have to access it with unsafe
// and manually ensure that only one mutable reference exists to it at a time
// but for passing memory between a host and wasm should be fine.
const WASM_MEMORY_BUFFER_SIZE: usize = 1024*1024;
static mut WASM_MEMORY_BUFFER: [u8; WASM_MEMORY_BUFFER_SIZE] = [0; WASM_MEMORY_BUFFER_SIZE];

// Function to return a pointer to our buffer
// in wasm memory
#[no_mangle]
pub fn buffer_pointer() -> *const u8 {
    unsafe { WASM_MEMORY_BUFFER.as_ptr() }
}