use {
    crate::proxy::{
        command::{
            MasterCommand,
            ShutdownToken,
        },
        pool::ProxyPool,
    },
    kanal::{
        unbounded_async,
        AsyncReceiver,
        AsyncSender,
    },
    std::future::Future,
    tokio::sync::oneshot,
};

pub struct State {
    pub pool: ProxyPool,

    trigger: oneshot::Sender<ShutdownToken>,
    tx: AsyncSender<MasterCommand>,
    rx: AsyncReceiver<MasterCommand>,
}

impl State {
    pub fn recv_command(
        &self,
    ) -> impl Future<Output = Result<MasterCommand, kanal::ReceiveError>> + '_
    {
        self.rx.recv()
    }

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

                pool: ProxyPool::default(),
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
