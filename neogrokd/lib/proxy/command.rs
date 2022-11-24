use kanal::AsyncSender;

pub enum ProxyCommand {}

#[derive(Debug)]
pub enum MasterCommand {
    Connected {
        tx: AsyncSender<ProxyCommand>,
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
