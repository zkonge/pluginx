use std::{collections::HashMap, ops::RangeInclusive, process::Command};

use prost_types::Duration;

use crate::handshake::HandshakeConfig;

pub struct ClientConfig {
    pub handshake_config: HandshakeConfig<'static>,
    pub cmd: Box<Command>,
    pub broker_multiplex: bool,
    pub port_range: RangeInclusive<u16>,
    pub startup_timeout: Duration,
}

// fn s(){
// Command::new("./").arg("arg").output()
// }
