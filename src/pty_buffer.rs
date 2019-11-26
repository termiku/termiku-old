// Group of character to be rendered, with probably in the future options to apply to them
#[derive(Debug)]
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
#[derive(Debug)]
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
}

pub struct PtyBuffer {
    current_line: CharacterLine,
    past_lines: Vec<CharacterLine>
}

impl PtyBuffer {
    pub fn new() -> PtyBuffer {
        let current_line = CharacterLine::new();
        let past_lines: Vec<CharacterLine> = vec![];
        
        Self {
            current_line,
            past_lines
        }
    }
    
    pub fn add_input(input: String) {
        
    }
}

impl std::default::Default for PtyBuffer {
    fn default() -> Self {
        Self::new()
    }
}