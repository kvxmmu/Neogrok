use {
    super::{
        commands::MasterCommand,
        pool::ClientsPool,
        state::InitState,
    },
    crate::slave::{
        commands::SlaveCommand,
        listener::listen_client,
    },
    hisui_codec::{
        common::{
            compression::{
                PayloadCompressor,
                PayloadDecompressor,
            },
            permissions::Rights,
            Compression,
            Protocol,
        },
        error::ReadError,
        protocol::frame::Frame,
        reader::HisuiReader,
        writer::HisuiWriter,
    },
    tokio::{
        io::{
            AsyncReadExt,
            AsyncWriteExt,
        },
        sync::mpsc::{
            unbounded_channel,
            UnboundedSender,
        },
    },
};

macro_rules! unexpected_frame {
    ($for_state:expr, $actual:expr) => {{
        log::error!(
            "Unexpected frame for state {:?}: {:#?}",
            $for_state,
            $actual
        );
        unreachable!()
    }};
}

#[allow(clippy::too_many_arguments)]
pub async fn handle_frame<Reader, Writer>(
    magic: &Option<String>,
    remote_port: u16,
    local_address: &str,
    master_tx: &UnboundedSender<MasterCommand>,

    init_state: &mut InitState,
    pool: &mut ClientsPool,

    reader: &mut HisuiReader<Reader>,
    writer: &mut HisuiWriter<Writer>,

    frame: Frame,
) -> Result<(), ReadError>
where
    Writer: AsyncWriteExt + Unpin,
    Reader: AsyncReadExt + Unpin,
{
    match *init_state {
        InitState::Polling => match frame {
            Frame::Error(e) => {
                log::error!("Error: {e}");
            }

            Frame::Forward { id, buffer } => {
                pool.send_to(id, SlaveCommand::Forward { buffer })
                    .unwrap_or_default();
            }
            Frame::Connected { id } => {
                log::info!("ID#{id} is connected to the server");

                let (cl_tx, cl_rx) = unbounded_channel();

                pool.push_client(id, cl_tx);
                tokio::spawn(listen_client(
                    master_tx.clone(),
                    cl_rx,
                    id,
                    local_address.to_owned(),
                ));
            }
            Frame::Disconnected { id } => {
                match pool.send_to(id, SlaveCommand::Disconnect) {
                    Ok(()) => {
                        log::info!("ID#{id} is disconnected");
                    }
                    Err(error) => {
                        log::error!(
                            "Failed to disconnect ID#{id} ({error}), \
                             please report to github"
                        );
                    }
                }

                pool.remove(id);
            }
            _ => unexpected_frame!(init_state, frame),
        },

        InitState::WaitingForServer => match frame {
            Frame::StartServerResponse { port } => {
                log::info!("Ready to receive connections on port {port}");
                *init_state = InitState::Polling;
            }

            Frame::Error(e) => {
                log::error!(
                    "Received error while trying to create remote \
                     server: {}, terminating...",
                    e
                );
                return Err(ReadError::InvalidString);
            }

            _ => unexpected_frame!(init_state, frame),
        },

        InitState::WaitingForPing => match frame {
            Frame::PingResponse {
                name,
                compression_algorithm,
                level,
            } => {
                log::info!(
                    "Connected to the {}! (Compression algorithm {:?} \
                     with level {})",
                    name,
                    compression_algorithm,
                    level
                );
                writer.replace_compressor(match compression_algorithm {
                    Compression::Deflate => {
                        PayloadCompressor::deflate(level as _)
                    }
                    Compression::Zstd => {
                        PayloadCompressor::zstd(level as _)
                    }
                    Compression::Last => unreachable!(),
                });
                reader.replace_decompressor(match compression_algorithm {
                    Compression::Deflate => PayloadDecompressor::deflate(),
                    Compression::Zstd => PayloadDecompressor::zstd(),
                    Compression::Last => unreachable!(),
                });

                if let Some(magic) = magic {
                    log::info!("Authorizing through magic...");
                    *init_state = InitState::WaitingForRightsUpdate;

                    writer.authorize_through_magic(magic).await?;
                } else {
                    writer
                        .request_server(remote_port, Protocol::Tcp)
                        .await?;
                    *init_state = InitState::WaitingForServer;
                }
            }

            _ => unexpected_frame!(init_state, frame),
        },

        InitState::WaitingForRightsUpdate => match frame {
            Frame::Error(e) => {
                log::error!("Auth error: {:?}", e);
                return Err(ReadError::InvalidString);
            }

            Frame::UpdateRights { rights } => {
                if !rights.contains(
                    Rights::CAN_CREATE_TCP
                        | if remote_port != 0 {
                            Rights::CAN_SELECT_TCP
                        } else {
                            Rights::empty()
                        },
                ) {
                    log::error!(
                        "Got insufficient rights ({rights:?}), \
                         terminating..."
                    );
                    return Err(ReadError::InvalidString);
                }

                log::info!("Received rights {:?}", rights);
                *init_state = InitState::WaitingForServer;
                writer
                    .request_server(remote_port, Protocol::Tcp)
                    .await?;
            }

            _ => unexpected_frame!(init_state, frame),
        },
    }

    Ok(())
}
