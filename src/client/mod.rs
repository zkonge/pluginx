pub mod config;

use std::{
    any::{Any, TypeId},
    process::Stdio,
};

use hashbrown::HashMap;
use tokio::{
    io::{AsyncBufReadExt, BufReader},
    net::UnixStream,
    process::Child,
};
use tonic::transport::{Channel, Endpoint, Uri};
use tower::service_fn;

use crate::{
    constant::{PLUGIN_MAX_PORT, PLUGIN_MIN_PORT},
    handshake::HandshakeMessage,
    plugin::PluginClient,
    Network,
};

use self::config::ClientConfig;

pub struct ClientBuilder {
    handshake: HandshakeMessage,
    proc: Child,
    channel: Channel,
    plugins: HashMap<TypeId, Box<dyn Any>>,
}

impl ClientBuilder {
    pub async fn new(mut config: ClientConfig) -> Self {
        // 1. start plugin
        let port_range = config.port_range.clone().unwrap_or(10000..=25000);
        let (magic_key, magic_value) = (
            config.handshake_config.magic_cookie_key.as_ref(),
            config.handshake_config.magic_cookie_value.as_ref(),
        );
        let mut proc = config
            .cmd
            .envs([
                (magic_key, magic_value),
                (PLUGIN_MIN_PORT, &port_range.start().to_string()),
                (PLUGIN_MAX_PORT, &port_range.end().to_string()),
            ])
            .stderr(Stdio::inherit())
            .stdout(Stdio::piped())
            .kill_on_drop(true)
            .spawn()
            .unwrap();

        let mut reader = BufReader::new(proc.stdout.as_mut().unwrap());
        let mut line = String::new();
        reader.read_line(&mut line).await.unwrap();

        let handshake = HandshakeMessage::parse(&line).unwrap();
        let channel = match &handshake.network {
            Network::Tcp(addr) => Channel::builder(
                Uri::builder()
                    .scheme("http")
                    .authority(addr.to_string())
                    .build()
                    .unwrap(),
            )
            .connect()
            .await
            .unwrap(),
            Network::Unix(path) => {
                let path = path.clone();
                Channel::from_static("http://[::1]")
                    .connect_with_connector(service_fn(move |_: Uri| {
                        UnixStream::connect(path.clone())
                    }))
                    .await
                    .unwrap()
            }
        };

        Self {
            handshake,
            proc,
            channel,
            plugins: HashMap::new(),
        }
    }

    pub async fn add_plugin<P: PluginClient + 'static>(&mut self, plugin: P) -> &mut Self {
        let plugin = plugin.client(self.channel.clone()).await;
        self.plugins
            .insert(TypeId::of::<P::Client>(), Box::new(plugin));
        self
    }

    pub fn build(self) -> Client {
        Client {
            handshake:self.handshake,
            proc: self.proc,
            plugins: self.plugins,
        }
    }
}

pub struct Client {
    handshake:HandshakeMessage,
    proc: Child,
    plugins: HashMap<TypeId, Box<dyn Any>>,
}

impl Client {
    pub fn dispense<P: PluginClient + 'static>(&self) -> Option<P::Client> {
        let id = TypeId::of::<P::Client>();
        self.plugins
            .get(&id)
            .and_then(|p| p.downcast_ref::<P::Client>())
            .cloned()
    }
}

impl Drop for Client {
    fn drop(&mut self) {
        let _ = self.proc.kill();
        match &self.handshake.network {
            Network::Tcp(_) => {}
            Network::Unix(path) => {
                let _ = std::fs::remove_file(path);
            }
        }
    }
}
