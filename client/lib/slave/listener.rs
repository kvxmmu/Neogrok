use flume::{
    Receiver,
    Sender,
};
use tokio::{
    io::{
        AsyncReadExt,
        AsyncWriteExt,
    },
    net::TcpStream,
};

use crate::commands::{
    MasterCommand,
    SlaveCommand,
};

pub async fn run_client(
    id: u16,
    address: String,

    self_rx: Receiver<SlaveCommand>,
    master_tx: Sender<MasterCommand>,
) {
    let mut stream = match TcpStream::connect(address).await {
        Ok(s) => s,
        Err(error) => {
            master_tx
                .send_async(MasterCommand::Disconnected { id })
                .await
                .unwrap_or_default();
            tracing::error!(%error, "failed to connect");
            return;
        }
    };
    let mut buffer = vec![0_u8; 4096];
    let mut forcibly_closed = false;

    loop {
        tokio::select! {
            read = stream.read(&mut buffer) => {
                let Ok(read @ 1..) = read else { break };
                let Ok(_) = master_tx.send_async(MasterCommand::Forward {
                    id,
                    buffer: Vec::from(&buffer[..read])
                }).await else { break };
            }

            command = self_rx.recv_async() => {
                let Ok(command) = command else { break };
                match command {
                    SlaveCommand::ForceDisconnect => {
                        forcibly_closed = true;
                        break;
                    }

                    SlaveCommand::Forward { buffer } => {
                        let Ok(_) = stream.write_all(&buffer).await else { break };
                    }
                }
            }
        }
    }

    if !forcibly_closed {
        master_tx
            .send_async(MasterCommand::Disconnected { id })
            .await
            .unwrap_or_default();
    }
}
