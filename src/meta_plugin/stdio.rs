use std::sync::Mutex;

use tokio::sync::mpsc::{self, error::SendError, Receiver, Sender};
use tokio_stream::wrappers::ReceiverStream;
use tonic::{transport::Channel, Request, Response, Status, Streaming};

use crate::proto::{
    grpc_stdio_client::GrpcStdioClient,
    grpc_stdio_server::{GrpcStdio, GrpcStdioServer},
    StdioData,
};

#[derive(Clone, Debug)]
pub struct StdioHandler {
    tx: Sender<Result<StdioData, Status>>,
}

impl StdioHandler {
    pub async fn write(&self, out_type: i32, data: Vec<u8>) -> Result<(), Vec<u8>> {
        let data = StdioData {
            channel: out_type,
            data,
        };

        self.tx
            .send(Ok(data))
            .await
            .map_err(|SendError(r)| r.unwrap().data)
    }
}

pub struct StdioServer(Mutex<Option<Receiver<Result<StdioData, Status>>>>);

impl StdioServer {
    pub fn new() -> (GrpcStdioServer<Self>, StdioHandler) {
        let (tx, rx) = mpsc::channel(1);
        (
            GrpcStdioServer::new(Self(Mutex::new(Some(rx)))),
            StdioHandler { tx },
        )
    }
}

#[tonic::async_trait]
impl GrpcStdio for StdioServer {
    type StreamStdioStream = ReceiverStream<Result<StdioData, Status>>;

    async fn stream_stdio(
        &self,
        _: Request<()>,
    ) -> Result<Response<Self::StreamStdioStream>, Status> {
        let receiver = match self.0.try_lock() {
            Ok(mut rx) => match rx.take() {
                Some(rx) => Ok(rx),
                None => Err(Status::unavailable("stdio stream is already in use")),
            },
            Err(_) => Err(Status::internal("mutex is poisoned")),
        }?;

        Ok(Response::new(ReceiverStream::new(receiver)))
    }
}

pub struct StdioClient {
    client: GrpcStdioClient<Channel>,
}

impl StdioClient {
    pub fn new(channel: Channel) -> Self {
        let client = GrpcStdioClient::new(channel);
        Self { client }
    }

    pub async fn read(&mut self) -> Result<Streaming<StdioData>, Status> {
        let s = self
            .client
            .stream_stdio(Request::new(()))
            .await?
            .into_inner();

        Ok(s)
    }
}
