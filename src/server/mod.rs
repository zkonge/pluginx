pub mod config;
pub mod util;

use std::{convert::Infallible, env, fs, io, mem, process::exit};

use http::{Request, Response};
use tokio::net::{TcpListener, UnixListener};
use tokio_stream::wrappers::UnixListenerStream;
use tonic::{
    body::BoxBody,
    server::NamedService,
    transport::{
        server::{RoutesBuilder, Server as TonicServer, TcpIncoming},
        Body,
    },
};
use tonic_health::ServingStatus;
use tower_service::Service;

use self::config::ServerConfig;
use crate::{
    handshake::{HandshakeMessage, Network, Protocol},
    meta_plugin, PluginxError,
};

enum TransportProvider {
    Unix(UnixListener),
    Tcp(TcpListener),
}

pub struct Server {
    protocol_version: u32,
    transport_provider: TransportProvider,

    exit_signal: meta_plugin::ControllerExitSignal,
    stdio_handler: meta_plugin::StdioHandler,
    broker_handler: meta_plugin::BrokerHandler,

    routes_builder: RoutesBuilder,
}

impl Server {
    pub async fn new(
        ServerConfig {
            handshake_config: hc,
        }: ServerConfig,
    ) -> io::Result<Self> {
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

        let transport_provider = if cfg!(windows) {
            TransportProvider::Tcp(util::find_available_tcp_listener().await?)
        } else {
            TransportProvider::Unix(util::find_available_unix_socket_listener()?)
        };

        let mut rb = RoutesBuilder::default();

        let (mut rep, svc) = tonic_health::server::health_reporter();
        rep.set_service_status("plugin", ServingStatus::Serving)
            .await;
        rb.add_service(svc);

        let (svc, exit_signal) = meta_plugin::ControllerServer::new();
        rb.add_service(svc);

        let (svc, stdio_handler) = meta_plugin::StdioServer::new();
        rb.add_service(svc);

        let (svc, broker_handler) = meta_plugin::BrokerServer::new();
        rb.add_service(svc);

        Ok(Self {
            protocol_version: hc.protocol_version,
            transport_provider,

            exit_signal,
            stdio_handler,
            broker_handler,

            routes_builder: rb,
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

    pub async fn add_plugin<S>(&mut self, plugin: S) -> &mut Self
    where
        S: Service<Request<Body>, Response = Response<BoxBody>, Error = Infallible>
            + NamedService
            + Clone
            + Send
            + 'static,
        S::Future: Send + 'static,
        S::Error: Into<Box<dyn std::error::Error + Send + Sync>> + Send,
    {
        self.routes_builder.add_service(plugin);
        self
    }

    pub async fn run(mut self) -> Result<(), PluginxError> {
        let mut unix_socket_path = None; // for cleanup

        let network = match &self.transport_provider {
            TransportProvider::Unix(listener) => {
                let path = listener.local_addr()?.as_pathname().unwrap().to_owned();
                unix_socket_path = Some(path.clone());
                Network::Unix(path)
            }
            TransportProvider::Tcp(listener) => Network::Tcp(listener.local_addr()?),
        };

        let hs = HandshakeMessage {
            core_protocol: 1,
            app_protocol: self.protocol_version,
            network,
            protocol: Protocol::Grpc,
        };
        println!("{hs}");

        let routes = mem::take(&mut self.routes_builder).routes();
        let exit_signal = self.exit_signal();

        match self.transport_provider {
            TransportProvider::Unix(u) => {
                let incoming = UnixListenerStream::new(u);

                TonicServer::builder()
                    .add_routes(routes)
                    .serve_with_incoming_shutdown(incoming, exit_signal.wait())
                    .await?;

                if let Some(path) = unix_socket_path {
                    _ = fs::remove_file(path);
                }
            }
            TransportProvider::Tcp(t) => {
                let server = TonicServer::builder().add_routes(routes);
                let incoming =
                    TcpIncoming::from_listener(t, true, None).expect("local_addr must existed");

                server
                    .serve_with_incoming_shutdown(incoming, exit_signal.wait())
                    .await?;
            }
        }

        Ok(())
    }
}
