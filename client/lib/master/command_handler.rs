use neogrok_protocol::{
    compression::types::CompressionStrategy,
    hisui::writer::HisuiWriter,
};
use tokio::io::AsyncWriteExt;

use super::state::State;
use crate::commands::MasterCommand;

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
