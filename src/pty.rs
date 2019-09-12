use libc::c_void;
use libc::{close, openpty};

use mio::unix::EventedFd;
use mio::Evented;
use mio::{Poll, PollOpt, Ready, Token};

use std::fs::File;
use std::io;
use std::io::Read;
use std::os::unix::io::{AsRawFd, RawFd};
use std::os::unix::{io::FromRawFd, process::CommandExt};
use std::process::{Child, Command, Stdio};
use std::ptr;

pub fn pty(program: &str, args: &[&str]) {
    let pty_pair = FdPtyPair::open_pty().unwrap();
    println!("ptmx {}, pts {}", pty_pair.ptmx, pty_pair.pts);
    let mut comm = pty_pair.spawn_process(program, args).unwrap();

    let size: usize = 16;

    let mut buff = vec![0; size];

    loop {
        match comm.io.read(&mut buff) {
            Ok(amount) => {
                let result = String::from_utf8_lossy(&buff[0..amount]);
                println!("{}", result);
                if amount != size {
                    println!("sleeping");
                    std::thread::sleep(std::time::Duration::from_millis(500));
                }
            }
            Err(_) => {
                println!("sleeping");
                std::thread::sleep(std::time::Duration::from_millis(500));
            }
        }
    }
}

struct FdPtyPair {
    pub ptmx: RawFd,
    pub pts: RawFd,
}

impl FdPtyPair {
    fn open_pty() -> Result<Self, ()> {
        let mut ptmx: RawFd = 0;
        let mut pts: RawFd = 0;
        let res = unsafe {
            openpty(
                &mut ptmx,
                &mut pts,
                ptr::null_mut(),
                ptr::null_mut(),
                ptr::null_mut(),
            )
        };
        if res == -1 {
            Err(())
        } else {
            Ok(FdPtyPair { ptmx, pts })
        }
    }
}

struct PtiedCommand {
    pub child: Child,
    pub io: File,
}

impl FdPtyPair {
    fn spawn_process(self, program: &str, args: &[&str]) -> std::io::Result<PtiedCommand> {
        let mut command = Command::new(program);
        command.args(args);
        command.stdin(unsafe { Stdio::from_raw_fd(self.pts) });
        command.stderr(unsafe { Stdio::from_raw_fd(self.pts) });
        command.stdout(unsafe { Stdio::from_raw_fd(self.pts) });

        let ptmx_fd = self.ptmx;

        unsafe {
            command.pre_exec(move || {
                let err = libc::setsid();
                if err == -1 {
                    return Err(std::io::Error::last_os_error());
                }
                close(self.ptmx);
                libc::ioctl(self.pts, libc::TIOCSCTTY, ptr::null::<c_void>());
                close(self.pts);
                Ok(())
            });
        }

        match command.spawn() {
            Ok(child) => {
                unsafe {
                    set_nonblocking(ptmx_fd);
                }
                Ok(PtiedCommand {
                    child,
                    io: unsafe { File::from_raw_fd(ptmx_fd) },
                })
            }
            Err(_) => Err(std::io::Error::last_os_error()),
        }
    }
}

impl Evented for PtiedCommand {
    fn register(
        &self,
        poll: &Poll,
        token: Token,
        interest: Ready,
        opts: PollOpt,
    ) -> io::Result<()> {
        EventedFd(&self.io.as_raw_fd()).register(poll, token, interest, opts)
    }

    fn reregister(
        &self,
        poll: &Poll,
        token: Token,
        interest: Ready,
        opts: PollOpt,
    ) -> io::Result<()> {
        EventedFd(&self.io.as_raw_fd()).reregister(poll, token, interest, opts)
    }

    fn deregister(&self, poll: &Poll) -> io::Result<()> {
        EventedFd(&self.io.as_raw_fd()).deregister(poll)
    }
}

// i'm trying stuff, got from alacritty, should try without ?
unsafe fn set_nonblocking(fd: RawFd) {
    use libc::{fcntl, F_GETFL, F_SETFL, O_NONBLOCK};

    let res = fcntl(fd, F_SETFL, fcntl(fd, F_GETFL, 0) | O_NONBLOCK);
    assert_eq!(res, 0);
}
