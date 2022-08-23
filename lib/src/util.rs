use crate::attrs::padding_attr::PaddingAttr;
use crate::constants::{MAGIC_COOKIE, TRANS_ID_LEN};
use crate::header::TransId;
use bytes::{BufMut, BytesMut};
use rand::prelude::*;
use std::fmt::Write as _;
use std::net::{Ipv4Addr, Ipv6Addr, SocketAddr, SocketAddrV4, SocketAddrV6};

pub fn print_bytes(buf: &[u8], separator: &str, row_width: usize) -> String {
    let mut hex = String::new();
    buf.iter().enumerate().for_each(|(x, y)| {
        // hex.push_str(&format!("{:02X}", y));
        let _ = write!(hex, "{:02X}", y);
        if (x + 1) % row_width == 0 {
            hex.push('\n');
        } else {
            hex.push_str(separator);
        }
    });

    hex
}

pub fn new_trans_id() -> TransId {
    let cookie_len = MAGIC_COOKIE.len();
    let mut trans_id = [0u8; TRANS_ID_LEN];

    trans_id[..cookie_len].copy_from_slice(&MAGIC_COOKIE[..]);
    rand::thread_rng().fill_bytes(&mut trans_id[4..]);
    trans_id
}

pub fn xor_address_v4(addr: SocketAddrV4) -> SocketAddrV4 {
    let port = addr.port();
    let magic_prefix = u16::from_be_bytes([MAGIC_COOKIE[0], MAGIC_COOKIE[1]]);
    let port = port ^ magic_prefix;

    let src_buf = addr.ip().octets();
    let mut buf = [0_u8; 4];
    for i in 0..buf.len() {
        buf[i] = src_buf[i] ^ MAGIC_COOKIE[i];
    }

    SocketAddrV4::new(Ipv4Addr::from(buf), port)
}

pub fn xor_address_v6(addr: SocketAddrV6, trans_id: &TransId) -> SocketAddrV6 {
    let port = addr.port();
    let magic_prefix = u16::from_be_bytes([MAGIC_COOKIE[0], MAGIC_COOKIE[1]]);
    let port = port ^ magic_prefix;

    let src_buf = addr.ip().octets();
    let mut buf = [0_u8; 16];
    for i in 0..buf.len() {
        if i < MAGIC_COOKIE.len() {
            buf[i] = src_buf[i] ^ MAGIC_COOKIE[i];
        } else {
            buf[i] = src_buf[i] ^ trans_id[i - MAGIC_COOKIE.len()];
        }
    }

    SocketAddrV6::new(Ipv6Addr::from(buf), port, 0, 0)
}

pub fn xor_address(addr: SocketAddr, trans_id: &TransId) -> SocketAddr {
    match addr {
        SocketAddr::V4(v) => SocketAddr::V4(xor_address_v4(v)),
        SocketAddr::V6(v) => SocketAddr::V6(xor_address_v6(v, trans_id)),
    }
}

// 4*n bytes
pub fn pack_error_reason(msg: &str) -> String {
    let buf = msg.as_bytes();
    let padding = match buf.len() % 4 {
        0 => 0,
        v => 4 - v,
    };

    match padding {
        0 => msg.to_string(),
        v => {
            let mut msg = msg.to_string();
            for _ in 0..v {
                msg.push(' ');
            }
            msg
        }
    }
}

pub fn pack_error_code(code: u16) -> u16 {
    let n1 = code / 100;
    let n2 = code % 100;

    n1 << 8 | n2
}

pub fn unpack_error_code(code: u16) -> u16 {
    let n2 = code & 0x00ff;
    let n1 = code << 8 & 0xff;
    n1 * 100 + n2
}

// len 需要是8的倍数 1500 < len  < 64K
pub fn new_padding_attr(len: u16) -> PaddingAttr {
    let new_len = len / 8 * 8;

    let mut data = BytesMut::with_capacity(new_len as usize);
    data.put_bytes(6, new_len as usize);

    PaddingAttr::new(data.freeze())
}
