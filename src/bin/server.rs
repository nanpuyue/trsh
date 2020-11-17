use std::convert::TryFrom;
use std::io::Result;

use tokio::net::TcpListener;
use tokio::stream::StreamExt;
use tokio::{io, select};
use tokio_fd::AsyncFd;

use rshell::*;

#[tokio::main]
async fn main() -> Result<()> {
    let mut listener = TcpListener::bind("0.0.0.0:8000").await?;
    let tcpstream = listener.next().await.unwrap()?;
    tcpstream.set_nodelay(true)?;
    let (reader, writer) = &mut io::split(tcpstream);
    let stdin = &mut AsyncFd::try_from(libc::STDIN_FILENO)?;
    let stdout = &mut AsyncFd::try_from(libc::STDOUT_FILENO)?;

    term::setup_terminal(libc::STDIN_FILENO, false)?;

    let link = select! {
        a = io::copy(stdin, writer) => {
            a
        },
        b = io::copy(reader, stdout) => {
            b
        }
    };

    link.map(drop)
}
