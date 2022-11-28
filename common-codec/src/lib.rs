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

    Last,
}

pub mod compression;
pub mod permissions;

pub use {
    deflate,
    zstd,
};

impl TryFrom<u8> for Compression {
    type Error = ();

    fn try_from(value: u8) -> Result<Self, Self::Error> {
        let last = Self::Last as u8;
        if value >= last {
            Err(())
        } else {
            unsafe { std::mem::transmute(value) }
        }
    }
}
