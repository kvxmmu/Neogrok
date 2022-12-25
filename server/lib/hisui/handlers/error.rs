use std::{
    io,
    net::SocketAddr,
};

use neogrok_declmacro::define_integral_enums;
use neogrok_protocol::hisui::{
    error::ReadError,
    writer::HisuiWriter,
};
use tokio::io::AsyncWriteExt;

define_integral_enums! {
    @easy ErrorType =
        NonFatal,
        NonFatalButDisconnect,
        Fatal;
}

pub async fn handle_error<Writer>(
    _writer: &mut HisuiWriter<Writer>,
    error: &ReadError,
    _address: &SocketAddr,
) -> io::Result<ErrorType>
where
    Writer: AsyncWriteExt + Unpin,
{
    Ok(match error {
        ReadError::Io(_) => ErrorType::NonFatalButDisconnect,
        ReadError::FailedToDecompress(err) => {
            tracing::error!(?err, "Failed to decompress compressed data");
            ErrorType::Fatal
        }

        err => {
            tracing::error!(?err, "Non-fatal error");
            ErrorType::NonFatal
        }
    })
}
