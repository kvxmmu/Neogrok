use neogrok_protocol::protocol::types::Rights;
use serde::Deserialize;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Deserialize)]
pub struct ProtocolEntry {
    pub tcp: bool,
    pub udp: bool,
    pub http: bool,
}

#[derive(Debug, Deserialize)]
pub struct PermissionCan {
    pub create: ProtocolEntry,
    pub select: ProtocolEntry,
}

#[derive(Debug, Deserialize)]
pub struct PermissionsEntry {
    pub can: PermissionCan,
}

#[derive(Debug, Deserialize)]
pub struct PermissionsCfg {
    pub base: PermissionsEntry,
    pub magic: PermissionsEntry,
}

impl PermissionCan {
    pub fn to_protocol_rights(&self) -> Rights {
        let mut flags = Rights::empty();
        if self.create.http {
            flags |= Rights::CAN_CREATE_HTTP;
        }
        if self.create.tcp {
            flags |= Rights::CAN_CREATE_TCP;
        }
        if self.create.udp {
            flags |= Rights::CAN_CREATE_UDP;
        }
        if self.select.tcp {
            flags |= Rights::CAN_SELECT_TCP;
        }
        if self.select.udp {
            flags |= Rights::CAN_SELECT_UDP;
        }
        if self.select.http {
            flags |= Rights::CAN_SELECT_HTTP;
        }

        flags
    }
}

impl PermissionsEntry {
    pub fn to_protocol_rights(&self) -> Rights {
        let mut flags = Rights::empty();

        // Can
        flags |= self.can.to_protocol_rights();

        flags
    }
}
