pub mod config;
pub mod message;

use std::num::ParseIntError;

use thiserror::Error;

pub use self::{
    config::HandshakeConfig,
    message::{HandshakeMessage, Network, Protocol},
};

#[derive(Error, Debug)]
pub enum HandshakeError {
    #[error("invalid handshake message")]
    InvalidHandshakeMessage,
    #[error("unsupported core protocol version")]
    UnsupportedCoreProtocolVersion,
    #[error("unsupported protocol version")]
    UnsupportedAppProtocolVersion,
    #[error("invalid handshake network type")]
    InvalidNetwork,
    #[error("invalid transport protocol")]
    InvalidTransportProtocol,
    #[error("parse number failed: {0}")]
    ParseNumberFailed(#[from] ParseIntError),
    #[error("plugin startup timeout")]
    StarupTimeout,
}
