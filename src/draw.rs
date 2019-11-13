use crate::atlas::*;
use crate::harfbuzz::*;
use crate::freetype::*;

use glium::Display;
use glium::program::Program;
use ::freetype::freetype::*;
use ::harfbuzz::sys::*;
use ::harfbuzz::Buffer;

// Struct containing a character, should be populated with colors, transformations, and such later on
// Maybe going to need the font info too when multifont ?
struct Character {
    glyph_id: u32
}

struct Line {
    characters: Vec<Character>
}

pub struct Drawer<'a> {
    display: &'a Display,
    harfbuzz: HarfbuzzWrapper,
    freetype: FreetypeWrapper,
    program: ProgramWrapper,
    atlas: Atlas,
    cell_size: RectSize
}

struct HarfbuzzWrapper {
    font: *mut hb_font_t,
    // Not sure how Pin works, so i'll recreate a pointer each time it's needed for now
    buffer: Buffer
}

struct FreetypeWrapper {
    lib: FT_Library,
    face: FT_Face
}

struct ProgramWrapper {
    pub char_program: Program
}

impl ProgramWrapper {
    pub fn new(display: &Display) -> Self {
        let char_program = program!(
        display,
        140 => {
                vertex: "
                    #version 140

                    in vec2 position;
                    in vec2 tex_coords;
                    in vec4 colour;

                    out vec2 v_tex_coords;
                    out vec4 v_colour;

                    void main() {
                        gl_Position = vec4(position, 0.0, 1.0);
                        v_tex_coords = tex_coords;
                        v_colour = colour;
                    }
                ",

                fragment: "
                    #version 140
                    uniform sampler2D tex;
                    in vec2 v_tex_coords;
                    in vec4 v_colour;
                    out vec4 f_colour;

                    void main() {
                        f_colour = v_colour * vec4(1.0, 1.0, 1.0, texture(tex, v_tex_coords).r);
                    }
                "
        })
        .unwrap();
        
        Self {
            char_program
        }
    }
}

impl <'a> Drawer<'a> {
    // TODO: probably should take a DrawConfig, or use a builder pattern
    pub fn new(display: &'a Display, font_path: &str) -> Self {
        let hb_font = create_harfbuzz_font(font_path).unwrap();
        let buffer = create_harfbuzz_buffer("");
        
        let hb_wrapper = HarfbuzzWrapper {
            font: hb_font,
            buffer
        };
        
        let freetype_lib = init_freetype().unwrap();
        let face = new_face(freetype_lib, font_path).unwrap();
        set_char_size(face).unwrap();
        
        let ft_wrapper = FreetypeWrapper {
            lib: freetype_lib,
            face
        };
        
        // let dpi_factor = display.gl_window().window().hidpi_factor();
        let atlas = Atlas::new(display, RectSize {
            height: 1280,
            width: 720
        });
        
        let program = ProgramWrapper::new(display);
        
        let cell_size = RectSize {
            width: 0,
            height: 0
        };
        
        let mut drawer = Self {
            display,
            harfbuzz: hb_wrapper,
            freetype: ft_wrapper,
            program,
            atlas,
            cell_size
        };
        
        drawer.guess_cell_size();
        
        drawer
    }
    
    fn rasterize(&mut self, characters: &str) -> Vec<FreeTypeGlyph> {
        self.harfbuzz.buffer.clear_contents();
        self.harfbuzz.buffer.add_str(characters);
        let glyphs = unsafe {
            get_buffer_glyph(self.harfbuzz.buffer.as_ptr())
        };
        render_glyphs(self.freetype.face, &glyphs).unwrap()
    }
    
    fn guess_cell_size(&mut self) {
        let rasterized = self.rasterize("abcdefghijklmnopqrstuvwxyz1234567890");
        
        let mut current_width = 0;
        let mut current_height = 0;
        
        for ftg in rasterized.iter() {
            let size = ftg.size();
            if size.height > current_height {
                current_height = size.height;
            }
            if size.width > current_width {
                current_width = size.width;
            }
        }
        
        println!("width: {}, height: {}", current_width, current_height);
        
        if current_width == 0 || current_height == 0 {
            println!("width: {}, height: {}", current_width, current_height);
            panic!("Cells are too tiny!");
        }
        
        self.cell_size.height = current_height;
        self.cell_size.width = current_width;
    }
}

