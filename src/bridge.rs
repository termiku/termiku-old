use crate::pty;
use mio::unix::EventedFd;
use mio::{Evented, Events, Poll, PollOpt, Ready, Token};
use mio_extras::channel::{channel, Sender};
use std::fs::File;
use std::io;
use std::io::{Read, Write};
use std::os::unix::io::{AsRawFd, FromRawFd};

const RECEIVER_TOKEN: usize = 0;
const STDIN_TOKEN: usize = 1;
const PROCESS_TOKEN: usize = 2;

pub fn spawn_process(program: &str, args: &[&str]) -> Sender<char> {
    unsafe {
        pty::set_nonblocking(0);
    }
    let mut stdin = EventedStdin::new();
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
                process_stdin(&mut stdin.stdin, &mut comm.pty, &mut buffer);
            } else if event.token() == Token(PROCESS_TOKEN) && event.readiness().is_readable() {
                read_and_print(&mut comm.pty, &mut buffer);
            }
        }
    });
    sender
}

fn process_input(input: char, ptmx: &mut File, buffer: &mut [u8]) {
    ptmx.write_all(input.encode_utf8(buffer).as_bytes())
        .unwrap();
}

fn process_stdin(stdin: &mut File, ptmx: &mut File, buffer: &mut [u8]) {
    while let Ok(amount) = stdin.read(buffer) {
        ptmx.write_all(&buffer[0..amount]).unwrap();
    }
}

fn read_and_print(file: &mut File, buffer: &mut [u8]) {
    while let Ok(amount) = file.read(buffer) {
        print!("{}", String::from_utf8_lossy(&buffer[0..amount]));
    }
}

struct EventedStdin {
    pub stdin: File,
}

impl EventedStdin {
    fn new() -> Self {
        Self {
            stdin: unsafe { File::from_raw_fd(0) },
        }
    }
}

impl Evented for EventedStdin {
    fn register(
        &self,
        poll: &Poll,
        token: Token,
        interest: Ready,
        opts: PollOpt,
    ) -> io::Result<()> {
        EventedFd(&self.stdin.as_raw_fd()).register(poll, token, interest, opts)
    }

    fn reregister(
        &self,
        poll: &Poll,
        token: Token,
        interest: Ready,
        opts: PollOpt,
    ) -> io::Result<()> {
        EventedFd(&self.stdin.as_raw_fd()).reregister(poll, token, interest, opts)
    }

    fn deregister(&self, poll: &Poll) -> io::Result<()> {
        EventedFd(&self.stdin.as_raw_fd()).deregister(poll)
    }
}
