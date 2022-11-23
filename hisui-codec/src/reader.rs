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
    common_codec::Protocol,
    std::{
        future::Future,
        io,
    },
    tokio::io::AsyncReadExt,
};

pub struct HisuiReader<Reader> {
    inner: Reader,
}

impl<Reader> HisuiReader<Reader>
where
    Reader: AsyncReadExt + Unpin,
{
    pub fn read_pkt_type(
        &mut self,
    ) -> impl Future<Output = io::Result<u8>> + '_ {
        self.inner.read_u8()
    }

    pub async fn read_frame(&mut self, pk: u8) -> Result<Frame, ReadError> {
        const fn decode_pkt_type(pkt_type: u8) -> (u8, PacketFlags) {
            let Some(flags) = PacketFlags::from_bits(pkt_type & 0b111) else {
                unreachable!()
            };

            (pkt_type >> 3, flags)
        }

        let (pkt_type, flags) = decode_pkt_type(pk);
        Ok(match pkt_type {
            Frame::ERROR => {
                let error_code = self.inner.read_u8().await?;
                Frame::Error(
                    ProtocolError::try_from(error_code)
                        .map_err(|_| ReadError::UnknownErrorVariant)?,
                )
            }
            Frame::PING => Frame::Ping,
            Frame::START_SERVER => Frame::StartServer {
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
            },

            Frame::AUTH_THROUGH_MAGIC => Frame::AuthThroughMagic {
                magic: self.read_string().await?,
            },

            _ => {
                return Err(ReadError::UnknownPacket {
                    packet: pkt_type,
                    flags,
                })
            }
        })
    }

    async fn read_string(&mut self) -> io::Result<Vec<u8>> {
        let length = self.inner.read_u8().await?;
        let mut buffer = vec![0; length as _];
        self.inner
            .read_exact(&mut buffer)
            .await
            .map(move |_| buffer)
    }

    pub fn new(inner: Reader) -> Self {
        Self { inner }
    }
}
