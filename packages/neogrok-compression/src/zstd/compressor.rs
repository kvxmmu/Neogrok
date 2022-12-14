use std::{
    ffi,
    ptr::NonNull,
};

use zstd_sys::{
    ZSTD_CCtx,
    ZSTD_compressCCtx,
    ZSTD_createCCtx,
    ZSTD_freeCCtx,
    ZSTD_isError,
};

use crate::error::{
    CompressorInitError,
    CompressorInitResult,
};

pub struct ZStdCctx {
    cctx: NonNull<ZSTD_CCtx>,
    level: u8,
}

impl ZStdCctx {
    pub fn compress(
        &mut self,
        buffer: &[u8],
        max_allocate_size: usize,
    ) -> Option<Vec<u8>> {
        unsafe {
            let mut out: Vec<u8> = Vec::with_capacity(max_allocate_size);

            let result;
            {
                let spare = out.spare_capacity_mut();
                let out_ptr = spare.as_ptr() as *mut ffi::c_void;

                result = ZSTD_compressCCtx(
                    self.cctx.as_ptr(),
                    out_ptr,
                    out.capacity(),
                    buffer.as_ptr() as *const _,
                    buffer.len(),
                    self.level as i32,
                );
            }

            if ZSTD_isError(result) == 1 {
                None
            } else {
                out.set_len(result);
                Some(out)
            }
        }
    }

    pub fn try_new(level: u8) -> CompressorInitResult<Self> {
        let cctx = unsafe { ZSTD_createCCtx() };
        NonNull::new(cctx)
            .map(|cctx| Self { cctx, level })
            .ok_or(CompressorInitError::FailedToAllocate)
    }

    pub fn new(level: u8) -> Self {
        Self::try_new(level)
            .expect("Failed to allocte ZStd compress context")
    }
}

impl Drop for ZStdCctx {
    fn drop(&mut self) {
        unsafe { ZSTD_freeCCtx(self.cctx.as_ptr()) };
    }
}
