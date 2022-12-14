use std::{
    net::SocketAddr,
    num::NonZeroU16,
    sync::Arc,
};

use neogrok_protocol::hisui::{
    error::ReadError,
    reader::HisuiReader,
    writer::HisuiWriter,
};
use tokio::io::{
    AsyncReadExt,
    AsyncWriteExt,
};

use crate::{
    commands::MasterCommand,
    config::Config,
    hisui::{
        handlers::{
            command::*,
            error::*,
            frame::*,
        },
        state::State,
    },
    infinite_future::infinite_future,
    user::User,
};

pub async fn listen_hisui_client<Reader, Writer>(
    mut reader: HisuiReader<Reader>,
    mut writer: HisuiWriter<Writer>,

    config: Arc<Config>,
    address: SocketAddr,

    buffer_read: u16,
) where
    Reader: AsyncReadExt + Unpin,
    Writer: AsyncWriteExt + Unpin,
{
    let compression_data = &config.compression.default;
    let mut user = User::new(config.permissions.base.to_protocol_rights());
    let mut state: Option<State> = None;
    let buffer_read = NonZeroU16::new(buffer_read);

    async fn wait_command(
        state: &mut Option<State>,
    ) -> Option<MasterCommand> {
        match state {
            Some(state) => state.rx.recv_async().await.ok(),
            None => {
                infinite_future().await;
                unreachable!()
            }
        }
    }

    loop {
        tokio::select! {
            command = wait_command(&mut state) => {
                let Some(command) = command else {
                    tracing::error!("master receiver is dropped (report this on project page)");
                    break;
                };

                if handle_command(
                    &mut writer,
                    &address,
                    state.as_mut().unwrap(),
                    command,
                    compression_data.threshold,
                ).await == CommandHandleResult::Terminate {
                    break;
                }
            }

            frame_type = reader.read_packet_type() => {
                let (pkt_type, flags) = match frame_type {
                    Ok(d) => d,
                    Err(error) => {
                        let Ok(error_type) = handle_error(
                            &mut writer,
                            &error,
                            &address
                        ).await else { break };
                        match error_type {
                            ErrorType::NonFatalButDisconnect => break,
                            ErrorType::Fatal => {
                                tracing::error!(?address, ?error, "fatal error");
                                break;
                            }

                            _ => { continue; }
                        }
                    }
                };

                let frame = match reader.read_frame(
                    pkt_type,
                    flags,
                    buffer_read,
                ).await {
                    Ok(f) => f,
                    Err(e) => {
                        tracing::error!(%e, "failed to read frame");
                        break;
                    }
                };

                match handle_frame(
                    &mut writer,
                    frame,
                    &config,
                    &address,
                    compression_data,
                    buffer_read.map(|i| i.get()).unwrap_or(u16::MAX),
                    &mut user,
                    &mut state,
                ).await {
                    Ok(()) => {},
                    Err(e) => {
                        tracing::error!(%e, "failed to handle frame");
                        let Ok(_) = handle_error(
                            &mut writer,
                            &ReadError::Io(e),
                            &address
                        ).await else { break };
                    }
                }
            }
        }
    }
}
