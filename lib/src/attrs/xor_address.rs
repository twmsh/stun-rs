use crate::attrs::RawAttr;
use crate::constants::*;
use bytes::{BufMut, BytesMut};
use std::net::SocketAddr;

use crate::attrs::address_attr::AddressAttr;
use crate::error::ParsePacketErr;
use crate::header::TransId;
use crate::util;

// xor-mapped-address 端口和ip需要混淆
// port 和 magic cookie 做 xor
// address(ipv4) 和 magic cookie做xor
// address(ipv6) 和 magic cookie + trans_id 做xor

#[derive(Debug, Clone)]
pub struct XorMappedAddress {
    pub address: SocketAddr,
    pub trans_id: TransId,
}

impl XorMappedAddress {
    pub fn new(trans_id: TransId, address: SocketAddr) -> Self {
        Self { trans_id, address }
    }

    pub fn from_base_attr(base_attr: RawAttr, trans_id: &TransId) -> Result<Self, ParsePacketErr> {
        let address_attr: AddressAttr = base_attr.try_into()?;

        let address = match address_attr.address {
            SocketAddr::V4(v) => SocketAddr::V4(util::xor_address_v4(v)),
            SocketAddr::V6(v) => SocketAddr::V6(util::xor_address_v6(v, trans_id)),
        };

        Ok(Self {
            address,
            trans_id: *trans_id,
        })
    }
}

impl From<XorMappedAddress> for RawAttr {
    fn from(attr: XorMappedAddress) -> Self {
        let xor_socket_addr = util::xor_address(attr.address, &attr.trans_id);

        let (family, port, ip_bytes, ip_len) = match &xor_socket_addr {
            SocketAddr::V4(addr) => {
                let ip_bytes: Vec<u8> = addr.ip().octets().into();
                (ATTR_FAMILY_IPV4, addr.port(), ip_bytes, 4)
            }
            SocketAddr::V6(addr) => {
                let ip_bytes: Vec<u8> = addr.ip().octets().into();
                (ATTR_FAMILY_IPV6, addr.port(), ip_bytes, 16)
            }
        };

        let mut bytes_buf = BytesMut::with_capacity(4 + ip_len);

        bytes_buf.put_u8(0);
        bytes_buf.put_u8(family);
        bytes_buf.put_u16(port);
        bytes_buf.put_slice(&ip_bytes);
        let value = bytes_buf.freeze();

        RawAttr::new(ATTR_XOR_MAPPED_ADDRESS, value)
    }
}
