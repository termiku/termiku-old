//! Contains the data structures for controlling several terminals at once (for example, for
//! tabbing support)

use crate::pty::PtyWithProcess;
use crate::pty;
use crate::pty_buffer::PtyBuffer;
use crate::config::*;
use crate::rasterizer::*;

use mio::unix::EventedFd;
use mio::{Events, Poll, PollOpt, Ready, Token};
use mio_extras::channel::{channel, Sender};

use std::io;
use std::io::{Read, Write};
use std::os::unix::io::RawFd;
use std::sync::{Arc, RwLock};

// Input received from Window
const RECEIVER_TOKEN: usize = 0;
// Input received from stdin
const STDIN_TOKEN:    usize = 1;

// Raw file descriptor for stdin (POSIX)
// See 5th paragraph in man 3 stdin (http://man7.org/linux/man-pages/man3/stdin.3.html)
const STDIN_FD: RawFd = 0;

// 0 and 1 are reserved
const FIRST_TERMINAL_UID: usize = 2;

pub struct Term {
    /// The process and pseudoterminal descriptors for this terminal.
    pub pty: PtyWithProcess,
    
    /// Buffer of the associated pty
    pub buffer: PtyBuffer,
    
    /// Unique identifier for this terminal, supplied by the TermFactory.
    pub uid: usize,

    /*
    /// We may want to implement visual bells (\a / 0x07 / ^G), like flashing the tab.
    alerted: bool,
    /// We probably want to implement terminal title setting on way or another.
    title: String,
    */
}

type WrappedTermList = Arc<RwLock<TermList>>;

struct TermList {
    inner: Vec<Term>,
    active_index: usize,
    
    char_buffer: [u8; 4]
}

impl TermList {
    pub fn new() -> Self {
        Self {
            inner: vec![],
            active_index: 0,
            
            char_buffer: [0; 4]
        }
    }
    
    pub fn push(&mut self, term: Term) {
        self.inner.push(term);
    }
    
    pub fn push_and_make_active(&mut self, term: Term) {
        self.inner.push(term);
        self.active_index = self.inner.len() - 1;
    }
    
    pub fn find_index(&self, uid: usize) -> Option<usize> {
        self.inner.iter().position(|el| { el.uid == uid
        })
    }
    
    pub fn get(&self, index: usize) -> Option<&Term> {
        self.inner.get(index)
    }
    
    pub fn get_uid(&self, uid: usize) -> Option<&Term> {
        self.get(self.find_index(uid).unwrap())
    }
    
    pub fn get_active(&self) -> Option<&Term> {
        self.get(self.active_index)
    }
    
    pub fn get_mut(&mut self, index: usize) -> Option<&mut Term> {
        self.inner.get_mut(index)
    }
    
    pub fn get_uid_mut(&mut self, uid: usize) -> Option<&mut Term> {
        let index = self.find_index(uid).unwrap();
        self.inner.get_mut(index)
    }
    
    pub fn get_active_mut(&mut self) -> Option<&mut Term> {
        self.get_mut(self.active_index)
    }
    
    pub fn write_buffer_to_pty(&mut self, buffer: &[u8], index: usize) {
        self.get_mut(index).unwrap().pty.pty.write_all(buffer).unwrap()
    }
    
    pub fn write_buffer_to_active_pty(&mut self, buffer: &[u8]) {
        self.write_buffer_to_pty(buffer, self.active_index)
    }
    
    pub fn write_buffer_to_uid_pty(&mut self, buffer: &[u8], uid: usize) {
        self.write_buffer_to_pty(buffer, self.find_index(uid).unwrap())
    }

}

/// Manage a Termlist
pub struct TermManager {
    config: Config,
    factory: TermFactory,
    poll: Arc<Poll>,
    sender: Sender<char>,
    list: WrappedTermList,
}

