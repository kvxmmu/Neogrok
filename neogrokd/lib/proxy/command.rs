use kanal::AsyncSender;

pub enum ProxyCommand {}

pub enum MasterCommand {
    Connected {
        tx: AsyncSender<ProxyCommand>,
        id: u16,
    },
}

pub struct ShutdownToken;
