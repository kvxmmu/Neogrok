use {
    crate::{
        commands::{
            MasterCommand,
            ShutdownToken,
        },
        proxy::client::run_tcp_client,
    },
    flume::Sender,
    idpool::prelude::FlatIdPool,
    std::{
        net::SocketAddr,
        sync::Arc,
    },
    tokio::{
        net::TcpListener,
        sync::{
            oneshot,
            Mutex,
        },
    },
};

pub async fn run_tcp_listener(
    listener: TcpListener,
    creator: SocketAddr,

    pool: Arc<Mutex<FlatIdPool<u16>>>,

    master: Sender<MasterCommand>,
    mut token: oneshot::Receiver<ShutdownToken>,

    per_client_size: usize,
) {
    let mut by_error = false;
    loop {
        tokio::select! {
            biased;

            _ = &mut token => {
                break;
            }

            result = listener.accept() => {
                let Ok((stream, address)) = result else {
                    by_error = true;
                    break;
                };

                let id = pool.lock().await.request_id();
                tracing::info!(
                    ?address,
                    ?creator,
                    ?id,
                    "client connected"
                );

                let (tx, rx) = flume::unbounded();
                let Ok(()) = master.send_async(MasterCommand::Connected { id, tx }).await else {
                    // state is dropped, so there is no sense in sending
                    // Closed to the master nor reporting in trace
                    break;
                };

                let master = Sender::clone(&master);
                let pool = Arc::clone(&pool);
                tokio::spawn(async move {
                    run_tcp_client(
                        stream,
                        master,
                        rx,
                        id,
                        per_client_size,
                    )
                    .await;

                    pool.lock()
                        .await
                        .return_id(id);
                });
            }
        }
    }

    if by_error {
        master
            .send_async(MasterCommand::Closed)
            .await
            .unwrap_or_default();
    }
}
