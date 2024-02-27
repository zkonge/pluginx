use std::{ops::RangeInclusive, time::Duration};

use tokio::process::Command;

use crate::handshake::HandshakeConfig;

pub struct ClientConfig {
    pub handshake_config: HandshakeConfig<'static>,
    pub cmd: Command,
    pub broker_multiplex: bool,
    pub port_range: Option<RangeInclusive<u16>>,
    pub startup_timeout: Duration,
}
