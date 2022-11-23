use {
    common_codec::permissions::Rights,
    std::net::SocketAddr,
};

#[derive(Debug, Clone)]
pub struct User {
    pub address: SocketAddr,
    pub permissions: Rights,
}

impl User {
    pub const fn new(address: SocketAddr, permissions: Rights) -> Self {
        Self {
            address,
            permissions,
        }
    }
}
