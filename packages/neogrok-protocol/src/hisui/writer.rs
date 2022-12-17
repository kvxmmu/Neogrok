use std::{
    future::Future,
    io::{
        self,
        IoSlice,
    },
};

use common::protocol::{
    error::ProtocolError,
    types::*,
};
use neogrok_compression::polymorphic::BufCompressor;
use tokio::io::AsyncWriteExt;

use super::{
    codec_utils::{
        encode_client_header,
        encode_fwd_header,
        encode_request_server_header,
        just_type,
    },
    frame::Frame,
};
use crate::compression::types::{
    CompressionStatus,
    CompressionStrategy,
};

macro_rules! impl_writer {
    (struct $name:ident<$gen:ident>;) => {};

    (
        struct $name:ident<$gen:ident>;
        impl { $($block:tt)* }
        $($tail:tt)*
    ) => {
        impl<$gen> $name<$gen>
        where $gen: AsyncWriteExt + Unpin {
            $($block)*
        }

        impl_writer!(struct $name<$gen>; $($tail)*);
    };
}

pub struct HisuiWriter<Writer> {
    inner: Writer,
    pub(crate) compressor: BufCompressor,
}

impl_writer! {
    struct HisuiWriter<Writer>;

    // Requestors
    impl {
        pub async fn request_server(
            &mut self,
            port: u16,
            protocol: Protocol,
        ) -> io::Result<()> {
            let (hdr, len) = encode_request_server_header(
                port,
                protocol
            );
            self.inner.write_all(&hdr[..len])
                .await
        }

        pub fn request_ping(&mut self) -> impl Future<Output = io::Result<()>> + '_ {
            self.inner.write_u8(just_type(Frame::PING))
        }
    }

    // Responders
    impl {
        pub async fn respond_update_rights(
            &mut self,
            rights: Rights
        ) -> io::Result<()> {
            self.inner.write_all(&[
                just_type(Frame::UPDATE_RIGHTS),
                rights.bits(),
            ])
            .await
        }

        pub async fn respond_server(
            &mut self,
            port: u16,
        ) -> io::Result<()> {
            self.inner.write_all(&[
                just_type(Frame::SERVER),
                (port & 0xff) as u8,
                (port >> 8) as u8,
            ])
            .await
        }

        pub async fn respond_error(&mut self, error: ProtocolError) -> io::Result<()> {
            self.inner.write_all(&[
                just_type(Frame::ERROR),
                error as _
            ]).await
        }

        pub async fn respond_ping(
            &mut self,
            server_name: &str,
            algorithm: CompressionAlgorithm,
            compression_level: u8
        ) -> io::Result<()> {
            self.write_vectored(
                &[
                    just_type(Frame::PING),
                    algorithm as _,
                    compression_level,

                    server_name.len() as u8,
                ],
                server_name.as_bytes()
            ).await
        }
    }

    // Writers
    impl {
        pub async fn write_auth_through_magic(
            &mut self,
            magic: &str
        ) -> io::Result<()> {
            self.write_vectored(
                &[just_type(Frame::AUTH_MAGIC), magic.len() as _],
                magic.as_bytes()
            )
            .await
        }

        pub fn write_disconnect(
            &mut self,
            id: u16
        ) -> impl Future<Output = io::Result<()>> + '_ {
            self.write_client_related_pkt(Frame::DISCONNECT, id)
        }

        pub async fn write_forward(
            &mut self,
            id: u16,
            buffer: &[u8],
            strategy: CompressionStrategy,
        ) -> io::Result<Option<CompressionStatus>> {
            let orig_len = buffer.len();
            let mut compressed: Vec<u8> = Vec::new();
            let mut buffer = buffer;

            if let CompressionStrategy::TryCompress { with_threshold } = strategy {
                if (orig_len as u16) >= with_threshold {
                    if let Some(succ) = self.compressor.compress(buffer, orig_len) {
                        compressed = succ;
                        buffer = &compressed;
                    }
                }
            }

            let (hdr, hdr_length) = encode_fwd_header(
                id,
                buffer.len() as _,
                compressed.capacity() != 0,
            );

            self.write_vectored(
                &hdr[..hdr_length],
                buffer
            )
            .await
            .map(|()| if compressed.capacity() != 0 {
                Some(CompressionStatus {
                    before: orig_len as _,
                    after: buffer.len() as _,
                })
            } else {
                None
            })
        }

        pub fn write_connect(
            &mut self,
            id: u16
        ) -> impl Future<Output = io::Result<()>> + '_ {
            self.write_client_related_pkt(Frame::CONNECT, id)
        }
    }

    // Helper utils
    impl {
        async fn write_client_related_pkt(
            &mut self,
            pkt_type: u8,
            id: u16
        ) -> io::Result<()> {
            let (hdr, len) = encode_client_header(pkt_type, id);
            self.inner.write_all(&hdr[..len])
                .await
        }

        async fn write_vectored(
            &mut self,
            prepend: &[u8],
            append: &[u8],
        ) -> io::Result<()> {
            let mut written: usize = 0;

            let mut prepend_io = IoSlice::new(prepend);
            let mut append_io = IoSlice::new(append);

            let prepend_len = prepend.len();
            let append_len = append.len();
            let total = prepend_len + append_len;

            loop {
                if written == total {
                    break Ok(());
                }

                let amount = self
                    .inner
                    .write_vectored(&[prepend_io, append_io])
                    .await?;
                written += amount;

                if written < prepend_len {
                    prepend_io = IoSlice::new(&prepend[amount..]);
                } else {
                    prepend_io = IoSlice::new(&[]);
                    append_io =
                        IoSlice::new(&append[(written - prepend_len)..]);
                }
            }
        }
    }
}

impl<Writer> HisuiWriter<Writer> {
    pub fn new(writer: Writer, compressor: BufCompressor) -> Self {
        Self {
            inner: writer,
            compressor,
        }
    }
}
