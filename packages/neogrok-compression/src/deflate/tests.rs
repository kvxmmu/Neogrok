use crate::{
    compressor::*,
    decompressor::*,
};

#[test]
fn test_compression() {
    let buffer = Vec::from(b"Hello world, guys" as &[u8]);

    let mut compressor = DeflateCompressor::new(12);
    let mut decompressor = DeflateDecompressor::new();

    let compressed_buf = compressor
        .compress(&buffer, buffer.len())
        .unwrap();
    let decompressed_buf = decompressor
        .decompress(&compressed_buf, buffer.len())
        .unwrap();

    assert_eq!(decompressed_buf, buffer);
}
