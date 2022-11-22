use {
    std::io,
    toml::de::Error as TomlError,
};

#[derive(Debug)]
pub enum Error {
    Io(io::Error),
    Toml(TomlError),
}

impl From<TomlError> for Error {
    fn from(value: TomlError) -> Self {
        Error::Toml(value)
    }
}

impl From<io::Error> for Error {
    fn from(value: io::Error) -> Error {
        Error::Io(value)
    }
}
