//! Contains the data structures for controlling several terminals at once (for example, for
//! tabbing support)

use std::io::{self, Read, Write};
use std::os::unix::io::RawFd;
use std::sync::{Arc, RwLock};

use mio::{Events, Poll, PollOpt, Ready, Token, unix::EventedFd};
use mio_extras::channel::{channel, Sender};

use crate::config::*;
use crate::pty::{self, PtyWithProcess};
use crate::pty_buffer::{event::*, PtyBuffer};
use crate::rasterizer::*;
use crate::window_event::*;
use crate::youtube::*;

// Input received from Window
const RECEIVER_TOKEN: usize = 0;
// Input received from stdin
const STDIN_TOKEN:    usize = 1;
// Input received from a PryBuffer::Screen, mainly after receiving a control sequence
const SCREEN_TOKEN:   usize = 2;

// Raw file descriptor for stdin (POSIX)
// See 5th paragraph in man 3 stdin (http://man7.org/linux/man-pages/man3/stdin.3.html)
const STDIN_FD: RawFd = 0;

// 0, 1 and 2 are reserved
const FIRST_TERMINAL_UID: usize = 3;

pub struct Term {
    /// The process and pseudoterminal descriptors for this terminal.
    pub pty: PtyWithProcess,
    
    /// Buffer of the associated pty
    pub buffer: PtyBuffer,
    
    /// Unique identifier for this terminal, supplied by the TermFactory.
    pub uid: usize,
    
    pub youtube: Option<WrappedYoutubeDlVlcInstance>,
    
    /*
    /// We may want to implement visual bells (\a / 0x07 / ^G), like flashing the tab.
    alerted: bool,
    /// We probably want to implement terminal title setting on way or another.
    title: String,
    */
   
   pub to_remove: bool,
}

type WrappedTermList = Arc<RwLock<TermList>>;

struct TermList {
    inner: Vec<Term>,
    active_uid: usize,
    
    char_buffer: [u8; 4]
}

impl TermList {
    pub fn new() -> Self {
        Self {
            inner: vec![],
            active_uid: FIRST_TERMINAL_UID,
            
            char_buffer: [0; 4]
        }
    }
    
    pub fn push(&mut self, term: Term) {
        self.inner.push(term);
    }
    
    pub fn push_and_make_active(&mut self, term: Term) {
        self.active_uid = term.uid;
        self.inner.push(term);
    }
    
    pub fn find_index(&self, uid: usize) -> Option<usize> {
        self.inner.iter().position(|el| { el.uid == uid
        })
    }
    
    pub fn get(&self, index: usize) -> Option<&Term> {
        self.inner.get(index)
    }
    
    pub fn get_uid(&self, uid: usize) -> Option<&Term> {
        match self.find_index(uid) {
            Some(index) => self.get(index),
            None => None
        }
        
    }
    
    pub fn get_active(&self) -> Option<&Term> {
        self.get_uid(self.active_uid)
    }
    
    pub fn get_mut(&mut self, index: usize) -> Option<&mut Term> {
        self.inner.get_mut(index)
    }
    
    pub fn get_uid_mut(&mut self, uid: usize) -> Option<&mut Term> {
        match self.find_index(uid) {
            Some(index) => self.inner.get_mut(index),
            None => None
        }
    }
    
    pub fn get_active_mut(&mut self) -> Option<&mut Term> {
        self.get_uid_mut(self.active_uid)
    }
    
    pub fn write_buffer_to_pty(&mut self, buffer: &[u8], index: usize) {
        if let Some(term) = self.get_mut(index) {
            term.pty.pty.write_all(buffer).unwrap();
        }
    }
    
    pub fn write_buffer_to_uid_pty(&mut self, buffer: &[u8], uid: usize) {
        if let Some(index) = self.find_index(uid) {
            self.write_buffer_to_pty(buffer, index);
        }
    }
    
    pub fn write_buffer_to_active_pty(&mut self, buffer: &[u8]) {
        self.write_buffer_to_uid_pty(buffer, self.active_uid)
    }
    
