use std::io;

use common::protocol::types::PacketFlags;
use thiserror::Error;

#[derive(Debug, Error)]
pub enum ReadError {
    #[error("Io error: {0:?}")]
    Io(io::Error),

    #[error("invalid packet flags: 0x{flags:x}")]
    InvalidPacketFlags { flags: u8 },

    #[error("invalid packet type: 0x{pkt_type:x} (flags: 0x{flags:x})")]
    InvalidPacketType { pkt_type: u8, flags: PacketFlags },

    #[error("invalid utf8 string")]
    InvalidString,

    #[error("invalid error code: 0x{code:x}")]
    InvalidErrorCode { code: u8 },

    #[error("failed to decompress forward payload")]
    FailedToDecompress,

    #[error("failed to read compression details")]
    FailedToReadCompressionDetails,

    #[error("invalid rights: 0x{rights:x}")]
    InvalidRights { rights: u8 },

    #[error("invalid network protocol")]
    InvalidProtocol,

    #[error("Too long buffer size")]
    TooLongBuffer,
}

impl From<io::Error> for ReadError {
    fn from(value: io::Error) -> Self {
        Self::Io(value)
    }
}
