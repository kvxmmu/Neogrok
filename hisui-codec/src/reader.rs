use tokio::io::AsyncReadExt;

pub struct HisuiReader<Reader> {
    inner: Reader,
}

impl<Reader> HisuiReader<Reader>
where
    Reader: AsyncReadExt + Unpin,
{
    pub fn new(inner: Reader) -> Self {
        Self { inner }
    }
}
