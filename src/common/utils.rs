use std::{
    io::{ErrorKind, Result},
    net::{Ipv4Addr, SocketAddrV4, TcpListener as StdTcpListener},
    ops::RangeInclusive,
    path::Path,
};

use tokio::net::{TcpListener, UnixListener};

/// Find an available TCP listener.
/// port_range: The range of ports to search for an available port.
pub(crate) fn find_available_tcp_listener(port_range: RangeInclusive<u16>) -> Result<TcpListener> {
    for port in port_range {
        let addr = SocketAddrV4::new(Ipv4Addr::new(127, 0, 0, 1), port);

        match StdTcpListener::bind(addr).and_then(TcpListener::from_std) {
            Ok(listener) => return Ok(listener),
            Err(e) if e.kind() == ErrorKind::AddrInUse => continue,
            e @ Err(_) => return e,
        }
    }

    Err(ErrorKind::AddrInUse.into())
}

/// Find an available Unix socket listener.
/// prefix: The prefix of the Unix socket file.
/// dir: The directory where the Unix socket file is created.
pub fn find_available_unix_socket_listener(
    prefix: &str,
    dir: Option<&Path>,
) -> Result<UnixListener> {
    let path = loop {
        let mut b = tempfile::Builder::new();
        let b = b.prefix(prefix);

        let r = if let Some(dir) = dir {
            b.tempfile_in(dir)
        } else {
            b.tempfile()
        };

        match r {
            Ok(f) => break f.path().to_owned(),
            Err(e) if e.kind() == ErrorKind::AlreadyExists => continue,
            Err(e) => return Err(e),
        }
    };

    UnixListener::bind(path)
}
