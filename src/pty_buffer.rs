use crate::rasterizer::*;

use std::collections::VecDeque;

// Group of character to be rendered, with probably in the future options to apply to them
#[derive(Debug, Clone)]
pub struct CharacterGroup {
    // maybe try to use &str?
    pub characters: String
}

impl CharacterGroup {
    pub fn with_capacity(capacity: usize) -> CharacterGroup {
        Self {
            characters: String::with_capacity(capacity)
        }
    }
    
    pub fn from_string(content: String) -> CharacterGroup {
        Self {
            characters: content
        }
    }
}

// Logical line, as in "here's a line with only one life feed at the end", as expected for the user
#[derive(Debug, Clone)]
pub struct CharacterLine {
    pub associated_string: String,
    pub line: Vec<CharacterGroup>,
    pub cell_lines: Vec<DisplayCellLine>
}

impl CharacterLine {
    pub fn new() -> CharacterLine {
        let character_group = CharacterGroup::with_capacity(16);

        Self {
            associated_string: String::from(""),
            line: vec![character_group],
            cell_lines: vec![]
        }
    }
    
    pub fn from_string(content: String) -> CharacterLine {
        let character_group = CharacterGroup::from_string(content);
        
        Self {
            associated_string: String::from(""),
            line: vec![character_group],
            cell_lines: vec![]
        }
    }
    
    pub fn single_line(content: String) -> Vec<CharacterLine> {
        vec![CharacterLine::from_string(content)]
    }
    
    pub fn basic_add_to_first(&mut self, content: &str, rasterizer: &mut Rasterizer) {
        self.line.get_mut(0).unwrap().characters.push_str(content);
        self.cell_lines = rasterizer.character_line_to_cell_lines(self, rasterizer.get_line_cell_width());
    }
}

pub struct PtyBuffer {
    rasterizer: WrappedRasterizer,
    buffer: VecDeque<CharacterLine>,
    updated: bool
}

impl PtyBuffer {
    pub fn new(rasterizer: WrappedRasterizer) -> PtyBuffer {
        let mut buffer: VecDeque<CharacterLine> = VecDeque::new();
        buffer.push_back(CharacterLine::new());
        
        Self {
            rasterizer,
            buffer,
            updated: false
        }
    }
    
    pub fn add_input(&mut self, input: &str) {
        self.updated = true;
        
        let mut lines = input.split('\n').peekable();
        
        loop {
            let next = lines.next();
            let is_last = lines.peek().is_none();
            match next {
                Some(data) => {                    
                    self.add_to_current_line(data);                    
                    if !is_last {
                        self.complete_current_line();
                    }
                },
                None => break
            };
        }
        
    }
    
    pub fn is_updated(&self) -> bool {
        self.updated
    }
    
    // Get a range of lines (from the last one pushed, aka the newest, to the first one pushed, aka the oldest)
    // Won't panic if there's more
    // Will panic if end < start
    pub fn get_range(&mut self, start: usize, end: usize) -> Vec<CharacterLine> {
        assert!(start <= end);
        self.updated = false;
        
        let mut data: Vec<CharacterLine> = Vec::new();
        
        for i in start..=end {
            if let Some(content) = self.buffer.get(i) {
                data.push(content.clone());
            }
        }
        
        data
    }
    
    fn add_to_current_line(&mut self, input: &str) {
        self.buffer.get_mut(0).unwrap().basic_add_to_first(input, &mut self.rasterizer.write().unwrap())
    }
    
    fn complete_current_line(&mut self) {
        self.buffer.push_front(CharacterLine::new());
    }
}
