use {
    libdeflate_sys::{
        libdeflate_alloc_decompressor,
        libdeflate_decompressor,
        libdeflate_deflate_decompress,
        libdeflate_free_decompressor,
        libdeflate_result_LIBDEFLATE_SUCCESS,
    },
    std::{
        ffi,
        ptr::NonNull,
    },
};

pub struct DeflateDecompressor {
    ptr: NonNull<libdeflate_decompressor>,
}

impl DeflateDecompressor {
    pub fn decompress(
        &mut self,
        buffer: &[u8],
        max_decompressed_size: usize,
    ) -> Option<Vec<u8>> {
        unsafe {
            let ptr = buffer.as_ptr();
            let mut buffer: Vec<u8> =
                Vec::with_capacity(max_decompressed_size);
            let mut actual_nbytes_ret = usize::MAX;

            let result;
            {
                let spare = buffer.spare_capacity_mut();
                let buf_ptr = spare.as_mut_ptr() as *mut ffi::c_void;

                result = libdeflate_deflate_decompress(
                    self.ptr.as_ptr(),
                    ptr as *const ffi::c_void,
                    buffer.len(),
                    buf_ptr,
                    buffer.capacity(),
                    &mut actual_nbytes_ret as *mut _,
                );
            }

            if result == libdeflate_result_LIBDEFLATE_SUCCESS {
                if actual_nbytes_ret == usize::MAX {
                    unreachable!();
                }

                buffer.set_len(actual_nbytes_ret);
                Some(buffer)
            } else {
                None
            }
        }
    }

    pub fn try_new() -> Option<Self> {
        NonNull::new(unsafe { libdeflate_alloc_decompressor() })
            .map(|ptr| Self { ptr })
    }

    pub fn new() -> Self {
        Self::default()
    }
}

impl Default for DeflateDecompressor {
    fn default() -> Self {
        Self::try_new().expect("Failed to allocate decompressor")
    }
}

impl Drop for DeflateDecompressor {
    fn drop(&mut self) {
        unsafe { libdeflate_free_decompressor(self.ptr.as_ptr()) }
    }
}
