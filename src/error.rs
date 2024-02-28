use thiserror::Error;

#[derive(Error, Debug)]
pub enum PluginxError {
    #[error("tonic: {0}")]
    TonicError(#[from] tonic::transport::Error),
    #[error("tokio task panic: {0}")]
    TokioTaskError(#[from] tokio::task::JoinError),
    #[error("io error: {0}")]
    IoError(#[from] std::io::Error),

    #[error(transparent)]
    HandshakeError(#[from] crate::handshake::HandshakeError),
}
