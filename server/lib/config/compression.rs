use integral_enum::IntegralEnum;
use neogrok_protocol::{
    compression::algorithms::polymorphic::{
        BufCompressor,
        BufDecompressor,
    },
    protocol::types::CompressionAlgorithm,
};
use serde::Deserialize;

#[derive(Deserialize, IntegralEnum)]
#[serde(rename_all = "snake_case")]
pub enum CfgCompressionAlgorithm {
    Deflate = 0,
    ZStd = 1,
}

#[derive(Debug, Deserialize)]
pub struct CompressionData {
    pub level: u8,
    pub algorithm: CfgCompressionAlgorithm,
    pub threshold: u16,
}

#[derive(Debug, Deserialize)]
pub struct CompressionCfg {
    pub default: CompressionData,
}

impl CfgCompressionAlgorithm {
    pub fn to_protocol(self) -> CompressionAlgorithm {
        match self {
            Self::Deflate => CompressionAlgorithm::Deflate,
            Self::ZStd => CompressionAlgorithm::ZStd,
        }
    }
}

impl CompressionData {
    pub fn to_pair(&self) -> (BufCompressor, BufDecompressor) {
        match self.algorithm {
            CfgCompressionAlgorithm::Deflate => (
                BufCompressor::deflate(self.level),
                BufDecompressor::deflate(),
            ),
            CfgCompressionAlgorithm::ZStd => {
                (BufCompressor::zstd(self.level), BufDecompressor::zstd())
            }
        }
    }
}
