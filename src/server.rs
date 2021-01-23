use std::convert::TryFrom;

use tokio::{io, select};
use tokio_fd::AsyncFd;

use crate::error::Result;
use crate::term::enter_raw_mode;
use crate::tls::{acceptor_context, context_digest, tls_accept};
use crate::util::listen_reuseport;

pub async fn server(addr: &str, cert: &str, key: &str) -> Result<()> {
    let ctx = acceptor_context(cert, key)?;
    println!("Server fingerprint: {}", context_digest(&ctx)?);

    let listener = listen_reuseport(addr)?;
    println!("Waiting for client to connect...");

    let (tcpstream, peer) = listener.accept().await?;
    println!("Client \"{}\" connected.\n", peer.to_string());

    drop(listener);
    tcpstream.set_nodelay(true)?;

    let tlsstream = tls_accept(tcpstream, &ctx).await?;
    let (reader, writer) = &mut io::split(tlsstream);
    let stdin = &mut AsyncFd::try_from(libc::STDIN_FILENO)?;
    let stdout = &mut AsyncFd::try_from(libc::STDOUT_FILENO)?;

    enter_raw_mode(libc::STDIN_FILENO, false)?;

    Ok(select! {
        a = io::copy(stdin, writer) => {
            a
        },
        b = io::copy(reader, stdout) => {
            b
        }
    }?)
    .map(drop)
}
