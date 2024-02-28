use tokio::sync::{
    oneshot::{channel, Receiver, Sender},
    Mutex,
};
use tonic::{Request, Response, Status, Streaming};

use crate::proto::{
    grpc_broker_server::{GrpcBroker, GrpcBrokerServer},
    grpc_controller_server::{GrpcController, GrpcControllerServer},
    grpc_stdio_server::{GrpcStdio, GrpcStdioServer},
    ConnInfo, Empty, StdioData,
};

// control service
pub struct Controller(Mutex<Option<Sender<()>>>);

impl Controller {
    pub fn new() -> (GrpcControllerServer<Self>, Receiver<()>) {
        let (tx, rx) = channel();
        (GrpcControllerServer::new(Self(Mutex::new(Some(tx)))), rx)
    }
}

#[tonic::async_trait]
impl GrpcController for Controller {
    async fn shutdown(&self, _: Request<Empty>) -> Result<Response<Empty>, Status> {
        self.0
            .lock()
            .await
            .take()
            .and_then(|x| x.send(()).ok())
            .ok_or_else(|| Status::internal("shutdown receiver dropped"))?;

        Ok(Response::new(Empty {}))
    }
}

// grpc broker
pub struct Broker;

impl Broker {
    pub fn new() -> GrpcBrokerServer<Self> {
        GrpcBrokerServer::new(Self)
    }
}

#[tonic::async_trait]
impl GrpcBroker for Broker {
    type StartStreamStream = tokio_stream::Pending<Result<ConnInfo, Status>>;

    async fn start_stream(
        &self,
        _: Request<Streaming<ConnInfo>>,
    ) -> Result<Response<Self::StartStreamStream>, Status> {
        Ok(Response::new(tokio_stream::pending()))
    }
}

// stdio service
pub struct Stdio;

impl Stdio {
    pub fn new() -> GrpcStdioServer<Self> {
        GrpcStdioServer::new(Self)
    }
}

#[tonic::async_trait]
impl GrpcStdio for Stdio {
    type StreamStdioStream = tokio_stream::Pending<Result<StdioData, Status>>;

    async fn stream_stdio(
        &self,
        _: Request<()>,
    ) -> Result<Response<Self::StreamStdioStream>, Status> {
        Ok(Response::new(tokio_stream::pending()))
    }
}
