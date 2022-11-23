use self::flags::PacketFlags;

pub mod flags;
pub mod frame;

pub const fn encode_type(pkt_type: u8, flags: PacketFlags) -> u8 {
    (pkt_type << 3) | flags.bits()
}
