use std::io;

use tokio::sync::{mpsc::Sender, oneshot, Mutex};
use tokio_stream::wrappers::ReceiverStream;
use tonic::{Request, Response, Status, Streaming};

use crate::proto::{
    grpc_broker_server::GrpcBroker,
    grpc_controller_server::{GrpcController, GrpcControllerServer},
    grpc_stdio_server::{GrpcStdio, GrpcStdioServer},
    ConnInfo, Empty, StdioData,
};

// control service

pub struct Controller(Mutex<Option<oneshot::Sender<()>>>);

impl Controller {
    pub fn new() -> (GrpcControllerServer<Self>, oneshot::Receiver<()>) {
        let (tx, rx) = oneshot::channel();
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

pub struct Broker();

#[tonic::async_trait]
impl GrpcBroker for Broker {
    type StartStreamStream = tokio_stream::Empty<Result<ConnInfo, Status>>;

    async fn start_stream(
        &self,
        _: Request<Streaming<ConnInfo>>,
    ) -> Result<Response<Self::StartStreamStream>, Status> {
        Err(Status::unimplemented("not finished"))
    }
}

// stdio service

type StdioDataResult = Result<StdioData, Status>;

pub struct GrpcStdoutWriter(Sender<StdioDataResult>);

impl io::Write for GrpcStdoutWriter {
    fn write(&mut self, buf: &[u8]) -> io::Result<usize> {
        self.0
            .blocking_send(Ok(StdioData {
                channel: 1, // stdout
                data: buf.to_vec(),
            }))
            .map_err(|_| io::ErrorKind::BrokenPipe)?;
        Ok(buf.len())
    }

    fn flush(&mut self) -> io::Result<()> {
        Ok(())
    }
}

pub struct GrpcStdioWriterMaker(Sender<StdioDataResult>);

impl tracing_subscriber::fmt::MakeWriter<'_> for GrpcStdioWriterMaker {
    type Writer = GrpcStdoutWriter;

    fn make_writer(&self) -> Self::Writer {
        GrpcStdoutWriter(self.0.clone())
    }
}

pub struct Stdio(Mutex<Option<ReceiverStream<StdioDataResult>>>);

impl Stdio {
    pub fn new() -> (GrpcStdioServer<Self>, GrpcStdioWriterMaker) {
        let (tx, rx) = tokio::sync::mpsc::channel(1);
        (
            GrpcStdioServer::new(Self(Mutex::new(Some(ReceiverStream::new(rx))))),
            GrpcStdioWriterMaker(tx),
        )
    }
}

#[tonic::async_trait]
impl GrpcStdio for Stdio {
    type StreamStdioStream = ReceiverStream<Result<StdioData, Status>>;

    async fn stream_stdio(
        &self,
        _: Request<()>,
    ) -> Result<Response<Self::StreamStdioStream>, Status> {
        let recv = match self.0.lock().await.take() {
            Some(recv) => recv,
            None => return Err(Status::internal("stream_stdio called twice")),
        };
        Ok(Response::new(recv))
    }
}
