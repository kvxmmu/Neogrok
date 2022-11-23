use {
    crate::protocol::flags::PacketFlags,
    std::io,
};

#[derive(Debug)]
pub enum ReadError {
    Io(io::Error),

    UnknownPacket { packet: u8, flags: PacketFlags },
    UnknownErrorVariant,
    InvalidRightsFlags,

    InvalidString,
}

impl From<io::Error> for ReadError {
    fn from(value: io::Error) -> Self {
        Self::Io(value)
    }
}
