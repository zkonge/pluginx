use std::{
    io::{self, ErrorKind},
    net::{Ipv4Addr, SocketAddrV4},
    ops::RangeInclusive,
};

use tokio::net::TcpListener;

pub async fn find_available_tcp_port(port_range: RangeInclusive<u16>) -> io::Result<TcpListener> {
    for port in port_range {
        match TcpListener::bind(SocketAddrV4::new(Ipv4Addr::new(127, 0, 0, 1), port)).await {
            Ok(listener) => return Ok(listener),
            Err(e) if e.kind() == ErrorKind::AddrInUse => continue,
            e @ Err(_) => return e,
        }
    }
    Err(ErrorKind::AddrInUse.into())
}
