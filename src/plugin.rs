use std::{convert::Infallible, future::Future};

use http::{Request, Response};
use tonic::{
    body::BoxBody,
    server::NamedService,
    transport::{Body, Channel},
};
use tower_service::Service;

pub trait PluginClient {
    type Client: Clone;

    fn client(&self, channel: Channel) -> impl Future<Output = Self::Client> + Send;
}

pub trait PluginServer {
    // type Fut: Send + 'static;
    // type Err: Into<Box<dyn std::error::Error + Send + Sync>> + Send;
    type Server: Service<Request<Body>, Response = Response<BoxBody>, Error = Infallible>
        + NamedService
        + Clone
        + Send
        + 'static;

    fn server(&self) -> impl Future<Output = Self::Server> + Send;
}
