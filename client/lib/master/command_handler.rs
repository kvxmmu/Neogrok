use {
    super::state::State,
    crate::commands::MasterCommand,
    neogrok_protocol::{
        compression::types::CompressionStrategy,
        hisui::writer::HisuiWriter,
    },
    tokio::io::AsyncWriteExt,
};

pub async fn handle_command<Writer>(
    writer: &mut HisuiWriter<Writer>,
    state: &mut State,
    command: MasterCommand,
) -> anyhow::Result<()>
where
    Writer: AsyncWriteExt + Unpin,
{
    match command {
        MasterCommand::Forward { id, buffer } => {
            writer
                .write_forward(
                    id,
                    &buffer,
                    CompressionStrategy::TryCompress {
                        with_threshold: 64,
                    },
                )
                .await?;
        }

        MasterCommand::Disconnected { id } => {
            tracing::info!(?id, "disconnected");

            state.remove_slave(id);
            writer.write_disconnect(id).await?;
        }
    }
    Ok(())
}
