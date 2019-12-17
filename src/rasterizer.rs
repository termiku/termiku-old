use crate::atlas::RectSize;
use crate::config::Config;
use crate::freetype::*;
use crate::harfbuzz::*;
use crate::pty_buffer::*;

use glium::Display;

use ::freetype::freetype::*;
use ::harfbuzz::sys::*;
use ::harfbuzz::Buffer;

use std::sync::{Arc, Mutex, RwLock};
use std::ptr::NonNull;

// Struct containing a character, should be populated with colors, transformations, and such later on
// Maybe going to need the font info too when multifont ?
#[derive(Debug, Clone)]
pub struct DisplayCell {
    pub ftg: FreeTypeGlyph,
    pub fg_color: Color,
    pub bg_color: Option<Color>,
    pub is_cursor: bool,
}

// Contains a cell line, aka a line of cell to be rendered.
// A character line can be segmented into multiple cell lines if this character line is too long
// to fit in a single cell line
#[derive(Debug, Clone)]
pub struct DisplayCellLine {
    pub cells: Vec<DisplayCell>
}

impl DisplayCellLine {
    pub fn empty() -> Self {
        Self {
            cells: vec![]
        }
    }
}


struct CLibsWrapper {
    font: NonNull<hb_font_t>,
    // Not sure how Pin works, so i'll recreate a pointer each time it's needed for now
    buffer: Buffer,
    lib: FT_Library,
    face: FT_Face
}

struct SendableCLibsWrapper(Arc<Mutex<CLibsWrapper>>);

unsafe impl Send for SendableCLibsWrapper {}
unsafe impl Sync for SendableCLibsWrapper {}

pub struct Rasterizer {
    config: Config,
    dimensions: RectSize,
    wrapper: SendableCLibsWrapper,
    pub cell_size: RectSize,
    pub delta_cell_height: u32,
}

pub type WrappedRasterizer = Arc<RwLock<Rasterizer>>;

impl Rasterizer {
    pub fn new(config: Config, dimensions: RectSize) -> Self {
        let hb_font = NonNull::new(create_harfbuzz_font(&config.font.path).unwrap()).unwrap();
        let buffer = create_harfbuzz_buffer(1);
        
        let freetype_lib = init_freetype().unwrap();
        let face = new_face(freetype_lib, &config.font.path).unwrap();
        set_char_size(face, config.font.size as i64).unwrap();
        
        let wrapper = SendableCLibsWrapper(Arc::new(Mutex::new(CLibsWrapper {
            font: hb_font,
            buffer,
            lib: freetype_lib,
            face
        })));
        
        let cell_size = RectSize {
            width: 0,
            height: 0
        };
        
        let mut rasterizer = Self {
            config,
            dimensions,
            wrapper,
            cell_size,
            delta_cell_height: 0,
        };
        
        rasterizer.guess_cell_size();
        
        rasterizer
    }  
    
    pub fn rasterize(&mut self, characters: &[u8]) -> Vec<FreeTypeGlyph> {
        let handle = self.wrapper.0.lock().unwrap();
        let buffer = create_harfbuzz_buffer(characters.len());
        let buffer_p = buffer.as_ptr();
        let glyphs = unsafe {
            add_slice_to_buffer(buffer_p, characters);
            harfbuzz_shape(handle.font.as_ptr(), buffer_p);
            get_buffer_glyph(buffer_p)
        };
        render_glyphs(handle.lib, handle.face, &glyphs).unwrap()
    }
    
    // update the dimensions of the drawer.
    // returns true if those dimensions have changed
    
    pub fn update_dimensions(&mut self, display: &Display) -> bool {
        let (width, height) = display.get_framebuffer_dimensions();
        
        let changed = self.dimensions.width != width || self.dimensions.height != height;
        
        if changed {
            self.dimensions = RectSize {
                width,
                height,
            };
        }
        
        changed
    }
    
    fn guess_cell_size(&mut self) {
        let rasterized = self.rasterize("▀  ▄  █".as_bytes());
        
        let mut current_width: i64 = 0;
        let mut current_height: i64 = 0;
        let mut current_delta_height: i64 = 0;
        
        for ftg in rasterized.iter() {
            if ftg.height > current_height {
                current_height = ftg.height;
            }
            
            if (ftg.height - ftg.bearing_y) > current_delta_height {
                current_delta_height = ftg.height - ftg.bearing_y;
            }
            
            if ftg.width > current_width {
                current_width = ftg.width;
            }
        }
        
        current_width = (current_width / 64) - 1;
        current_height = (current_height / 64) - 1;
        current_delta_height = (current_delta_height / 64) - 1;
        
        if current_width < 2 || current_height < 2 {
            println!("width: {}, height: {}, delta_height: {}", current_width, current_height, current_delta_height);
            panic!("Cells are too tiny!");
        }
        
        self.cell_size.height = current_height as u32;
        self.cell_size.width = current_width as u32;
        self.delta_cell_height = current_delta_height as u32;
    }
    
    /// Get the maximum number of cell per row
    pub fn get_line_cell_width(&self) -> u32 {
        let screen_width = self.dimensions.width as f32;
        let cell_width = self.cell_size.width as f32;
        
        let mut cell_number = (screen_width / cell_width).floor() as u32;
        
        if cell_number == 0 {
            cell_number += 1;
        }
        
        cell_number
    }
    
    /// Get the maximum number of cell per column
    pub fn get_line_cell_height(&self) -> u32 {
        let screen_height = self.dimensions.height as f32;
        let cell_height = self.cell_size.height as f32;
        
        let mut cell_number = (screen_height / cell_height).floor() as u32;
        
        if cell_number == 0 {
            cell_number += 1;
        }
        
        cell_number
    }
    
    pub fn get_line_cell_size(&self) -> RectSize {
        RectSize {
            width: self.get_line_cell_width(),
            height: self.get_line_cell_height()
        }
    }
    
    pub fn get_winsize(&self) -> libc::winsize {
        libc::winsize {
            ws_col: self.get_line_cell_width() as u16,
            ws_row: self.get_line_cell_height() as u16,
            ws_xpixel: self.dimensions.width as u16,
            ws_ypixel: self.dimensions.height as u16,
        }
    }
    
    pub fn cells_to_display_cell_lines(&mut self, cells: &[Cell]) -> Vec<DisplayCellLine> {
        let line_cell_width = self.get_line_cell_width();
        
        let mut display_cell_lines = Vec::<DisplayCellLine>::new();
        
        let mut to_rasterize = String::with_capacity(cells.len());
        
        for cell in cells.iter() {
            match cell.state {
                CellState::Filled(content) => to_rasterize.push(content),
                CellState::Empty => to_rasterize.push(' '),
                CellState::Invalid | CellState::Filling(_) => to_rasterize.push('�')
            }
        }
        
        let mut rasterized = self.rasterize(to_rasterize.as_bytes());
        let mut cell_index = 0usize;
        loop {
            if rasterized.is_empty() {
                break
            }
            
            let number_to_remove = if rasterized.len() < line_cell_width as usize {
                rasterized.len()
            } else {
                line_cell_width as usize
            };
            
            let drain = rasterized.drain(0..number_to_remove);
            let cells: Vec<DisplayCell> = drain.map(|ftg| {    
                let cell = cells[cell_index];
                let display = DisplayCell {
                    ftg,
                    fg_color: cell.properties.fg,
                    bg_color: cell.properties.bg,
                    is_cursor: false
                };
                cell_index += 1;
                display
            }).collect();
            
            display_cell_lines.push(DisplayCellLine {
                cells
            });
        }
        
        display_cell_lines
    }
}
