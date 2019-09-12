use libc::{self, c_void};

use mio::{unix::EventedFd, Evented, Poll, PollOpt, Ready, Token};

use std::fs::File;
use std::io;
use std::os::unix::{io::*, process::CommandExt};
use std::process::{Child, Command, Stdio};
use std::ptr;

#[repr(C)]
#[derive(Copy, Clone, Debug)]
pub struct PtyFdPair {
    pub ptmx: RawFd,
    pub pts: RawFd,
}

pub fn open() -> io::Result<PtyFdPair> {
    let mut ptmx = -1;
    let mut pts = -1;

    let res = unsafe {
        libc::openpty(
            &mut ptmx,
            &mut pts,
            ptr::null_mut(),
            ptr::null(),
            ptr::null(),
        )
    };

    if res == -1 {
        Err(io::Error::last_os_error())
    } else {
        Ok(PtyFdPair { ptmx, pts })
    }
}

pub struct ProcessWithPty {
    pub process: Child,
    pub pty: File,
}

pub fn spawn_process(program: &str, args: &[&str]) -> io::Result<ProcessWithPty> {
    let fds = self::open()?;

    let mut command = Command::new(program);
    command
        .args(args)
        .stdin(unsafe { Stdio::from_raw_fd(fds.pts) })
        .stdout(unsafe { Stdio::from_raw_fd(fds.pts) })
        .stderr(unsafe { Stdio::from_raw_fd(fds.pts) });
    set_envs(&mut command);

    unsafe {
        command.pre_exec(move || {
            let err = libc::setsid();

            if err == -1 {
                return Err(io::Error::last_os_error());
            }

            let err = libc::ioctl(fds.pts, libc::TIOCSCTTY, ptr::null::<c_void>());

            if err == -1 {
                return Err(io::Error::last_os_error());
            }

            libc::close(fds.ptmx);
            libc::close(fds.pts);
            Ok(())
        });
    }

    let child = command.spawn()?;

    unsafe {
        set_nonblocking(fds.ptmx);
    }

    Ok(ProcessWithPty {
        process: child,
        pty: unsafe { File::from_raw_fd(fds.ptmx) },
    })
}

impl Evented for ProcessWithPty {
    fn register(
        &self,
        poll: &Poll,
        token: Token,
        interest: Ready,
        opts: PollOpt,
    ) -> io::Result<()> {
        EventedFd(&self.pty.as_raw_fd()).register(poll, token, interest, opts)
    }

    fn reregister(
        &self,
        poll: &Poll,
        token: Token,
        interest: Ready,
        opts: PollOpt,
    ) -> io::Result<()> {
        EventedFd(&self.pty.as_raw_fd()).reregister(poll, token, interest, opts)
    }

    fn deregister(&self, poll: &Poll) -> io::Result<()> {
        EventedFd(&self.pty.as_raw_fd()).deregister(poll)
    }
}

fn set_envs(command: &mut Command) {
    command.env("TERM", "dumb");
}

// Got from alacritty, not sure if the impl is correct
// (propably) makes the fd non blocking, so that it can returns "WouldBlock" when there's no more data
// This is needed so that when polling the fd with mio, we know when to stop
pub unsafe fn set_nonblocking(fd: RawFd) {
    use libc::{fcntl, F_GETFL, F_SETFL, O_NONBLOCK};

    let fl = fcntl(fd, F_GETFL, 0);
    let res = fcntl(fd, F_SETFL, fl | O_NONBLOCK);

    assert_eq!(res, 0);
}
