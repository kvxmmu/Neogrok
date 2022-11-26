use {
    crate::protocol::flags::PacketFlags,
    std::{
        fmt::Display,
        io,
    },
};

#[derive(Debug)]
pub enum ReadError {
    Io(io::Error),

    UnknownPacket { packet: u8, flags: PacketFlags },
    UnknownErrorVariant { variant: u8 },
    InvalidRightsFlags { flags: u8 },

    InvalidString,

    TooLongBuffer { expected: usize, found: usize },
    FailedToDecompress,

    InvalidCompressionAlgorithm { code: u8 },
}

impl std::error::Error for ReadError {}

impl Display for ReadError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        if let Self::Io(e) = self {
            return e.fmt(f);
        }

        match self {
            Self::InvalidCompressionAlgorithm { code } => {
                f.write_fmt(format_args!(
                    "Invalid compression algorithm: 0x{:x}",
                    code
                ))
            }

            Self::FailedToDecompress => {
                f.write_str("Failed to decompress buffer")
            }
            Self::TooLongBuffer { expected, found } => {
                f.write_fmt(format_args!(
                    "The buffer is too long, expected: {expected}, \
                     actual: {found}"
                ))
            }
            Self::UnknownErrorVariant { variant } => f.write_fmt(
                format_args!("Received unknown error code: 0x{variant:x}"),
            ),
            Self::InvalidRightsFlags { flags } => {
                f.write_fmt(format_args!(
                    "Received invalid rights flag: 0x{flags:x} (maximum \
                     valid 0x{:x})",
                    PacketFlags::all()
                ))
            }

            Self::InvalidString => f.write_str("Invalid string format"),

            Self::UnknownPacket { packet, flags } => {
                f.write_fmt(format_args!(
                    "Unknown packet 0x{packet:x} with flags 0x{flags:?}",
                ))
            }

            Self::Io(_) => unreachable!(),
        }
    }
}

impl From<io::Error> for ReadError {
    fn from(value: io::Error) -> Self {
        Self::Io(value)
    }
}
