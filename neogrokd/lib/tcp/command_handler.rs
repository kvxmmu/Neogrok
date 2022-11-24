use {
    super::state::State,
    crate::proxy::command::MasterCommand,
    hisui_codec::{
        protocol::frame::ProtocolError,
        writer::HisuiWriter,
    },
    std::io,
    tokio::io::AsyncWriteExt,
};

pub(crate) async fn handle_command<Writer>(
    state: &mut State,
    writer: &mut HisuiWriter<Writer>,

    command: MasterCommand,
) -> io::Result<()>
where
    Writer: AsyncWriteExt + Unpin,
{
    match command {
        MasterCommand::Connected { tx, id } => {
            state.pool.create_client(id, tx);
        }

        MasterCommand::Disconnected { id } => {
            match state.pool.remove_client(id) {
                Ok(()) => {
                    todo!()
                }

                Err(_) => {
                    writer
                        .respond_error(ProtocolError::NoSuchClient)
                        .await?;
                }
            }
        }
    }

    Ok(())
}
