use {
    crate::args::{
        Args,
        NSub,
    },
    hisui_codec::{
        common::{
            permissions::Rights,
            Protocol,
        },
        error::ReadError,
        protocol::frame::Frame,
        reader::HisuiReader,
        writer::HisuiWriter,
    },
    tokio::net::TcpStream,
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

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum InitState {
    WaitingForPing,
    WaitingForRightsUpdate,
    WaitingForServer,

    Polling,
}

pub async fn listen_server(
    mut stream: TcpStream,
    args: Args,
) -> Result<(), ReadError> {
    let (reader, writer) = stream.split();
    let mut init_state = InitState::WaitingForPing;
    let (mut reader, mut writer) =
        (HisuiReader::client(reader), HisuiWriter::new(writer));

    log::info!("Requesting server information...");
    writer.request_ping().await?;

    #[allow(irrefutable_let_patterns)]
    let NSub::Tcp { address: local_address, port: remote_port } = args.subcommand else {
        log::error!("Specified implementation is not implemented");
        return Err(ReadError::InvalidString)
    };

    loop {
        let pkt_id = reader.read_pkt_type().await?;
        let frame = reader.read_frame(pkt_id).await?;

        match init_state {
            InitState::Polling => {
                todo!()
            }

            InitState::WaitingForServer => match frame {
                Frame::StartServerResponse { port } => {
                    log::info!(
                        "Ready to receive connections on port {port}"
                    );
                    init_state = InitState::Polling;
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
                Frame::PingResponse { name } => {
                    log::info!("Connected to the {}!", name);

                    if let Some(ref magic) = args.magic {
                        log::info!("Authorizing through magic...");
                        init_state = InitState::WaitingForRightsUpdate;

                        writer.authorize_through_magic(magic).await?;
                    } else {
                        writer
                            .request_server(remote_port, Protocol::Tcp)
                            .await?;
                        init_state = InitState::WaitingForServer;
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
                    init_state = InitState::WaitingForServer;
                    writer
                        .request_server(remote_port, Protocol::Tcp)
                        .await?;
                }

                _ => unexpected_frame!(init_state, frame),
            },
        }
    }
}
