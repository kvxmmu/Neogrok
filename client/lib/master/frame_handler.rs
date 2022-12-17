use neogrok_protocol::hisui::{
    frame::Frame,
    writer::HisuiWriter,
};
use tokio::io::AsyncWriteExt;

use super::state::{
    SendResult,
    State,
};
use crate::{
    commands::SlaveCommand,
    slave::listener::run_client,
};

pub async fn handle_frame<Writer>(
    _writer: &mut HisuiWriter<Writer>,
    state: &mut State,
    frame: Frame,
    address: &str,
) -> anyhow::Result<()>
where
    Writer: AsyncWriteExt + Unpin,
{
    match frame {
        Frame::Forward { id, buffer } => {
            state
                .send_to(id, SlaveCommand::Forward { buffer })
                .await
                .ignore();
        }

        Frame::Disconnect { id } => {
            match state
                .send_to(id, SlaveCommand::ForceDisconnect)
                .await
            {
                SendResult::Ok => {}
                error => {
                    tracing::error!(?error, "failed to disconnect client");
                }
            }

            state.remove_slave(id);
        }

        Frame::Connect { id } => {
            let (tx, rx) = flume::unbounded();
            state.insert_slave(id, tx);
            tokio::spawn(run_client(
                id,
                address.to_owned(),
                rx,
                state.tx.clone(),
            ));
        }

        Frame::Error(err) => {
            tracing::error!(?err, "error");
        }

        frame => unimplemented!("{frame:#?}"),
    }

    Ok(())
}