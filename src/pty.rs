use libc::c_int;
use libc::{close, openpty};

use std::fs::File;
use std::io::Read;
use std::os::unix::{io::FromRawFd, process::CommandExt};
use std::process::{Child, Command, Stdio};
use std::ptr;

pub fn pty() {
    let pty_pair = open_pty().unwrap();
    println!("master {}, slave {}", pty_pair.master, pty_pair.slave);
    let mut comm = pty_pair.prepare_process("ping", &["8.8.8.8"]).unwrap();

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
    pub master: c_int,
    pub slave: c_int,
}

struct PtiedCommand {
    pub child: Child,
    pub io: File,
}

impl FdPtyPair {
    fn prepare_process(self, program: &str, args: &[&str]) -> std::io::Result<PtiedCommand> {
        let mut command = Command::new(program);
        command.args(args);
        command.stdin(unsafe { Stdio::from_raw_fd(self.slave) });
        command.stderr(unsafe { Stdio::from_raw_fd(self.slave) });
        command.stdout(unsafe { Stdio::from_raw_fd(self.slave) });

        let master_fd = self.master;

        unsafe {
            command.pre_exec(move || {
                let err = libc::setsid();
                if err == -1 {
                    return Err(std::io::Error::last_os_error());
                }
                close(self.master);
                close(self.slave);
                libc::signal(libc::SIGCHLD, libc::SIG_DFL);
                libc::signal(libc::SIGHUP, libc::SIG_DFL);
                libc::signal(libc::SIGINT, libc::SIG_DFL);
                libc::signal(libc::SIGQUIT, libc::SIG_DFL);
                libc::signal(libc::SIGTERM, libc::SIG_DFL);
                libc::signal(libc::SIGALRM, libc::SIG_DFL);
                Ok(())
            });
        }

        match command.spawn() {
            Ok(child) => {
                unsafe {
                    set_nonblocking(master_fd);
                }
                Ok(PtiedCommand {
                    child,
                    io: unsafe { File::from_raw_fd(master_fd) },
                })
            }
            Err(_) => Err(std::io::Error::last_os_error()),
        }
    }
}

fn open_pty() -> Result<FdPtyPair, ()> {
    let mut master: c_int = 0;
    let mut slave: c_int = 0;
    let res = unsafe {
        openpty(
            &mut master,
            &mut slave,
            ptr::null_mut(),
            ptr::null_mut(),
            ptr::null_mut(),
        )
    };
    if res == -1 {
        Err(())
    } else {
        Ok(FdPtyPair { master, slave })
    }
}

// i'm trying stuff
unsafe fn set_nonblocking(fd: c_int) {
    use libc::{fcntl, F_GETFL, F_SETFL, O_NONBLOCK};

    let res = fcntl(fd, F_SETFL, fcntl(fd, F_GETFL, 0) | O_NONBLOCK);
    assert_eq!(res, 0);
}
