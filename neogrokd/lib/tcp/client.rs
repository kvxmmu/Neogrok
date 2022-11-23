use {
    crate::user::User,
    config::Config,
    hisui_codec::{
        error::ReadError,
        protocol::frame::Frame,
        reader::HisuiReader,
        writer::HisuiWriter,
    },
    std::{
        io,
        net::SocketAddr,
        sync::Arc,
    },
    tokio::io::{
        AsyncReadExt,
        AsyncWriteExt,
    },
};

pub async fn listen_client<Reader, Writer>(
    config: Arc<Config>,
    user: User,

    reader: Reader,
    writer: Writer,
    address: SocketAddr,
) -> Result<(), ReadError>
where
    Reader: AsyncReadExt + Unpin,
    Writer: AsyncWriteExt + Unpin,
{
    let (mut reader, mut writer) =
        (HisuiReader::new(reader), HisuiWriter::new(writer));

    loop {
        let pkt_type = reader.read_pkt_type().await?;
        let frame = reader.read_frame(pkt_type).await?;

        match frame {
            Frame::Error(error) => {
                log::error!("{} Error: {:?}", address, error);
            }
            Frame::Ping => {
                writer
                    .respond_ping(config.server.name.as_bytes())
                    .await?;
            }

            _ => {
                log::error!("Not implemented");
            }
        }
    }
}
