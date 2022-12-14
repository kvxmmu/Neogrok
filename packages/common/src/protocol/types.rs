use bitflags::bitflags;
use integral_enum::IntegralEnum;

#[derive(IntegralEnum)]
pub enum CompressionAlgorithm {
    Deflate = 0,
    ZStd = 1,
}

#[derive(IntegralEnum)]
pub enum Protocol {
    Tcp = 0,
    Udp = 1,
}

#[derive(IntegralEnum)]
pub enum CodecSide {
    Client = 0,
    Server = 1,
}

bitflags! {
    #[repr(transparent)]
    pub struct PacketFlags: u8 {
        const SHORT      = 1 << 0;
        const SHORT2     = 1 << 1;
        const COMPRESSED = 1 << 2;
    }

    #[repr(transparent)]
    pub struct Rights: u8 {
        const CAN_CREATE_TCP  = 1 << 0;
        const CAN_SELECT_TCP  = 1 << 1;

        const CAN_CREATE_UDP  = 1 << 2;
        const CAN_SELECT_UDP  = 1 << 3;

        const CAN_CREATE_HTTP = 1 << 4;
        const CAN_SELECT_HTTP = 1 << 5;
    }
}

impl Rights {
    pub const fn allowed_to(self, to: Rights) -> bool {
        self.contains(to)
    }
}
