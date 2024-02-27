use std::future::Future;

use tonic::transport::Channel;

pub trait PluginClient {
    type Client: Clone;

    fn client(&self, channel: Channel) -> impl Future<Output = Self::Client> + Send;
}

pub trait PluginServer {
    type Server: Clone;

    fn server(&self) -> impl Future<Output = Self::Server> + Send;
}
