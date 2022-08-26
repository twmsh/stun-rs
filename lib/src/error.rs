#[derive(Debug)]
pub enum Error {
    Parse(ParsePacketErr),
    Validate(ValidateErr),
}
#[derive(Debug)]
pub struct ValidateErr(pub String);

#[derive(Debug)]
pub enum ParsePacketErr {
    // 长度或值不匹配
    NotMatch(String),

    // buf不够
    BufSize(String),

    //字段的值不合规
    BadValue(String),

    // 不是utf8字符串
    NotUtf8,

    // attribute过多
    TooManyAttrs,
}

pub trait AttrValidator {
    fn validate(&self) -> Option<ValidateErr>;
}
