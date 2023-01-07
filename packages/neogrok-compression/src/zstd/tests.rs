use super::{
    compressor::*,
    decompressor::*,
};

#[test]
fn test_compression() {
    let buffer: &'static [u8] = &[0; 1024];

    let mut compressor = ZStdCctx::new(10);
    let mut decompressor = ZStdDctx::new();

    let c_buf = compressor.compress(buffer, buffer.len()).unwrap();
    let d_buf = decompressor
        .decompress(&c_buf, buffer.len())
        .unwrap();

    assert_eq!(buffer, d_buf);
}
