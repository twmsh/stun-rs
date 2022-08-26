use crate::attrs::RawAttr;
use crate::constants::*;
use crate::error::{AttrValidator, ParsePacketErr, ValidateErr};
use bytes::{BufMut, BytesMut};
use std::ops::Deref;

#[derive(Debug, Clone)]
pub struct ResponsePort {
    pub port: u16,
}

impl ResponsePort {
    pub fn new(port: u16) -> Self {
        Self { port }
    }
}

impl From<ResponsePort> for RawAttr {
    fn from(attr: ResponsePort) -> Self {
        let mut bytes_buf = BytesMut::with_capacity(4);
        bytes_buf.put_u16(attr.port);
        bytes_buf.put_u16(0);

        let value = bytes_buf.freeze();
        RawAttr::new(ATTR_RESPONSE_PORT, value)
    }
}

impl TryFrom<RawAttr> for ResponsePort {
    type Error = ParsePacketErr;

    fn try_from(base_attr: RawAttr) -> Result<Self, Self::Error> {
        if base_attr.value.len() != 4 {
            return Err(ParsePacketErr::BufSize(format!(
                "response_port attr buf len:{} != 4",
                base_attr.value.len()
            )));
        }

        let value = base_attr.value.deref();
        let port = u16::from_be_bytes([value[0], value[1]]);

        Ok(Self { port })
    }
}

impl AttrValidator for ResponsePort {
    fn validate(&self) -> Option<ValidateErr> {
        // 检查 port

        let port = self.port;
        if port > 0 && port < 65535 {
            return None;
        }

        let err_msg = format!("wrong port: {}", self.port);
        Some(ValidateErr(err_msg))
    }
}
