#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Protocol {
    Tcp = 0,
    Udp = 1,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CodecSide {
    Client,
    Server,
}

pub mod permissions;
