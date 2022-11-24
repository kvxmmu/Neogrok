use kanal::AsyncSender;

pub enum ProxyCommand {}

#[derive(Debug)]
pub enum MasterCommand {
    Connected {
        tx: AsyncSender<ProxyCommand>,
        id: u16,
    },

    Disconnected {
        id: u16,
    },
}

pub struct ShutdownToken;
