use std::io;

use thiserror::Error;

#[derive(Debug, Error)]
pub enum ConfigLoadError {
    #[error("io error: {0}")]
    Io(io::Error),

    #[error("toml load filed: {0}")]
    Format(toml::de::Error),
}

impl From<toml::de::Error> for ConfigLoadError {
    fn from(value: toml::de::Error) -> Self {
        Self::Format(value)
    }
}

impl From<io::Error> for ConfigLoadError {
    fn from(value: io::Error) -> Self {
        Self::Io(value)
    }
}
