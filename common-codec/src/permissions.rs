use bitflags::bitflags;

bitflags! {
    #[repr(transparent)]
    pub struct Rights: u8 {
        const CAN_CREATE_TCP  = 1 << 0;
        const CAN_CREATE_UDP  = 1 << 1;
        const CAN_CREATE_HTTP = 1 << 2;

        const CAN_SELECT_TCP  = 1 << 3;
        const CAN_SELECT_UDP  = 1 << 4;
        const CAN_SELECT_HTTP = 1 << 5;
    }
}
