use std::{
    fmt::{Debug, Formatter, Result as FmtResult},
    future::Future,
    io::{ErrorKind, Result as IoResult},
    net::{Ipv4Addr, SocketAddrV4, TcpListener as StdTcpListener},
    ops::RangeInclusive,
    path::Path,
    task::{Context, Poll},
};

use tokio::net::{TcpListener, UnixListener};
use tower_service::Service;

/// Find an available TCP listener.
/// port_range: The range of ports to search for an available port.
pub(crate) fn find_available_tcp_listener(
    port_range: RangeInclusive<u16>,
) -> IoResult<TcpListener> {
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
pub(crate) fn find_available_unix_socket_listener(
    prefix: &str,
    dir: Option<&Path>,
) -> IoResult<UnixListener> {
    let path = loop {
        let mut b = tempfile::Builder::new();
        let b = b.prefix(prefix).rand_bytes(8);

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

// following code is copied from tower-rs/tower and remove the dependency on tower crate

pub(crate) fn service_fn<T>(f: T) -> ServiceFn<T> {
    ServiceFn { f }
}

#[derive(Copy, Clone)]
pub(crate) struct ServiceFn<T> {
    f: T,
}

impl<T> Debug for ServiceFn<T> {
    fn fmt(&self, f: &mut Formatter<'_>) -> FmtResult {
        f.debug_struct("ServiceFn")
            .field("f", &format_args!("{}", std::any::type_name::<T>()))
            .finish()
    }
}

impl<T, F, Request, R, E> Service<Request> for ServiceFn<T>
where
    T: FnMut(Request) -> F,
    F: Future<Output = Result<R, E>>,
{
    type Response = R;
    type Error = E;
    type Future = F;

    fn poll_ready(&mut self, _: &mut Context<'_>) -> Poll<Result<(), E>> {
        Ok(()).into()
    }

    fn call(&mut self, req: Request) -> Self::Future {
        (self.f)(req)
    }
}
