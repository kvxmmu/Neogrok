use {
    super::commands::SlaveCommand,
    crate::master::commands::MasterCommand,
    kanal::{
        AsyncReceiver,
        AsyncSender,
    },
    tokio::{
        io::{
            AsyncReadExt,
            AsyncWriteExt,
        },
        net::TcpStream,
    },
};

pub async fn listen_client(
    master: AsyncSender<MasterCommand>,

    self_rx: AsyncReceiver<SlaveCommand>,
    self_id: u16,
    address: String,
) {
    let mut stream = match TcpStream::connect(&address).await {
        Ok(s) => s,
        Err(e) => {
            log::error!("Failed to connect to the {}: {}", address, e);
            master
                .send(MasterCommand::Disconnected { id: self_id })
                .await
                .unwrap_or_default();
            return;
        }
    };

    // TODO: Add configuration for buffer length
    let mut buffer = vec![0; 4096];
    let mut gracefully = true;

    loop {
        tokio::select! {
            read = stream.read(&mut buffer) => {
                let Ok(read @ 1..) = read else { break };
                let Ok(_) = master.send(MasterCommand::Forward {
                    id: self_id,
                    buffer: Vec::from(&buffer[..read])
                }).await else {
                    break
                };
            }

            command = self_rx.recv() => {
                let Ok(command) = command else { break };

                match command {
                    SlaveCommand::Forward { buffer } => {
                        let Ok(_) = stream.write_all(&buffer).await else { break };
                    }

                    SlaveCommand::Disconnect => {
                        log::info!("ID#{self_id} is forcibly disconnected");
                        gracefully = false;
                        break;
                    }
                }
            }
        }
    }

    if gracefully {
        master
            .send(MasterCommand::Disconnected { id: self_id })
            .await
            .unwrap_or_default();
    }
}
