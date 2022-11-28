use std::fmt::Display;

#[derive(Debug)]
pub enum SendError {
    ClientDoesNotExists,
    SendError,
}

impl Display for SendError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::ClientDoesNotExists => {
                f.write_str("Client is not exists")
            }
            Self::SendError => f.write_str("Send error"),
        }
    }
}
