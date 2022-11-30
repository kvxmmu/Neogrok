#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CompressionStrategy {
    TryCompress { with_threshold: u16 },
    Disable,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct CompressionStatus {
    pub before: u16,
    pub after: u16,
}

impl CompressionStatus {
    pub fn ratio(self) -> f64 {
        (self.before as f64) / (self.after as f64)
    }
}
