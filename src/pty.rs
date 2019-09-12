use libc::{self, c_void};

use mio::{
    Evented,
    Poll,
    PollOpt,
    Ready,
    Token,
    unix::EventedFd
};

use std::ffi::OsStr;
use std::fs::File;
use std::io;
use std::os::unix::{io::*, process::CommandExt};
use std::process::{Child, Command, Stdio};
use std::ptr;

#[repr(C)]
#[derive(Copy, Clone, Debug)]
pub struct PtyFdPair {
    pub ptmx: RawFd,
    pub pts:  RawFd,
}

pub fn open() -> io::Result<PtyFdPair> {
    let mut ptmx = -1;
    let mut pts  = -1;

    let res = unsafe {
        libc::openpty(
            &mut ptmx,
            &mut pts,
            ptr::null_mut(),
            ptr::null(),
            ptr::null()
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
    pub pty: File
}

pub fn spawn_process(program: &str, args: &[&str]) -> io::Result<ProcessWithPty> {
    let fds = self::open()?;

    let mut command = Command::new(program);
            command.args(args)
                   .stdin( unsafe { Stdio::from_raw_fd(fds.pts) })
                   .stdout(unsafe { Stdio::from_raw_fd(fds.pts) })
                   .stderr(unsafe { Stdio::from_raw_fd(fds.pts) });

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
        use libc::{fcntl, F_GETFL, F_SETFL, O_NONBLOCK};

        let fl  = fcntl(fds.ptmx, F_GETFL, 0);
        let res = fcntl(fds.ptmx, F_SETFL, fl | O_NONBLOCK);

        assert_eq!(res, 0);
    }

    Ok(ProcessWithPty {
        process: child,
        pty: unsafe { File::from_raw_fd(fds.ptmx) }
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
