use thiserror::Error;

pub type CompressorInitResult<T> = Result<T, CompressorInitError>;
pub type DecompressorInitResult<T> = Result<T, DecompressorError>;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Error)]
pub enum DecompressorError {
    #[error("failed to allocate decompressor")]
    FailedToAllocate,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Error)]
pub enum CompressorInitError {
    #[error("failed to allocate compressor")]
    FailedToAllocate,

    #[error("Invalid compression level specified")]
    InvalidCompressionLevel,
}
