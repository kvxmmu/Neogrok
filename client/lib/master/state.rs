use {
    crate::commands::{
        MasterCommand,
        SlaveCommand,
    },
    flume::{
        Receiver,
        Sender,
    },
    rustc_hash::FxHashMap,
};

#[must_use]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SendResult {
    Ok,
    NotFound,
    Closed,
}

pub struct State {
    pub tx: Sender<MasterCommand>,
    pub rx: Receiver<MasterCommand>,

    slaves: FxHashMap<u16, Sender<SlaveCommand>>,
}

impl State {
    pub async fn send_to(
        &mut self,
        id: u16,
        command: SlaveCommand,
    ) -> SendResult {
        match self.slaves.get(&id) {
            Some(slave) => {
                if slave.send_async(command).await.is_err() {
                    SendResult::Closed
                } else {
                    SendResult::Ok
                }
            }

            _ => SendResult::NotFound,
        }
    }

    pub fn remove_slave(&mut self, id: u16) {
        self.slaves.remove(&id);
    }

    pub fn insert_slave(&mut self, id: u16, tx: Sender<SlaveCommand>) {
        self.slaves.insert(id, tx);
    }
}

impl SendResult {
    pub const fn ignore(self) {}
}

impl Default for State {
    fn default() -> Self {
        let (tx, rx) = flume::unbounded();

        Self {
            tx,
            rx,
            slaves: Default::default(),
        }
    }
}
