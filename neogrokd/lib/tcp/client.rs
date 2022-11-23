use {
    crate::user::User,
    config::Config,
    std::{
        io,
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

    mut reader: Reader,
    writer: Writer,
) -> io::Result<()>
where
    Reader: AsyncReadExt + Unpin,
    Writer: AsyncWriteExt,
{
    Ok(())
}
