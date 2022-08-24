use crate::attrs::RawAttr;
use crate::constants::*;
use crate::error::ParsePacketErr;
use bytes::{BufMut, BytesMut};
use std::ops::Deref;

#[derive(Debug, Clone)]
pub struct ChangeRequest {
    pub change_ip: bool,
    pub change_port: bool,
}

impl ChangeRequest {
    pub fn new(change_ip: bool, change_port: bool) -> Self {
        Self {
            change_ip,
            change_port,
        }
    }
}

impl From<ChangeRequest> for RawAttr {
    fn from(attr: ChangeRequest) -> Self {
        let mut flag: u32 = 0;
        if attr.change_ip {
            flag |= 0x04;
        }
        if attr.change_port {
            flag |= 0x02;
        }
        let mut bytes_buf = BytesMut::with_capacity(4);
        bytes_buf.put_u32(flag);
        let value = bytes_buf.freeze();
        RawAttr::new(ATTR_CHANGE_REQUEST, value)
    }
}

impl TryFrom<RawAttr> for ChangeRequest {
    type Error = ParsePacketErr;

    fn try_from(base_attr: RawAttr) -> Result<Self, Self::Error> {
        if base_attr.value.len() != 4 {
            return Err(ParsePacketErr::BufSize(format!(
                "change_request attr len:{} !=4",
                base_attr.value.len()
            )));
        }

        let value = base_attr.value.deref();
        let flag = value[4];

        let change_ip = flag & 0x04 == 0x04;
        let change_port = flag & 0x02 == 0x02;
        Ok(Self {
            change_ip,
            change_port,
        })
    }
}
