// 0x2112A442
pub const MAGIC_COOKIE: [u8; 4] = [0x21, 0x12, 0xA4, 0x42];

pub const TRANS_ID_LEN: usize = 16;
pub const HEADER_LEN: usize = 20;

pub const MESSAGE_TYPE_BIND_REQ: u16 = 0x0001;
pub const MESSAGE_TYPE_BIND_RES: u16 = 0x0101;
pub const MESSAGE_TYPE_BIND_ERR_RES: u16 = 0x0111;

pub const ATTR_FAMILY_IPV4: u8 = 0x01;
pub const ATTR_FAMILY_IPV6: u8 = 0x02;

pub const ATTR_MAPPED_ADDRESS: u16 = 0x0001;
pub const ATTR_CHANGE_REQUEST: u16 = 0x0003;
pub const ATTR_SOURCE_ADDRESS: u16 = 0x0004;
pub const ATTR_CHANGED_ADDRESS: u16 = 0x0005;
pub const ATTR_ERROR_CODE: u16 = 0x0009;
pub const ATTR_PADDING: u16 = 0x0026;
pub const ATTR_RESPONSE_PORT: u16 = 0x0027;

pub const ATTR_XOR_MAPPED_ADDRESS: u16 = 0x8020;
pub const ATTR_RESPONSE_ORIGIN: u16 = 0x802b;
pub const ATTR_OTHER_ADDRESS: u16 = 0x802c;

