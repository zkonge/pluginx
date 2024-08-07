use std::any::{Any, TypeId};

use ahash::AHashMap;
use futures::TryFutureExt;
use hyper_util::rt::TokioIo;
use tokio::net::UnixStream;
use tonic::transport::{Channel, Uri};
use tower::service_fn;

use crate::{
    handshake::{HandshakeError, Network},
    PluginxError,
};

pub(crate) struct Client {
    channel: Channel,
    service: AHashMap<TypeId, Box<dyn Any + Send + Sync>>,
}

impl Client {
    pub(crate) async fn new(network: &Network) -> Result<Self, PluginxError> {
        let channel = match network {
            Network::Tcp(addr) => {
                let uri = Uri::builder()
                    .scheme("http")
                    .authority(addr.to_string())
                    .build()
                    .map_err(|_| PluginxError::HandshakeError {
                        error: HandshakeError::InvalidNetwork,
                        message: addr.to_string(),
                    })?;
                Channel::builder(uri).connect().await?
            }
            Network::Unix(path) => {
                let path = path.to_owned();
                Channel::from_static("http://[::1]")
                    .connect_with_connector(service_fn(move |_: Uri| {
                        UnixStream::connect(path.clone()).map_ok(TokioIo::new)
                    }))
                    .await?
            }
        };

        Ok(Self {
            channel,
            service: AHashMap::new(),
        })
    }

    pub(crate) fn channel(&self) -> &Channel {
        &self.channel
    }

    #[inline]
    pub(crate) fn add_service<S: Clone + Send + Sync + 'static>(
        &mut self,
        service: S,
    ) -> &mut Self {
        self.service.insert(TypeId::of::<S>(), Box::new(service));
        self
    }

    #[inline]
    pub(crate) fn dispense<S: Clone + 'static>(&self) -> Option<S> {
        let id = TypeId::of::<S>();
        self.service
            .get(&id)
            .and_then(|p| p.downcast_ref::<S>())
            .cloned()
    }
}
