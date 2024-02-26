use std::{convert::Infallible, future::Future};

use futures::stream::{self, Empty};
use http_body::Body;
use tonic::{
    body::BoxBody, server::NamedService, transport::Channel, Request, Response, Status, Streaming,
};
use tower_service::Service;

use crate::proto::{
    grpc_broker_client::GrpcBrokerClient,
    grpc_broker_server::{GrpcBroker, GrpcBrokerServer},
    ConnInfo,
};

// pub trait Plugin {
//     type Client<C>;
//     type Server;

//     fn client(&self, channel: Channel) -> impl Future<Output = Self::Client<Channel>> + Send;
//     fn server(&self) -> impl Future<Output = Self::Server> + Send;
// }
#[tonic::async_trait]
pub trait Plugin {
    type Client;
    type Server;

    async fn client(&self, channel: Channel) -> Self::Client;
    async fn server(&self) -> Self::Server;
}

#[derive(Debug)]
struct ServerT;

#[tonic::async_trait]
impl GrpcBroker for ServerT {
    type StartStreamStream = Empty<Result<ConnInfo, Status>>;
    async fn start_stream(
        &self,
        _: Request<Streaming<ConnInfo>>,
    ) -> Result<Response<Self::StartStreamStream>, Status> {
        Ok(Response::new(stream::empty()))
    }
}

struct Test1;

// impl Plugin for Test1 {
//     // there is no way to force the following types to be tonic types :(
//     type Client<C> = GrpcBrokerClient<Channel>;
//     type Server = GrpcBrokerServer<ServerT>;

//     async fn client(&self, channel: Channel) -> Self::Client<Channel> {
//         GrpcBrokerClient::new(channel)
//     }

//     async fn server(&self) -> Self::Server {
//         GrpcBrokerServer::new(ServerT)
//     }
// }
#[tonic::async_trait]
impl Plugin for Test1 {
    // there is no way to force the following types to be tonic types :(
    type Client = GrpcBrokerClient<Channel>;
    type Server = GrpcBrokerServer<ServerT>;

    async fn client(&self, channel: Channel) -> Self::Client {
        GrpcBrokerClient::new(channel)
    }

    async fn server(&self) -> Self::Server {
        GrpcBrokerServer::new(ServerT)
    }
}
fn ss() {
    let p = Box::new(Test1);
    let p = p as Box<
        dyn Plugin<Client = GrpcBrokerClient<Channel>, Server = GrpcBrokerServer<ServerT>>,
    >;
}

struct Client {}

// pub struct PluginMap {
//     map: HashMap<String, Box<dyn Any>>,
// }

// impl PluginClientMap {
//     fn new() -> Self {
//         Self {
//             map: HashMap::new(),
//         }
//     }

//     fn insert<P: Plugin + 'static>(&mut self, client_builder: impl FnOnce(#[must_use] C) -> P) -> Option<P>
//     where
//         P: Plugin + 'static,
//         C: tonic::client::GrpcService<tonic::body::BoxBody>,
//         C::Error: Into<StdError>,
//         C::ResponseBody: Body<Data = Bytes> + Send + 'static,
//         <C::ResponseBody as Body>::Error: Into<StdError> + Send,
//     {
//         self.map.insert(std::any::type_name::<P>().to_string(), Box::new(client_builder));
//     }

//     fn get<P: Plugin + 'static>(&self) -> Option<&P> {
//         self.map.get(&std::any::type_name::<P>()).and_then(|p| p.downcast_ref())
//     }
// }
