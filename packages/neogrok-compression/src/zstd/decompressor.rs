use std::{
    ffi,
    ptr::NonNull,
};

use zstd_sys::{
    ZSTD_DCtx,
    ZSTD_createDCtx,
    ZSTD_decompressDCtx,
    ZSTD_freeDCtx,
    ZSTD_getDecompressedSize,
    ZSTD_isError,
};

use crate::error::{
    DecompressError,
    DecompressResult,
    DecompressorError,
    DecompressorInitResult,
};

pub struct ZStdDctx {
    dctx: NonNull<ZSTD_DCtx>,
}

impl ZStdDctx {
    pub fn decompress(
        &mut self,
        in_buffer: &[u8],
        max_allocate_size: usize,
    ) -> DecompressResult<Vec<u8>> {
        let size = unsafe {
            ZSTD_getDecompressedSize(
                in_buffer.as_ptr() as *const _,
                in_buffer.len(),
            )
        };
        if size == 0 || size > max_allocate_size as u64 {
            return Err(DecompressError::InsufficientSpace);
        }

        unsafe {
            let mut buffer: Vec<u8> = Vec::with_capacity(size as usize);
            let result;
            {
                let spare = buffer.spare_capacity_mut();
                let spare_ptr = spare.as_ptr() as *mut ffi::c_void;

                result = ZSTD_decompressDCtx(
                    self.dctx.as_ptr(),
                    spare_ptr,
                    buffer.capacity(),
                    in_buffer.as_ptr() as *const _,
                    in_buffer.len(),
                );
            }

            if ZSTD_isError(result) == 1 {
                Err(DecompressError::InvalidCompressedData)
            } else {
                buffer.set_len(result);
                Ok(buffer)
            }
        }
    }

    pub fn try_new() -> DecompressorInitResult<Self> {
        NonNull::new(unsafe { ZSTD_createDCtx() })
            .map(|dctx| Self { dctx })
            .ok_or(DecompressorError::FailedToAllocate)
    }

    pub fn new() -> Self {
        Self::default()
    }
}

impl Default for ZStdDctx {
    fn default() -> Self {
        Self::try_new()
            .expect("Failed to allocate ZStd decompression context")
    }
}

impl Drop for ZStdDctx {
    fn drop(&mut self) {
        unsafe {
            ZSTD_freeDCtx(self.dctx.as_ptr());
        }
    }
}
