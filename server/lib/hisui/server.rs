use {
    crate::{
        config::Config,
        hisui::main::listen_hisui_client,
    },
    neogrok_protocol::{
        compression::algorithms::polymorphic::{
            BufCompressor,
            BufDecompressor,
        },
        hisui::{
            reader::HisuiReader,
            writer::HisuiWriter,
        },
    },
    std::{
        io,
        sync::Arc,
    },
    tokio::{
        io::{
            AsyncRead,
            BufReader,
        },
        net::TcpListener,
    },
};

pub async fn listen_hisui(config: Arc<Config>) -> io::Result<()> {
    let listener = TcpListener::bind(&config.server.listen).await?;
    let addr = listener.local_addr()?;
    tracing::info!(%addr, "started Neogrok main server");

    loop {
        let (mut stream, addr) = listener.accept().await?;
        tracing::info!(%addr, "new user connected to the main server");

        if let Err(error) = stream.set_nodelay(true) {
            tracing::error!(%error, ?addr, "failed to set TCP nodelay, closing connection");
            continue;
        }

        let config = Arc::clone(&config);
        let buffer_read = config.server.buffer.read;

        tokio::spawn(async move {
            let (reader, writer) = stream.split();
            let (comp, decomp) = config.compression.default.to_pair();
            let (reader, writer) = create_rw_handles(
                reader,
                writer,
                comp,
                decomp,
                buffer_read,
            );

            listen_hisui_client(reader, writer, config, addr, buffer_read)
                .await;
            tracing::info!(?addr, "disconnected from the main server");
        });
    }
}

fn create_rw_handles<Reader: AsyncRead, Writer>(
    reader: Reader,
    writer: Writer,

    compressor: BufCompressor,
    decompressor: BufDecompressor,

    buffer_size: usize,
) -> (HisuiReader<BufReader<Reader>>, HisuiWriter<Writer>) {
    let reader = HisuiReader::server(
        BufReader::with_capacity(buffer_size, reader),
        decompressor,
    );
    let writer = HisuiWriter::new(writer, compressor);

    (reader, writer)
}
