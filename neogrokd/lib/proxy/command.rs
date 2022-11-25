use tokio::sync::mpsc::UnboundedSender;

#[derive(Debug)]
pub enum ProxyCommand {
    ForceDisconnect,
    Forward { buffer: Vec<u8> },
}

#[derive(Debug)]
pub enum MasterCommand {
    Connected {
        tx: UnboundedSender<ProxyCommand>,
        id: u16,
    },

    Forward {
        id: u16,
        buffer: Vec<u8>,
    },

    Disconnected {
        id: u16,
    },
}

pub struct ShutdownToken;