    /// Cleanup every child that has exited.
    /// Returns the number of terminals inside inner after the cleanup.
    pub fn cleanup_exited_children(&mut self) -> usize {        
        for term in self.inner.iter_mut() {
            if let Ok(status) = term.pty.process.try_wait() {
                if status.is_some() {
                    term.to_remove = true;
                }
            }
        }
        
        self.inner.retain(|term| !term.to_remove);
        
        self.inner.len()
    }
}

/// Manage a Termlist
pub struct TermManager {
    config: Config,
    factory: TermFactory,
    poll: Arc<Poll>,
    screen_sender: Sender<ScreenEvent>,
    window_sender: Sender<TermikuWindowEvent>,
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
        
        // Channel for receiving pty_buffers's screen events
        let (screen_sender, screen_receiver) = channel::<ScreenEvent>();
        
        // Channel used for receiving window events
        let (window_sender, window_receiver) = channel::<TermikuWindowEvent>();
        
        let termlist = Arc::new(RwLock::new(TermList::new()));
        
        // Register the receiver of Termiku's window input
        poll.register(
            &window_receiver,
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
        
        poll.register(
            &screen_receiver,
            Token(SCREEN_TOKEN),
            Ready::readable(),
            PollOpt::edge()
        ).unwrap();
        
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
                        // Should panic if poisoned.
                        let mut handle = cloned_termlist.write().unwrap();
                        
                        while let Ok(event) = window_receiver.try_recv() {
                            handle_window_event(event, &mut handle, &mut char_buffer);
                        }
                    // This is input from the shell who started Termiku. We redirect to the active term
                    // We're leaving this to control the spawned process,
                    // but this should disappear eventually
                    } else if event.token() == Token(STDIN_TOKEN) && event.readiness().is_readable() {
                        // Should panic if poisoned.
                        let mut handle = cloned_termlist.write().unwrap();
                        
                        while let Ok(amount) = io::stdin().read(&mut buffer) {
                            handle.write_buffer_to_active_pty(&buffer[0..amount]);
                        }
                    // This is input from a screen
                    } else if event.token() == Token(SCREEN_TOKEN) && event.readiness().is_readable() {
                        // Should panic if poisoned.
                        let mut handle = cloned_termlist.write().unwrap();
                        
                        while let Ok(event) = screen_receiver.try_recv() {
                            handle_screen_event(event, &mut handle);
                        }
                    // This is output from a pty.
                    // We write it to a PtyBuffer, and to Termiku's STDOUT
                    } else {
                        let uid = event.token().0;
                        
                        if event.readiness().is_readable() {
                            // Should panic if poisoned.
                            let mut handle = cloned_termlist.write().unwrap();
                            
                            // Shouldn't panic, can just be that the term has exited and has been
                            // cleaned up.
                            if let Some(term) = handle.get_uid_mut(uid) {
                                let mut input: Vec<u8> = Vec::with_capacity(32);
                                
                                while let Ok(amount) = term.pty.pty.read(&mut buffer) {
                                    // println!("{}", String::from_utf8_lossy(&buffer[0..amount]));
                                    
                                    // Should panic if there's an error .
                                    io::stdout().flush().unwrap();
                                    input.extend(&buffer[0..amount]);
                                }
                                
                                term.buffer.add_input(input)
                            }
                            
                        }
                    }
                }
            });
        }
        
        let factory = TermFactory::new(config.clone(), rasterizer, screen_sender.clone());
        
        let mut term_manager = Self {
            config,
            factory,
            poll,
            screen_sender,
            window_sender,
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
            // Should panic if poisoned.
            let mut list = self.list.write().unwrap(); 
            list.push(term);
        }
    }
    
    pub fn send_event(&mut self, event: TermikuWindowEvent) {
        // Should panic if poisoned.
        self.window_sender.send(event).unwrap();
    }
    
    pub fn get_lines_from_active(&mut self, start: usize, end: usize) -> Option<Vec<DisplayCellLine>> {
        let updated = {
            // Should panic if poisoned.
            let list = self.list.read().unwrap();
            
            match list.get_active() {
                Some(term) => term.buffer.is_updated(),
                // We shouldn't try to update something that doesn't exist anymore.
                None => false
            }
        };
        
        if updated {
            Some(self.get_lines_from_active_force(start, end))
        } else {
            None
        }
    }
    
    pub fn get_lines_from_active_force(&mut self, start: usize, end: usize) -> Vec<DisplayCellLine> {
        // Should panic if poisoned.
        let mut list = self.list.write().unwrap();
        
        match list.get_active_mut() {
            Some(term) => term.buffer.get_range(start, end),
            None => vec![]
        }
    }
    
    pub fn get_youtube_frame_from_active(&mut self) -> Option<Vec<u8>> {
        let list = self.list.write().unwrap();
        
        match list.get_active() {
            Some(term) => match &term.youtube {
                Some(youtube) => {
                    let mut youtube_handle = youtube.0.lock().unwrap();
                    Some(youtube_handle.player.get_frame())
                }
                None => None
            },
            None => None
        }
    }
    
    pub fn is_active_updated(&self) -> bool {
        // Should panic if poisoned.
        let list = self.list.read().unwrap();
        
        match list.get_active() {
            Some(term) => term.buffer.is_updated(),
            None => false
        }
    }
    
    pub fn dimensions_updated(&mut self) {
        // Should panic if poisoned.
        let mut list = self.list.write().unwrap();

        for term in list.inner.iter_mut() {
            term.buffer.dimensions_updated();
        }
    }
    
    /// Cleanup every exited terminals.
    /// Return if the window should exit (i.e. there's no more terminals to display).
    pub fn cleanup_exited_terminals(&mut self) -> bool {
        // Should panic if poisoned.
        let mut list = self.list.write().unwrap();
        
        list.cleanup_exited_children() == 0
    }
}

