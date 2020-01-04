mod handle_control_sequence;

pub mod event;
pub mod sgr;

use std::collections::VecDeque;
use std::sync::{Arc, Mutex};

use crate::atlas::RectSize;
use crate::control::*;
use crate::rasterizer::*;
use crate::unicode::*;

use event::*;

const BELL_BYTE: u8 = 0x07;
const BACKSPACE_BYTE: u8 = 0x08;
const TABULATION_BYTE: u8 = 0x09;
const LINE_FEED_BYTE: u8 = 0x0A;
const CARRIAGE_RETURN_BYTE: u8 = 0x0D;

fn is_special_byte(byte: u8) -> bool {
    byte == BELL_BYTE ||
    byte == BACKSPACE_BYTE ||
    byte == TABULATION_BYTE ||
    byte == LINE_FEED_BYTE ||
    byte == CARRIAGE_RETURN_BYTE
}

const TAB_LENGTH: usize = 8;

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
// White is 255, 255, 255
#[derive(Copy, Clone, Debug)]
// FIXME: pty_buffer::Color should have named fields instead of being a tuple struct.
pub struct Color(pub u8, pub u8, pub u8, pub u8);

pub const DEFAULT_FG: Color = Color(255, 255, 255, 255);

impl Color {
    pub fn u8_to_f32(byte: u8) -> f32 {
        // FIXME: pty_buffer::Color::u8_to_f32 has a redundant if block.
        if byte == 0 {
            0.0
        } else {
            byte as f32 / 255.0
        }
    }
    
    pub fn from_rgb(r: u8, g: u8, b: u8) -> Self {
        Self(r, g, b, 255)
    }
    
