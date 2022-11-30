use {
    super::{
        error::ReadError,
        frame::{
            Compression,
            Frame,
        },
    },
    common::protocol::{
        error::ProtocolError,
        types::{
            CodecSide,
            CompressionAlgorithm,
            PacketFlags,
            Protocol,
            Rights,
        },
    },
    neogrok_compression::polymorphic::BufDecompressor,
    std::{
        future::{
            poll_fn,
            Future,
        },
        io,
        pin::Pin,
    },
    tokio::io::{
        AsyncRead,
        AsyncReadExt,
        ReadBuf,
    },
};

pub struct HisuiReader<Reader> {
    inner: Reader,
    side: CodecSide,

    pub(crate) decompressor: BufDecompressor,
}

impl<Reader> HisuiReader<Reader>
where
    Reader: AsyncReadExt + AsyncRead + Unpin,
{
    pub async fn read_frame_inconcurrent(
        &mut self,
    ) -> Result<Frame, ReadError> {
        let (pkt_type, flags) = self.read_packet_type().await?;
        self.read_frame(pkt_type, flags).await
    }

    pub async fn read_frame(
        &mut self,
        pkt_type: u8,
        flags: PacketFlags,
    ) -> Result<Frame, ReadError> {
        Ok(match pkt_type {
            Frame::SERVER if self.side == CodecSide::Server => {
                return self.read_req_server_frame(flags).await
            }
            Frame::SERVER if self.side == CodecSide::Client => {
                Frame::ServerResponse {
                    port: self.inner.read_u16_le().await?,
                }
            }

            Frame::UPDATE_RIGHTS => {
                let rights = self.inner.read_u8().await?;
                Frame::UpdateRights {
                    new_rights: Rights::from_bits(rights)
                        .ok_or(ReadError::InvalidRights { rights })?,
                }
            }

            Frame::PING if self.side == CodecSide::Server => {
                Frame::PingRequest
            }
            Frame::PING if self.side == CodecSide::Client => {
                Frame::PingResponse {
                    compression: self.read_compression_details().await?,
                    server_name: self.read_string_prefixed().await?,
                }
            }

            Frame::CONNECT => Frame::Connect {
                id: self.read_client_id(flags).await?,
            },

            Frame::FORWARD => {
                let id = self.read_client_id(flags).await?;
                let length = self.read_length(flags).await? as usize;
                let buffer = self
                    .read_fwd_payload(length, flags, length << 1)
                    .await?;

                Frame::Forward { id, buffer }
            }

            Frame::DISCONNECT => Frame::Disconnect {
                id: self.read_client_id(flags).await?,
            },

            Frame::AUTH_MAGIC => Frame::AuthThroughMagic {
                magic: self.read_string_prefixed().await?,
            },

            Frame::ERROR => {
                let code = self.inner.read_u8().await?;
                Frame::Error(
                    ProtocolError::try_from(code).map_err(|()| {
                        ReadError::InvalidErrorCode { code }
                    })?,
                )
            }

            _ => {
                return Err(ReadError::InvalidPacketType {
                    pkt_type,
                    flags,
                })
            }
        })
    }

    pub async fn read_packet_type(
        &mut self,
    ) -> Result<(u8, PacketFlags), ReadError> {
        let data = self.inner.read_u8().await?;
        let (pkt_type, flags) = (
            data >> 3,
            PacketFlags::from_bits(data & 0b111).ok_or(
                ReadError::InvalidPacketFlags {
                    flags: data & 0b111,
                },
            )?,
        );

        Ok((pkt_type, flags))
    }

    // Helpers

    async fn read_req_server_frame(
        &mut self,
        flags: PacketFlags,
    ) -> Result<Frame, ReadError> {
        let protocol = if flags.contains(PacketFlags::COMPRESSED) {
            Protocol::Udp
        } else if flags.contains(PacketFlags::SHORT2) {
            Protocol::Tcp
        } else {
            return Err(ReadError::InvalidProtocol);
        };
        let port = if flags.contains(PacketFlags::SHORT) {
            0
        } else {
            self.inner.read_u16_le().await?
        };

        Ok(Frame::ServerRequest { port, protocol })
    }

    async fn read_compression_details(
        &mut self,
    ) -> Result<Compression, ReadError> {
        let algorithm =
            CompressionAlgorithm::try_from(self.inner.read_u8().await?)
                .map_err(|()| ReadError::FailedToReadCompressionDetails)?;
        let level = self.inner.read_u8().await?;
        Ok(Compression { algorithm, level })
    }

    async fn read_fwd_payload(
        &mut self,
        length: usize,
        flags: PacketFlags,
        max_size: usize,
    ) -> Result<Vec<u8>, ReadError> {
        let mut buffer = self.read_exact(length).await?;
        if flags.contains(PacketFlags::COMPRESSED) {
            if let Some(buf) =
                self.decompressor.decompress(&buffer, max_size)
            {
                buffer = buf;
            } else {
                return Err(ReadError::FailedToDecompress);
            }
        }

        Ok(buffer)
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

    async fn read_variadic(
        &mut self,
        flags: PacketFlags,
        need: PacketFlags,
    ) -> io::Result<u16> {
        if flags.contains(need) {
            let data = self.inner.read_u8().await?;
            Ok(data as u16)
        } else {
            self.inner.read_u16_le().await
        }
    }

    async fn read_string_prefixed(&mut self) -> Result<String, ReadError> {
        let buffer = self.read_buffer_prefixed().await?;
        String::from_utf8(buffer).map_err(|_| ReadError::InvalidString)
    }

    async fn read_buffer_prefixed(&mut self) -> io::Result<Vec<u8>> {
        let length = self.inner.read_u8().await? as usize;
        self.read_exact(length).await
    }

    async fn read_exact(
        &mut self,
        capacity: usize,
    ) -> io::Result<Vec<u8>> {
        let mut buffer: Vec<u8> = Vec::with_capacity(capacity);
        let mut rd_buffer = ReadBuf::uninit(buffer.spare_capacity_mut());

        while rd_buffer.filled().len() != capacity {
            poll_fn(|cx| {
                Pin::new(&mut self.inner).poll_read(cx, &mut rd_buffer)
            })
            .await?;
        }

        let filled = rd_buffer.filled().len();
        unsafe { buffer.set_len(filled) }

        Ok(buffer)
    }
}

impl<Reader> HisuiReader<Reader> {
    // Creation

    pub fn server(reader: Reader, decompressor: BufDecompressor) -> Self {
        Self::new(reader, CodecSide::Server, decompressor)
    }

    pub fn client(reader: Reader, decompressor: BufDecompressor) -> Self {
        Self::new(reader, CodecSide::Client, decompressor)
    }

    pub fn new(
        reader: Reader,
        side: CodecSide,
        decompressor: BufDecompressor,
    ) -> Self {
        Self {
            inner: reader,
            side,
            decompressor,
        }
    }
}
