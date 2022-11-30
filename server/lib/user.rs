use neogrok_protocol::protocol::types::Rights;

#[derive(Debug)]
pub struct User {
    pub rights: Rights,
}

impl User {
    pub fn new(rights: Rights) -> Self {
        Self { rights }
    }
}

impl Default for User {
    fn default() -> Self {
        User {
            rights: Rights::empty(),
        }
    }
}
