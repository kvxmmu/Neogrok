#[derive(Debug)]
pub enum MasterCommand {
    Disconnected { id: u16 },
    Forward { id: u16, buffer: Vec<u8> },
}
