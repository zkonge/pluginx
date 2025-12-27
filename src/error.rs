use std::io;

use thiserror::Error;

use crate::handshake::HandshakeError;

#[derive(Error, Debug)]
pub enum PluginxError {
    #[error("tonic: {0}")]
    Tonic(#[from] tonic::transport::Error),
    #[error("tokio task panic: {0}")]
    TokioTask(#[from] tokio::task::JoinError),
    #[error("io error: {0}")]
    Io(#[from] io::Error),

    #[error("handshake failed: {error}, message: {message}")]
    Handshake {
        error: HandshakeError,
        message: String,
    },
}

/// fast convert for [`HandshakeError`] that doesn't provides any message
impl From<HandshakeError> for PluginxError {
    fn from(error: HandshakeError) -> Self {
        Self::Handshake {
            error,
            message: String::new(),
        }
    }
}
