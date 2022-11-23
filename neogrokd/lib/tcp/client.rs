use {
    crate::user::User,
    config::Config,
    hisui_codec::{
        reader::HisuiReader,
        writer::HisuiWriter,
    },
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

    reader: Reader,
    writer: Writer,
) -> io::Result<()>
where
    Reader: AsyncReadExt + Unpin,
    Writer: AsyncWriteExt + Unpin,
{
    let (mut reader, mut writer) =
        (HisuiReader::new(reader), HisuiWriter::new(writer));

    Ok(())
}
