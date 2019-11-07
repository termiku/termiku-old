use crate::pty::{self, Pty};

use mio::unix::EventedFd;
use mio::{Events, Poll, PollOpt, Ready, Token};
use mio_extras::channel::{channel, Sender};

use std::io;
use std::io::{Read, Write};
use std::os::unix::io::RawFd;

// Input received from Window
const RECEIVER_TOKEN: usize = 0;
// Input received from stdin
const STDIN_TOKEN:    usize = 1;
// Output received from Pty
const PROCESS_TOKEN:  usize = 2;

// Raw file descriptor for stdin (POSIX)
// See 5th paragraph in man 3 stdin (http://man7.org/linux/man-pages/man3/stdin.3.html)
const STDIN_FD: RawFd = 0;
// const STDOUT_FD: RawFd = 1;
// const STDERR_FD: RawFd = 2;

/// Create a Pty with Process attached and set up an mio event loop.
/// 
/// This function:
/// 1. Sets our stdin to be nonblocking (for later reading)
/// 2. Creates an mio::EventedFd for stdin
/// 3. Creates a Pty and a Process attached to it
/// 4. Creates a new mio::Poll
/// 5. Creates a new mio::channel::<char>
/// 6. Registers EventedFd, Pty, and channel for reading (edge-triggered)
/// 7. Creates a new thread with the event loop running inside
/// 8. Returns the Sender part of the channel created in *5.*
pub fn spawn_process(program: &str, args: &[&str]) -> Sender<char> {
    // 1.
    unsafe {
        use libc::{F_GETFL, F_SETFL, O_NONBLOCK};

        let flags = libc::fcntl(STDIN_FD, F_GETFL, 0 /* should be ptr::null() but whateves */);
        let res   = libc::fcntl(STDIN_FD, F_SETFL, flags | O_NONBLOCK);

        assert_eq!(res, 0);
    }

    // 2.
    let stdin = EventedFd(&STDIN_FD);

    // 3.
    let mut comm = pty::spawn_process(program, args).unwrap();

    // 4.
    let poll = Poll::new().unwrap();
    let mut events = Events::with_capacity(1024);

    // 5.
    let (sender, receiver) = channel::<char>();

    // 6.
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

    // 7.
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

    // 8.
    sender
}

// Handles events from window (RECEIVER_TOKEN)
fn process_input(input: char, ptmx: &mut Pty, buffer: &mut [u8]) {
    ptmx.write_all(input.encode_utf8(buffer).as_bytes())
        .unwrap();
}

// Handles events from stdin (STDIN_TOKEN)
fn process_stdin(ptmx: &mut Pty, buffer: &mut [u8]) {
    while let Ok(amount) = io::stdin().read(buffer) {
        ptmx.write_all(&buffer[0..amount]).unwrap();
    }
}

// Handles events from Pty (PROCESS_TOKEN)
fn read_and_print(pty: &mut Pty, buffer: &mut [u8]) {
    while let Ok(amount) = pty.read(buffer) {
        print!("{}", String::from_utf8_lossy(&buffer[0..amount]));
        io::stdout().flush().unwrap();
    }
}
