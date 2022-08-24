use crate::attrs::RawAttr;
use crate::constants::ATTR_PADDING;
use crate::error::ParsePacketErr;
use bytes::Bytes;

#[derive(Debug, Clone)]
pub struct PaddingAttr {
    // 4 * 2 * n , 4字节的偶数倍数
    pub data: Bytes,
}

impl PaddingAttr {
    // data的长度需要是4的偶数倍
    pub fn new(data: Bytes) -> Self {
        Self { data }
    }
}

impl From<PaddingAttr> for RawAttr {
    fn from(attr: PaddingAttr) -> Self {
        RawAttr::new(ATTR_PADDING, attr.data)
    }
}

impl TryFrom<RawAttr> for PaddingAttr {
    type Error = ParsePacketErr;

    fn try_from(base_attr: RawAttr) -> Result<Self, Self::Error> {
        if base_attr.value.len() % 8 != 0 {
            return Err(ParsePacketErr::BadValue(format!(
                "padding attr buf len:{}",
                base_attr.value.len()
            )));
        }

        Ok(Self {
            data: base_attr.value,
        })
    }
}
