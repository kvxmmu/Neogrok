use {
    super::{
        reader::HisuiReader,
        writer::HisuiWriter,
    },
    neogrok_compression::polymorphic::{
        BufCompressor,
        BufDecompressor,
    },
};

pub fn replace_compression<Reader, Writer>(
    reader: &mut HisuiReader<Reader>,
    writer: &mut HisuiWriter<Writer>,

    compressor: BufCompressor,
    decompressor: BufDecompressor,
) {
    reader.decompressor = decompressor;
    writer.compressor = compressor;
}
