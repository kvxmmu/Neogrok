use crate::{
    deflate::{
        compressor::DeflateCompressor,
        decompressor::DeflateDecompressor,
    },
    error::{
        DecompressError,
        DecompressResult,
    },
    zstd::{
        compressor::ZStdCctx,
        decompressor::ZStdDctx,
    },
};

pub enum BufCompressor {
    ZStd(ZStdCctx),
    Deflate(DeflateCompressor),
}

pub enum BufDecompressor {
    ZStd(ZStdDctx),
    Deflate(DeflateDecompressor),
}

impl BufDecompressor {
    pub fn decompress_unconstrained(
        &mut self,
        src: &[u8],
    ) -> DecompressResult<Vec<u8>> {
        fn try_decompress(
            src: &[u8],
            max_size: usize,
            decompressor: &mut BufDecompressor,
        ) -> DecompressResult<Vec<u8>> {
            match decompressor {
                BufDecompressor::Deflate(deflate) => {
                    deflate.decompress(src, max_size)
                }

                _ => unreachable!(),
            }
        }

        #[allow(clippy::single_match)]
        match self {
            // ZStd decompressor will not allocate whole `usize::MAX` size,
            // only needed size
            Self::ZStd(zstd) => {
                return zstd.decompress(src, usize::MAX);
            }

            _ => {}
        }

        let mut max_size = 4096_usize;
        Ok(loop {
            match try_decompress(src, max_size, self) {
                Ok(b) => break b,
                Err(DecompressError::InsufficientSpace) => {}
                Err(e) => return Err(e),
            }

            // max_size *= 1.5
            max_size = (max_size << 1) - (max_size >> 1);
        })
    }

    pub fn decompress_constrained(
        &mut self,
        src: &[u8],
        max_size: usize,
    ) -> DecompressResult<Vec<u8>> {
        match self {
            Self::ZStd(zstd) => zstd.decompress(src, max_size),
            Self::Deflate(deflate) => deflate.decompress(src, max_size),
        }
    }

    pub fn deflate() -> Self {
        Self::Deflate(DeflateDecompressor::new())
    }

    pub fn zstd() -> Self {
        Self::ZStd(ZStdDctx::new())
    }
}

impl BufCompressor {
    pub fn compress(
        &mut self,
        src: &[u8],
        max_size: usize,
    ) -> Option<Vec<u8>> {
        match self {
            Self::Deflate(deflate) => deflate.compress(src, max_size),
            Self::ZStd(zstd) => zstd.compress(src, max_size),
        }
    }

    pub fn deflate(level: u8) -> Self {
        Self::Deflate(DeflateCompressor::new(level))
    }

    pub fn zstd(level: u8) -> Self {
        Self::ZStd(ZStdCctx::new(level))
    }
}

unsafe impl Send for BufCompressor {}
unsafe impl Send for BufDecompressor {}
