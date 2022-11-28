use {
    super::error::SendError,
    crate::slave::commands::SlaveCommand,
    rustc_hash::FxHashMap,
    tokio::sync::mpsc::UnboundedSender,
};

#[derive(Debug, Default)]
pub struct ClientsPool {
    map: FxHashMap<u16, UnboundedSender<SlaveCommand>>,
}

impl ClientsPool {
    pub fn push_client(
        &mut self,
        id: u16,
        tx: UnboundedSender<SlaveCommand>,
    ) {
        self.map.insert(id, tx);
    }

    pub fn remove(&mut self, id: u16) {
        let _ = self.map.remove(&id);
    }

    pub fn send_to(
        &self,
        id: u16,
        command: SlaveCommand,
    ) -> Result<(), SendError> {
        match self.map.get(&id) {
            Some(tx) => match tx.send(command) {
                Ok(()) => Ok(()),
                Err(_) => Err(SendError::SendError),
            },

            None => Err(SendError::ClientDoesNotExists),
        }
    }
}
