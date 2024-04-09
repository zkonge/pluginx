use thiserror::Error;

use crate::handshake::HandshakeError;

#[derive(Error, Debug)]
pub enum PluginxError {
    #[error("tonic: {0}")]
    TonicError(#[from] tonic::transport::Error),
    #[error("tokio task panic: {0}")]
    TokioTaskError(#[from] tokio::task::JoinError),
    #[error("io error: {0}")]
    IoError(#[from] std::io::Error),

    #[error("handshake failed: {error}, message: {message}")]
    HandshakeError {
        error: HandshakeError,
        message: String,
    },
}

/// fast convert for [`HandshakeError`] that doesn't provides any message
impl From<HandshakeError> for PluginxError {
    fn from(error: HandshakeError) -> Self {
        Self::HandshakeError {
            error,
            message: String::new(),
        }
    }
}
