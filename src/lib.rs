pub mod broker;
pub mod client;
pub mod common;
pub mod constant;
pub mod error;
pub mod handshake;
pub mod meta_plugin;
pub mod plugin;
pub mod proto;
pub mod server;

pub use tonic::{async_trait, server::NamedService, Request, Response, Status, Streaming};

pub use self::error::PluginxError;

type StdError = Box<dyn std::error::Error + Send + Sync>;
