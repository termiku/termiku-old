use crate::pty::{self, Pty};
use mio::unix::EventedFd;
use mio::{Events, Poll, PollOpt, Ready, Token};
use mio_extras::channel::{channel, Sender};
use std::io;
use std::io::{Read, Write};
use std::os::unix::io::RawFd;

const RECEIVER_TOKEN: usize = 0;
const STDIN_TOKEN: usize = 1;
const PROCESS_TOKEN: usize = 2;

const STDIN_FD: RawFd = 0;

pub fn spawn_process(program: &str, args: &[&str]) -> Sender<char> {
    // Set stdin to be nonblocking (which doesn't actually affect epoll's behavior...)
    // This will disappear later anyway.
    unsafe {
        use libc::{F_GETFL, F_SETFL, O_NONBLOCK};

        let flags = libc::fcntl(STDIN_FD, F_GETFL, 0 /* should be ptr::null() but whateves */);
        let res   = libc::fcntl(STDIN_FD, F_SETFL, flags | O_NONBLOCK);

        assert_eq!(res, 0);
    }

    // Replaced EventedStdin with EventedFd
    let mut stdin = EventedFd(&STDIN_FD);
    let mut comm = pty::spawn_process(program, args).unwrap();
    let poll = Poll::new().unwrap();
    let mut events = Events::with_capacity(1024);
    let (sender, receiver) = channel::<char>();

    poll.register(
        &receiver,
        Token(RECEIVER_TOKEN),
        Ready::readable(),
        PollOpt::edge(),
    )
    .unwrap();
    poll.register(
        &stdin,
        Token(STDIN_TOKEN),
        Ready::readable(),
        PollOpt::edge(),
    )
    .unwrap();
    poll.register(
        &comm,
        Token(PROCESS_TOKEN),
        Ready::readable(),
        PollOpt::edge(),
    )
    .unwrap();
    let mut buffer = [0; 256];
    let mut char_buffer = [0; 4];

    read_and_print(&mut comm.pty, &mut buffer);

    std::thread::spawn(move || loop {
        poll.poll(&mut events, None).unwrap();
        for event in &events {
            if event.token() == Token(RECEIVER_TOKEN) && event.readiness().is_readable() {
                while let Ok(input) = receiver.try_recv() {
                    process_input(input, &mut comm.pty, &mut char_buffer);
                }
            } else if event.token() == Token(STDIN_TOKEN) && event.readiness().is_readable() {
                // We're leaving this to control the spawned process,
                // but this should disappear eventually
                process_stdin(&mut comm.pty, &mut buffer);
            } else if event.token() == Token(PROCESS_TOKEN) && event.readiness().is_readable() {
                read_and_print(&mut comm.pty, &mut buffer);
            }
        }
    });
    sender
}

fn process_input(input: char, ptmx: &mut Pty, buffer: &mut [u8]) {
    ptmx.write_all(input.encode_utf8(buffer).as_bytes())
        .unwrap();
}

fn process_stdin(ptmx: &mut Pty, buffer: &mut [u8]) {
    while let Ok(amount) = io::stdin().read(buffer) {
        ptmx.write_all(&buffer[0..amount]).unwrap();
    }
}

fn read_and_print(pty: &mut Pty, buffer: &mut [u8]) {
    while let Ok(amount) = pty.read(buffer) {
        print!("{}", String::from_utf8_lossy(&buffer[0..amount]));
    }
}
