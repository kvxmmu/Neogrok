use {
    std::{
        ffi,
        ptr::NonNull,
    },
    zstd_sys::{
        ZSTD_DCtx,
        ZSTD_createDCtx,
        ZSTD_decompressDCtx,
        ZSTD_freeDCtx,
        ZSTD_getDecompressedSize,
        ZSTD_isError,
    },
};

pub struct ZStdDctx {
    dctx: NonNull<ZSTD_DCtx>,
}

impl ZStdDctx {
    pub fn decompress(
        &mut self,
        in_buffer: &[u8],
        max_allocate_size: usize,
    ) -> Option<Vec<u8>> {
        let size = unsafe {
            ZSTD_getDecompressedSize(
                in_buffer.as_ptr() as *const _,
                in_buffer.len(),
            )
        };
        if size == 0 || size > max_allocate_size as u64 {
            return None;
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
                None
            } else {
                buffer.set_len(result);
                Some(buffer)
            }
        }
    }

    pub fn try_new() -> Option<Self> {
        NonNull::new(unsafe { ZSTD_createDCtx() })
            .map(|dctx| Self { dctx })
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
