#[derive(Debug)]
pub enum Error {
    Parse(ParsePacketErr),

}


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