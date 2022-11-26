use {
    crate::protocol::{
        encode_type,
        flags::PacketFlags,
        frame::{
            Frame,
            ProtocolError,
        },
    },
    common_codec::{
        compression::PayloadCompressor,
        permissions::Rights,
        Protocol,
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
    compressor: PayloadCompressor,
}

impl<Writer> HisuiWriter<Writer>
where
    Writer: AsyncWriteExt + Unpin,
{
    pub async fn authorize_through_magic(
        &mut self,
        magic: impl AsRef<str>,
    ) -> io::Result<()> {
        let magic = magic.as_ref();
        self.write_vectored(
            &[
                encode_type(
                    Frame::AUTH_THROUGH_MAGIC,
                    PacketFlags::empty(),
                ),
                magic.len() as _,
            ],
            magic.as_bytes(),
        )
        .await
    }

    pub async fn write_forward(
        &mut self,
        id: u16,
        mut buffer: &[u8],
        compress_threshold: usize,
    ) -> io::Result<bool> {
        let mut compressed = Vec::<u8>::new();
        let mut flags = PacketFlags::empty();

        if buffer.len() >= compress_threshold {
            let comp_buf = self.compressor.compress(buffer, buffer.len());
            if let Some(buf) = comp_buf {
                compressed = buf;
                buffer = &compressed;

                flags |= PacketFlags::COMPRESSED;
            }
        }

        let mut hdr = [0u8; 5];
        let mut offset = 1;
        let buf_len = buffer.len();

        offset += if id <= 0xff {
            flags |= PacketFlags::SHORT2;
            hdr[offset] = id as _;

            1
        } else {
            hdr[offset] = (id & 0xff) as _;
            hdr[offset + 1] = (id >> 8) as _;

            2
        };

        offset += if buf_len <= 0xff {
            flags |= PacketFlags::SHORT;
            hdr[offset] = buf_len as _;

            1
        } else {
            hdr[offset] = (buf_len & 0xff) as _;
            hdr[offset + 1] = (buf_len >> 8) as _;

            2
        };

        hdr[0] = encode_type(Frame::FORWARD, flags);

        self.write_vectored(&hdr[..offset], buffer)
            .await?;
        Ok(compressed.capacity() != 0)
    }

    pub fn write_connected(
        &mut self,
        id: u16,
    ) -> impl Future<Output = io::Result<()>> + '_ {
        self.write_client_id_pkt(id, Frame::CONNECTED)
    }

    pub fn write_disconnected(
        &mut self,
        id: u16,
    ) -> impl Future<Output = io::Result<()>> + '_ {
        self.write_client_id_pkt(id, Frame::DISCONNECTED)
    }

    async fn write_client_id_pkt(
        &mut self,
        id: u16,
        pkt_type: u8,
    ) -> io::Result<()> {
        if id <= 0xff {
            self.inner
                .write_all(&[
                    encode_type(pkt_type, PacketFlags::SHORT2),
                    id as u8,
                ])
                .await
        } else {
            self.inner
                .write_all(&[
                    encode_type(pkt_type, PacketFlags::empty()),
                    (id & 0xff) as u8,
                    (id >> 8) as u8,
                ])
                .await
        }
    }

    pub async fn request_server(
        &mut self,
        port: u16,
        protocol: Protocol,
    ) -> io::Result<()> {
        let mut flags = PacketFlags::empty();
        let mut buffer = [0; 4];
        let mut offset = 1;

        offset += if port == 0 {
            flags |= PacketFlags::SHORT;

            0
        } else {
            buffer[1] = (port & 0xff) as u8;
            buffer[2] = (port >> 8) as u8;

            2
        };

        offset += if protocol == Protocol::Tcp {
            flags |= PacketFlags::SHORT2;

            0
        } else {
            buffer[offset] = protocol as u8;

            1
        };

        buffer[0] = encode_type(Frame::START_SERVER, flags);
        self.inner.write_all(&buffer[..offset]).await
    }

    pub async fn respond_server(&mut self, port: u16) -> io::Result<()> {
        self.inner
            .write_all(&[
                encode_type(Frame::START_SERVER, PacketFlags::empty()),
                (port & 0xff) as u8,
                (port >> 8) as u8,
            ])
            .await
    }

    pub async fn respond_update_rights(
        &mut self,
        rights: Rights,
    ) -> io::Result<()> {
        self.inner
            .write_all(&[
                encode_type(Frame::UPDATE_RIGHTS, PacketFlags::empty()),
                rights.bits(),
            ])
            .await
    }

    pub async fn respond_error(
        &mut self,
        error: ProtocolError,
    ) -> io::Result<()> {
        self.inner
            .write_all(&[
                encode_type(Frame::ERROR, PacketFlags::empty()),
                error as u8,
            ])
            .await
    }

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
            if written == total {
                break Ok(());
            }

            let amount = self
                .inner
                .write_vectored(&[prepend_io, buffer_io])
                .await?;
            written += amount;

            match written {
                prep_in_progress if written < prep_len => {
                    prepend_io =
                        IoSlice::new(&prepend[prep_in_progress..]);
                }

                buffer_in_progress => {
                    buffer_io = IoSlice::new(
                        &buffer[buffer_in_progress - prep_len..],
                    );
                }
            }
        }
    }

    pub fn request_ping(
        &mut self,
    ) -> impl Future<Output = io::Result<()>> + '_ {
        self.inner
            .write_u8(encode_type(Frame::PING, PacketFlags::empty()))
    }

    pub fn new(inner: Writer, compressor: PayloadCompressor) -> Self {
        Self { inner, compressor }
    }
}
