use std::borrow::Cow;

tonic::include_proto!("example");

pub const HANDSHAKE_CONFIG: pluginx::HandshakeConfig = pluginx::HandshakeConfig {
    protocol_version: 1,
    magic_cookie_key: Cow::Borrowed("key"),
    magic_cookie_value: Cow::Borrowed("value"),
};
