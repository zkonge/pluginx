use std::sync::{
    atomic::{AtomicBool, Ordering},
    Arc,
};

use tokio::sync::Notify;
use tonic::{transport::Channel, Request, Response, Status};

use crate::proto::{
    grpc_controller_client::GrpcControllerClient,
    grpc_controller_server::{GrpcController, GrpcControllerServer},
    Empty,
};

#[derive(Debug)]
pub struct ControllerExitSignal(Arc<(Notify, AtomicBool)>);

impl ControllerExitSignal {
    pub async fn wait(&self) {
        let (notify, is_exit) = self.0.as_ref();

        if is_exit.load(Ordering::Acquire) {
            return;
        }

        notify.notified().await;
    }
}

impl Clone for ControllerExitSignal {
    fn clone(&self) -> Self {
        Self(self.0.clone())
    }
}

pub struct ControllerServer(Arc<(Notify, AtomicBool)>);

impl ControllerServer {
    pub fn new() -> (GrpcControllerServer<Self>, ControllerExitSignal) {
        let n = Arc::new((Notify::new(), Default::default()));
        (
            GrpcControllerServer::new(Self(n.clone())),
            ControllerExitSignal(n),
        )
    }
}

#[tonic::async_trait]
impl GrpcController for ControllerServer {
    async fn shutdown(&self, _: Request<Empty>) -> Result<Response<Empty>, Status> {
        let (notify, is_exit) = self.0.as_ref();

        is_exit.store(true, Ordering::Release);

        notify.notify_waiters();

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
