use {
    super::{
        command::ProxyCommand,
        error::SendError,
    },
    idpool::flat::FlatIdPool,
    kanal::{
        unbounded_async,
        AsyncReceiver,
        AsyncSender,
    },
    std::{
        collections::HashMap,
        sync::Arc,
    },
    tokio::sync::Mutex,
};

pub struct IdResource {
    id: u16,
    pool: Arc<Mutex<FlatIdPool<u16>>>,
}

pub struct ProxyPool {
    map: HashMap<u16, AsyncSender<ProxyCommand>>,
    pool: Arc<Mutex<FlatIdPool<u16>>>,
}

impl ProxyPool {
    pub async fn create_client(
        &mut self,
    ) -> (AsyncReceiver<ProxyCommand>, IdResource) {
        let id = self.pool.lock().await.request_id();
        let resource = IdResource {
            id,
            pool: Arc::clone(&self.pool),
        };

        let (tx, rx) = unbounded_async();
        self.map.insert(id, tx);

        (rx, resource)
    }

    pub async fn send_to(
        &self,
        id: u16,
        command: ProxyCommand,
    ) -> Result<(), SendError> {
        if let Some(tx) = self.map.get(&id) {
            tx.send(command)
                .await
                .map_err(|_| SendError::Closed)
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
