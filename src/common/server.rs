use std::{convert::Infallible, fs, future::Future, mem, ops::RangeInclusive, path::Path};

use http::{Request, Response};
use tokio::net::{TcpListener, UnixListener};
use tokio_stream::wrappers::UnixListenerStream;
use tonic::{
    body::BoxBody,
    server::NamedService,
    service::RoutesBuilder,
    transport::server::{Server as TonicServer, TcpIncoming},
};
use tower::Service;

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
    transport: Transport,
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

        Ok(Self {
            transport,
            routes_builder: RoutesBuilder::default(),
        })
    }

    pub(crate) fn network(&self) -> Result<Network, PluginxError> {
        let n = match &self.transport {
            Transport::Unix(listener) => {
                Network::Unix(listener.local_addr()?.as_pathname().unwrap().to_owned())
            }
            Transport::Tcp(listener) => Network::Tcp(listener.local_addr()?),
        };

        Ok(n)
    }

    #[inline]
    pub(crate) fn add_service<S>(&mut self, service: S) -> &mut Self
    where
        S: Service<Request<BoxBody>, Response = Response<BoxBody>, Error = Infallible>
            + NamedService
            + Clone
            + Send
            + 'static,
        S::Future: Send + 'static,
        S::Error: Into<Box<dyn std::error::Error + Send + Sync>> + Send,
    {
        self.routes_builder.add_service(service);
        self
    }

    pub(crate) async fn run(
        mut self,
        exiter: impl Future<Output = ()>,
    ) -> Result<(), PluginxError> {
        let unix_socket_path = match self.network()? {
            Network::Unix(p) => Some(p),
            _ => None,
        };

        let routes = mem::take(&mut self.routes_builder).routes();

        match self.transport {
            Transport::Unix(u) => {
                let incoming = UnixListenerStream::new(u);

                TonicServer::builder()
                    .add_routes(routes)
                    .serve_with_incoming_shutdown(incoming, exiter)
                    .await?;

                if let Some(path) = unix_socket_path {
                    _ = fs::remove_file(path);
                }
            }
            Transport::Tcp(t) => {
                let server = TonicServer::builder().add_routes(routes);
                let incoming =
                    TcpIncoming::from_listener(t, true, None).expect("local_addr must existed");

                server
                    .serve_with_incoming_shutdown(incoming, exiter)
                    .await?;
            }
        }

        Ok(())
    }
}
