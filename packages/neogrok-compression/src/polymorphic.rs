use crate::{
    deflate::{
        compressor::DeflateCompressor,
        decompressor::DeflateDecompressor,
    },
    error::DecompressResult,
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
    pub fn decompress(
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
