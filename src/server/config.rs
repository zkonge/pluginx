use std::collections::HashMap;

use crate::handshake::HandshakeConfig;

pub struct ServerConfig {
    handshake_config: HandshakeConfig<'static>,
    plugins: HashMap<String, ()>,
}
