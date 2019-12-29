use wasmer_runtime::{Memory, types::WasmExternType};

#[derive(Clone, Copy)]
pub struct U8WasmPtr {
	offset: u32
}
unsafe impl WasmExternType for U8WasmPtr {
    type Native = i32;

    fn to_native(self) -> Self::Native {
        self.offset as i32
    }
    fn from_native(n: Self::Native) -> Self {
        Self {
            offset: n as u32
        }
    }
}
impl U8WasmPtr {
    /// Get a u8 slice
    /// # Safety
    /// Slice pointing to memory managed by WASM module, it is recommended to use it only 
    /// within the same thread where WASM module executes making sure that WASM module 
    /// does not overtake control while slice is used. 
    pub unsafe fn get_slice(self, memory: &Memory, len: u32) -> Option<&[u8]> {
        if self.offset as usize + len as usize > memory.size().bytes().0 {
            return None;
        }
        let ptr = memory.view::<u8>().as_ptr().add(self.offset as usize) as *const u8;
        Some(std::slice::from_raw_parts(ptr, len as usize))
    }
    /// Get a u8 slice
    /// # Safety
    /// Slice pointing to memory managed by WASM module, it is recommended to use it only 
    /// within the same thread where WASM module executes making sure that WASM module 
    /// does not overtake control while slice is used. 
    pub unsafe fn get_mut_slice(self, memory: &Memory, len: u32) -> Option<&mut [u8]> {
        if self.offset as usize + len as usize > memory.size().bytes().0 {
            return None;
        }
        let ptr = memory.view::<u8>().as_ptr().add(self.offset as usize) as *mut u8;
        Some(std::slice::from_raw_parts_mut(ptr, len as usize))
    }
    /// Copies memory slice to new Vec<u8>
    /// Safe version of a `get_slice`
    pub fn to_vec(self, memory: &Memory, len: u32) -> Option<Vec<u8>> {
        unsafe { self.get_slice(memory, len) }.map(|s| s.to_vec())
    }
}
