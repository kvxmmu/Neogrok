use std::fmt::Display;

#[derive(Debug)]
pub enum SendError {
    ClientDoesNotExists,
    Kanal(kanal::SendError),
}

impl Display for SendError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::ClientDoesNotExists => {
                f.write_str("Client is not exists")
            }
            Self::Kanal(k) => {
                f.write_fmt(format_args!("kanal error: {}", k))
            }
        }
    }
}
