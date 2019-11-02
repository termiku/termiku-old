// mod termlist
//! Contains the data structures for controlling several terminals at once (for example, for
//! tabbing support)

use crate::pty::ProcessWithPty as PTY;

pub struct Term {
    /// The process and pseudoterminal descriptors for this terminal.
    pty: PTY,
    /// Unique identifier for this terminal, supplied by the TermFactory.
    uid: u32,

    /*
    /// We may want to implement visual bells (\a / 0x07 / ^G), like flashing the tab.
    alerted: bool,
    /// We probably want to implement terminal title setting on way or another.
    title: String,
    */
}

/// Wraps a list of `Term`s.
/// FIXME: Would this work as a single-field tuple struct?
/// FIXME: This is a bunch of jank because I'm not 100% sure how we wanna design access to this later.
/// FIXME: For now we just add a bunch of convenience methods and let the inner vec be accessible.
pub struct TermList {
    pub inner: Vec<Term>
}

impl TermList {
    fn get_by_uid(&self, uid: u32) -> Option<&Term> {
        self.inner.iter()
                  .find(|&t| t.uid == uid)
    }

    fn get_by_index(&self, index: u32) -> Option<&Term> {
        self.inner.iter()
                  .find(|&t| t.index == index)
    }

    fn get_by_uid_mut(&mut self, uid: u32) -> Option<&mut Term> {
        self.inner.iter_mut()
                  .find(|&t| t.uid == uid)
    }

    fn get_by_index_mut(&mut self, index: u32) -> Option<&mut Term> {
        self.inner.iter_mut()
                  .find(|&t| t.index == index)
    }
}

/// Creates sequentially numbered `Term`s for us so we don't need to rely on a global counter.
/// FIXME(Luna): Move PTY creation into TermFactory
pub struct TermFactory {
    count: u32
}

impl TermFactory {
    pub fn new() -> Self {
        // 0 is reserved (ShinySaana)
        TermFactory {
            count: 1
        }
    }

    /// Wraps a ProcessWithPty in a Term struct with a new uid.
    pub fn make_term(&mut self, pty: PTY) -> Term {
        if self.count == u32::max_value() {
            panic!("Exhausted Term UIds.");
        }

        let term = Term {
            pty,

            uid: self.count,
        }

        self.count += 1;
        term
    }
}


