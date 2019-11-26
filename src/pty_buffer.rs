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
    pub line: Vec<CharacterGroup>
}

impl CharacterLine {
    pub fn new() -> CharacterLine {
        let character_group = CharacterGroup::with_capacity(16);

        Self {
            line: vec![character_group]
        }
    }
    
    pub fn from_string(content: String) -> CharacterLine {
        let character_group = CharacterGroup::from_string(content);
        
        Self {
            line: vec![character_group]
        }
    }
    
    pub fn single_line(content: String) -> Vec<CharacterLine> {
        vec![CharacterLine::from_string(content)]
    }
    
    pub fn basic_add_to_first(&mut self, content: &str) {
        self.line.get_mut(0).unwrap().characters.push_str(content);
    }
}

#[derive(Debug)]
pub struct PtyBuffer {
    current_line: CharacterLine,
    past_lines: VecDeque<CharacterLine>
}

impl PtyBuffer {
    pub fn new() -> PtyBuffer {
        let current_line = CharacterLine::new();
        let past_lines: VecDeque<CharacterLine> = VecDeque::new();
        
        Self {
            current_line,
            past_lines
        }
    }
    
    pub fn add_input(&mut self, input: &str) {
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
    
    // Get a range of lines (from the last one pushed, aka the newest, to the first one pushed, aka the oldest)
    // Won't panic if there's more
    // Will panic if end < start
    pub fn get_range(&self, start: usize, end: usize) -> Vec<CharacterLine> {
        assert!(start <= end);
        let number_of_line_requested = end - start + 1;
        let mut to_return: Vec<CharacterLine>;
        
        if self.past_lines.len() + 1 < number_of_line_requested {
            let mut data = self.past_lines.clone();
            data.push_front(self.current_line.clone());
            to_return = data.into();
        } else if end == 0 {
            to_return = vec![self.current_line.clone()];
        } else {
            let mut data: Vec<CharacterLine> = Vec::new();
            if start == 0 {
                data.push(self.current_line.clone());
                
                for i in start..end {
                    if let Some(content) = self.past_lines.get(i) {
                        data.push(content.clone());
                    }
                }
            } else {
                for i in (start - 1)..end {
                    if let Some(content) = self.past_lines.get(i) {
                        data.push(content.clone());
                    }
                }
            }
            
            to_return = data;
        }
        
        to_return.reverse();
        to_return
    }
    
    fn add_to_current_line(&mut self, input: &str) {
        self.current_line.basic_add_to_first(input)
    }
    
    fn complete_current_line(&mut self) {
        let completed = std::mem::replace(&mut self.current_line, CharacterLine::new());
        self.past_lines.push_front(completed);        
    }
}

impl std::default::Default for PtyBuffer {
    fn default() -> Self {
        Self::new()
    }
}