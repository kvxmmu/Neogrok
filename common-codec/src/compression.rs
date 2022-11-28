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

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CompressionStatus {
    pub before: u16,
    pub after: u16,
}

pub enum PayloadCompressor {
    Deflate(DeflateCompressor),
    ZStd(ZStdCctx),
}

pub enum PayloadDecompressor {
    Deflate(DeflateDecompressor),
    ZStd(ZStdDctx),
}

impl CompressionStatus {
    pub fn ratio(&self) -> f64 {
        (self.before as f64) / (self.after as f64)
    }
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

// SAFETY: This is safe because corresponding function
// taking mutable reference so they can not be used across
// threads simulatenously
unsafe impl Send for PayloadCompressor {}
unsafe impl Send for PayloadDecompressor {}
