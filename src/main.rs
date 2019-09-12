use libc;
use termiku::pty;

use std::ffi::CStr;
use std::io::prelude::*;
use std::os::unix::io::*;

fn main() {
    let mut child = pty::spawn_process("ping", &["127.0.0.1"]).unwrap();

    unsafe {
        let ptmx = child.pty.as_raw_fd();
        let ptsn = libc::ptsname(ptmx);
        let cstr = CStr::from_ptr(ptsn);
        println!("Spawned process on {}", cstr.to_str().unwrap());
    }

    let mut buf = vec![0; 256];

    loop {
        match child.pty.read(&mut buf) {
            Ok(nread) => {
                let out = String::from_utf8_lossy(&buf[0..nread]);
                print!("{}", out);
            }
            Err(_) => {
                println!("sleeping...");
                std::thread::sleep(std::time::Duration::from_millis(500));
            }
        }
    }
}
