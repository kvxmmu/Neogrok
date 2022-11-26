use {
    deflate::{
        compressor::DeflateCompressor,
        decompressor::DeflateDecompressor,
    },
    zstd::{
        compressor::ZStdCctx,
        decompressor::ZStdDctx,
    },
};

pub enum PayloadCompressor {
    Deflate(DeflateCompressor),
    ZStd(ZStdCctx),
}

pub enum PayloadDecompressor {
    Deflate(DeflateDecompressor),
    ZStd(ZStdDctx),
}

impl PayloadDecompressor {
    pub fn decompress(
        &mut self,
        buffer: &[u8],
        max_allocate_size: usize,
    ) -> Option<Vec<u8>> {
        match self {
            Self::Deflate(deflate) => {
                deflate.decompress(buffer, max_allocate_size)
            }
            Self::ZStd(zstd) => zstd.decompress(buffer, max_allocate_size),
        }
    }

    pub fn zstd() -> Self {
        Self::ZStd(ZStdDctx::new())
    }

    pub fn deflate() -> Self {
        Self::Deflate(DeflateDecompressor::new())
    }
}

impl PayloadCompressor {
    pub fn compress(
        &mut self,
        buffer: &[u8],
        max_allocate_size: usize,
    ) -> Option<Vec<u8>> {
        match self {
            Self::Deflate(deflate) => {
                deflate.compress(buffer, max_allocate_size)
            }
            Self::ZStd(zstd) => zstd.compress(buffer, max_allocate_size),
        }
    }

    pub fn zstd(level: i32) -> Self {
        Self::ZStd(ZStdCctx::new(level))
    }

    pub fn deflate(level: i32) -> Self {
        Self::Deflate(DeflateCompressor::new(level))
    }
}
