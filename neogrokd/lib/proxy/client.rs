use {
    super::{
        command::{
            MasterCommand,
            ProxyCommand,
        },
        pool::IdResource,
    },
    kanal::{
        AsyncReceiver,
        AsyncSender,
    },
    std::{
        io,
        net::SocketAddr,
    },
    tokio::{
        io::{
            AsyncReadExt,
            AsyncWriteExt,
        },
        net::TcpStream,
    },
};

pub async fn listen_proxy_client(
    res: IdResource,

    mut stream: TcpStream,
    address: SocketAddr,

    master_tx: AsyncSender<MasterCommand>,
    self_rx: AsyncReceiver<ProxyCommand>,

    buffer_alloc_size: usize,
) -> io::Result<()> {
    let mut buffer = vec![0; buffer_alloc_size];

    loop {
        tokio::select! {
            command = self_rx.recv() => {
                let Ok(command) = command else { break };

                match command {
                    ProxyCommand::ForceDisconnect => break,
                    ProxyCommand::Forward { buffer } => {
                        let Ok(_) = stream.write_all(&buffer).await else { break };
                    }
                }
            }

            read = stream.read(&mut buffer) => {
                let Ok(read) = read else { break };
                let Ok(_) = master_tx.send(MasterCommand::Forward {
                    id: res.id(), buffer: Vec::from(&buffer[..read]) }).await else { break };
            }
        }
    }

    res.return_self().await;
    master_tx
        .send(MasterCommand::Disconnected { id: res.id() })
        .await
        .unwrap_or_default();
    Ok(())
}
