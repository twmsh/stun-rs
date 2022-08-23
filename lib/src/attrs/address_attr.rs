use crate::attrs::RawAttr;
use crate::constants::*;
use bytes::{BufMut, BytesMut};
use std::net::{IpAddr, Ipv4Addr, Ipv6Addr, SocketAddr};
use std::ops::Deref;
use crate::error::ParsePacketErr;

// 地址类的attribute
//
// mapped-address  response-origin   other-address

// ipv4: family: 0x01, 4 bytes
// ipv6: family: 0x02, 16 bytes

#[derive(Debug, Clone)]
pub struct AddressAttr {
    pub attr_type: u16,
    pub address: SocketAddr,
}

impl AddressAttr {
    pub fn new(attr_type: u16, address: SocketAddr) -> Self {
        Self { attr_type, address }
    }
}

impl From<AddressAttr> for RawAttr {
    fn from(attr: AddressAttr) -> Self {
        let (family, port, ip_bytes, ip_len) = match &attr.address {
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

        RawAttr::new(attr.attr_type, value)
    }
}

impl TryFrom<RawAttr> for AddressAttr {
    type Error = ParsePacketErr;

    fn try_from(base_attr: RawAttr) -> Result<Self, Self::Error> {
        let attr_type = base_attr.attr_type;

        // 从 value中解析
        let mut index = 0_usize;
        let value = base_attr.value.deref();

        if value.len() < 4 {
            return Err(ParsePacketErr::BufSize(format!("attr buf len:{}",
                                                       value.len())));
        }

        index += 1;
        let family = value[index];

        index += 1;
        let port = u16::from_be_bytes([value[index], value[index + 1]]);

        index += 2;

        let address = match family {
            ATTR_FAMILY_IPV4 => {
                // 4 bytes
                if index + 4 > value.len() {
                    return Err(ParsePacketErr::BufSize(
                        "ipv4 buf len < 4".to_string()));
                }
                let mut addr = [0_u8; 4];
                addr.copy_from_slice(&value[index..index + 4]);
                SocketAddr::new(IpAddr::V4(Ipv4Addr::from(addr)), port)
            }
            ATTR_FAMILY_IPV6 => {
                // 16 bytes
                if index + 16 > value.len() {
                    return Err(ParsePacketErr::BufSize(
                        "ipv6 buf len < 16".to_string()));
                }
                let mut addr = [0_u8; 16];
                addr.copy_from_slice(&value[index..index + 16]);
                SocketAddr::new(IpAddr::V6(Ipv6Addr::from(addr)), port)
            }
            v => {
                return Err(ParsePacketErr::BadValue(
                    format!("ip family: {}",
                            v)
                ));
            }
        };

        Ok(Self { attr_type, address })
    }
}
