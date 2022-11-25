use {
    super::{
        command::{
            MasterCommand,
            ProxyCommand,
        },
        pool::IdResource,
    },
    std::net::SocketAddr,
    tokio::{
        io::{
            AsyncReadExt,
            AsyncWriteExt,
        },
        net::TcpStream,
        sync::mpsc::{
            UnboundedReceiver,
            UnboundedSender,
        },
    },
};

pub async fn listen_proxy_client(
    res: IdResource,

    mut stream: TcpStream,
    address: SocketAddr,

    master_tx: UnboundedSender<MasterCommand>,
    mut self_rx: UnboundedReceiver<ProxyCommand>,

    buffer_alloc_size: usize,
) {
    let mut buffer = vec![0; buffer_alloc_size];
    let mut gracefully = true;

    loop {
        tokio::select! {
            command = self_rx.recv() => {
                let Some(command) = command else { break };

                match command {
                    ProxyCommand::ForceDisconnect => {
                        log::info!("{address} Is forcibly disconnected");
                        gracefully = false;
                        break;
                    }

                    ProxyCommand::Forward { buffer } => {
                        let Ok(_) = stream.write_all(&buffer).await else { break };
                    }
                }
            }

            read = stream.read(&mut buffer) => {
                let Ok(read @ 1..) = read else { break };
                let Ok(_) = master_tx.send(MasterCommand::Forward {
                    id: res.id(), buffer: Vec::from(&buffer[..read]) }) else { break };
            }
        }
    }

    res.return_self().await;
    if gracefully {
        master_tx
            .send(MasterCommand::Disconnected { id: res.id() })
            .unwrap_or_default();
    }
}
