use std::net::SocketAddr;
use std::ptr::{null, null_mut};
use std::str::FromStr;

use tokio::net::{TcpListener, TcpSocket};

use crate::Result;

pub(crate) trait AsPtr<T> {
    fn as_ptr(&self) -> *const T;
    fn as_mut_ptr(&mut self) -> *mut T;
}

impl<T> AsPtr<T> for Option<T> {
    fn as_ptr(&self) -> *const T {
        match self {
            Some(x) => x as *const T,
            None => null(),
        }
    }

    fn as_mut_ptr(&mut self) -> *mut T {
        match self {
            Some(x) => x as *mut T,
            None => null_mut(),
        }
    }
}

pub(crate) fn listen_reuseport(addr: &str) -> Result<TcpListener> {
    let addr = SocketAddr::from_str(addr)?;
    let socket = match addr {
        SocketAddr::V4(_) => TcpSocket::new_v4(),
        SocketAddr::V6(_) => TcpSocket::new_v6(),
    }?;
    socket.set_reuseport(true)?;
    socket.bind(addr)?;
    Ok(socket.listen(0)?)
}
