use crate::rasterizer::*;
use crate::atlas::RectSize;
use crate::unicode::*;

use std::collections::VecDeque;


// Cursor positions
// They are 1 based
// They start from the top left
#[derive(Copy, Clone, Debug)]
pub struct Position {
    x: usize,
    y: usize,
}

impl Position {
    pub fn new() -> Self {
        Self {
            x: 1,
            y: 1
        }
    }
}

// R G B A
// Black is 0,0,0
// White is 1,1,1
#[derive(Copy, Clone, Debug)]
pub struct Color(pub u8, pub u8, pub u8, pub u8);

impl Color {
    pub fn u8_to_f32(byte: u8) -> f32 {
        if byte == 0 {
            0.0
        } else {
            byte as f32 / 255.0
        }
    }
    
    pub fn to_opengl_color(&self) -> [f32; 4] {
        [
            Self::u8_to_f32(self.0),
            Self::u8_to_f32(self.1),
            Self::u8_to_f32(self.2),
            Self::u8_to_f32(self.3),
        ]
    }
}

#[derive(Copy, Clone, Debug)]
pub struct CellProperties {
    pub fg: Color,
    pub bg: Color
}

// Should probably need a Config from somewhere
impl CellProperties {
    pub fn new() -> Self {
        Self {
            fg: Color(255, 0, 0, 255),
            bg: Color(1, 1, 1, 255)
        }
    }
}

// Cursor can hold its current position and its saved position, if it exists
// Also holds the cell properties of the next cells to create (fg and gb colors, bold, italic, etc)
// It's different than the cursor displayed on screen, and therefore should not hold any
// information relating to its display state (block vs line, blinking or not, etc)
#[derive(Copy, Clone, Debug)]
pub struct Cursor {
    position: Position,
    saved: Option<Position>,
    properties: CellProperties,
}

impl Cursor {
    pub fn new() -> Self {
        Self {
            position: Position::new(),
            saved: None,
            properties: CellProperties::new()
        }
    }
    
    pub fn save(&mut self, position: Position) {
        self.saved = Some(position);
    }
    
    pub fn restore(&mut self) {
        if let Some(position) = &self.saved {
            self.position = position.clone();
        }
    }
}

// Group of character to be rendered, with probably in the future options to apply to them
#[derive(Debug, Clone)]
pub struct CharacterGroup {
    pub characters: Vec<u8>
}

impl CharacterGroup {
    pub fn with_capacity(capacity: usize) -> CharacterGroup {
        Self {
            characters: Vec::with_capacity(capacity)
        }
    }
    
    pub fn from_string(content: String) -> CharacterGroup {
        Self {
            characters: content.into()
        }
    }
}

#[derive(Copy, Clone, Debug)]
pub enum CellState {
    Empty,
    Filling(Utf8Parser),
    Filled(char),
    Invalid
}

impl CellState {    
    fn get_start(first_byte: u8) -> Self {
        let parser = Utf8Parser::new();
        Self::get_cell_from_parser_and_byte(parser, first_byte)
    }
    
    #[inline]
    fn get_cell_from_parser_and_byte(mut parser: Utf8Parser, first_byte: u8) -> Self {
        match parser.parse_byte(first_byte) {
            Ok(maybe_char) => match maybe_char {
                Some(char) => CellState::Filled(char),
                None => CellState::Filling(parser)
            },
            Err(_) => CellState::Invalid
        }
    }
    
    pub fn next_state(&mut self, new_byte: u8) -> CellState {
        match self {
            CellState::Filled(_) | CellState::Invalid  | CellState::Empty => CellState::get_start(new_byte),
            CellState::Filling(parser) => Self::get_cell_from_parser_and_byte(*parser, new_byte)
        }
    }
}

#[derive(Copy, Clone, Debug)]
pub struct Cell {
    pub state: CellState,
    pub properties: CellProperties
}

impl Cell {
    fn empty(properties: CellProperties) -> Self {
        Self {
            properties,
            state: CellState::Empty
        }
    }
}

#[derive(Clone, Debug)]
pub struct CellLine {
    pub cells: Vec<Cell>,
    pub display: Vec<DisplayCellLine>
}

impl CellLine {
    pub fn new(width: usize, properties: CellProperties) -> Self {
        Self {
            cells: vec![Cell::empty(properties); width],
            display: Vec::new()
        }
    }
    
    pub fn rasterize(&mut self, rasterizer: &mut Rasterizer) {
        self.display = rasterizer.cells_to_display_cell_lines(&self.cells);
    }
}

pub struct Screen {
    pub line_cell_width: usize,
    pub line_cell_height: usize,
    pub screen_lines: Vec<CellLine>,
    pub history: VecDeque<CellLine>,
    pub cursor: Cursor,
    pub buffer: Vec<u8>
}

