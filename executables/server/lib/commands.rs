#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ShutdownToken;

#[derive(Debug)]
pub enum MasterCommand {
    Disconnected {
        id: u16,
    },
    Connected {
        id: u16,
        tx: flume::Sender<SlaveCommand>,
    },

    Forward {
        id: u16,
        buffer: Vec<u8>,
    },

    Closed,
}

#[derive(Debug)]
pub enum SlaveCommand {
    Forward { buffer: Vec<u8> },
    ForceDisconnect,
}
