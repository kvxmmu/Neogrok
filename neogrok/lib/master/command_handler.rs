use {
    super::{
        commands::MasterCommand,
        pool::ClientsPool,
    },
    hisui_codec::writer::HisuiWriter,
    std::io,
    tokio::io::AsyncWriteExt,
};

pub async fn handle_command<Writer>(
    writer: &mut HisuiWriter<Writer>,
    command: MasterCommand,
    pool: &mut ClientsPool,

    compress_threshold: usize,
) -> io::Result<()>
where
    Writer: AsyncWriteExt + Unpin,
{
    match command {
        MasterCommand::Forward { id, buffer } => {
            writer
                .write_forward(id, &buffer, compress_threshold)
                .await?;
        }
        MasterCommand::Disconnected { id } => {
            writer
                .write_disconnected(id)
                .await
                .unwrap_or_default();
            pool.remove(id);
        }
    }

    Ok(())
}
