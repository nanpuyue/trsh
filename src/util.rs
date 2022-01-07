use std::net::SocketAddr;
use std::pin::Pin;
use std::ptr::{null, null_mut};
use std::task::{Context, Poll, Poll::*};

use tokio::io::{self, AsyncRead, ReadBuf};
use tokio::net::{TcpListener, TcpSocket};

use crate::error::Result;

pub trait AsPtr<T> {
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

pub struct Merge<T, U> {
    a: T,
    b: U,
    a_first: bool,
}

pub fn merge_reader<T, U>(a: T, b: U) -> Merge<T, U>
where
    T: AsyncRead + Unpin,
    U: AsyncRead + Unpin,
{
    Merge {
        a,
        b,
        a_first: false,
    }
}

impl<T: AsyncRead + Unpin, U: AsyncRead + Unpin> AsyncRead for Merge<T, U> {
    fn poll_read(
        mut self: Pin<&mut Self>,
        cx: &mut Context<'_>,
        buf: &mut ReadBuf<'_>,
    ) -> Poll<io::Result<()>> {
        let mut pending = false;

        loop {
            self.a_first = !self.a_first;

            return match if self.a_first {
                Pin::new(&mut self.a).poll_read(cx, buf)
            } else {
                Pin::new(&mut self.b).poll_read(cx, buf)
            } {
                Pending if !pending => {
                    pending = true;
                    continue;
                }
                x => x,
            };
        }
    }
}

pub fn listen_reuseport(addr: SocketAddr) -> Result<TcpListener> {
    let socket = match addr {
        SocketAddr::V4(_) => TcpSocket::new_v4(),
        SocketAddr::V6(_) => TcpSocket::new_v6(),
    }?;
    socket.set_reuseport(true)?;
    socket.bind(addr)?;
    Ok(socket.listen(64)?)
}
