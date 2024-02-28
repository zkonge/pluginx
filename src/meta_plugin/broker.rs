use tokio_stream::Pending;
use tonic::{Request, Response, Status, Streaming};

use crate::proto::{
    grpc_broker_server::{GrpcBroker, GrpcBrokerServer},
    ConnInfo,
};

#[derive(Clone, Debug)]
pub struct BrokerHandler;

pub struct BrokerServer;

impl BrokerServer {
    pub fn new() -> (GrpcBrokerServer<Self>, BrokerHandler) {
        (GrpcBrokerServer::new(Self), BrokerHandler)
    }
}

#[tonic::async_trait]
impl GrpcBroker for BrokerServer {
    type StartStreamStream = Pending<Result<ConnInfo, Status>>;

    async fn start_stream(
        &self,
        _: Request<Streaming<ConnInfo>>,
    ) -> Result<Response<Self::StartStreamStream>, Status> {
        Ok(Response::new(tokio_stream::pending()))
    }
}
