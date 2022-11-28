use {
    crate::proxy::{
        command::{
            MasterCommand,
            ShutdownToken,
        },
        pool::ProxyPool,
    },
    std::future::Future,
    tokio::sync::{
        mpsc::{
            unbounded_channel,
            UnboundedReceiver,
            UnboundedSender,
        },
        oneshot,
    },
};

pub struct State {
    pub pool: ProxyPool,

    trigger: Option<oneshot::Sender<ShutdownToken>>,
    tx: UnboundedSender<MasterCommand>,
    rx: UnboundedReceiver<MasterCommand>,
}

impl State {
    pub fn recv_command(
        &mut self,
    ) -> impl Future<Output = Option<MasterCommand>> + '_ {
        self.rx.recv()
    }

    pub fn new() -> (Self, oneshot::Receiver<ShutdownToken>) {
        let (tx, rx) = unbounded_channel();
        let (otx, orx) = oneshot::channel();

        (
            Self {
                trigger: Some(otx),

                pool: ProxyPool::default(),
                tx,
                rx,
            },
            orx,
        )
    }

    pub fn clone_master_tx(&self) -> UnboundedSender<MasterCommand> {
        self.tx.clone()
    }
}

impl Drop for State {
    fn drop(&mut self) {
        std::mem::take(&mut self.trigger)
            .unwrap()
            .send(ShutdownToken)
            .unwrap_or_default();
    }
}
