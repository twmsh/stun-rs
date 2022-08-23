use crate::attrs::RawAttr;
use crate::constants::HEADER_LEN;
use crate::header::Header;
use bytes::{BufMut, Bytes, BytesMut};
use crate::error::ParsePacketErr;

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
            return Err(ParsePacketErr::BufSize(format!("header buf len:{} < {}",
                                                       buf_bytes.len(), HEADER_LEN)));
        }

        let header_buf = buf_bytes.split_to(HEADER_LEN);
        let header = Header::unpack(header_buf)?;
        let origin_header_len = header.msg_len;

        if header.msg_len as usize != buf_bytes.len() {
            return Err(ParsePacketErr::NotMatch(
                format!("header len:{} != {}", header.msg_len, buf_bytes.len()))
            );
        }

        let mut attr_list = vec![];

        let mut max_attr = 32_usize;

        while buf_bytes.len() >= 4 {
            if max_attr == 0 {
                return Err(ParsePacketErr::TooManyAttrs);
            }

            let attr_len = u16::from_be_bytes([buf_bytes[2], buf_bytes[3]]);
            println!("attr_len: {}", attr_len);

            if buf_bytes.len() < attr_len as usize + 4 {
                return Err(ParsePacketErr::BufSize(
                    format!("attr buf len:{} < {}",
                            buf_bytes.len(),
                            attr_len + 4)));
            }
            let attr_buf = buf_bytes.split_to(attr_len as usize + 4);
            let attr = RawAttr::unpack(attr_buf)?;
            attr_list.push(attr);

            max_attr -= 1;
        }

        let packet = Packet::new(header, attr_list);
        if packet.header.msg_len != origin_header_len {


            return Err(ParsePacketErr::NotMatch(
                format!("packet data len:{} != packet msg len:{}",
                        packet.header.msg_len,
                        origin_header_len)));

        }

        Ok(packet)
    }
}
