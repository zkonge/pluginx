mod config;
mod plugin_map;

use std::any::{Any, TypeId};

use hashbrown::HashMap;
use tonic::transport::Channel;

use crate::plugin::Plugin;

use self::config::ClientConfig;

pub struct ClientBuilder {
    config: self::config::ClientConfig,
    plugins: HashMap<TypeId, Box<dyn Any>>,
}

impl ClientBuilder {
    pub fn new(mut config: ClientConfig) -> Self {
        // 1. start plugin
        let proc = config.cmd.spawn().unwrap();
        

        Self {
            config,
            plugins: HashMap::new(),
        }
    }

    pub fn add_plugin<P: Plugin + 'static>(&mut self, plugin: P) -> &mut Self {
        self.config
            .plugins
            .insert(TypeId::of::<P>(), Box::new(plugin));
        self
    }
}

pub struct Client {
    c: Channel,
    plugins: HashMap<TypeId, Box<dyn Any>>,
}

impl Client {
    pub fn dispense<P: Plugin + 'static>(&self) -> Option<&P::Client> {
        let id = TypeId::of::<P>();
        self.plugins
            .get(&id)
            .and_then(|p| p.downcast_ref::<P::Client>())
    }
}

fn useit() {
    let client: Client = todo!();

    // client.dispense::<proto::>()
}
