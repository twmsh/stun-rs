use std::ops::Deref;

use crate::attrs::RawAttr;
use crate::constants::ATTR_ERROR_CODE;
use crate::error::{AttrValidator, ParsePacketErr, ValidateErr};
use crate::util;
use bytes::{BufMut, BytesMut};

// class:  3 bit        1-6
// number: 8 bit        0-99

#[derive(Debug, Clone)]
pub struct ErrcodeAttr {
    pub code: u16,
    pub msg: String,
}

impl ErrcodeAttr {
    pub fn new(code: u16, msg: &str) -> Self {
        Self {
            code,
            msg: msg.to_string(),
        }
    }
}

impl From<ErrcodeAttr> for RawAttr {
    fn from(attr: ErrcodeAttr) -> Self {
        let mut bytes_buf = BytesMut::with_capacity(4);
        bytes_buf.put_u16(0);
        bytes_buf.put_u16(util::pack_error_code(attr.code));
        bytes_buf.put_slice(util::pack_error_reason(&attr.msg).as_bytes());

        let value = bytes_buf.freeze();
        RawAttr::new(ATTR_ERROR_CODE, value)
    }
}

impl TryFrom<RawAttr> for ErrcodeAttr {
    type Error = ParsePacketErr;

    fn try_from(base_attr: RawAttr) -> Result<Self, Self::Error> {
        if base_attr.value.len() < 4 {
            return Err(ParsePacketErr::BufSize(format!(
                "err_code attr buf len:{} < 4",
                base_attr.value.len()
            )));
        }

        // 从 value中解析
        let mut index = 0_usize;
        let value = base_attr.value.deref();

        index += 2;
        let code = u16::from_be_bytes([value[index], value[index + 1]]);
        let code = util::unpack_error_code(code);

        index += 2;
        let msg = &base_attr.value[index..];
        let msg = match String::from_utf8(msg.into()) {
            Ok(v) => v.trim().to_string(),
            Err(_e) => {
                return Err(ParsePacketErr::NotUtf8);
            }
        };

        Ok(Self { code, msg })
    }
}
impl AttrValidator for ErrcodeAttr {
    fn validate(&self) -> Option<ValidateErr> {
        if self.code > 100 && self.code < 700 {
            return None;
        }

        let err_msg = format!("wrong code: {}", self.code);
        Some(ValidateErr(err_msg))
    }
}
