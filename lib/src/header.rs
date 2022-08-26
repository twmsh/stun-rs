#![allow(clippy::len_without_is_empty)]

use crate::constants::*;
use bytes::{BufMut, Bytes, BytesMut};

use crate::error::{ParsePacketErr, ValidateErr};
use std::ops::Deref;

pub type TransId = [u8; TRANS_ID_LEN];

// rfc 3489, 11.1
#[derive(Debug, Clone)]
pub struct Header {
    pub msg_type: u16,

    // 不包括header的20字节
    pub msg_len: u16,

    pub trans_id: TransId,
}

impl Header {
    pub fn new(msg_type: u16, msg_len: u16, trans_id: TransId) -> Self {
        Self {
            msg_type,
            msg_len,
            trans_id,
        }
    }

    pub fn len(&self) -> usize {
        HEADER_LEN
    }

    pub fn pack(&self) -> Bytes {
        let mut buf = BytesMut::with_capacity(HEADER_LEN);
        buf.put_u16(self.msg_type);
        buf.put_u16(self.msg_len);
        buf.put_slice(&self.trans_id);
        buf.freeze()
    }

    pub fn unpack(buf_bytes: Bytes) -> Result<Self, ParsePacketErr> {
        let buf = buf_bytes.deref();

        // 只检查长度，不检查有效性
        if buf.len() < HEADER_LEN {
            return Err(ParsePacketErr::BufSize(format!(
                "header buf len:{} < {}",
                buf.len(),
                HEADER_LEN
            )));
        }

        let mut index = 0_usize;
        let msg_type = u16::from_be_bytes([buf[index], buf[index + 1]]);

        index += 2;
        let msg_len = u16::from_be_bytes([buf[index], buf[index + 1]]);

        index += 2;
        let mut trans_id = [0_u8; TRANS_ID_LEN];
        trans_id.copy_from_slice(&buf[index..]);

        Ok(Self {
            msg_type,
            msg_len,
            trans_id,
        })
    }

    pub fn validate(&self) -> Option<ValidateErr> {
        // 检查 stun message type

        if self.msg_type == MESSAGE_TYPE_BIND_REQ
            || self.msg_type == MESSAGE_TYPE_BIND_RES
            || self.msg_type == MESSAGE_TYPE_BIND_ERR_RES
        {
            return None;
        }

        let err_msg = format!("not support message type: {}", self.msg_type);
        Some(ValidateErr(err_msg))
    }
}
