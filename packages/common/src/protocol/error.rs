use integral_enum::IntegralEnum;
use thiserror::Error;

#[derive(IntegralEnum, Error)]
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
}
