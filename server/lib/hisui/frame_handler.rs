use {
    super::state::{
        SendResult,
        State,
    },
    crate::{
        commands::SlaveCommand,
        config::{
            compression::CompressionData,
            Config,
        },
        proxy::listener::run_tcp_listener,
        user::User,
    },
    neogrok_protocol::{
        hisui::{
            frame::Frame,
            writer::HisuiWriter,
        },
        protocol::{
            error::ProtocolError,
            types::Rights,
        },
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

macro_rules! with_server {
    ($writer:expr, $state:ident($id:expr, $frame:expr) as $state_pat:ident => $ok:expr) => {
        match $state {
            Some($state_pat) => {
                match $state_pat.send_to($id, $frame).await {
                    SendResult::Ok => $ok,
                    SendResult::NoSuchClient | SendResult::Closed => {
                        $writer
                            .respond_error(ProtocolError::NoSuchClient)
                            .await?;
                    }
                }
            }

            None => {
                $writer
                    .respond_error(ProtocolError::NoSuchClient)
                    .await?;
            }
        }
    };
}

pub async fn handle_frame<Writer>(
    writer: &mut HisuiWriter<Writer>,
    frame: Frame,
    config: &Arc<Config>,
    address: &SocketAddr,

    compression_data: &CompressionData,
    user: &mut User,
    state: &mut Option<State>,
) -> io::Result<()>
where
    Writer: AsyncWriteExt + Unpin,
{
    match frame {
        Frame::Forward { id, buffer } => {
            with_server!(writer, state(id, SlaveCommand::Forward { buffer }) as state => ())
        }

        Frame::Disconnect { id } => {
            with_server!(writer, state(id, SlaveCommand::ForceDisconnect) as state => {
                let from = address;
                tracing::info!(?id, ?from, "disconnected from the proxy");
                state.remove_client(id);
            })
        }

        Frame::ServerRequest { port, .. } => {
            // TODO: add protocol selection
            if !user.rights.allowed_to(Rights::CAN_CREATE_TCP) {
                tracing::error!(
                    ?address,
                    "access denied to create tcp server"
                );
                writer
                    .respond_error(ProtocolError::AccessDenied)
                    .await?;
                return Ok(());
            }

            // TODO: add protocol selection
            if port != 0 && !user.rights.allowed_to(Rights::CAN_SELECT_TCP)
            {
                tracing::error!(
                    ?address,
                    "access denied to select tcp port"
                );
                writer
                    .respond_error(ProtocolError::AccessDenied)
                    .await?;
                return Ok(());
            }

            let listener = match TcpListener::bind(&format!(
                "0.0.0.0:{port}"
            ))
            .await
            {
                Ok(l) => l,
                Err(error) => {
                    tracing::error!(
                        %error,
                        "failed to create tcp listener"
                    );
                    return writer
                        .respond_error(ProtocolError::FailedToCreateServer)
                        .await;
                }
            };
            let newly_created_address = match listener.local_addr() {
                Ok(a) => a,
                Err(error) => {
                    tracing::error!(
                        ?error,
                        "failed to get listener address"
                    );
                    writer
                        .respond_error(ProtocolError::FailedToCreateServer)
                        .await?;
                    return Ok(());
                }
            };

            let (new_state, token) = State::new();
            tokio::spawn(run_tcp_listener(
                listener,
                *address,
                new_state.clone_pool(),
                new_state.clone_tx(),
                token,
                config.server.buffer.per_client,
            ));

            tracing::info!(?newly_created_address, "Created server");

            *state = Some(new_state);
            writer
                .respond_server(newly_created_address.port())
                .await?;
        }

        Frame::AuthThroughMagic { magic } => {
            if magic == config.server.magic {
                let new_rights =
                    config.permissions.magic.to_protocol_rights();
                user.rights = new_rights;
                tracing::info!(
                    ?address,
                    ?new_rights,
                    "authorized through magic"
                );

                writer.respond_update_rights(new_rights).await?;
            } else {
                tracing::error!(?address, %magic, "failed to authorize using magic");
                writer
                    .respond_error(ProtocolError::InvalidCredentials)
                    .await?;
            }
        }

        Frame::PingRequest => {
            tracing::info!(?address, "ping request");

            writer
                .respond_ping(
                    &config.server.name,
                    compression_data.algorithm.to_protocol(),
                    compression_data.level,
                )
                .await?;
        }

        frame => {
            tracing::error!(?frame, "unexpected frame sent");
            writer
                .respond_error(ProtocolError::UnexpectedFrame)
                .await?;
        }
    }

    Ok(())
}
