#[derive(Debug, Clone)]
pub enum SlaveCommand {
    Disconnect,
    Forward { buffer: Vec<u8> },
}
