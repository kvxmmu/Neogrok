use common_codec::Protocol;

#[derive(Debug, Clone)]
pub enum Frame {
    Ping,

    StartServer { port: u16, protocol: Protocol },
    StartHttpServer,
}

#[rustfmt::skip]
impl Frame {
    pub const PING: u8              = 0;
    pub const START_SERVER: u8      = 1;
    pub const START_HTTP_SERVER: u8 = 2;
}