    pub fn to_opengl_color(self) -> [f32; 4] {
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
    pub bg: Option<Color>
}

// Should probably need a Config from somewhere
impl CellProperties {
    pub fn new() -> Self {
        Self {
            fg: DEFAULT_FG,
            bg: None
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
    
    pub fn save(&mut self) {
        self.saved = Some(self.position);
    }
    
    pub fn restore(&mut self) {
        if let Some(position) = &self.saved {
            self.position = *position;
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
    
    // FIXME: pty_buffer::CellState::get_cell_from_parser_and_byte() probably shouldn't be marked inline...
    #[inline]
    fn get_cell_from_parser_and_byte(mut parser: Utf8Parser, byte: u8) -> CellState {
        match parser.parse_byte(byte) {
            Ok(maybe_char) => match maybe_char {
                Some(char) => CellState::Filled(char),
                None => CellState::Filling(parser)
            },
            Err(err) => {
                if let Utf8ParserError::InvalidContinuationByte = err {
                    match parser.parse_byte(byte) {
                        Ok(maybe_char) => match maybe_char {
                            Some(char) => CellState::Filled(char),
                            None => CellState::Filling(parser)
                        },
                        Err(_) => CellState::Invalid
                    }
                } else {
                    CellState::Invalid
                }
            }
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
            display: vec![DisplayCellLine::empty()]
        }
    }
    
    pub fn rasterize(&mut self, rasterizer: &mut Rasterizer) {
        self.display = rasterizer.cells_to_display_cell_lines(&self.cells);
    }
}

#[derive(Copy, Clone, Default)]
pub struct ScreenState {
    /// Alternative buffer state
    pub is_alternative: bool
}

pub struct Screen {
    pub line_cell_width: usize,
    pub line_cell_height: usize,
    pub history: VecDeque<CellLine>,
    pub control_parser: ControlSequenceParser,
    pub screen_lines: Vec<CellLine>,
    pub cursor: Cursor,
    pub alternative_screen_lines: Vec<CellLine>,
    // FIXME Screen::alternative_cursor can probably be removed. The alternate screen switching should save/restore the cursor.
    pub alternative_cursor: Cursor,
    pub state: ScreenState,
    pub sender: Arc<Mutex<mio_extras::channel::Sender<ScreenEvent>>>,
    pub id: usize
}

impl Screen {
    pub fn empty(sender: mio_extras::channel::Sender<ScreenEvent>, rasterizer: &mut Rasterizer, id: usize) -> Self {
        let line_cell_size = rasterizer.get_line_cell_size();
        
        let line_cell_width = line_cell_size.width as usize;
        let line_cell_height = line_cell_size.height as usize;
        let mut screen_lines: Vec<CellLine> = vec![CellLine::new(line_cell_width, CellProperties::new()); line_cell_height];
        
        for line in screen_lines.iter_mut() {
            line.rasterize(rasterizer)
        }
        
        let history: VecDeque<CellLine> = VecDeque::new();
        
        let cursor = Cursor::new();
        
        Self {
            line_cell_width,
            line_cell_height,
            history,
            control_parser: ControlSequenceParser::new(),
            
            screen_lines: screen_lines.clone(),
            cursor,
            
            alternative_screen_lines: screen_lines,
            alternative_cursor: cursor,
            
            state: ScreenState::default(),
            sender: Arc::new(Mutex::new(sender)),
            id
        }
    }
    
    // FIXME: Can Screen::update_line_cell_dimensions be removed?
    pub fn update_line_cell_dimensions(&mut self, _line_cell_size: RectSize) {
        // self.line_cell_height = line_cell_size.height as usize;
        // self.line_cell_width = line_cell_size.width as usize;
    }
    
    pub fn add_to_buffer(&mut self, data: &[u8], rasterizer: &mut Rasterizer) {
        for byte in data.iter() {
            if self.control_parser.is_parsing() {
                match self.control_parser.parse_byte(*byte) {
                    Ok(maybe_control) => {
                        if let Some(control) = maybe_control {
                            self.handle_control_sequence(control, rasterizer);
                        }
                    },
                    Err(_) => {
                        let mut buffer = self.control_parser.flush();
                        if *byte == CSI_1 {
                            self.control_parser.parse_byte(*byte)
                                .expect("Can't parse a CSI after being reset");
                        } else {
                            buffer.push(*byte);
                        }
                        
                        for invalid_byte in buffer.into_iter() {
                            self.push_byte_to_screen(invalid_byte, rasterizer);
                        }
                    }
                }
            } else if *byte == CSI_1 {
                self.control_parser.parse_byte(*byte)
                    .expect("Can't parse a CSI after being reset");
            } else {
                self.push_byte_to_screen(*byte, rasterizer);
            }
        }
    }

    pub fn next_line(&mut self, rasterizer: &mut Rasterizer) {
        if self.cursor.position.y == self.line_cell_height {
            self.push_line_to_history(rasterizer);
        } else {
            self.cursor.position.y += 1;
        }
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
    
    fn handle_special_byte(&mut self, byte: u8, rasterizer: &mut Rasterizer) {
        println!("special byte received! {:#04X?}", byte);
        
        match byte {
            
            BELL_BYTE => {
                // Do nothing for now
            },
            
            BACKSPACE_BYTE => {
                let (row_number, column_number) = self.get_position_pointed_by_cursor();
                
                // self.screen_lines[row_number].cells[column_number].state = CellState::Empty;
                self.screen_lines[row_number].rasterize(rasterizer);
                
                if column_number != 0 {
                    self.cursor.position.x -= 1;
                }
            },
            
            TABULATION_BYTE => {
                let (_, column_number) = self.get_position_pointed_by_cursor();
                
                // FIXME: Screen::handle_special_byte tab calculation doesn't look correct to me...
                let mut new_column = (column_number / TAB_LENGTH) * TAB_LENGTH + TAB_LENGTH;
                
                if new_column >= self.line_cell_width {
                    new_column = self.line_cell_width - 1;
                }
                
                self.cursor.position.x = new_column;
            },
            
            CARRIAGE_RETURN_BYTE => {
                self.cursor.position.x = 1;
            }
            
            // Should never reach a line feed for now (handled upper in the stack), so i want to
            // crash if we somehow do encouter it.
            _ => unreachable!()
        }
    }
    
    fn push_byte_to_screen(&mut self, byte: u8, rasterizer: &mut Rasterizer) {
        // Handle special bytes, like the bell or a backspace, and do not draw anything on screen
        if is_special_byte(byte) {
            self.handle_special_byte(byte, rasterizer);
        } else {
            let (mut row_number, mut column_number) = self.get_position_pointed_by_cursor();
            
            let cell_state = self.screen_lines[row_number].cells[column_number].state.next_state(byte);
            
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
                        self.push_line_to_history(rasterizer);
                    }
                }
            }
            
            self.cursor.position.x = column_number + 1;
            self.cursor.position.y = row_number + 1;
        }
        
    }
    
    fn push_line_to_history(&mut self, rasterizer: &mut Rasterizer) {
        let line = self.screen_lines.remove(0);
        
        // If we're in the alternative buffer state, we don't want to polute the main history.
        if !self.state.is_alternative {
            self.history.push_front(line);
        }
        
        let mut new = CellLine::new(self.line_cell_width, CellProperties::new());
        new.rasterize(rasterizer);
        self.screen_lines.push(new);
    }
}

pub struct PtyBuffer {
    rasterizer: WrappedRasterizer,
    screen: Screen,
    updated: bool,
}

impl PtyBuffer {
    pub fn new(rasterizer: WrappedRasterizer, sender: mio_extras::channel::Sender<ScreenEvent>, id: usize) -> PtyBuffer {    
        let screen = Screen::empty(sender, &mut rasterizer.write().unwrap(), id);
        
        Self {
            rasterizer,
            screen,
            updated: false
        }
    }
    
    pub fn add_input(&mut self, input: Vec<u8>) {
        self.updated = true;
        
        let mut lines = input.split(|x| x == &LINE_FEED_BYTE).peekable();
        
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
        
        let mut display_lines: Vec<DisplayCellLine> = display_lines.iter().flatten().cloned().collect();
        
        if let Some(line) = display_lines.get_mut(self.screen.cursor.position.y - 1) {
            if let Some(cell) = line.cells.get_mut(self.screen.cursor.position.x - 1) {
                cell.is_cursor = true;
            }
        }
        
        display_lines.reverse();
        
        display_lines
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
        self.screen.next_line(&mut self.rasterizer.write().unwrap());
    }
}
