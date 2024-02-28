use tokio::sync::broadcast::{self, Receiver, Sender};
use tonic::{transport::Channel, Request, Response, Status};

use crate::proto::{
    grpc_controller_client::GrpcControllerClient,
    grpc_controller_server::{GrpcController, GrpcControllerServer},
    Empty,
};

#[derive(Debug)]
pub struct ControllerExitSignal(Receiver<()>);

impl ControllerExitSignal {
    pub async fn wait(mut self) {
        _ = self.0.recv().await;
    }
}

impl Clone for ControllerExitSignal {
    fn clone(&self) -> Self {
        Self(self.0.resubscribe())
    }
}

pub struct ControllerServer(Sender<()>);

impl ControllerServer {
    pub fn new() -> (GrpcControllerServer<Self>, ControllerExitSignal) {
        let (tx, rx) = broadcast::channel(1);
        (
            GrpcControllerServer::new(Self(tx)),
            ControllerExitSignal(rx),
        )
    }
}

#[tonic::async_trait]
impl GrpcController for ControllerServer {
    async fn shutdown(&self, _: Request<Empty>) -> Result<Response<Empty>, Status> {
        // don't care about if anyone is wait for shutdown
        _ = self.0.send(());

        Ok(Response::new(Empty {}))
    }
}

pub struct ControllerClient {
    client: GrpcControllerClient<Channel>,
}

impl ControllerClient {
    pub fn new(channel: Channel) -> Self {
        Self {
            client: GrpcControllerClient::new(channel),
        }
    }

    pub async fn shutdown(&mut self) -> Result<(), Status> {
        self.client
            .shutdown(Request::new(Empty {}))
            .await?
            .into_inner();
        Ok(())
    }
}
