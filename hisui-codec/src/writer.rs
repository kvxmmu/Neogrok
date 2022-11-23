use tokio::io::AsyncWriteExt;

pub struct HisuiWriter<Writer> {
    inner: Writer,
}

impl<Writer> HisuiWriter<Writer>
where
    Writer: AsyncWriteExt + Unpin,
{
    pub fn new(inner: Writer) -> Self {
        Self { inner }
    }
}
