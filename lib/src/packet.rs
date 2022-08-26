use crate::attrs;
use crate::attrs::address_attr::AddressAttr;
use crate::attrs::errcode_attr::ErrcodeAttr;
use crate::attrs::response_port::ResponsePort;
use crate::attrs::xor_address::XorMappedAddress;
use crate::attrs::RawAttr;
use crate::constants::*;
use crate::error::{AttrValidator, ParsePacketErr, ValidateErr};
use crate::header::Header;
use bytes::{BufMut, Bytes, BytesMut};
use std::fmt::Debug;

// 是否是一个正确的stun 包
// message_type 在范围内
// 验证message length
// 验证 magic cookie
// 属性解析是否正常

#[derive(Debug, Clone)]
pub struct Packet {
    pub header: Header,
    pub attrs: Vec<RawAttr>,
}

impl Packet {
    pub fn new(header: Header, attrs: Vec<RawAttr>) -> Self {
        let mut packet = Self { header, attrs };
        packet.update_header_len();
        packet
    }

    fn update_header_len(&mut self) {
        let total = self.attrs.iter().fold(0_usize, |acc, x| acc + x.len());
        self.header.msg_len = total as u16;
    }

    pub fn add_attr(&mut self, attr: RawAttr) {
        self.attrs.push(attr);
        self.update_header_len();
    }

    pub fn add_attrs(&mut self, mut attrs: Vec<RawAttr>) {
        self.attrs.append(&mut attrs);
        self.update_header_len();
    }

    pub fn pack(&self) -> Bytes {
        let mut buf = BytesMut::new();
        buf.put_slice(&self.header.pack());
        for v in self.attrs.iter() {
            buf.put_slice(&v.pack());
        }

        buf.freeze()
    }

    pub fn unpack(mut buf_bytes: Bytes) -> Result<Self, ParsePacketErr> {
        if buf_bytes.len() < HEADER_LEN {
            return Err(ParsePacketErr::BufSize(format!(
                "header buf len:{} < {}",
                buf_bytes.len(),
                HEADER_LEN
            )));
        }

        let header_buf = buf_bytes.split_to(HEADER_LEN);
        let header = Header::unpack(header_buf)?;
        let origin_header_len = header.msg_len;

        if header.msg_len as usize != buf_bytes.len() {
            return Err(ParsePacketErr::NotMatch(format!(
                "header len:{} != {}",
                header.msg_len,
                buf_bytes.len()
            )));
        }

        let mut attr_list = vec![];

        let mut max_attr = 32_usize;

        while buf_bytes.len() >= 4 {
            if max_attr == 0 {
                return Err(ParsePacketErr::TooManyAttrs);
            }

            let attr_len = u16::from_be_bytes([buf_bytes[2], buf_bytes[3]]);

            if buf_bytes.len() < attr_len as usize + 4 {
                return Err(ParsePacketErr::BufSize(format!(
                    "attr buf len:{} < {}",
                    buf_bytes.len(),
                    attr_len + 4
                )));
            }
            let attr_buf = buf_bytes.split_to(attr_len as usize + 4);
            let attr = RawAttr::unpack(attr_buf)?;
            attr_list.push(attr);

            max_attr -= 1;
        }

        let packet = Packet::new(header, attr_list);
        if packet.header.msg_len != origin_header_len {
            return Err(ParsePacketErr::NotMatch(format!(
                "packet data len:{} != packet msg len:{}",
                packet.header.msg_len, origin_header_len
            )));
        }

        Ok(packet)
    }

    pub fn validate(&self) -> Option<ValidateErr> {
        if let Some(v) = self.header.validate() {
            return Some(v);
        }

        for v in self.attrs.iter() {
            if AddressAttr::is_like_mapped_addr(v.attr_type) {
                if let Some(e) = validate_attr::<AddressAttr>(v) {
                    return Some(e);
                }
            }
            if v.attr_type == ATTR_ERROR_CODE {
                if let Some(e) = validate_attr::<ErrcodeAttr>(v) {
                    return Some(e);
                }
            }
            if v.attr_type == ATTR_RESPONSE_PORT {
                if let Some(e) = validate_attr::<ResponsePort>(v) {
                    return Some(e);
                }
            }
            if v.attr_type == ATTR_XOR_MAPPED_ADDRESS {
                let xor = match XorMappedAddress::from_base_attr(v.clone(), &self.header.trans_id) {
                    Ok(v) => v,
                    Err(e) => return Some(ValidateErr(format!("{:?}", e))),
                };

                if let Some(e) = xor.validate() {
                    return Some(e);
                }
            }
        }

        None
    }
}

fn validate_attr<T>(raw_attr: &RawAttr) -> Option<ValidateErr>
where
    T: AttrValidator + TryFrom<RawAttr>,
    <T as std::convert::TryFrom<attrs::RawAttr>>::Error: Debug,
{
    let attr: Result<T, _> = raw_attr.clone().try_into();
    match attr {
        Ok(v) => v.validate(),
        Err(e) => Some(ValidateErr(format!("{:?}", e))),
    }
}
