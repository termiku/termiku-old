use super::*;

use crate::control::control_type::*;
use ControlType::*;

impl Screen {
    pub fn handle_control_sequence(&mut self, control: ControlType) {
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
                
                *cursor_y = *cursor_y + value;
                if *cursor_y > self.line_cell_height {
                    *cursor_y = self.line_cell_height;
                }
            },
            
            // Make the cursor go n rows right, stopping at the max number of columns.
            CursorRight(value) => {
                let cursor_x = &mut self.cursor.position.x;
                let value = value as usize;
                
                *cursor_x = *cursor_x + value;
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
                *cursor_y = *cursor_y + value;
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
                
                println!("row: {}, column: {}", row, column);
                println!("before: cursor x: {}, cursor_y: {}", cursor_x, cursor_y);
                
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
                
                println!("after: cursor x: {}, cursor_y: {}", cursor_x, cursor_y);
                
            },
            _ => {}
        }
    }
}