/// Creates sequentially numbered `Term`s for us so we don't need to rely on a global counter.
/// FIXME(Luna): Move PTY creation into TermFactory
pub struct TermFactory {
    rasterizer: WrappedRasterizer,
    config: Config,
    count: usize,
    sender: mio_extras::channel::Sender<ScreenEvent>
}

impl TermFactory {
    pub fn new(config: Config, rasterizer: WrappedRasterizer, sender: mio_extras::channel::Sender<ScreenEvent>) -> Self {
        TermFactory {
            config,
            rasterizer,
            count: FIRST_TERMINAL_UID,
            sender
        }
    }

    /// Wraps a ProcessWithPty in a Term struct with a new uid.
    pub fn make_term(&mut self) -> Term {
        if self.count == usize::max_value() {
            panic!("Exhausted Term UIds.");
        }
        
        let pty = pty::spawn_process(
            &self.config.shell.program,
            self.config.shell.args.as_slice(),
            &self.config.env,
            self.rasterizer.read().unwrap().get_winsize()
        ).unwrap();
        
        let buffer = PtyBuffer::new(self.rasterizer.clone(), self.sender.clone(), self.count);

        
        let term = Term {
            pty,
            youtube: None,
            buffer,
            uid: self.count,
            to_remove: false,
        };

        self.count += 1;
        term
    }
}

fn handle_window_event(event: TermikuWindowEvent, termlist: &mut TermList, char_buffer: &mut [u8]) {
    use TermikuWindowEvent::*;
    
    match event {
        CharacterInput(character) => termlist.write_buffer_to_active_pty(character.encode_utf8(char_buffer).as_bytes()),
        KeyboardArrow(arrow) => termlist.write_buffer_to_active_pty(arrow.to_control_sequence().as_bytes()),
    }
}

fn handle_screen_event(event: ScreenEvent, termlist: &mut TermList) {
    use ScreenEventType::*;
    
    match event.event {
        PlayYoutubeVideo(video_id) => {
            let term = termlist.get_uid_mut(event.terminal_id);   
            if let Some(term) = term {
                if let Some(youtube) = term.youtube.take() {
                    youtube.0.lock().unwrap().cleanup();
                }
                
                let ytdl = YoutubeDlInstance::new(&video_id);
                
                term.youtube = Some(
                    WrappedYoutubeDlVlcInstance::new(ytdl)
                )
            }
        }
    }
}
