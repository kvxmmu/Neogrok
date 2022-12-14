use std::{
    ffi,
    ptr::NonNull,
};

use libdeflate_sys::{
    libdeflate_alloc_compressor,
    libdeflate_compressor,
    libdeflate_deflate_compress,
    libdeflate_deflate_compress_bound,
    libdeflate_free_compressor,
};

use crate::error::{
    CompressorInitError,
    CompressorInitResult,
};

pub struct DeflateCompressor {
    ptr: NonNull<libdeflate_compressor>,
}

impl DeflateCompressor {
    pub fn compress(
        &mut self,
        buffer: &[u8],
        max_available: usize,
    ) -> Option<Vec<u8>> {
        let mut out: Vec<u8> = Vec::with_capacity(unsafe {
            libdeflate_deflate_compress_bound(
                self.ptr.as_ptr(),
                max_available,
            )
        });

        unsafe {
            let compressed_size;
            {
                let out_spare = out.spare_capacity_mut();
                let out_ptr = out_spare.as_mut_ptr() as *mut ffi::c_void;
                compressed_size = libdeflate_deflate_compress(
                    self.ptr.as_ptr(),
                    buffer.as_ptr() as *const ffi::c_void,
                    buffer.len(),
                    out_ptr,
                    out.capacity(),
                );
            }

            if compressed_size != 0 {
                out.set_len(compressed_size);
                Some(out)
            } else {
                None
            }
        }
    }

    pub fn try_new(level: u8) -> CompressorInitResult<Self> {
        if level > 12 {
            return Err(CompressorInitError::InvalidCompressionLevel);
        }

        let compressor =
            unsafe { libdeflate_alloc_compressor(level as _) };

        NonNull::new(compressor)
            .map(|ptr| Self { ptr })
            .ok_or(CompressorInitError::FailedToAllocate)
    }

    pub fn new(level: u8) -> Self {
        Self::try_new(level).expect("Failed to create compressor")
    }
}

impl Drop for DeflateCompressor {
    fn drop(&mut self) {
        unsafe {
            libdeflate_free_compressor(self.ptr.as_ptr());
        }
    }
}
