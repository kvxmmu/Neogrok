use neogrok_protocol::{
    compression::algorithms::polymorphic::{
        BufCompressor,
        BufDecompressor,
    },
    hisui::{
        frame::Frame,
        reader::HisuiReader,
        utils::replace_compression,
        writer::HisuiWriter,
    },
    protocol::types::{
        CompressionAlgorithm,
        Protocol,
        Rights,
    },
};
use tokio::{
    io::{
        AsyncReadExt,
        AsyncWriteExt,
        BufReader,
    },
    net::TcpStream,
};

use super::state::State;
use crate::{
    args::{
        Args,
        Command,
    },
    master::{
        command_handler::handle_command,
        frame_handler::handle_frame,
    },
};

const BUF_SIZE: usize = 4096 << 1;

async fn listen_to<Reader, Writer>(
    mut reader: HisuiReader<Reader>,
    mut writer: HisuiWriter<Writer>,
    args: Args,
) -> anyhow::Result<()>
where
    Writer: AsyncWriteExt + Unpin,
    Reader: AsyncReadExt + Unpin,
{
    let Command::Tcp { ref address, .. } = args.subcommand;
    let mut state = State::default();
    loop {
        tokio::select! {
            frame_type = reader.read_packet_type() => {
                let (pkt_type, flags) = frame_type?;
                let frame = reader.read_frame(pkt_type, flags, BUF_SIZE).await?;

                handle_frame(&mut writer, &mut state, frame, address).await?;
            }

            command = state.rx.recv_async() => {
                let Ok(command) = command else { break };
                handle_command(&mut writer, &mut state, command).await?;
            }
        }
    }

    Ok(())
}

// Initial state

pub async fn run_listener(
    mut stream: TcpStream,
    args: Args,
) -> anyhow::Result<()> {
    let (reader, writer) = stream.split();
    let (mut reader, mut writer) =
        create_rw_handles(BufReader::with_capacity(4096, reader), writer);
    tracing::info!("Requesting server information...");
    writer.request_ping().await?;

    let Frame::PingResponse {
        server_name,
        compression
    } = read_ping(&mut reader).await? else {
        unreachable!()
    };

    let (comp, decomp) = match compression.algorithm {
        CompressionAlgorithm::Deflate => (
            BufCompressor::deflate(compression.level),
            BufDecompressor::deflate(),
        ),
        CompressionAlgorithm::ZStd => (
            BufCompressor::zstd(compression.level),
            BufDecompressor::zstd(),
        ),
        CompressionAlgorithm::Reserved => unreachable!(),
    };

    replace_compression(&mut reader, &mut writer, comp, decomp);
    tracing::info!(?compression, ?server_name, "Connected");

    if let Some(ref magic) = args.password {
        tracing::info!(?magic, "authorizing using magic...");
        writer.write_auth_through_magic(magic).await?;

        let new_rights = read_auth_state(&mut reader).await?;
        tracing::info!(?new_rights, "received new rights");

        // TODO: add check
    }

    let Command::Tcp { bind_port, .. } = args.subcommand;
    tracing::info!("Requesting server start");
    writer
        .request_server(bind_port.unwrap_or(0), Protocol::Tcp)
        .await?;
    let port = read_server_response(&mut reader).await?;

    tracing::info!("Server started at port {port}");

    listen_to(reader, writer, args).await
}

async fn read_server_response<Reader>(
    reader: &mut HisuiReader<Reader>,
) -> anyhow::Result<u16>
where
    Reader: AsyncReadExt + Unpin,
{
    let frame = reader.read_frame_inconcurrent(BUF_SIZE).await?;
    match frame {
        Frame::Error(error) => {
            tracing::error!(?error, "server creation failed");
            panic!()
        }

        Frame::ServerResponse { port } => Ok(port),
        frame => unreachable!("{frame:#?}"),
    }
}

async fn read_auth_state<Reader>(
    reader: &mut HisuiReader<Reader>,
) -> anyhow::Result<Rights>
where
    Reader: AsyncReadExt + Unpin,
{
    let frame = reader.read_frame_inconcurrent(BUF_SIZE).await?;
    match frame {
        Frame::UpdateRights { new_rights } => Ok(new_rights),

        Frame::Error(error) => {
            tracing::error!(%error, "got error");
            panic!();
        }

        _ => unreachable!(),
    }
}

async fn read_ping<Reader>(
    reader: &mut HisuiReader<Reader>,
) -> anyhow::Result<Frame>
where
    Reader: AsyncReadExt + Unpin,
{
    let pkt_type = reader.read_packet_type().await?;
    let frame = reader
        .read_frame(pkt_type.0, pkt_type.1, BUF_SIZE)
        .await?;

    assert!(matches!(&frame, Frame::PingResponse { .. }));

    Ok(frame)
}

fn create_rw_handles<Reader, Writer>(
    reader: Reader,
    writer: Writer,
) -> (HisuiReader<Reader>, HisuiWriter<Writer>) {
    let reader = HisuiReader::client(reader, BufDecompressor::deflate());
    let writer = HisuiWriter::new(writer, BufCompressor::deflate(10));

    (reader, writer)
}
