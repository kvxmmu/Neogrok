use {
    super::state::State,
    crate::{
        proxy::listener::proxy_listener,
        user::User,
    },
    common_codec::{
        permissions::Rights,
        Protocol,
    },
    config::Config,
    hisui_codec::{
        error::ReadError,
        protocol::frame::{
            Frame,
            ProtocolError,
        },
        reader::HisuiReader,
        writer::HisuiWriter,
    },
    std::{
        io,
        net::SocketAddr,
        sync::Arc,
    },
    tokio::{
        io::{
            AsyncReadExt,
            AsyncWriteExt,
        },
        net::TcpListener,
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
    let (mut reader, mut writer) =
        (HisuiReader::server(reader), HisuiWriter::new(writer));

    let mut state: Option<State> = None;

    loop {
        let Ok(pkt_type) = reader.read_pkt_type().await else { break };

        // TODO: Properly handle read_frame errors
        let Ok(frame) = reader.read_frame(pkt_type).await else { break };

        // TODO: handle master commands
        handle_frame(
            &mut state,
            &address,
            &config,
            &mut writer,
            &mut user,
            frame,
        )
        .await?;
    }

    log::info!("{} Disconnected from the main server", address);
    if let Some(state) = state {
        state.trigger_shutdown();
        log::debug!("{} Removed initiated proxy server", address);
    }

    Ok(())
}

async fn handle_frame<Writer>(
    state: &mut Option<State>,

    address: &SocketAddr,
    config: &Arc<Config>,

    writer: &mut HisuiWriter<Writer>,
    user: &mut User,

    frame: Frame,
) -> io::Result<()>
where
    Writer: AsyncWriteExt + Unpin,
{
    match frame {
        Frame::StartServer { port, protocol } => {
            if state.is_some() {
                return writer
                    .respond_error(ProtocolError::ServerIsAlreadyCreated)
                    .await;
            }
            match protocol {
                Protocol::Tcp
                    if user
                        .permissions
                        .intersects(Rights::CAN_CREATE_TCP) =>
                {
                    if port != 0
                        && !user
                            .permissions
                            .intersects(Rights::CAN_SELECT_TCP)
                    {
                        return writer
                            .respond_error(ProtocolError::AccessDenied)
                            .await;
                    }

                    let Ok(listener) = TcpListener::bind(format!(
                        "{}:{}",
                        config.server.bind_host, port
                    ))
                    .await else {
                        return writer.respond_error(ProtocolError::FailedToBindPort).await;
                    };
                    let port = listener.local_addr()?.port();

                    let (new_state, shutdown_rx) = State::new();
                    tokio::spawn(proxy_listener(
                        listener,
                        new_state.clone_master_tx(),
                        new_state.pool.clone(),
                        shutdown_rx,
                    ));

                    *state = Some(new_state);
                    writer.respond_server(port).await?;

                    log::info!(
                        "{} Created server at {}:{}",
                        address,
                        config.server.bind_host,
                        port
                    );
                }

                Protocol::Udp
                    if user
                        .permissions
                        .intersects(Rights::CAN_CREATE_UDP) =>
                {
                    log::error!(
                        "{} NOT IMPLEMENTED (Protocol::Udp)",
                        address
                    );
                    return writer
                        .respond_error(ProtocolError::NotImplemented)
                        .await;
                }

                _ => {
                    return writer
                        .respond_error(ProtocolError::AccessDenied)
                        .await;
                }
            }
        }

        Frame::AuthThroughMagic { magic } => {
            if let Some(ref actual_magic) = config.server.magic {
                if actual_magic.as_bytes() == magic {
                    let rights =
                        config.permissions.magic_authorized.into_rights();
                    log::info!(
                        "{} Authorized through magic, received rights: \
                         {:?}",
                        address,
                        rights
                    );

                    user.promote_to(rights);
                    writer.respond_update_rights(rights).await?;
                } else {
                    log::error!(
                        "{} Tried to authorize using magic: magic did \
                         not matched",
                        address
                    );
                    writer
                        .respond_error(ProtocolError::AccessDenied)
                        .await?;
                }
            } else {
                log::error!(
                    "{} Tried to authorize using magic: magic \
                     authorization turned off",
                    address
                );
                writer
                    .respond_error(ProtocolError::MagicAuthIsTurnedOff)
                    .await?;
            }
        }

        Frame::Error(error) => {
            log::error!("{} Error: {:?}", address, error);
        }

        Frame::Ping => {
            log::info!("{} Ping request", address);
            writer
                .respond_ping(config.server.name.as_bytes())
                .await?;
        }

        _ => {
            log::error!("{} Error: Not implemented", address);
            writer
                .respond_error(ProtocolError::NotImplemented)
                .await?;
        }
    }

    Ok(())
}
