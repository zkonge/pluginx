use std::{net::SocketAddr, path::PathBuf, str::FromStr};

use super::HandshakeError;

#[derive(Debug)]
pub enum Network {
    Tcp(SocketAddr),
    Unix(PathBuf),
}

impl Network {
    pub fn parse(type_: &str, addr: &str) -> Result<Self, HandshakeError> {
        let e = HandshakeError::InvalidNetwork;
        match type_ {
            "tcp" => Ok(Self::Tcp(addr.parse().map_err(|_| e)?)),
            "unix" => Ok(Self::Unix(addr.parse().map_err(|_| e)?)),
            _ => Err(e),
        }
    }
}

#[derive(Debug)]
pub enum Protocol {
    Grpc,
}

impl FromStr for Protocol {
    type Err = HandshakeError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "grpc" => Ok(Self::Grpc),
            _ => Err(HandshakeError::InvalidTransportProtocol),
        }
    }
}

#[derive(Debug)]
pub struct HandshakeMessage {
    pub core_protocol: u32,
    pub app_protocol: u32,
    pub network: Network,
    pub protocol: Protocol,
}

impl HandshakeMessage {
    // CORE-PROTOCOL-VERSION | APP-PROTOCOL-VERSION | NETWORK-TYPE | NETWORK-ADDR | PROTOCOL
    pub fn parse(s: &str) -> Result<Self, HandshakeError> {
        let mut it: Vec<_> = s.split('|').map(str::trim).collect();
        if it.len() < 5 {
            return Err(HandshakeError::InvalidHandshakeMessage);
        }

        Ok(Self {
            core_protocol: it[0].parse()?,
            app_protocol: it[1].parse()?,
            network: Network::parse(it[2], it[3])?,
            protocol: it[4].parse()?,
        })
    }
}
