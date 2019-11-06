use libc;

use std::io;
use std::mem;
use std::os::unix::{io::*, process::CommandExt};
use std::process::{Child, Command, Stdio};
use std::ptr;

/// A pair of `RawFds` referring to the pseudoterminal multiplexer device (ptmx), and the associated pseudoterminal sub-device (pts).
#[derive(Clone, Copy, Debug)]
pub struct RawPtyFds {
    pub ptmx: RawFd,
    pub pts:  RawFd,
}

/// A pseudoterminal (sometimes abbreviated "pty") is a pair of virtual character devices that provide a bidirectional communication channel.
///
/// For more information, see, the Linux manual pages [pty(7)] and [pts(4)].
/// 
/// [pty(7)]: http://man7.org/linux/man-pages/man7/pty.7.html
/// [pts(4)]: http://man7.org/linux/man-pages/man4/pts.4.html
#[derive(Debug)]
pub struct Pty {
    fds: RawPtyFds
}

impl Pty {
    /// Opens a new pseudoterminal.
    /// The descriptor for the ptmx device is additionally set to be nonblocking.
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
    /// See also the [`std::os::unix::io::AsRawFd` trait.][AsRawFd]
    /// 
    /// [AsRawFd]: https://doc.rust-lang.org/std/os/unix/io/trait.AsRawFd.html
    pub fn raw_ptmx_fd(&self) -> RawFd {
        self.fds.ptmx
    }

    /// Returns the `RawFd` for this pseudoterminal's pts device.
    /// This does not transfer ownership of the file descriptor to the caller.
    /// 
    /// See also the [`std::os::unix::io::AsRawFd` trait.][AsRawFd]
    /// 
    /// [AsRawFd]: https://doc.rust-lang.org/std/os/unix/io/trait.AsRawFd.html
    pub fn raw_pts_fd(&self) -> RawFd {
        self.fds.pts
    }

    /// Returns the `RawFd` pair for this pseudoterminal.
    /// This does not transfer ownership of the file descriptors to the caller.
    ///
    /// See also the [`std::os::unix::io::AsRawFd` trait.][AsRawFd]
    /// 
    /// [AsRawFd]: https://doc.rust-lang.org/std/os/unix/io/trait.AsRawFd.html
    pub fn as_raw_fds(&self) -> RawPtyFds {
        self.fds
    }

    /// Consumes this pseudoterminal, returning its `RawFd` pair.
    /// This transfers ownership of the file descriptors to the caller.
    /// 
    /// See also the [`std::os::unix::io::IntoRawFd` trait.][IntoRawFd]
    /// 
    /// [IntoRawFd]: https://doc.rust-lang.org/std/os/unix/io/trait.IntoRawFd.html
    pub fn into_raw_fds(self) -> RawPtyFds {
        let fds = self.fds;
        mem::forget(self);
        fds
    }
}

impl io::Read for Pty {
    /// Reads data directly from the ptmx descriptor for this pseudoterminal.
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
    /// Writes data directly to the ptmx descriptor for this pseudoterminal.
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

    /// This function is a no-op as the pseudoterminal never buffers any data.
    fn flush(&mut self) -> io::Result<()> {
        Ok(())
    }
}

use std::fmt;

impl fmt::Display for Pty {
    /// Shows the path for this pseudoterminal's pts device.
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        use std::ffi::CStr;

        unsafe {
            let ptsname = libc::ptsname(self.fds.ptmx);
            let cstr    = CStr::from_ptr(ptsname)
                               .to_str()
                               .unwrap();
            f.write_str(cstr)
        }
    }
}

impl Drop for Pty {
    /// Closes the pseudoterminal.
    /// 
    /// No error checks are performed for the same reason Rust itself doesn't:
    /// *"[...] if an error occurs we don't actually know if he file descriptor was closed or not"*
    /// See [the implementation for file descriptors in Rust.][github]
    /// 
    /// [github]: https://github.com/rust-lang/rust/blob/6de4924b6c1ff5a99397ca1a3894c51f085f3e6f/src/libstd/sys/unix/fd.rs#L274.
    fn drop(&mut self) {
        unsafe {
            libc::close(self.fds.ptmx);
            libc::close(self.fds.pts);
        }
    }
}

use mio::{unix::EventedFd, Evented, Poll, PollOpt, Ready, Token};


// FIXME(LunarLambda) Stuff below should probably be moved into a process module.

pub struct PtyWithProcess {
    pub pty: Pty,
    pub process: Child,
}

pub fn spawn_process(program: &str, args: &[&str]) -> io::Result<PtyWithProcess> {
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

    Ok(PtyWithProcess {
        process: child,
        pty,
    })
}

impl Evented for PtyWithProcess {
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