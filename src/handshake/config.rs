use std::borrow::Cow;


pub struct HandshakeConfig<'a> {
    pub protocol_version: u32,
    pub magic_cookie_key: Cow<'a, str>,
    pub magic_cookie_value: Cow<'a, str>,
}
