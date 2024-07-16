use std::{convert::Infallible, future::Future};

use http::{Request, Response};
use tonic::{body::BoxBody, server::NamedService, transport::Channel};
use tower::Service;

pub trait PluginClient {
    type Client: Clone + Send + Sync;

    fn client(&self, channel: Channel) -> impl Future<Output = Self::Client> + Send;
}

pub trait PluginServer {
    type Server: Service<Request<BoxBody>, Response = Response<BoxBody>, Error = Infallible>
        + NamedService
        + Clone
        + Send
        + 'static;

    fn server(&self) -> impl Future<Output = Self::Server> + Send;
}

/// for those service doesn't need broker
impl<T> PluginServer for T
where
    T: Service<Request<BoxBody>, Response = Response<BoxBody>, Error = Infallible>
        + NamedService
        + Clone
        + Send
        + Sync
        + 'static,
    T::Future: Send + 'static,
    T::Error: Into<Box<dyn std::error::Error + Send + Sync>> + Send,
{
    type Server = T;

    #[inline]
    async fn server(&self) -> Self::Server {
        self.clone()
    }
}
