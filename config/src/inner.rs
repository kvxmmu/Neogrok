use {
    crate::{
        error,
        permissions,
    },
    log::LevelFilter,
    serde::Deserialize,
    std::{
        fs,
        path::{
            Path,
            PathBuf,
        },
    },
    url::Url,
};

#[derive(Debug, Clone, Copy, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum LogLevelEntry {
    None,
    Info,
    Debug,
    Error,
}

#[derive(Debug, Clone, Copy, Deserialize)]
#[repr(u8)]
#[serde(rename_all = "snake_case")]
pub enum CompressionAlgorithm {
    Deflate = 0,
    ZStandard = 1,
}

#[derive(Debug, Clone, Deserialize)]
pub struct PermissionsConfig {
    pub base: permissions::PermissionEntry,
    pub magic_authorized: permissions::PermissionEntry,
}

#[derive(Debug, Clone, Deserialize)]
pub struct LoggingConfig {
    pub level: LogLevelEntry,
    pub files: Vec<PathBuf>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct DatabaseConfig {
    pub url: Url,
}

#[derive(Debug, Clone, Copy, Deserialize)]
pub struct CompressionConfig {
    pub algorithm: CompressionAlgorithm,
    pub level: u8,
    pub threshold: usize,
}

#[derive(Debug, Clone, Deserialize)]
pub struct TcpBufferCapacity {
    pub read: usize,
    pub per_client: usize,
}

#[derive(Debug, Clone, Deserialize)]
pub struct ServerConfig {
    pub listen: String,
    pub name: String,

    pub magic: Option<String>,
    pub workers: usize,

    pub buffer: TcpBufferCapacity,

    pub bind_host: String,
}

#[derive(Debug, Clone, Deserialize)]
pub struct Config {
    pub server: ServerConfig,
    pub compression: CompressionConfig,
    pub database: DatabaseConfig,
    pub logging: LoggingConfig,

    pub permissions: PermissionsConfig,
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

impl LogLevelEntry {
    pub const fn into_log(self) -> LevelFilter {
        match self {
            Self::Debug => LevelFilter::Debug,
            Self::Error => LevelFilter::Error,
            Self::Info => LevelFilter::Info,
            Self::None => LevelFilter::Off,
        }
    }
}
