#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Protocol {
    Tcp = 0,
    Udp = 1,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CodecSide {
    Client,
    Server,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Compression {
    Zstd = 0,
    Deflate = 1,
}

pub mod compression;
pub mod permissions;

pub use {
    deflate,
    zstd,
};
