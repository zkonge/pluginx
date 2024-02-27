use crate::handshake::HandshakeConfig;

pub struct ServerConfig {
    pub handshake_config: HandshakeConfig<'static>,
}
