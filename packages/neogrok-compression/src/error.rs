use neogrok_declmacro::{
    compress::define_copyable_compress_errors,
    define_results,
};

define_results! {
    CompressorInitResult<T>   = <CompressorInitError>,
    DecompressorInitResult<T> = <DecompressorError>,
    DecompressResult<T>       = <DecompressError>,
}

define_copyable_compress_errors! {
    enum DecompressError {
        InvalidCompressedData = "Got invalid compressed data",
        InsufficientSpace = "Insufficient destination buffer size for the compressed data",
    }

    enum DecompressorError {
        FailedToAllocate = "Failed to allocate decompressor"
    }

    enum CompressorInitError {
        FailedToAllocate = "Failed to allocate compressor",
        InvalidCompressionLevel = "Invalid compression level specified"
    }
}
