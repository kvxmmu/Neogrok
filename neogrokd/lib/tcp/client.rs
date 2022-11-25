use {
    super::state::State,
    crate::{
        infinite_future::wait_forever,
        proxy::command::MasterCommand,
        tcp::{
            command_handler::handle_command,
            frame_handler::handle_frame,
        },
        user::User,
    },
    config::Config,
    hisui_codec::{
        self,
        error::ReadError,
        reader::HisuiReader,
        writer::HisuiWriter,
    },
    std::{
        net::SocketAddr,
        sync::Arc,
    },
    tokio::{
        io::{
            AsyncReadExt,
            AsyncWriteExt,
        },
        select,
    },
};

pub async fn listen_client<Reader, Writer>(
    config: Arc<Config>,
    mut user: User,

    reader: Reader,
    writer: Writer,
    address: SocketAddr,
) -> Result<(), ReadError>
where
    Reader: AsyncReadExt + Unpin,
    Writer: AsyncWriteExt + Unpin,
{
    async fn recv_command_or_wait(
        state: &mut Option<State>,
    ) -> Option<MasterCommand> {
        if let Some(state) = state {
            state.recv_command().await
        } else {
            wait_forever().await;
            unreachable!()
        }
    }

    let (mut reader, mut writer) =
        (HisuiReader::server(reader), HisuiWriter::new(writer));

    let mut state: Option<State> = None;

    loop {
        select! {
            command = recv_command_or_wait(&mut state) => {
                let Some(command) = command else { break };
                let Ok(_) = handle_command(
                    &address,
                    state.as_mut().unwrap(),
                    &mut writer,
                    command
                ).await else { break };
            },

            pkt_type = reader.read_pkt_type() => {
                let Ok(pkt_type) = pkt_type else { break };

                match reader.read_frame(pkt_type).await {
                    Ok(frame) => {
                        let Ok(_) = handle_frame(
                            &mut state,
                            &address,
                            &config,
                            &mut writer,
                            &mut user,
                            frame,
                        )
                        .await else { break };
                    }

                    Err(e) => {
                        log::error!("{address} framing-level error: {:?}", e);
                    }
                }
            }
        }
    }

    log::info!("{} Disconnected from the main server", address);

    Ok(())
}
