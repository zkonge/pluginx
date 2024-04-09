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
    select, time,
};
pub use tonic::transport::Channel;
use tonic::transport::Uri;
use tower::service_fn;

use self::config::ClientConfig;
use crate::{
    constant::{PLUGIN_MAX_PORT, PLUGIN_MIN_PORT},
    handshake::{HandshakeError, HandshakeMessage, Network},
    meta_plugin::{ControllerClient, StdioClient},
    plugin::PluginClient,
    PluginxError,
};

pub struct ClientBuilder {
    handshake: HandshakeMessage,
    plugin_host: Child,
    channel: Channel,
    plugins: HashMap<TypeId, Box<dyn Any>>,

    controller: ControllerClient,
    stdio: StdioClient,
}

impl ClientBuilder {
    pub async fn new(mut config: ClientConfig) -> Result<Self, PluginxError> {
        // 1. build plugin env
        let port_range = config.port_range.clone().unwrap_or(10000..=25000);
        let (magic_key, magic_value) = (
            config.handshake_config.magic_cookie_key.as_ref(),
            config.handshake_config.magic_cookie_value.as_ref(),
        );

        // 2. spawn plugin process
        let mut plugin_process = config
            .cmd
            .envs([
                (magic_key, magic_value),
                (PLUGIN_MIN_PORT, &port_range.start().to_string()),
                (PLUGIN_MAX_PORT, &port_range.end().to_string()),
                // TODO: unix socket dir
            ])
            .stderr(Stdio::piped())
            .stdout(Stdio::piped())
            .kill_on_drop(true)
            .spawn()?;

        // 3. wait for handshake
        let mut reader = BufReader::new(plugin_process.stdout.as_mut().unwrap());
        let mut line = String::new();
        select! {
            _ = time::sleep(config.startup_timeout) => {
                return Err(PluginxError::HandshakeError(HandshakeError::StarupTimeout));
            }
            _ = reader.read_line(&mut line) => {}
        }
        let handshake = HandshakeMessage::parse(&line).unwrap();

        // 4. connect with gRPC
        let channel = match &handshake.network {
            Network::Tcp(addr) => {
                let uri = Uri::builder()
                    .scheme("http")
                    .authority(addr.to_string())
                    .build()
                    .map_err(|_| HandshakeError::InvalidNetwork)?;
                Channel::builder(uri).connect().await?
            }
            Network::Unix(path) => {
                let path = path.clone();
                Channel::from_static("http://[::1]")
                    .connect_with_connector(service_fn(move |_: Uri| {
                        UnixStream::connect(path.clone())
                    }))
                    .await?
            }
        };

        // 5. load builtin plugins
        let controller = ControllerClient::new(channel.clone());
        let stdio = StdioClient::new(channel.clone());

        Ok(Self {
            handshake,
            plugin_host: plugin_process,
            channel,
            plugins: HashMap::new(),

            controller,
            stdio,
        })
    }

    pub async fn add_plugin<P: PluginClient + 'static>(mut self, plugin: P) -> Self {
        let plugin = plugin.client(self.channel.clone()).await;
        self.plugins
            .insert(TypeId::of::<P::Client>(), Box::new(plugin));
        self
    }

    pub fn build(self) -> Client {
        Client {
            handshake: self.handshake,
            plugin_host: self.plugin_host,
            plugins: self.plugins,

            controller: self.controller,
            stdio: self.stdio,
        }
    }
}

pub struct Client {
    handshake: HandshakeMessage,
    #[allow(unused)]
    plugin_host: Child,
    plugins: HashMap<TypeId, Box<dyn Any>>,

    controller: ControllerClient,
    #[allow(unused)]
    stdio: StdioClient,
}

impl Client {
    pub fn dispense<P: PluginClient + 'static>(&self) -> Option<P::Client> {
        let id = TypeId::of::<P::Client>();
        self.plugins
            .get(&id)
            .and_then(|p| p.downcast_ref::<P::Client>())
            .cloned()
    }

    pub async fn shutdown(mut self) {
        _ = self.controller.shutdown().await;
    }
}

impl Drop for Client {
    fn drop(&mut self) {
        // TODO: shutdown in sync context
        match &self.handshake.network {
            Network::Tcp(_) => {}
            Network::Unix(path) => {
                let _ = std::fs::remove_file(path);
            }
        }
    }
}
