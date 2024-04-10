use std::borrow::Cow;

use pluginx::{handshake::HandshakeConfig, plugin::PluginClient};
use tonic::transport::Channel;

tonic::include_proto!("proto");

pub const HANDSHAKE_CONFIG: HandshakeConfig = HandshakeConfig {
    protocol_version: 1,
    magic_cookie_key: Cow::Borrowed("BASIC_PLUGIN"),
    magic_cookie_value: Cow::Borrowed("hello"),
};

#[derive(Debug)]
pub struct KvPlugin;

impl PluginClient for KvPlugin {
    type Client = kv_client::KvClient<Channel>;

    async fn client(&self, channel: Channel) -> Self::Client {
        kv_client::KvClient::new(channel)
    }
}
