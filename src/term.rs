use std::io::{Error, Result};
use std::mem::MaybeUninit;
use std::os::unix::io::RawFd;
use std::ptr::null_mut;

use libc::*;

use crate::util::AsPtr;

pub static mut PTY_MASTER: Option<c_int> = None;
static mut ORIGINAL_TERMIOS: Option<termios> = None;

fn get_winsize(fd: RawFd) -> Result<winsize> {
    let mut winsize = MaybeUninit::uninit();
    unsafe {
        if ioctl(fd, TIOCGWINSZ, winsize.as_mut_ptr()) != 0 {
            Err(Error::last_os_error())
        } else {
            Ok(winsize.assume_init())
        }
    }
}

fn set_winsize(fd: RawFd, winsize: &winsize) -> Result<()> {
    if unsafe { ioctl(fd, TIOCSWINSZ, winsize) } != 0 {
        Err(Error::last_os_error())
    } else {
        Ok(())
    }
}

fn get_termios(fd: RawFd) -> Result<termios> {
    let mut termios = MaybeUninit::uninit();
    unsafe {
        if tcgetattr(fd, termios.as_mut_ptr()) != 0 {
            Err(Error::last_os_error())
        } else {
            Ok(termios.assume_init())
        }
    }
}

fn set_termios(fd: RawFd, termios: &termios) -> Result<()> {
    if unsafe { tcsetattr(fd, TCSANOW, termios) } != 0 {
        Err(Error::last_os_error())
    } else {
        Ok(())
    }
}

pub fn fork_pty() -> Result<(c_int, pid_t)> {
    let mut amaster = 0;
    let (mut termios, mut winsize) = if unsafe { isatty(STDIN_FILENO) } == 1 {
        (
            Some(get_termios(STDIN_FILENO)?),
            Some(get_winsize(STDIN_FILENO)?),
        )
    } else {
        (None, None)
    };

    let pid = unsafe {
        forkpty(
            &mut amaster,
            null_mut(),
            termios.as_mut_ptr(),
            winsize.as_mut_ptr(),
        )
    };
    if pid < 0 {
        Err(Error::last_os_error())
    } else {
        Ok((amaster, pid))
    }
}

extern "C" fn sigwinch_handler(_signal: c_int) {
    if let Ok(winsize) = get_winsize(STDIN_FILENO) {
        if let Some(pty_master) = unsafe { PTY_MASTER } {
            let _ = set_winsize(pty_master, &winsize);
        }
    }
}

extern "C" fn sigchld_handler(_signal: c_int) {
    unsafe { exit(0) };
}

extern "C" fn sigint_handler(_signal: c_int) {
    unsafe { exit(0) };
}

fn register_signal_handler(signal: c_int, handler: extern "C" fn(c_int)) -> Result<()> {
    let mut act: sigaction = unsafe { MaybeUninit::zeroed().assume_init() };
    act.sa_sigaction = handler as sighandler_t;
    act.sa_flags = SA_RESTART;
    unsafe {
        if sigemptyset(&mut act.sa_mask) != 0 {
            return Err(Error::last_os_error());
        }
        if sigaction(signal, &act, null_mut()) != 0 {
            return Err(Error::last_os_error());
        }
    }
    Ok(())
}

pub fn restore_termios() -> Result<()> {
    if let Some(termios) = unsafe { ORIGINAL_TERMIOS.take() } {
        set_termios(STDIN_FILENO, &termios)
    } else {
        Ok(())
    }
}

extern "C" fn exit_handler() {
    let _ = restore_termios();
    println!("exited.");
}

pub fn set_exit_handler() -> Result<()> {
    if unsafe { libc::atexit(exit_handler) } != 0 {
        Err(Error::last_os_error())
    } else {
        Ok(())
    }
}

pub fn setup_terminal(fd: RawFd, isig: bool) -> Result<()> {
    if unsafe { isatty(STDIN_FILENO) } == 1 {
        enter_raw_mode(STDIN_FILENO, isig)?;

        let winsize = get_winsize(STDIN_FILENO)?;
        set_winsize(fd, &winsize)?;

        register_signal_handler(SIGWINCH, sigwinch_handler)?;
    }

    register_signal_handler(SIGCHLD, sigchld_handler)?;

    if isig {
        register_signal_handler(SIGINT, sigint_handler)?;
    }

    Ok(())
}

pub fn enter_raw_mode(fd: RawFd, isig: bool) -> Result<()> {
    let mut termios = get_termios(fd)?;
    unsafe {
        ORIGINAL_TERMIOS = Some(termios);
        cfmakeraw(&mut termios)
    };
    if isig {
        termios.c_lflag |= ISIG;
    }
    set_termios(fd, &termios)?;
    Ok(())
}
