use {
    super::state::State,
    crate::{
        proxy::{
            command::ProxyCommand,
            listener::proxy_listener,
        },
        user::User,
    },
    config::Config,
    hisui_codec::{
        common::{
            permissions::Rights,
            Compression,
            Protocol,
        },
        protocol::frame::{
            Frame,
            ProtocolError,
        },
        writer::HisuiWriter,
    },
    std::{
        io,
        net::SocketAddr,
        sync::Arc,
    },
    tokio::{
        io::AsyncWriteExt,
        net::TcpListener,
    },
};

macro_rules! send_to_sessioned {
    ($state:ident: try($id:expr, $message:expr) or write_to $writer:expr) => {
        match $state {
            Some(state) => match state.pool.send_to($id, $message) {
                Ok(()) => {}
                Err(_) => {
                    $writer
                        .respond_error(ProtocolError::NoSuchClient)
                        .await?;
                }
            },

            _ => {
                $writer
                    .respond_error(ProtocolError::ServerIsNotCreated)
                    .await?;
            }
        }
    };
}

pub(crate) async fn handle_frame<Writer>(
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
        Frame::Forward { id, buffer } => send_to_sessioned!(
            state: try(id, ProxyCommand::Forward { buffer }) or write_to writer
        ),
        Frame::Disconnected { id } => send_to_sessioned!(
            state: try(id, ProxyCommand::ForceDisconnect) or write_to writer
        ),

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
                        new_state.pool.clone_id_pool(),
                        shutdown_rx,
                        config.server.buffer.per_client,
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
                .respond_ping(
                    config.server.name.as_bytes(),
                    Compression::try_from(
                        config.compression.algorithm as u8,
                    )
                    .unwrap(),
                    config.compression.level,
                )
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
