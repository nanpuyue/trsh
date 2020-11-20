use std::convert::TryFrom;

use tokio::{io, select};
use tokio_fd::AsyncFd;

use crate::*;

pub async fn server(addr: &str, cert: &str, key: &str) -> Result<()> {
    println!("fingerprint: {}", tls::cert_digest(cert)?);
    let listener = util::listen_reuseport(addr)?;
    let tcpstream = listener.accept().await?.0;
    drop(listener);
    tcpstream.set_nodelay(true)?;

    let tlsstream = tls::tls_accept(tcpstream, cert, key).await?;
    let (reader, writer) = &mut io::split(tlsstream);
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

    Ok(link.map(drop)?)
}
