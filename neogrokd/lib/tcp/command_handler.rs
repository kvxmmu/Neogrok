use {
    super::state::State,
    crate::proxy::command::MasterCommand,
    hisui_codec::{
        protocol::frame::ProtocolError,
        writer::HisuiWriter,
    },
    std::{
        io,
        net::SocketAddr,
    },
    tokio::io::AsyncWriteExt,
};

pub(crate) async fn handle_command<Writer>(
    address: &SocketAddr,
    state: &mut State,
    writer: &mut HisuiWriter<Writer>,

    command: MasterCommand,
) -> io::Result<()>
where
    Writer: AsyncWriteExt + Unpin,
{
    match command {
        MasterCommand::Forward { id, buffer } => {
            writer.write_forward(id, &buffer).await?;
        }

        MasterCommand::Connected { tx, id } => {
            log::info!("ID#{id} is connected to the {address}'s server");
            state.pool.create_client(id, tx);
            writer.write_connected(id).await?;
        }

        MasterCommand::Disconnected { id } => {
            log::info!(
                "ID#{id} is disconnected from the {address}'s server"
            );
            match state.pool.remove_client(id) {
                Ok(()) => {
                    writer.write_disconnected(id).await?;
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
