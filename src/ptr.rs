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
    pub fn get_slice<'a>(self, memory: &'a Memory, len: u32) -> Option<&'a [u8]> {
        if self.offset as usize + len as usize > memory.size().bytes().0 {
            return None;
        }
        let ptr = unsafe { memory.view::<u8>().as_ptr().add(self.offset as usize) as *const u8 };
        Some(unsafe { std::slice::from_raw_parts(ptr, len as usize) })
    }
    /// Get a u8 slice
    pub fn get_mut_slice<'a>(self, memory: &'a Memory, len: u32) -> Option<&'a mut [u8]> {
        if self.offset as usize + len as usize > memory.size().bytes().0 {
            return None;
        }
        let ptr = unsafe { memory.view::<u8>().as_ptr().add(self.offset as usize) as *mut u8 };
        Some(unsafe { std::slice::from_raw_parts_mut(ptr, len as usize) })
    }
}
