use {
    crate::{
        tcp::client::listen_client,
        user::User,
    },
    config::Config,
    std::{
        io,
        sync::Arc,
    },
    tokio::{
        io::BufReader,
        net::TcpListener,
    },
};

pub async fn listen_to(config: Arc<Config>) -> io::Result<()> {
    let listener = TcpListener::bind(&config.server.listen).await?;
    log::info!("Listening on {}", listener.local_addr()?);

    loop {
        let (stream, address) = listener.accept().await?;
        if let Err(e) = stream.set_nodelay(true) {
            log::error!(
                "Failed to set TCP_NODELAY for {address}: {:?} (Closing \
                 connection)",
                e
            );
            continue;
        }

        let (reader, writer) = stream.into_split();
        let reader =
            BufReader::with_capacity(config.server.buffer.read, reader);
        let config = Arc::clone(&config);

        tokio::spawn(async move {
            log::info!("{address} Connected to the main server");

            let rights = config.permissions.base.into_rights();
            listen_client(config, User::new(address, rights), reader, writer)
                .await
                .unwrap_or_default();

            log::info!("{address} Disconnected from the main server");
        });
    }
}
