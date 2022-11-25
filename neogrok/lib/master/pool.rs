use {
    super::error::SendError,
    crate::slave::commands::SlaveCommand,
    kanal::AsyncSender,
    std::collections::HashMap,
};

#[derive(Debug, Default)]
pub struct ClientsPool {
    map: HashMap<u16, AsyncSender<SlaveCommand>>,
}

impl ClientsPool {
    pub fn push_client(&mut self, id: u16, tx: AsyncSender<SlaveCommand>) {
        self.map.insert(id, tx);
    }

    pub fn remove(&mut self, id: u16) {
        let _ = self.map.remove(&id);
    }

    pub async fn send_to(
        &self,
        id: u16,
        command: SlaveCommand,
    ) -> Result<(), SendError> {
        match self.map.get(&id) {
            Some(tx) => match tx.send(command).await {
                Ok(()) => Ok(()),
                Err(e) => Err(SendError::Kanal(e)),
            },

            None => Err(SendError::ClientDoesNotExists),
        }
    }
}
