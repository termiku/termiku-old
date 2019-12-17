use super::*;

use crate::control::control_type::*;
use ControlType::*;

impl Screen {
    #[allow(clippy::cognitive_complexity)] // I won't comment on this.
    pub fn handle_control_sequence(&mut self, control: ControlType, rasterizer: &mut Rasterizer) {
        println!("control sequence received! {:?}", control);
        
        match control {
        
            // Here begins control sequences related to the cursor position.
            // A Cursor position starts as 1 and is stored as such, so no mistake should me made
            // when implementing these control functions.
        
            // Make the cursor go n rows up, stopping at 1.
            CursorUp(value) => {
                let cursor_y = &mut self.cursor.position.y;
                let value = value as usize;
                
                *cursor_y = if value < *cursor_y {
                    *cursor_y - value
                } else {
                    1usize
                };
            },
            
            // Make the cursor go n rows down, stopping at the max number of rows.
            CursorDown(value) => {                
                let cursor_y = &mut self.cursor.position.y;
                let value = value as usize;
                
                *cursor_y += value;
                if *cursor_y > self.line_cell_height {
                    *cursor_y = self.line_cell_height;
                }
            },
            
            // Make the cursor go n rows right, stopping at the max number of columns.
            CursorRight(value) => {                
                let cursor_x = &mut self.cursor.position.x;
                let value = value as usize;
                
                *cursor_x += value;
                if *cursor_x > self.line_cell_width {
                    *cursor_x = self.line_cell_width;
                }
            }
            
            // Make the cursor go n rows left, stopping at 1.
            CursorLeft(value) => {
                let cursor_x = &mut self.cursor.position.x;
                let value = value as usize;
                
                *cursor_x = if value < *cursor_x {
                    *cursor_x - value
                } else {
                    1usize
                }
            },
            
            // Make the cursor gp n rows down and at the start of the line.
            // If it would go past last line, go to the first line and not beyond.
            // If already on the last line, it would effectively make the cursor got to the start
            // of the current line.
            CursorNextLine(value) => {
                let cursor_x = &mut self.cursor.position.x;
                let cursor_y = &mut self.cursor.position.y;
                let value = value as usize;
                
                *cursor_x = 1;
                *cursor_y += value;
                if *cursor_y > self.line_cell_height {
                    *cursor_y = self.line_cell_height;
                }
            }
            
            // Make the cursor gp n rows up and at the start of the line.
            // If it would go past first line, go to the first line and not beyond.
            // If already on the first line, it would effectively make the cursor go to the start 
            // of the current line.
            CursorPrecedingLine(value) => {
                let cursor_x = &mut self.cursor.position.x;
                let cursor_y = &mut self.cursor.position.y;
                let value = value as usize;
                
                *cursor_x = 1;
                *cursor_y = if value < *cursor_y {
                    *cursor_y - value
                } else {
                    1usize
                };
            },
            
            // Make the cursor to the nth column.
            // If it would go past the last column, go to the last column.
            // If it is 0, go to the first column.
            CursorCharacterAbsolute(value) => {
                let cursor_x = &mut self.cursor.position.x;
                let value = value as usize;
                
                *cursor_x = {
                    if value == 0 {
                        1
                    } else if value > self.line_cell_width {
                        self.line_cell_width
                    } else {
                        value
                    }
                }
            },
            
            // Make the cursor go the nth row, mth column.
            // Behavior for value 0 and values too big is the same as the other cursor control
            // functions.
            CursorPosition(row, column) => {
                let cursor_x = &mut self.cursor.position.x;
                let cursor_y = &mut self.cursor.position.y;
                
                let row = row as usize;
                let column = column as usize;
                
                *cursor_x = {
                    if column == 0 {
                        1
                    } else if column > self.line_cell_width {
                        self.line_cell_width
                    } else {
                        column
                    }
                };
                
                *cursor_y = {
                    if row == 0 {
                        1
                    } else if row > self.line_cell_height {
                        self.line_cell_height
                    } else {
                        row
                    }
                };                
            },
            
            // Erase cells of the current page.
            // If parameter = 0, erase everything after and including the cursor.
            // If parameter = 1, erase everything before and including the cursor.
            // If parameter = 2, erase everything.
            EraseInPage(parameter) => {
                match parameter {
                    0 => {
                        let mut new_line = CellLine::new(
                            self.line_cell_width, 
                            CellProperties::new()
                        );
                        
                        new_line.rasterize(rasterizer);
                        
                        for index in self.cursor.position.y .. self.line_cell_height {
                            self.screen_lines[index] = new_line.clone();
                        }
                        
                        for index in self.cursor.position.x .. self.line_cell_width {
                            self.screen_lines[self.cursor.position.y - 1].cells[index] = Cell::empty(CellProperties::new())
                        }
                        
                        self.screen_lines[self.cursor.position.y - 1].rasterize(rasterizer);
                    },
                    1 => {
                        let mut new_line = CellLine::new(
                            self.line_cell_width, 
                            CellProperties::new()
                        );
                        
                        new_line.rasterize(rasterizer);
                        
                        for index in 0 .. self.cursor.position.y - 1 {
                            self.screen_lines[index] = new_line.clone();
                        }
                        
                        for index in 0 .. self.cursor.position.x - 1 {
                            self.screen_lines[self.cursor.position.y - 1].cells[index] = Cell::empty(CellProperties::new())
                        }
                        
                        self.screen_lines[self.cursor.position.y - 1].rasterize(rasterizer);
                    },
                    2 => {
                        let mut new_line = CellLine::new(
                            self.line_cell_width, 
                            CellProperties::new()
                        );
                        
                        new_line.rasterize(rasterizer);
                        
                        for index in 0..self.line_cell_height {
                            self.screen_lines[index] = new_line.clone();
                        }
                    },
                    _ => {}
                }
            }
            
            // Delete the current and the n-1 following lines, then make the the cursor go to
            // column = 1.
            // If = 0, treats it as n = 1.
            // If the number of line to delete would be too high and go past the number of lines,
            // delete until the last line.
            DeleteLine(parameter) => {
                let number_to_delete = if parameter == 0 {
                    1
                } else {
                    parameter
                };
                
                let cursor_y = self.cursor.position.y;
                
                let number_to_delete = if (number_to_delete - 1) as usize + cursor_y > self.line_cell_height {
                    (self.line_cell_height - cursor_y + 1) as u16
                } else {
                    number_to_delete
                };
                
                for _ in 0..number_to_delete {
                    self.screen_lines.remove(cursor_y - 1);
                }
                
                while self.screen_lines.len() < self.line_cell_height {
                    let mut new_line = CellLine::new(self.line_cell_width, CellProperties::new());
                    new_line.rasterize(rasterizer);
                    self.screen_lines.push(new_line);
                }
                
                assert!(self.screen_lines.len() == self.line_cell_height);
                
                self.cursor.position.x = 1;
            }
            
            // Set the mode in which the terminal will operate in. Currently only implements
            // an alternative buffer (smcup).
            SetMode(parameters) => {
                let mut index = 0usize;
                
                loop {
                    if index >= parameters.len() {
                        break
                    }
                    
                    index = self.exec_sm_property(&parameters, index);
                }
            },
            
            // Reset the mode in which the terminal will operate in. Currently only implements
            // an alternative buffer (rmcup).
            ResetMode(parameters) => {
                let mut index = 0usize;
                
                loop {
                    if index >= parameters.len() {
                        break
                    }
                    
                    index = self.exec_rm_property(&parameters, index);
                }
            }
            
            // One of the heaviest control sequence, which changes the way characters are now
            // printed on screen.
            // Support a wide variety of parameters and parameters length.
            // https://en.wikipedia.org/wiki/ANSI_escape_code#SGR_parameters
            // 
            // Dispatches to functions inside sgr.rs
            SelectGraphicRendition(parameters) => {
                if parameters.is_empty() {
                    // If length is 0, treats it as a reset
                    self.reset_graphics();
                } else {                    
                    let mut index = 0usize;
                    
                    loop {
                        if index >= parameters.len() {
                            break
                        }
                        
                        index = self.exec_sgr_property(&parameters, index);
                    }
                    
                    
                }
            },
            
            SaveCursor => {
                self.cursor.save();
            },
            
            RestoreCursor => {
                self.cursor.restore();
            },
            
            _ => {}
        }
    }
    
