use crate::pty;
use mio::{Events, Poll, PollOpt, Ready, Token};
use std::fs::File;
use std::io::Read;

pub fn spawn_process(program: &str, args: &[&str]) {
    let mut comm = pty::spawn_process(program, args).unwrap();

    let poll = Poll::new().unwrap();
    let mut events = Events::with_capacity(1024);
    poll.register(&comm, Token(0), Ready::readable(), PollOpt::edge())
        .unwrap();
    let mut buffer = [0; 256];

    loop {
        poll.poll(&mut events, None).unwrap();
        for event in &events {
            if event.token() == Token(0) && event.readiness().is_readable() {
                read_and_print(&mut comm.pty, &mut buffer);
            }
        }
    }
}

fn read_and_print(file: &mut File, buffer: &mut [u8]) {
    while let Ok(amount) = file.read(buffer) {
        print!("{}", String::from_utf8_lossy(&buffer[0..amount]));
    }
}
