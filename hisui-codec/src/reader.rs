use {
    crate::{
        error::ReadError,
        protocol::{
            flags::PacketFlags,
            frame::{
                Frame,
                ProtocolError,
            },
        },
    },
    common_codec::{
        permissions::Rights,
        CodecSide,
        Protocol,
    },
    std::{
        future::Future,
        io,
        pin::Pin,
    },
    tokio::{
        io::{
            AsyncRead,
            AsyncReadExt,
            ReadBuf,
        },
        macros::support::poll_fn,
    },
};

pub struct HisuiReader<Reader> {
    inner: Reader,
    side: CodecSide,
}

impl<Reader> HisuiReader<Reader>
where
    Reader: AsyncReadExt + AsyncRead + Unpin,
{
    pub fn read_pkt_type(
        &mut self,
    ) -> impl Future<Output = io::Result<u8>> + '_ {
        self.inner.read_u8()
    }

    pub async fn read_frame(
        &mut self,
        pk: u8,
        max_forward_size: usize,
    ) -> Result<Frame, ReadError> {
        const fn decode_pkt_type(pkt_type: u8) -> (u8, PacketFlags) {
            let Some(flags) = PacketFlags::from_bits(pkt_type & 0b111) else {
                unreachable!()
            };

            (pkt_type >> 3, flags)
        }

        let (pkt_type, flags) = decode_pkt_type(pk);
        Ok(match pkt_type {
            Frame::CONNECTED => Frame::Connected {
                id: self.read_client_id(flags).await?,
            },
            Frame::DISCONNECTED => Frame::Disconnected {
                id: self.read_client_id(flags).await?,
            },

            Frame::FORWARD => {
                let id = self.read_client_id(flags).await?;
                let length = self.read_length(flags).await? as usize;

                if length > max_forward_size {
                    self.skip_read(length).await?;
                    return Err(ReadError::TooLongBuffer {
                        expected: max_forward_size,
                        found: length,
                    });
                }

                let mut buffer = Vec::with_capacity(length);
                let mut rd_buffer =
                    ReadBuf::uninit(buffer.spare_capacity_mut());

                while rd_buffer.filled().len() < length {
                    poll_fn(|cx| {
                        Pin::new(&mut self.inner)
                            .poll_read(cx, &mut rd_buffer)
                    })
                    .await?;
                }

                unsafe { buffer.set_len(length) };

                Frame::Forward { id, buffer }
            }
            Frame::UPDATE_RIGHTS => {
                let flags = self.inner.read_u8().await?;
                Frame::UpdateRights {
                    rights: Rights::from_bits(flags)
                        .ok_or(ReadError::InvalidRightsFlags { flags })?,
                }
            }

            Frame::ERROR => {
                let error_code = self.inner.read_u8().await?;
                Frame::Error(ProtocolError::try_from(error_code).map_err(
                    |_| ReadError::UnknownErrorVariant {
                        variant: error_code,
                    },
                )?)
            }

            Frame::PING if self.side == CodecSide::Server => Frame::Ping,
            Frame::PING if self.side == CodecSide::Client => {
                Frame::PingResponse {
                    name: self.read_string().await?,
                }
            }

            Frame::START_SERVER if self.side == CodecSide::Server => {
                Frame::StartServer {
                    port: if flags.intersects(PacketFlags::SHORT) {
                        0
                    } else {
                        self.inner.read_u16_le().await?
                    },
                    protocol: if flags.intersects(PacketFlags::SHORT2) {
                        Protocol::Tcp
                    } else {
                        Protocol::Udp
                    },
                }
            }
            Frame::START_SERVER if self.side == CodecSide::Client => {
                Frame::StartServerResponse {
                    port: self.inner.read_u16_le().await?,
                }
            }

            Frame::AUTH_THROUGH_MAGIC => Frame::AuthThroughMagic {
                magic: self.read_buffer_prefixed().await?,
            },

            _ => {
                return Err(ReadError::UnknownPacket {
                    packet: pkt_type,
                    flags,
                })
            }
        })
    }

    async fn skip_read(&mut self, n: usize) -> io::Result<()> {
        let mut skip_buf = [0; 64];
        let mut skipped = 0;
        while skipped < n {
            let chunk_size = (n - skipped).min(skip_buf.len());
            self.inner
                .read_exact(&mut skip_buf[..chunk_size])
                .await?;
            skipped += chunk_size;
        }

        Ok(())
    }

    async fn read_variadic(
        &mut self,
        flags: PacketFlags,
        need_flags: PacketFlags,
    ) -> io::Result<u16> {
        if flags.contains(need_flags) {
            self.inner.read_u8().await.map(|v| v as _)
        } else {
            self.inner.read_u16_le().await
        }
    }

    fn read_length(
        &mut self,
        flags: PacketFlags,
    ) -> impl Future<Output = io::Result<u16>> + '_ {
        self.read_variadic(flags, PacketFlags::SHORT)
    }

    fn read_client_id(
        &mut self,
        flags: PacketFlags,
    ) -> impl Future<Output = io::Result<u16>> + '_ {
        self.read_variadic(flags, PacketFlags::SHORT2)
    }

    async fn read_string(&mut self) -> Result<String, ReadError> {
        let data = self.read_buffer_prefixed().await?;
        String::from_utf8(data).map_err(|_| ReadError::InvalidString)
    }

    async fn read_buffer_prefixed(&mut self) -> io::Result<Vec<u8>> {
        let length = self.inner.read_u8().await?;
        let mut buffer = vec![0; length as _];
        self.inner
            .read_exact(&mut buffer)
            .await
            .map(move |_| buffer)
    }

    pub const fn client(inner: Reader) -> Self {
        Self::new(inner, CodecSide::Client)
    }

    pub const fn server(inner: Reader) -> Self {
        Self::new(inner, CodecSide::Server)
    }

    pub const fn new(inner: Reader, side: CodecSide) -> Self {
        Self { inner, side }
    }
}
