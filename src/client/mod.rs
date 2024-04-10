pub mod config;

use std::process::Stdio;

use futures::{Stream, StreamExt};
use tokio::{
    io::AsyncReadExt,
    process::{Child, ChildStderr, ChildStdout},
    select, time,
};
pub use tonic::transport::Channel;

use self::config::ClientConfig;
use crate::{
    common::client::Client as InnerClient,
    constant::{PLUGIN_MAX_PORT, PLUGIN_MIN_PORT},
    handshake::{HandshakeError, HandshakeMessage, Network},
    meta_plugin::{ControllerClient, StdioClient},
    plugin::PluginClient,
    proto::stdio_data,
    PluginxError,
};

pub struct ClientBuilder {
    handshake: HandshakeMessage,
    plugin_host: Child,

    controller: ControllerClient,
    stdio: StdioClient,

    client: InnerClient,
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
        let mut plugin_host = config
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
        let stdout = plugin_host
            .stdout
            .as_mut()
            .expect("stdout is pipe, must success");
        let mut buf = Vec::new();
        select! {
            _ = time::sleep(config.startup_timeout) => {
                return Err(HandshakeError::StarupTimeout.into());
            }
            _ = stdout.read_buf(&mut buf) => {}
        }
        let stdout = String::from_utf8_lossy(&buf);
        let handshake = HandshakeMessage::parse(stdout.trim()).map_err(|error| {
            PluginxError::HandshakeError {
                error,
                message: stdout.to_string(),
            }
        })?;

        // 4. connect with gRPC
        let client = InnerClient::new(&handshake.network).await?;

        // 5. load builtin plugins
        let controller = ControllerClient::new(client.channel().clone());
        let stdio = StdioClient::new(client.channel().clone());

        Ok(Self {
            handshake,
            plugin_host,

            controller,
            stdio,

            client,
        })
    }

    pub async fn add_plugin<P: PluginClient + 'static>(&mut self, plugin: P) -> &mut Self {
        let plugin = plugin.client(self.client.channel().clone()).await;
        self.client.add_service(plugin);
        self
    }

    pub fn build(self) -> Client {
        Client {
            handshake: self.handshake,
            plugin_host: self.plugin_host,

            controller: self.controller,
            stdio: Some(self.stdio),

            client: self.client,
        }
    }
}

#[derive(Default, Debug)]
pub enum StdioData {
    #[default]
    Invalid,
    Stdout(Vec<u8>),
    Stderr(Vec<u8>),
}

pub struct Client {
    handshake: HandshakeMessage,
    plugin_host: Child,

    controller: ControllerClient,
    stdio: Option<StdioClient>,

    client: InnerClient,
}

impl Client {
    pub fn dispense<P: PluginClient + 'static>(&self) -> Option<P::Client> {
        self.client.dispense::<P::Client>()
    }

    /// stdout/stderr data sent from plugin host, it can be only called once, or it will return [`None`]
    pub async fn stdio(&mut self) -> Option<impl Stream<Item = StdioData>> {
        let s = self.stdio.take()?.read().await.ok()?;

        Some(s.map(|x| x.unwrap_or_default()).map(|x| match x.channel() {
            stdio_data::Channel::Invalid => StdioData::Invalid,
            stdio_data::Channel::Stdout => StdioData::Stdout(x.data),
            stdio_data::Channel::Stderr => StdioData::Stderr(x.data),
        }))
    }

    /// raw stdout from process instead of RPC, can only be called once.
    pub fn raw_stdout(&mut self) -> Option<ChildStdout> {
        self.plugin_host.stdout.take()
    }

    /// raw stderr from process instead of RPC, can only be called once.
    pub fn raw_stderr(&mut self) -> Option<ChildStderr> {
        self.plugin_host.stderr.take()
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
