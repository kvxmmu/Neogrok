use {
    common_codec::permissions::Rights,
    serde::Deserialize,
};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Deserialize)]
pub struct PermissionEntry {
    pub can_create_tcp: bool,
    pub can_create_udp: bool,
    pub can_create_http: bool,

    pub can_select_tcp: bool,
    pub can_select_udp: bool,
    pub can_select_http: bool,
}

macro_rules! apply_flags {
    ($rights:ident = $($flag_bool:expr => $flag:expr),+ $(,)?) => {
        $(
            if $flag_bool {
                $rights |= $flag;
            }
        );+
    };
}

impl PermissionEntry {
    pub fn into_rights(self) -> Rights {
        let mut rights = Rights::empty();
        apply_flags!(
            rights = self.can_create_tcp  => Rights::CAN_CREATE_TCP,
                     self.can_create_udp  => Rights::CAN_CREATE_UDP,
                     self.can_create_http => Rights::CAN_CREATE_HTTP,

                     self.can_select_tcp  => Rights::CAN_SELECT_TCP,
                     self.can_select_udp  => Rights::CAN_SELECT_UDP,
                     self.can_select_http => Rights::CAN_SELECT_HTTP,
        );

        rights
    }
}
