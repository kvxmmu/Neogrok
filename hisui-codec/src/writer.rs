use {
    crate::protocol::{
        encode_type,
        flags::PacketFlags,
        frame::Frame,
    },
    std::{
        future::Future,
        io::{
            self,
            IoSlice,
        },
    },
    tokio::io::AsyncWriteExt,
};

pub struct HisuiWriter<Writer> {
    inner: Writer,
}

impl<Writer> HisuiWriter<Writer>
where
    Writer: AsyncWriteExt + Unpin,
{
    pub async fn respond_ping(&mut self, data: &[u8]) -> io::Result<()> {
        self.write_vectored(
            &[
                encode_type(Frame::PING, PacketFlags::empty()),
                data.len() as u8,
            ],
            data,
        )
        .await
    }

    async fn write_vectored(
        &mut self,
        prepend: &[u8],
        buffer: &[u8],
    ) -> io::Result<()> {
        let mut written: usize = 0;

        let prep_len = prepend.len();
        let buf_len = buffer.len();
        let total = prep_len + buf_len;

        let mut prepend_io = IoSlice::new(prepend);
        let mut buffer_io = IoSlice::new(buffer);

        loop {
            let amount = self
                .inner
                .write_vectored(&[prepend_io, buffer_io])
                .await?;
            written += amount;

            match written {
                _ if written == total => break Ok(()),
                prep_in_progress if written < prepend.len() => {
                    prepend_io = IoSlice::new(&prepend[prep_in_progress..]);
                }

                buffer_in_progress if written >= prepend.len() => {
                    buffer_io =
                        IoSlice::new(&buffer[buffer_in_progress - prep_len..]);
                }

                _ => {}
            }
        }
    }

    pub fn request_ping(
        &mut self,
    ) -> impl Future<Output = io::Result<()>> + '_ {
        self.inner
            .write_u8(encode_type(Frame::PING, PacketFlags::empty()))
    }

    pub fn new(inner: Writer) -> Self {
        Self { inner }
    }
}
