use std::convert::TryFrom;
use std::ffi::CString;
use std::future::{pending, Future};
use std::io::{stdin, Error};
use std::pin::Pin;
use std::ptr::null;

use tokio::io::{duplex, AsyncRead, AsyncReadExt, AsyncWriteExt};
use tokio::net::TcpStream;
use tokio::{io, select};
use tokio_fd::AsyncFd;

use crate::error::Result;
use crate::term::{fork_pty, setup_terminal, PTY_MASTER};
use crate::tls::{peer_digest, tls_connect};
use crate::util::merge_reader;

pub async fn client(addr: &str, sni: &str, verify: bool, readonly: bool) -> Result<()> {
    let tcpstream = TcpStream::connect(addr).await?;
    tcpstream.set_nodelay(true)?;
    let tlsstream = tls_connect(tcpstream, sni, verify).await?;

    println!("Server fingerprint: {}", peer_digest(&tlsstream)?);
    if !verify {
        println!("Do you want continue? [y/N]");
        let buf = &mut String::new();
        stdin().read_line(buf)?;
        if !buf.to_ascii_lowercase().starts_with('y') {
            return Ok(());
        }
    }

    if readonly {
        println!("You can use \"Ctrl + C\" to disconnect at any time.\n");
    }

    let (pty_master, pid) = fork_pty()?;

    if pid == 0 {
        let exe = CString::new("/usr/bin/env").unwrap();
        let argv: Vec<_> = vec!["", "bash", "-l"]
            .iter()
            .map(|&x| CString::new(x).unwrap())
            .collect();
        let mut argv: Vec<_> = argv.iter().map(|x| x.as_ptr()).collect();
        argv.push(null());
        unsafe { libc::execv(exe.as_ptr(), argv.as_ptr()) };
        return Err(Error::last_os_error().into());
    }

    unsafe { PTY_MASTER = Some(pty_master) };
    setup_terminal(pty_master, readonly)?;

    let pty = AsyncFd::try_from(pty_master)?;
    let (pty_reader, pty_writer) = &mut io::split(pty);
    let (tcp_reader, mut tcp_writer) = io::split(tlsstream);

    let stdin = &mut AsyncFd::try_from(libc::STDIN_FILENO)?;
    let stdout = &mut AsyncFd::try_from(libc::STDOUT_FILENO)?;

    let mut input: Box<dyn AsyncRead + Unpin>;
    let read: Pin<Box<dyn Future<Output = Result<()>>>>;
    if readonly {
        input = Box::new(tcp_reader);
        read = Box::pin(async {
            let buf = &mut vec![0; 2048];
            while stdin.read(buf).await? > 0 {
                continue;
            }
            Ok(())
        });
    } else {
        input = Box::new(merge_reader(tcp_reader, stdin));
        read = Box::pin(pending());
    };

    let (sender, receiver) = &mut duplex(65536);

    let link1 = async {
        let buf = &mut vec![0; 2048];
        loop {
            match pty_reader.read(buf).await? {
                0 => return Ok(()),
                n => {
                    sender.write_all(&buf[..n]).await?;
                    tcp_writer.write_all(&buf[..n]).await?;
                }
            }
        }
    };

    let link2 = async { Ok(io::copy(&mut input, pty_writer).await.map(drop)?) };

    let echo = async { Ok(io::copy(receiver, stdout).await.map(drop)?) };

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
