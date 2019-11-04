use libc;

use mio::{unix::EventedFd, Evented, Poll, PollOpt, Ready, Token};

use std::io;
use std::mem;
use std::os::unix::{io::*, process::CommandExt};
use std::process::{Child, Command, Stdio};
use std::ptr;

/// A pair of `RawFds` referring to the pseudoterminal multiplexer device (ptmx), and its associated pseudoterminal sub-device (pts).
#[derive(Clone, Copy, Debug)]
pub struct RawPtyFds {
    pub ptmx: RawFd,
    pub pts:  RawFd,
}

/// A Pseudoterminal.
pub struct Pty {
    fds: RawPtyFds
}

impl Pty {
    /// Opens a new pseudoterminal.
    /// The created file descriptors have the flags `O_RDWR`, `O_NOCTTY`.
    /// The descriptor for the multiplexer device additionally has `O_NONBLOCK` set.
    pub fn open() -> io::Result<Pty> {
        let mut ptmx = -1;
        let mut pts  = -1;

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
            unsafe {
                use libc::{c_void, F_GETFL, F_SETFL, O_NONBLOCK};

                let flags = libc::fcntl(ptmx, F_GETFL, ptr::null::<c_void>());
                let res   = libc::fcntl(ptmx, F_SETFL, flags | O_NONBLOCK);

                if res == -1 {
                    Err(io::Error::last_os_error())
                } else {
                    Ok(Pty { fds: { RawPtyFds { ptmx, pts }}})
                }
            }
        }
    }

    /// Returns the `RawFd` for this pseudoterminal's ptmx device.
    /// This does not transfer ownership of the file descriptor to the caller.
    /// 
    /// See also the `std::os::unix::io::AsRawFd` trait.
    pub fn raw_ptmx_fd(&self) -> RawFd {
        self.fds.ptmx
    }

    /// Returns the `RawFd` for this pseudoterminal's pts device.
    /// This does not transfer ownership of the file descriptor to the caller.
    /// 
    /// See also the `std::os::unix::io::AsRawFd` trait.
    pub fn raw_pts_fd(&self) -> RawFd {
        self.fds.pts
    }

    /// Returns the `RawFd` pair for this pseudoterminal.
    /// This does not transfer ownership of the file descriptors to the caller.
    ///
    /// See also the `std::os::unix::io::AsRawFd` trait.
    pub fn as_raw_fds(&self) -> RawPtyFds {
        self.fds
    }

    /// Consumes this pseudoterminal, returning its `RawFd` pair.
    /// This transfers ownership of the file descriptors to the caller.
    /// 
    /// See also the `std::os::unix::io::IntoRawFd` trait.
    pub fn into_raw_fds(self) -> RawPtyFds {
        let fds = self.fds;
        mem::forget(self);
        fds
    }
}

impl io::Read for Pty {
    fn read(&mut self, buf: &mut [u8]) -> io::Result<usize> {
        let res = unsafe {
            libc::read(self.fds.ptmx, buf.as_mut_ptr() as _, buf.len())
        };

        if res == -1 {
            Err(io::Error::last_os_error())
        } else {
            Ok(res as usize)
        }
    }
}

impl io::Write for Pty {
    fn write(&mut self, buf: &[u8]) -> io::Result<usize> {
        let res = unsafe {
            libc::write(self.fds.ptmx, buf.as_ptr() as _, buf.len())
        };

        if res == -1 {
            Err(io::Error::last_os_error())
        } else {
            Ok(res as usize)
        }
    }

    /// Always returns Ok because we don't have any buffered data to flush.
    fn flush(&mut self) -> io::Result<()> {
        Ok(())
    }
}

impl Drop for Pty {
    /// Closes the pseudoterminal.
    /// 
    /// We do not check for errors for the same reason Rust itself doesn't:
    /// "[...] if an error occurs we don't actually know if he file descriptor was closed or not"
    /// See https://github.com/rust-lang/rust/blob/6de4924b6c1ff5a99397ca1a3894c51f085f3e6f/src/libstd/sys/unix/fd.rs#L274.
    fn drop(&mut self) {
        unsafe {
            libc::close(self.fds.ptmx);
            libc::close(self.fds.pts);
        }
    }
}

// FIXME(LunarLambda) Stuff below should probably be moved into a process module.

pub struct ProcessWithPty {
    pub process: Child,
    pub pty: Pty,
}

pub fn spawn_process(program: &str, args: &[&str]) -> io::Result<ProcessWithPty> {
    let pty = Pty::open()?;
    let fds = pty.as_raw_fds();

    let mut command = Command::new(program);

    command
        .args(args)
        .stdin( unsafe { Stdio::from_raw_fd(fds.pts) })
        .stdout(unsafe { Stdio::from_raw_fd(fds.pts) })
        .stderr(unsafe { Stdio::from_raw_fd(fds.pts) });

    set_envs(&mut command);

    unsafe {
        command.pre_exec(move || {
            use libc::{c_void, TIOCSCTTY};

            let res = libc::setsid();

            if  res == -1 {
                return Err(io::Error::last_os_error());
            }

            let res = libc::ioctl(fds.pts, TIOCSCTTY, ptr::null::<c_void>());

            if  res == -1 {
                return Err(io::Error::last_os_error());
            }

            libc::close(fds.ptmx);
            libc::close(fds.pts);

            Ok(())
        });
    }

    let child = command.spawn()?;

    Ok(ProcessWithPty {
        process: child,
        pty,
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
        EventedFd(&self.pty.raw_ptmx_fd()).register(poll, token, interest, opts)
    }

    fn reregister(
        &self,
        poll: &Poll,
        token: Token,
        interest: Ready,
        opts: PollOpt,
    ) -> io::Result<()> {
        EventedFd(&self.pty.raw_ptmx_fd()).reregister(poll, token, interest, opts)
    }

    fn deregister(&self, poll: &Poll) -> io::Result<()> {
        EventedFd(&self.pty.raw_ptmx_fd()).deregister(poll)
    }
}

fn set_envs(command: &mut Command) {
    command.env("TERM", "dumb");
}