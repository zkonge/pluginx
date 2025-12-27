use std::{convert::Infallible, fs, mem, ops::RangeInclusive, path::Path};

use http::{Request, Response};
use tokio::net::{TcpListener, UnixListener};
use tokio_stream::wrappers::UnixListenerStream;
use tonic::{
    body::Body,
    server::NamedService,
    service::RoutesBuilder,
    transport::server::{Server as TonicServer, TcpIncoming},
};
use tower_service::Service;

use super::utils;
use crate::{handshake::Network, PluginxError};

pub(crate) enum TransportConfig {
    Unix {
        prefix: Box<str>,
        dir: Option<Box<Path>>,
    },
    Tcp {
        port_range: RangeInclusive<u16>,
    },
}

pub(crate) struct ServerConfig {
    pub transport_config: TransportConfig,
}

pub(crate) enum Transport {
    Unix(UnixListener),
    Tcp(TcpListener),
}

pub(crate) struct Server {
    // the Option makes Drop trait available while we use moving self in run()
    transport: Option<Transport>,
    network: Network,
    routes_builder: RoutesBuilder,
}

impl Server {
    pub(crate) async fn new(config: ServerConfig) -> Result<Self, PluginxError> {
        let transport = match config.transport_config {
            TransportConfig::Unix { prefix, dir } => Transport::Unix(
                utils::find_available_unix_socket_listener(&prefix, dir.as_deref())?,
            ),
            TransportConfig::Tcp { port_range } => {
                Transport::Tcp(utils::find_available_tcp_listener(port_range)?)
            }
        };

        let network = match &transport {
            Transport::Unix(listener) => Network::Unix(
                listener
                    .local_addr()?
                    .as_pathname()
                    .expect("uses a named UDS")
                    .to_owned(),
            ),
            Transport::Tcp(listener) => Network::Tcp(listener.local_addr()?),
        };

        Ok(Self {
            transport: Some(transport),
            network,
            routes_builder: RoutesBuilder::default(),
        })
    }

    pub(crate) fn network(&self) -> &Network {
        &self.network
    }

    #[inline]
    pub(crate) fn add_service<S>(&mut self, service: S) -> &mut Self
    where
        S: Service<Request<Body>, Response = Response<Body>, Error = Infallible>
            + NamedService
            + Clone
            + Send
            + Sync
            + 'static,
        S::Future: Send + 'static,
        S::Error: Into<Box<dyn std::error::Error + Send + Sync>> + Send,
    {
        self.routes_builder.add_service(service);
        self
    }

    pub(crate) async fn run(mut self) -> Result<(), PluginxError> {
        let routes = mem::take(&mut self.routes_builder).routes();

        match self.transport.take().expect("transport is always Some") {
            Transport::Unix(u) => {
                TonicServer::builder()
                    .add_routes(routes)
                    .serve_with_incoming(UnixListenerStream::new(u))
                    .await?
            }
            Transport::Tcp(t) => {
                TonicServer::builder()
                    .add_routes(routes)
                    .serve_with_incoming(TcpIncoming::from(t))
                    .await?
            }
        }

        Ok(())
    }
}

impl Drop for Server {
    fn drop(&mut self) {
        if let Network::Unix(path) = &self.network {
            _ = fs::remove_file(path);
        }
    }
}
