use std::{
    mem,
    sync::Arc,
};

use flume::{
    unbounded,
    Receiver,
    Sender,
};
use idpool::prelude::FlatIdPool;
use integral_enum::IntegralEnum;
use rustc_hash::FxHashMap;
use tokio::sync::{
    oneshot,
    Mutex,
};

use crate::{
    commands::{
        MasterCommand,
        ShutdownToken,
        SlaveCommand,
    },
    utils::cold_path,
};

#[derive(IntegralEnum)]
pub enum SendResult {
    Ok,
    NoSuchClient,
    Closed,
}

pub struct State {
    slaves: FxHashMap<u16, Sender<SlaveCommand>>,

    pub rx: Receiver<MasterCommand>,
    tx: Sender<MasterCommand>,

    token: Option<oneshot::Sender<ShutdownToken>>,
    pool: Arc<Mutex<FlatIdPool<u16>>>,
}

impl State {
    pub fn insert_slave(&mut self, id: u16, slave: Sender<SlaveCommand>) {
        self.slaves.insert(id, slave);
    }

    pub fn remove_client(&mut self, id: u16) {
        self.slaves.remove(&id).expect("Unreachable");
    }

    pub async fn send_to(
        &mut self,
        id: u16,
        command: SlaveCommand,
    ) -> SendResult {
        if let Some(tx) = self.slaves.get(&id) {
            if tx.send_async(command).await.is_err() {
                cold_path();
                SendResult::Closed
            } else {
                SendResult::Ok
            }
        } else {
            cold_path();
            SendResult::NoSuchClient
        }
    }

    pub fn clone_tx(&self) -> Sender<MasterCommand> {
        self.tx.clone()
    }

    pub fn clone_pool(&self) -> Arc<Mutex<FlatIdPool<u16>>> {
        Arc::clone(&self.pool)
    }

    pub fn new() -> (Self, oneshot::Receiver<ShutdownToken>) {
        let (tx, rx) = unbounded();
        let (stk, rtk) = oneshot::channel();

        (
            Self {
                tx,
                rx,
                token: Some(stk),
                slaves: Default::default(),
                pool: Arc::new(FlatIdPool::zero().into()),
            },
            rtk,
        )
    }
}

impl Drop for State {
    fn drop(&mut self) {
        let token = mem::take(&mut self.token).unwrap();
        token.send(ShutdownToken).unwrap_or_default();
    }
}
