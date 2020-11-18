use std::convert::TryFrom;
use std::ffi::CString;
use std::io::Error;
use std::ptr::null;

use tokio::io::{duplex, AsyncReadExt, AsyncWriteExt};
use tokio::net::TcpStream;
use tokio::{io, select};
use tokio_fd::AsyncFd;

use crate::*;

pub async fn client(addr: &str, sni: &str, verify: bool) -> Result<()> {
    let tcpstream = TcpStream::connect(addr).await?;
    tcpstream.set_nodelay(true)?;
    let tlsstream = tls::tls_connect(tcpstream, sni, verify).await?;

    let (pty_master, pid) = term::fork_pty()?;
    unsafe { term::PTY_MASTER = Some(pty_master) };

    if pid == 0 {
        let exe = CString::new("/usr/bin/env").unwrap();
        let argv: Vec<_> = vec!["", "bash"]
            .iter()
            .map(|&x| CString::new(x).unwrap())
            .collect();
        let mut argv: Vec<_> = argv.iter().map(|x| x.as_ptr()).collect();
        argv.push(null());
        unsafe {
            libc::execv(exe.as_ptr(), argv.as_ptr());
        }
        return Err(Error::last_os_error().into());
    }

    term::setup_terminal(pty_master, true)?;
    let pty = AsyncFd::try_from(pty_master)?;
    let (pty_reader, pty_writer) = &mut io::split(pty);
    let (tcp_reader, tcp_writer) = &mut io::split(tlsstream);

    let stdin = &mut AsyncFd::try_from(libc::STDIN_FILENO)?;
    let stdout = &mut AsyncFd::try_from(libc::STDOUT_FILENO)?;

    let (mut ds1, mut ds2) = duplex(2048);

    let link1 = async {
        let buf = &mut vec![0; 1024];
        loop {
            let n = pty_reader.read(buf).await?;
            ds1.write_all(&buf[..n]).await?;
            tcp_writer.write_all(&buf[..n]).await?;
        }
    };

    let link2 = async { Ok(io::copy(tcp_reader, pty_writer).await.map(drop)?) };

    let echo = async {
        let buf = &mut vec![0; 1024];
        loop {
            let n = ds2.read(buf).await?;
            stdout.write_all(&buf[..n]).await?;
        }
    };

    let read = async {
        let buf = &mut vec![0; 1024];
        loop {
            if stdin.read(buf).await? == 0 {
                break;
            }
        }
        Ok(())
    };

    select! {
        a = link1 => {
            a
        }
        b = link2 => {
            b
        }
        c = echo => {
            c
        }
        d = read => {
            d
        }
    }
}
