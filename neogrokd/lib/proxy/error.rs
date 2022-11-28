use std::io;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum RemoveError {
    ClientDoesNotExists,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SendError {
    NotFound,
    Closed,
}

#[derive(Debug)]
pub enum ListenerError {
    Io(io::Error),
    SendError,
}

impl From<io::Error> for ListenerError {
    fn from(value: io::Error) -> Self {
        ListenerError::Io(value)
    }
}
