#[derive(Debug)]
pub enum MasterCommand {
    Disconnected { id: u16 },
    Forward { id: u16, buffer: Vec<u8> },
}

#[derive(Debug)]
pub enum SlaveCommand {
    Forward { buffer: Vec<u8> },
    ForceDisconnect,
}
