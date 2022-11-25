use {
    super::{
        command::ProxyCommand,
        error::{
            RemoveError,
            SendError,
        },
    },
    idpool::flat::FlatIdPool,
    std::{
        collections::HashMap,
        sync::Arc,
    },
    tokio::sync::{
        mpsc::UnboundedSender,
        Mutex,
    },
};

pub struct IdResource {
    id: u16,
    pool: Arc<Mutex<FlatIdPool<u16>>>,
}

pub struct ProxyPool {
    map: HashMap<u16, UnboundedSender<ProxyCommand>>,
    pool: Arc<Mutex<FlatIdPool<u16>>>,
}

impl ProxyPool {
    pub fn clone_id_pool(&self) -> Arc<Mutex<FlatIdPool<u16>>> {
        Arc::clone(&self.pool)
    }

    pub fn remove_client(&mut self, id: u16) -> Result<(), RemoveError> {
        if self.map.remove(&id).is_some() {
            Ok(())
        } else {
            Err(RemoveError::ClientDoesNotExists)
        }
    }

    pub fn create_client(
        &mut self,
        id: u16,
        tx: UnboundedSender<ProxyCommand>,
    ) {
        self.map.insert(id, tx);
    }

    pub fn send_to(
        &self,
        id: u16,
        command: ProxyCommand,
    ) -> Result<(), SendError> {
        if let Some(tx) = self.map.get(&id) {
            tx.send(command).map_err(|_| SendError::Closed)
        } else {
            Err(SendError::NotFound)
        }
    }
}

impl IdResource {
    pub async fn return_self(&self) {
        self.pool.lock().await.return_id(self.id())
    }

    pub const fn id(&self) -> u16 {
        self.id
    }

    pub const fn new(id: u16, pool: Arc<Mutex<FlatIdPool<u16>>>) -> Self {
        Self { id, pool }
    }
}

impl Default for ProxyPool {
    fn default() -> Self {
        Self {
            map: HashMap::new(),
            pool: Arc::new(FlatIdPool::zero_with_capacity(4).into()),
        }
    }
}
