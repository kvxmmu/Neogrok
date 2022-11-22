use {
    crate::error,
    serde::Deserialize,
    std::{
        fs,
        path::Path,
    },
    url::Url,
};

#[derive(Debug, Clone, Copy, Deserialize)]
#[repr(u8)]
pub enum CompressionAlgorithm {
    Deflate = 0,
    ZStandard = 1,
}

#[derive(Debug, Clone, Deserialize)]
pub struct DatabaseConfig {
    pub url: Url,
}

#[derive(Debug, Clone, Copy, Deserialize)]
pub struct CompressionConfig {
    pub algorithm: CompressionAlgorithm,
    pub level: u8,
}

#[derive(Debug, Clone, Deserialize)]
pub struct ServerConfig {
    pub listen: String,
    pub name: String,

    pub magic: Option<String>,
    pub workers: usize,
}

#[derive(Debug, Clone, Deserialize)]
pub struct Config {
    pub server: ServerConfig,
    pub compression: CompressionConfig,
    pub database: DatabaseConfig,
}

impl Config {
    pub fn load(path: impl AsRef<Path>) -> Self {
        Self::try_load(path).expect("Failed to read toml config")
    }

    pub fn try_load(path: impl AsRef<Path>) -> Result<Self, error::Error> {
        let text = fs::read_to_string(path)?;
        toml::from_str(&text).map_err(|e| e.into())
    }
}
