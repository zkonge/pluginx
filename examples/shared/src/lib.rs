use std::borrow::Cow;
use std::future::Future;

use pluginx::plugin::PluginClient;
use tonic::transport::Channel;

tonic::include_proto!("proto");

pub const HANDSHAKE_CONFIG: pluginx::HandshakeConfig = pluginx::HandshakeConfig {
    protocol_version: 1,
    magic_cookie_key: Cow::Borrowed("BASIC_PLUGIN"),
    magic_cookie_value: Cow::Borrowed("hello"),
};

#[derive(Debug)]
pub struct KvPlugin;

impl PluginClient for KvPlugin {
    type Client = kv_client::KvClient<Channel>;

    fn client(&self, channel: Channel) -> impl Future<Output = Self::Client> + Send {
        async { kv_client::KvClient::new(channel) }
    }
}
