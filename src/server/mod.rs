pub mod config;
pub mod utils;

use std::{env, process::exit};

use http::Request;
use tonic::body::Body;
use tower_service::Service;

use self::config::ServerConfig;
use crate::{
    common::server::{Server as InnerServer, ServerConfig as InnerServerConfig},
    handshake::{HandshakeMessage, Protocol},
    meta_plugin,
    plugin::PluginServer,
    PluginxError, StdError,
};

pub struct Server {
    protocol_version: u32,

    exit_signal: meta_plugin::ControllerExitSignal,
    stdio_handler: meta_plugin::StdioHandler,
    broker_handler: meta_plugin::BrokerHandler,

    server: InnerServer,
}

impl Server {
    pub async fn new(
        ServerConfig {
            handshake_config: hc,
        }: ServerConfig,
    ) -> Result<Self, PluginxError> {
        if hc.magic_cookie_key.is_empty() || hc.magic_cookie_value.is_empty() {
            eprintln!(
                r"Misconfigured ServeConfig given to serve this plugin: no magic cookie
key or value was set. Please notify the plugin author and report
this as a bug."
            );
            exit(-1);
        }

        if env::var(hc.magic_cookie_key.as_ref()).as_deref() != Ok(hc.magic_cookie_value.as_ref()) {
            eprintln!(
                r"This binary is a plugin. These are not meant to be executed directly.
Please execute the program that consumes these plugins, which will
load any plugins automatically."
            );
            exit(-1);
        }

        let transport_config = if cfg!(windows) {
            utils::tcp_transport_config_from_env()?
        } else {
            utils::unix_transport_config_from_env()?
        };

        let mut server = InnerServer::new(InnerServerConfig { transport_config }).await?;

        #[cfg(feature = "health")]
        {
            use tonic_health::ServingStatus;
            let (reporter, svc) = tonic_health::server::health_reporter();
            reporter
                .set_service_status("plugin", ServingStatus::Serving)
                .await;
            server.add_service(svc);
        }

        let (svc, exit_signal) = meta_plugin::ControllerServer::new();
        server.add_service(svc);

        let (svc, stdio_handler) = meta_plugin::StdioServer::new();
        server.add_service(svc);

        let (svc, broker_handler) = meta_plugin::BrokerServer::new();
        server.add_service(svc);

        Ok(Self {
            protocol_version: hc.protocol_version,

            exit_signal,
            stdio_handler,
            broker_handler,

            server,
        })
    }

    pub fn exit_signal(&self) -> meta_plugin::ControllerExitSignal {
        self.exit_signal.clone()
    }

    pub fn stdio_handler(&self) -> meta_plugin::StdioHandler {
        self.stdio_handler.clone()
    }

    pub fn broker_handler(&self) -> meta_plugin::BrokerHandler {
        self.broker_handler.clone()
    }

    #[inline]
    pub async fn add_plugin<P: PluginServer + 'static>(&mut self, plugin: P) -> &mut Self
    where
        <P::Server as Service<Request<Body>>>::Future: Send + 'static,
        <P::Server as Service<Request<Body>>>::Error: Into<StdError> + Send,
    {
        let plugin = plugin.server().await;
        self.server.add_service(plugin);
        self
    }

    pub async fn run(self) -> Result<(), PluginxError> {
        let network = self.server.network().clone();

        let hs = HandshakeMessage {
            core_protocol: 1,
            app_protocol: self.protocol_version,
            network,
            protocol: Protocol::Grpc,
        };
        println!("{hs}");

        let exiter = self.exit_signal();

        self.server.run(exiter.wait()).await
    }
}
