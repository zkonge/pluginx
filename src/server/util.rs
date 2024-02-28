use std::{
    env,
    io::{self, ErrorKind},
    net::{Ipv4Addr, SocketAddrV4},
};

use tokio::net::{TcpListener, UnixListener};

use crate::constant::{PLUGIN_MAX_PORT, PLUGIN_MIN_PORT, PLUGIN_UNIX_SOCKET_DIR};

const PLUGIN_UNIX_SOCKET_PREFIX: &str = "plugin-";

pub async fn find_available_tcp_listener() -> io::Result<TcpListener> {
    let port_start = env::var(PLUGIN_MIN_PORT).ok().and_then(|x| x.parse().ok());
    let port_end = env::var(PLUGIN_MAX_PORT).ok().and_then(|x| x.parse().ok());

    let port_range = match (port_start, port_end) {
        (Some(start), Some(end)) => start..=end,
        _ => 10000..=25000u16,
    };

    for port in port_range {
        match TcpListener::bind(SocketAddrV4::new(Ipv4Addr::new(127, 0, 0, 1), port)).await {
            Ok(listener) => return Ok(listener),
            Err(e) if e.kind() == ErrorKind::AddrInUse => continue,
            e @ Err(_) => return e,
        }
    }

    Err(ErrorKind::AddrInUse.into())
}

pub fn find_available_unix_socket_listener() -> io::Result<UnixListener> {
    let path = {
        let mut b = tempfile::Builder::new();
        let b = b.prefix(PLUGIN_UNIX_SOCKET_PREFIX);

        let f = if let Ok(dir) = env::var(PLUGIN_UNIX_SOCKET_DIR) {
            b.tempfile_in(dir)
        } else {
            b.tempfile()
        }?;
        f.path().to_owned()
    };

    UnixListener::bind(path)
}
