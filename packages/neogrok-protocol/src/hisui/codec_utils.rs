use common::protocol::types::*;

use super::frame::Frame;

pub(crate) fn encode_request_server_header(
    port: u16,
    protocol: Protocol,
) -> ([u8; 3], usize) {
    let mut hdr = [0; 3];
    let mut flags = PacketFlags::empty();
    let mut size = 3;

    if port == 0 {
        flags |= PacketFlags::SHORT;
        size = 1;
        0
    } else {
        hdr[1] = (port & 0xff) as u8;
        hdr[2] = (port >> 8) as u8;

        2
    };
    match protocol {
        Protocol::Tcp => {
            flags |= PacketFlags::SHORT2;
        }
        Protocol::Udp => {
            flags |= PacketFlags::COMPRESSED;
        }

        Protocol::Reserved => unreachable!(),
    };

    hdr[0] = encode_type(Frame::SERVER, flags);

    (hdr, size)
}

pub(crate) const fn encode_client_header(
    pkt_type: u8,
    id: u16,
) -> ([u8; 3], usize) {
    if id <= 0xff {
        ([encode_type(pkt_type, PacketFlags::SHORT2), id as u8, 0], 2)
    } else {
        ([just_type(pkt_type), (id & 0xff) as u8, (id >> 8) as u8], 3)
    }
}

pub(crate) fn encode_fwd_header(
    id: u16,
    length: u16,
    compressed: bool,
) -> ([u8; 5], usize) {
    let mut hdr = [0_u8; 5];
    let mut flags = if compressed {
        PacketFlags::COMPRESSED
    } else {
        PacketFlags::empty()
    };
    let mut offset = 1_usize;

    offset += if id <= 0xff {
        hdr[offset] = id as u8;
        flags |= PacketFlags::SHORT2;

        1
    } else {
        hdr[offset] = (id & 0xff) as u8;
        hdr[offset + 1] = (id >> 8) as u8;
        2
    };

    offset += if length <= 0xff {
        hdr[offset] = length as u8;
        flags |= PacketFlags::SHORT;

        1
    } else {
        hdr[offset] = (length & 0xff) as u8;
        hdr[offset + 1] = (length >> 8) as u8;
        2
    };

    hdr[0] = encode_type(Frame::FORWARD, flags);

    (hdr, offset)
}

pub(crate) const fn encode_type(pkt_type: u8, flags: PacketFlags) -> u8 {
    unsafe { raw_encode_type(pkt_type, flags.bits()) }
}

pub(crate) const fn just_type(pkt_type: u8) -> u8 {
    unsafe { raw_encode_type(pkt_type, 0) }
}

pub(crate) const unsafe fn raw_encode_type(pkt_type: u8, flags: u8) -> u8 {
    (pkt_type << 3) | flags
}
