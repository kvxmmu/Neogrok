use common::protocol::{
    error::ProtocolError,
    types::*,
};

macro_rules! impl_variants {
    (impl $frame:ident { $(const $id:ident = $expr:expr;)* }) => {
        impl $frame {
            $(
                pub const $id: u8 = $expr;
            )*
        }
    };
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Compression {
    pub level: u8,
    pub algorithm: CompressionAlgorithm,
}

#[derive(Debug, Clone)]
pub enum Frame {
    ServerRequest {
        port: u16,
        protocol: Protocol,
    },
    ServerResponse {
        port: u16,
    },

    PingRequest,
    PingResponse {
        server_name: String,
        compression: Compression,
    },

    UpdateRights {
        new_rights: Rights,
    },
    Error(ProtocolError),

    Connect {
        id: u16,
    },
    Forward {
        id: u16,
        buffer: Vec<u8>,
    },
    Disconnect {
        id: u16,
    },

    AuthThroughMagic {
        magic: String,
    },
}

impl_variants! {
    impl Frame {
        const PING          = 0;
        const ERROR         = 1;

        const CONNECT       = 2;
        const FORWARD       = 3;
        const DISCONNECT    = 4;

        const SERVER        = 5;
        const AUTH_MAGIC    = 6;

        const UPDATE_RIGHTS = 7;
    }
}
