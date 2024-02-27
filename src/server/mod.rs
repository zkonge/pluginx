pub mod config;
pub mod util;

use std::{convert::Infallible, env, io, mem, path::PathBuf, process::exit};

use http::{Request, Response};
use tokio::net::UnixListener;
use tokio_stream::wrappers::UnixListenerStream;
use tonic::{
    body::BoxBody,
    server::NamedService,
    transport::{
        server::{RoutesBuilder, Server as TonicServer},
        Body,
    },
};
use tower_service::Service;

use self::config::ServerConfig;
use crate::{constant::PLUGIN_UNIX_SOCKET_DIR, HandshakeMessage, Network, Protocol};

pub struct Server {
    protocol_version: u32,
    routes: RoutesBuilder,
    socket_path: PathBuf,
}

impl Server {
    pub fn new(
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

        let uds_socket_dir = std::env::var(PLUGIN_UNIX_SOCKET_DIR);

        let avaiable_path = {
            let mut b = tempfile::Builder::new();
            let b = b.prefix("plugin-");
            let f = if let Ok(dir) = uds_socket_dir {
                b.tempfile_in(dir)
            } else {
                b.tempfile()
            };
            f?.path().to_owned()
        };

        Ok(Self {
            protocol_version: hc.protocol_version,
            socket_path: avaiable_path,
            routes: RoutesBuilder::default(),
        })
    }

    pub async fn add_plugin<S>(&mut self, plugin: S)
    where
        S: Service<Request<Body>, Response = Response<BoxBody>, Error = Infallible>
            + NamedService
            + Clone
            + Send
            + 'static,
        S::Future: Send + 'static,
        S::Error: Into<Box<dyn std::error::Error + Send + Sync>> + Send,
    {
        self.routes.add_service(plugin);
    }

    pub async fn run(mut self) -> io::Result<()> {
        let uds = UnixListenerStream::new(UnixListener::bind(&self.socket_path)?);

        let server = TonicServer::builder().add_routes(mem::take(&mut self.routes).routes());
        let server = tokio::spawn(server.serve_with_incoming(uds));

        let handshake_message = HandshakeMessage {
            core_protocol: 1,
            app_protocol: self.protocol_version,
            network: Network::Unix(self.socket_path.clone()),
            protocol: Protocol::Grpc,
        };

        println!("{handshake_message}");

        // TODO: no unwrap
        Ok(server.await.unwrap().unwrap())
    }
}

impl Drop for Server {
    fn drop(&mut self) {
        let _ = std::fs::remove_file(&self.socket_path);
    }
}
