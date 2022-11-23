use {
    crate::proxy::command::{
        MasterCommand,
        ShutdownToken,
    },
    idpool::prelude::FlatIdPool,
    kanal::{
        unbounded_async,
        AsyncReceiver,
        AsyncSender,
    },
    std::sync::Arc,
    tokio::sync::{
        oneshot,
        Mutex,
    },
};

pub struct State {
    pub pool: Arc<Mutex<FlatIdPool<u16>>>,

    trigger: oneshot::Sender<ShutdownToken>,
    tx: AsyncSender<MasterCommand>,
    rx: AsyncReceiver<MasterCommand>,
}

impl State {
    pub fn trigger_shutdown(self) {
        self.trigger
            .send(ShutdownToken)
            .unwrap_or_default()
    }

    pub fn new() -> (Self, oneshot::Receiver<ShutdownToken>) {
        let (tx, rx) = unbounded_async();
        let (otx, orx) = oneshot::channel();

        (
            Self {
                trigger: otx,

                pool: Arc::new(FlatIdPool::zero().into()),
                tx,
                rx,
            },
            orx,
        )
    }

    pub fn clone_master_tx(&self) -> AsyncSender<MasterCommand> {
        self.tx.clone()
    }
}
