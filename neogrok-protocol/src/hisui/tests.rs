use {
    super::codec_utils::encode_request_server_header,
    crate::hisui::{
        codec_utils::{
            encode_client_header,
            encode_fwd_header,
            encode_type,
            just_type,
        },
        frame::Frame,
    },
    common::protocol::types::*,
};

#[test]
fn test_req_server_encoder() {
    assert_eq!(
        encode_request_server_header(10, Protocol::Tcp),
        ([encode_type(Frame::SERVER, PacketFlags::SHORT2), 10, 0,], 3)
    );
    assert_eq!(
        encode_request_server_header(10, Protocol::Udp),
        (
            [encode_type(Frame::SERVER, PacketFlags::COMPRESSED), 10, 0],
            3
        )
    );
    assert_eq!(
        encode_request_server_header(0, Protocol::Udp),
        (
            [
                encode_type(
                    Frame::SERVER,
                    PacketFlags::SHORT | PacketFlags::COMPRESSED
                ),
                0,
                0
            ],
            1,
        )
    );

    assert_eq!(
        encode_request_server_header(0, Protocol::Tcp),
        (
            [
                encode_type(
                    Frame::SERVER,
                    PacketFlags::SHORT | PacketFlags::SHORT2
                ),
                0,
                0
            ],
            1
        )
    );
}

#[test]
fn test_just_type_equality() {
    assert_eq!(
        encode_type(Frame::DISCONNECT, PacketFlags::empty()),
        just_type(Frame::DISCONNECT)
    );
}

#[test]
fn test_client_flags_application() {
    assert_eq!(
        encode_client_header(Frame::CONNECT, 10),
        (
            [encode_type(Frame::CONNECT, PacketFlags::SHORT2), 10, 0,],
            2
        )
    );

    assert_eq!(
        encode_client_header(Frame::DISCONNECT, 1024),
        ([just_type(Frame::DISCONNECT), 0, 4], 3)
    );
}

#[test]
fn test_fwd_flags_application() {
    assert_eq!(
        encode_fwd_header(10, 1024, true),
        (
            [
                encode_type(
                    Frame::FORWARD,
                    PacketFlags::SHORT2 | PacketFlags::COMPRESSED
                ),
                10,
                0,
                4,
                0
            ],
            4
        )
    );

    assert_eq!(
        encode_fwd_header(10, 10, false),
        (
            [
                encode_type(
                    Frame::FORWARD,
                    PacketFlags::SHORT | PacketFlags::SHORT2
                ),
                10,
                10,
                0,
                0,
            ],
            3
        )
    );

    assert_eq!(
        encode_fwd_header(1024, 1024, true),
        (
            [
                encode_type(Frame::FORWARD, PacketFlags::COMPRESSED),
                0,
                4,
                0,
                4
            ],
            5
        )
    )
}
