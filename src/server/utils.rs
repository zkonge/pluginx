use std::{env, io, path::PathBuf};

use crate::{
    common::server::TransportConfig,
    constant::{PLUGIN_MAX_PORT, PLUGIN_MIN_PORT, PLUGIN_UNIX_SOCKET_DIR},
};

const PLUGIN_UNIX_SOCKET_PREFIX: &str = "plugin-";

pub(crate) fn tcp_transport_config_from_env() -> io::Result<TransportConfig> {
    let port_start = env::var(PLUGIN_MIN_PORT).ok().and_then(|x| x.parse().ok());
    let port_end = env::var(PLUGIN_MAX_PORT).ok().and_then(|x| x.parse().ok());

    let port_range = match (port_start, port_end) {
        (Some(start), Some(end)) => start..=end,
        _ => 10000..=25000u16,
    };

    Ok(TransportConfig::Tcp { port_range })
}

pub(crate) fn unix_transport_config_from_env() -> io::Result<TransportConfig> {
    let dir = env::var(PLUGIN_UNIX_SOCKET_DIR)
        .map(PathBuf::from)
        .map(Into::into)
        .ok();

    Ok(TransportConfig::Unix {
        prefix: PLUGIN_UNIX_SOCKET_PREFIX.into(),
        dir,
    })
}
