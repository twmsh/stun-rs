#![allow(clippy::len_without_is_empty)]

use bytes::{BufMut, Bytes, BytesMut};
use std::ops::Deref;
use crate::error::ParsePacketErr;

pub mod address_attr;
pub mod change_request;
pub mod errcode_attr;
pub mod padding_attr;
pub mod response_port;
pub mod xor_address;

#[derive(Debug, Clone)]
pub struct RawAttr {
    pub attr_type: u16,
    pub attr_len: u16,
    pub value: Bytes,
}

impl RawAttr {
    pub fn new(attr_type: u16, value: Bytes) -> Self {
        Self {
            attr_type,
            attr_len: value.len() as u16,
            value,
        }
    }

    pub fn len(&self) -> usize {
        self.attr_len as usize + 4
    }

    pub fn pack(&self) -> Bytes {
        let buf_len = self.attr_len + 4;
        let mut buf = BytesMut::with_capacity(buf_len as usize);

        buf.put_u16(self.attr_type);
        buf.put_u16(self.attr_len);
        buf.put_slice(&self.value);

        buf.freeze()
    }

    pub fn unpack(buf_bytes: Bytes) -> Result<Self, ParsePacketErr> {
        let buf = buf_bytes.deref();

        if buf.len() < 4 {
            return Err(ParsePacketErr::BufSize(
                format!("attr buf len:{}",buf.len())));
        }

        let mut index = 0_usize;
        let attr_type = u16::from_be_bytes([buf[index], buf[index + 1]]);

        index += 2;
        let attr_len = u16::from_be_bytes([buf[index], buf[index + 1]]);

        if buf.len() < (attr_len + 4) as usize {
            return Err(ParsePacketErr::BufSize(format!("attr buf len:{} < {}",
                                                       buf.len(),attr_len + 4)));
        }

        index += 2;
        let mut value = BytesMut::with_capacity(attr_len as usize);
        value.put_slice(&buf[index..index + attr_len as usize]);

        let value = value.freeze();

        Ok(Self {
            attr_type,
            attr_len,
            value,
        })
    }
}
