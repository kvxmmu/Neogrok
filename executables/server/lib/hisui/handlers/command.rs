use std::net::SocketAddr;

use integral_enum::IntegralEnum;
use neogrok_protocol::{
    compression::types::CompressionStrategy,
    hisui::writer::HisuiWriter,
};
use tokio::io::AsyncWriteExt;

use crate::{
    commands::MasterCommand,
    hisui::state::State,
};

#[must_use]
#[derive(IntegralEnum)]
pub enum CommandHandleResult {
    Terminate,
    Ok,
}

pub async fn handle_command<Writer>(
    writer: &mut HisuiWriter<Writer>,
    address: &SocketAddr,
    state: &mut State,

    command: MasterCommand,
    with_threshold: u16,
) -> CommandHandleResult
where
    Writer: AsyncWriteExt + Unpin,
{
    match command {
        MasterCommand::Closed => {
            tracing::error!(
                ?address,
                "unexpected behavior: listener closed"
            );
            return CommandHandleResult::Terminate;
        }

        MasterCommand::Connected { id, tx } => {
            state.insert_slave(id, tx);
            let Ok(_) = writer.write_connect(id).await else {
                return CommandHandleResult::Terminate;
            };
        }

        MasterCommand::Disconnected { id } => {
            let Ok(_) = writer.write_disconnect(id).await else {
                return CommandHandleResult::Terminate;
            };
        }

        MasterCommand::Forward { id, buffer } => {
            let Ok(_) = writer.write_forward(
                id,
                &buffer,
                CompressionStrategy::TryCompress { with_threshold }
            ).await else {
                return CommandHandleResult::Terminate;
            };
        }
    }

    CommandHandleResult::Ok
}
