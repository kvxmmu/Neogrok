use {
    super::{
        command::{
            MasterCommand,
            ShutdownToken,
        },
        error::ListenerError,
    },
    crate::proxy::{
        client::listen_proxy_client,
        pool::IdResource,
    },
    idpool::prelude::FlatIdPool,
    kanal::{
        unbounded_async,
        AsyncSender,
    },
    std::sync::Arc,
    tokio::{
        net::TcpListener,
        select,
        sync::{
            oneshot::Receiver,
            Mutex,
        },
    },
};

pub async fn proxy_listener(
    listener: TcpListener,
    master: AsyncSender<MasterCommand>,

    pool: Arc<Mutex<FlatIdPool<u16>>>,
    mut shutdown: Receiver<ShutdownToken>,

    buffer_per_client: usize,
) -> Result<(), ListenerError> {
    loop {
        select! {
            biased;

            _ = &mut shutdown => {
                break Ok(())
            }

            pair = listener.accept() => {
                let (stream, address) = pair?;
                let id = pool.lock().await.request_id();
                let (tx, rx) = unbounded_async();

                master.send(MasterCommand::Connected { tx, id }).await?;
                tokio::spawn(
                    listen_proxy_client(
                        IdResource::new(id, Arc::clone(&pool)),
                        stream,
                        address,
                        master.clone(),
                        rx,
                        buffer_per_client
                    )
                );
            }
        }
    }
}
