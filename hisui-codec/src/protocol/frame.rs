use {
    common_codec::{
        permissions::Rights,
        Protocol,
    },
    std::mem,
};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(u8)]
pub enum ProtocolError {
    /// Functionality is not implemented at the moment
    NotImplemented = 0,

    /// Sent invalid error code
    InvalidErrorCode = 1,

    ///
    ServerIsNotStarted = 2,

    /// Client has no access to perform action
    AccessDenied = 3,

    ///
    MagicAuthIsTurnedOff = 4,

    ///
    ServerIsAlreadyCreated = 5,

    ///
    FailedToBindPort = 6,

    ///
    NoSuchClient = 7,

    /// Reserved error code
    Reserved,
}

#[derive(Debug, Clone)]
pub enum Frame {
    Ping,
    PingResponse { name: String },

    StartServer { port: u16, protocol: Protocol },
    StartHttpServer,

    StartServerResponse { port: u16 },

    AuthThroughMagic { magic: Vec<u8> },
    UpdateRights { rights: Rights },

    Error(ProtocolError),
}

#[rustfmt::skip]
impl Frame {
    pub const PING: u8               = 0;
    pub const START_SERVER: u8       = 1;
    pub const START_HTTP_SERVER: u8  = 2;
    pub const ERROR: u8              = 3;

    pub const AUTH_THROUGH_MAGIC: u8 = 4;
    pub const UPDATE_RIGHTS: u8      = 5;
}

impl TryFrom<u8> for ProtocolError {
    type Error = ();

    fn try_from(value: u8) -> Result<Self, Self::Error> {
        let last = (Self::Reserved as u8) - 1;
        if value <= last {
            Ok(unsafe { mem::transmute(value) })
        } else {
            Err(())
        }
    }
}