impl TermManager {
    pub fn new(config: Config, rasterizer: WrappedRasterizer) -> Self {
        Self::setup();
        
        // Creates an mio::EventedFd for stdin
        let stdin = EventedFd(&STDIN_FD);
        
        // Creates a new mio::Poll
        let poll = Arc::new(Poll::new().unwrap());
        let mut events = Events::with_capacity(1024);
        
        // Channel used for receiving channel events
        let (sender, receiver) = channel::<char>();
        
        let termlist = Arc::new(RwLock::new(TermList::new()));
        
        // Register the receiver of Termiku's window input
        poll.register(
            &receiver,
            Token(RECEIVER_TOKEN),
            Ready::readable(),
            PollOpt::edge(),
        )
        .unwrap();
        
        // Regiter STDIN of the Termiku process
        poll.register(
            &stdin,
            Token(STDIN_TOKEN),
            Ready::readable(),
            PollOpt::edge(),
        )
        .unwrap();
        
        let cloned_poll = poll.clone();
        let cloned_termlist = termlist.clone();
        
        let mut buffer = [0; 256];
        let mut char_buffer = [0; 4];

        {
            std::thread::spawn(move || loop {                                
                cloned_poll.poll(&mut events, None).unwrap();
                for event in &events {
                    // This is a window event. We redirect to the active term
                    if event.token() == Token(RECEIVER_TOKEN) && event.readiness().is_readable() {
                        let mut handle = cloned_termlist.write().unwrap();
                        
                        while let Ok(input) = receiver.try_recv() {
                            handle.write_buffer_to_active_pty(input.encode_utf8(&mut char_buffer).as_bytes());
                        }
                    // This is input from the shell who started Termiku. We redirect to the active term
                    // We're leaving this to control the spawned process,
                    // but this should disappear eventually
                    } else if event.token() == Token(STDIN_TOKEN) && event.readiness().is_readable() {
                        let mut handle = cloned_termlist.write().unwrap();
                        
                        while let Ok(amount) = io::stdin().read(&mut buffer) {
                            handle.write_buffer_to_active_pty(&buffer[0..amount]);
                        }
                    // This is output from a pty.
                    // We write it to a PtyBuffer, and to Termiku's STDOUT
                    } else {
                        let uid = event.token().0;
                        
                        if event.readiness().is_readable() {
                            let mut handle = cloned_termlist.write().unwrap();
                            
                            let term = handle.get_uid_mut(uid).unwrap();
                            
                            let mut input: Vec<u8> = Vec::with_capacity(32);
                            
                            while let Ok(amount) = term.pty.pty.read(&mut buffer) {
                                print!("{}", String::from_utf8_lossy(&buffer[0..amount]));
                                io::stdout().flush().unwrap();
                                input.extend(&buffer[0..amount]);
                            }
                            
                            term.buffer.add_input(input)
                        }
                    }
                }
            });
        }
        
        let factory = TermFactory::new(config.clone(), rasterizer);
        
        let mut term_manager = Self {
            config,
            factory,
            poll,
            sender,
            list: termlist
        };
        
        term_manager.add_new_term();

        term_manager
    }
    
    fn setup() {
        //  Sets our stdin to be nonblocking (for later reading)
        unsafe {
            use libc::{F_GETFL, F_SETFL, O_NONBLOCK};

            let flags = libc::fcntl(STDIN_FD, F_GETFL, 0 /* should be ptr::null() but whateves */);
            let res   = libc::fcntl(STDIN_FD, F_SETFL, flags | O_NONBLOCK);

            assert_eq!(res, 0);
        }
    }
    
    pub fn add_new_term(&mut self) {
        let term = self.factory.make_term();
        
        self.poll.register(&term.pty, Token(term.uid), Ready::readable(), PollOpt::edge()).unwrap();
        
        {
            let mut list = self.list.write().unwrap(); 
            list.push(term);
        }
    }
    
    pub fn send_input(&mut self, input: char) {
        self.sender.send(input).unwrap();
    }
    
    pub fn get_lines_from_active(&mut self, start: usize, end: usize) -> Option<Vec<DisplayCellLine>> {
        let updated = {
            let list = self.list.read().unwrap();
            list.get_active().unwrap().buffer.is_updated()
        };
        
        if updated {
            Some(self.get_lines_from_active_force(start, end))
        } else {
            None
        }
    }
    
    pub fn get_lines_from_active_force(&mut self, start: usize, end: usize) -> Vec<DisplayCellLine> {
        let mut list = self.list.write().unwrap();
        list.get_active_mut().unwrap().buffer.get_range(start, end)
    }
    
    pub fn dimensions_updated(&mut self) {
        let mut list = self.list.write().unwrap();

        for term in list.inner.iter_mut() {
            term.buffer.dimensions_updated();
        }
    }
}

/// Creates sequentially numbered `Term`s for us so we don't need to rely on a global counter.
/// FIXME(Luna): Move PTY creation into TermFactory
pub struct TermFactory {
    rasterizer: WrappedRasterizer,
    config: Config,
    count: usize
}

impl TermFactory {
    pub fn new(config: Config, rasterizer: WrappedRasterizer) -> Self {
        TermFactory {
            config,
            rasterizer,
            count: FIRST_TERMINAL_UID
        }
    }

    /// Wraps a ProcessWithPty in a Term struct with a new uid.
    pub fn make_term(&mut self) -> Term {
        let pty = pty::spawn_process(
            &self.config.shell.program,
            self.config.shell.args.as_slice(),
            &self.config.env
        ).unwrap();
        
        let buffer = PtyBuffer::new(self.rasterizer.clone());

        if self.count == usize::max_value() {
            panic!("Exhausted Term UIds.");
        }

        let term = Term {
            pty,
            buffer,
            uid: self.count,
        };

        self.count += 1;
        term
    }
}

