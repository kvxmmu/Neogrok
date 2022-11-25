use {
    crate::{
        args::{
            Args,
            NSub,
        },
        master::{
            command_handler::handle_command,
            commands::MasterCommand,
            frame_handler::handle_frame,
            pool::ClientsPool,
            state::InitState,
        },
    },
    hisui_codec::{
        self,
        error::ReadError,
        reader::HisuiReader,
        writer::HisuiWriter,
    },
    tokio::{
        net::TcpStream,
        sync::mpsc::unbounded_channel,
    },
};

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

    let mut pool = ClientsPool::default();
    let (tx, mut rx) = unbounded_channel::<MasterCommand>();

    loop {
        tokio::select! {
            command = rx.recv() => {
                handle_command(
                    &mut writer,
                    command.unwrap(),
                    &mut pool
                ).await?;
            }

            pkt_type = reader.read_pkt_type() => {
                let pkt_type = pkt_type?;
                let frame = reader.read_frame(pkt_type, usize::MAX).await?;

                handle_frame(
                    &args.magic,
                    remote_port,
                    &local_address,
                    &tx,
                    &mut init_state,
                    &mut pool,
                    &mut writer,
                    frame
                ).await?;
            }
        }
    }
}