impl Screen {
    pub fn empty(line_cell_size: RectSize) -> Self {
        let line_cell_width = line_cell_size.width as usize;
        let line_cell_height = line_cell_size.height as usize;
        let screen_lines: Vec<CellLine> = vec![CellLine::new(line_cell_width, CellProperties::new()); line_cell_height];
        let history: VecDeque<CellLine> = VecDeque::new();
        
        Self {
            line_cell_width,
            line_cell_height,
            screen_lines,
            history,
            cursor: Cursor::new(),
            buffer: Vec::with_capacity(32),
        }
    }
    
    pub fn update_line_cell_dimensions(&mut self, line_cell_size: RectSize) {
        // self.line_cell_height = line_cell_size.height as usize;
        // self.line_cell_width = line_cell_size.width as usize;
    }
    
    pub fn add_to_buffer(&mut self, data: &[u8], rasterizer: &mut Rasterizer) {
        for byte in data.iter() {
            self.buffer.push(*byte);
            if self.is_buffer_enough() {
                self.handle_buffer(rasterizer);
            }
        }
    }
    
    // incorrect. Should only go down one line, not go back at the beginning, but whatever for now,
    // im done
    pub fn next_line(&mut self) {
        self.cursor.position.x = 1;
        
        if self.cursor.position.y == self.line_cell_height {
            self.push_line_to_history();
        } else {
            self.cursor.position.y += 1;
        }
    }
     
    // check here for escape sequences and whatnot
    fn is_buffer_enough(&self) -> bool {
        true
    }
    
    fn handle_buffer(&mut self, rasterizer: &mut Rasterizer) {
        // Or later, handle the escape sequence    
        self.push_buffer(rasterizer);
    }
    
    fn push_buffer(&mut self, rasterizer: &mut Rasterizer) {
        self.push_buffer_to_screen(rasterizer);
        self.buffer.clear();
    }
    
    fn get_position_pointed_by_cursor(&self) -> (usize, usize) {
        let mut row_number = self.cursor.position.y - 1;
        if row_number >= self.line_cell_height {
            row_number = self.line_cell_height - 1;
        }
        
        let mut column_number = self.cursor.position.x - 1;
        if column_number >= self.line_cell_width {
            column_number = self.line_cell_width - 1;
        }
        
        (row_number, column_number)
    }
    
    fn push_buffer_to_screen(&mut self, rasterizer: &mut Rasterizer) {
        let (mut row_number, mut column_number) = self.get_position_pointed_by_cursor();
        let buffer = self.buffer.clone();
        
        for data in buffer {
            let cell_state = self.screen_lines[row_number].cells[column_number].state.next_state(data);
            
            let advance = match cell_state {
                CellState::Filled(_) | CellState::Invalid => true,
                _ => false
            };
            
            self.screen_lines[row_number].cells[column_number].state = cell_state;
            self.screen_lines[row_number].cells[column_number].properties = self.cursor.properties;
            
            self.screen_lines[row_number].rasterize(rasterizer);
            
            if advance {
                column_number += 1;
                if column_number >= self.line_cell_width {
                    row_number += 1;
                    column_number = 0;
                    if row_number >= self.line_cell_height {
                        self.push_line_to_history();
                    }
                }
            }
            
            self.cursor.position.x = column_number + 1;
            self.cursor.position.y = row_number + 1;
        }
    }
    
    fn push_line_to_history(&mut self) {
        let line = self.screen_lines.remove(0);
        self.history.push_front(line);
        self.screen_lines.push(CellLine::new(self.line_cell_width, CellProperties::new()));
    }
}

pub struct PtyBuffer {
    rasterizer: WrappedRasterizer,
    screen: Screen,
    updated: bool
}

impl PtyBuffer {
    pub fn new(rasterizer: WrappedRasterizer) -> PtyBuffer {
        let screen = Screen::empty(rasterizer.read().unwrap().get_line_cell_size());
        
        Self {
            rasterizer,
            screen,
            updated: false
        }
    }
    
    pub fn add_input(&mut self, input: Vec<u8>) {
        self.updated = true;
        
        let mut lines = input.split(|x| x == &10).peekable();
        
        loop {
            let next = lines.next();
            let is_last = lines.peek().is_none();
            match next {
                Some(data) => {
                    self.add_to_screen_buffer(data);                 
                    if !is_last {
                        self.complete_line();
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
    pub fn get_range(&mut self, start: usize, end: usize) -> Vec<DisplayCellLine> {
        assert!(start <= end);
        self.updated = false;
        
        let mut display_lines: Vec<Vec<DisplayCellLine>> = vec![];
        for line in self.screen.screen_lines.iter() {
            display_lines.push(line.display.clone());
        }
        
        display_lines.iter().flat_map(|x| x).rev().cloned().collect()
    }
    
    pub fn dimensions_updated(&mut self) {        
        // for line in self.screen.history.iter_mut() {
        //     line.rasterize_to_cells(&mut self.rasterizer.write().unwrap());
        // }
        
        self.updated = true;
    }
    
    fn add_to_screen_buffer(&mut self, data: &[u8]) {
        self.screen.add_to_buffer(data, &mut self.rasterizer.write().unwrap());
    }
    
    fn complete_line(&mut self) {
        self.screen.next_line();
    }
}
