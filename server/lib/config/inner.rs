use {
    super::{
        compression::CompressionCfg,
        error::ConfigLoadError,
        permissions::PermissionsCfg,
    },
    serde::Deserialize,
    std::{
        fs,
        path::Path,
    },
};

#[derive(Debug, Clone, PartialEq, Eq, Deserialize)]
pub struct TcpBufferCfg {
    pub per_client: usize,
    pub read: usize,
}

#[derive(Debug, Clone, Deserialize)]
pub struct ServerCfg {
    pub listen: String,
    pub buffer: TcpBufferCfg,

    pub magic: String,
    pub name: String,
}

#[derive(Debug, Clone, Deserialize)]
pub struct RuntimeCfg {
    pub workers: usize,
}

#[derive(Debug, Deserialize)]
pub struct Config {
    pub server: ServerCfg,
    pub runtime: RuntimeCfg,

    pub compression: CompressionCfg,
    pub permissions: PermissionsCfg,
}

impl Config {
    pub fn try_load_from(
        path: impl AsRef<Path>,
    ) -> Result<Self, ConfigLoadError> {
        let string = fs::read_to_string(path)?;
        toml::from_str(&string).map_err(ConfigLoadError::Format)
    }
}
