use thiserror::Error;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Error)]
#[repr(u8)]
pub enum ProtocolError {
    #[error("functionality currently is not implemented")]
    NotImplemented = 0,

    #[error("no access to this command")]
    AccessDenied = 1,

    #[error("invalid credentials specified")]
    InvalidCredentials = 2,

    #[error("unexpected frame sent")]
    UnexpectedFrame = 3,

    #[error("unknown frame sent")]
    UnknownFrame = 4,

    #[error("failed to create server with specified properties")]
    FailedToCreateServer = 5,

    #[error("server is not created")]
    ServerIsNotCreated = 6,

    #[error("no such client")]
    NoSuchClient = 7,

    #[error("reserved field")]
    Reserved,
}

crate::impl_transmute!(ProtocolError);
