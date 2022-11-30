use {
    crate::commands::{
        MasterCommand,
        SlaveCommand,
    },
    flume::{
        Receiver,
        Sender,
    },
    tokio::{
        io::{
            AsyncReadExt,
            AsyncWriteExt,
        },
        net::TcpStream,
    },
};

pub async fn run_tcp_client(
    mut stream: TcpStream,
    master: Sender<MasterCommand>,
    self_rx: Receiver<SlaveCommand>,

    id: u16,
    per_client_size: usize,
) {
    let mut buffer = vec![0; per_client_size];
    let mut forcibly_disconnected = false;

    loop {
        tokio::select! {
            read = stream.read(&mut buffer) => {
                let Ok(read @ 1..) = read else { break };
                let Ok(_) = master.send_async(
                    MasterCommand::Forward { id, buffer: Vec::from(&buffer[..read]) }
                ).await else {
                    break;
                };
            }

            item = self_rx.recv_async() => {
                let Ok(item) = item else { break };
                match item {
                    SlaveCommand::ForceDisconnect => {
                        forcibly_disconnected = true;
                        break;
                    }

                    SlaveCommand::Forward { buffer } => {
                        let Ok(_) = stream.write_all(&buffer).await else {
                            break;
                        };
                    }
                }
            }
        }
    }

    if !forcibly_disconnected {
        master
            .send_async(MasterCommand::Disconnected { id })
            .await
            .unwrap_or_default();
    }
}
