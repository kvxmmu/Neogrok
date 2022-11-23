use bitflags::bitflags;

bitflags! {
    #[repr(transparent)]
    pub struct PacketFlags: u8 {
        const SHORT      = 1 << 0;
        const SHORT2     = 1 << 1;
        const COMPRESSED = 1 << 2;
    }
}