    #[allow(clippy::single_match)]
    fn exec_sm_property(&mut self, parameters: &[u16], index: usize) -> usize {
        let property = parameters[index];
        
        match property {
            1049 => {
                if !self.state.is_alternative {
                    self.state.is_alternative = true;
                    
                    std::mem::swap(&mut self.cursor, &mut self.alternative_cursor);
                    std::mem::swap(&mut self.screen_lines, &mut self.alternative_screen_lines);
                }
            },
            
            _ => {}
        }
        
        index + 1
    }
    
    #[allow(clippy::single_match)]
    fn exec_rm_property(&mut self, parameters: &[u16], index: usize) -> usize {
        let property = parameters[index];
        
        match property {
            1049 => {
                if self.state.is_alternative {
                    self.state.is_alternative = false;
                    
                    std::mem::swap(&mut self.cursor, &mut self.alternative_cursor);
                    std::mem::swap(&mut self.screen_lines, &mut self.alternative_screen_lines);
                }
            },
            
            _ => {}
        }
        
        index + 1
    }
    
    fn exec_sgr_property(&mut self, parameters: &[u16], index: usize) -> usize {
        let property = parameters[index];
        
        let mut index = index;
        
        match property {
            0 => self.reset_graphics(),
            
            
            30..=37 => self.simple_color_foreground(property as u8 - 30),
            
            38 => if parameters.len() >= index + 3 {
                
                match parameters[index + 1] {
                    // 256 colors
                    5 =>  {
                        index += 2;
                        
                        match parameters[index] {
                            0..=15 => {
                                self.simple_color_foreground(parameters[index] as u8)
                            },
                            
                            16..=231 => self.cube_color_foreground(parameters[index] as u8 - 16),
                            
                            232..=255 => self.grayscale_color_foreground(parameters[index] as u8 - 232),
                            
                            _ => {}
                        }
                    },
                    
                    // Truecolor
                    2 => if parameters.len() >= index + 5 {
                            let r = parameters[index + 2] as u8;
                            let g = parameters[index + 3] as u8;
                            let b = parameters[index + 4] as u8;
                            
                            index += 4;
                            
                            self.true_color_foreground(r, g, b);
                    },
                    
                    _ => {}
                }
            },
            
            39 => self.default_color_foreground(),
            
            40..=47 => self.simple_color_background(property as u8 - 40),
            
            48 => if parameters.len() >= index + 3 {
                
                match parameters[index + 1] {
                    
                    // 256 colors
                    5 => {
                        index += 2;
                        match parameters[index] {
                            
                            0..=15 => {
                                self.simple_color_background(parameters[index] as u8)
                            },
                            
                            16..=231 => self.cube_color_background(parameters[index] as u8 - 16),
                            
                            232..=255 => self.grayscale_color_background(parameters[index] as u8 - 232),
                            
                            _ => {}
                        }
                    },
                    
                    // Truecolor
                    2 => if parameters.len() >= index + 5 {
                            let r = parameters[index + 2] as u8;
                            let g = parameters[index + 3] as u8;
                            let b = parameters[index + 4] as u8;
                            
                            index += 4;
                            
                            self.true_color_background(r, g, b);
                    },
                    
                    _ => {}
                }
            },
            
            49 => self.default_color_background(),
            
            90..=97 => self.simple_color_foreground(property as u8 - 90 + 8),
            100..=107 => self.simple_color_background(property as u8 - 100 + 8),
            
            _ => {}
        };
        
        index += 1;
        
        index
    }
}