use crate::atlas::*;
use crate::harfbuzz::*;
use crate::freetype::*;

use glium::{Display, Frame, VertexBuffer, IndexBuffer, DrawParameters, Surface};
use glium::program::Program;
use glium::uniforms::Uniforms;
use glium::index::PrimitiveType;

use ::freetype::freetype::*;
use ::harfbuzz::sys::*;
use ::harfbuzz::Buffer;

#[derive(Copy, Clone)]
struct Vertex {
    position: [f32; 2],
    tex_coords: [f32; 2],
    colour: [f32; 4],
}
implement_vertex!(Vertex, position, tex_coords, colour);

// Group of character to be rendered, with probably in the future options to apply to them
#[derive(Debug)]
pub struct CharacterGroup {
    // maybe try to use &str?
    characters: String
}

// Logical line, as in "here's a line to be rendered", as expected for the user
#[derive(Debug)]
pub struct CharacterLine {
    line: Vec<CharacterGroup>
}

impl CharacterLine {
    pub fn from_string(content: String) -> CharacterLine {
        let character_group = CharacterGroup {
            characters: content
        };
        
        Self {
            line: vec![character_group]
        }
    }
    
    pub fn single_line(content: String) -> Vec<CharacterLine> {
        vec![CharacterLine::from_string(content)]
    }
}

// Struct containing a character, should be populated with colors, transformations, and such later on
// Maybe going to need the font info too when multifont ?
#[derive(Debug)]
struct DisplayCell {
    ftg: FreeTypeGlyph
}

#[derive(Debug)]
struct DisplayCellLine {
    cells: Vec<DisplayCell>
}

pub struct Drawer<'a> {
    display: &'a Display,
    harfbuzz: HarfbuzzWrapper,
    freetype: FreetypeWrapper,
    program: ProgramWrapper,
    index_buffer: IndexBuffer<u16>,
    draw_parameters: DrawParameters<'a>,
    atlas: Atlas,
    pub cell_size: RectSize
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
        
        let index_buffer =
            IndexBuffer::new(display, PrimitiveType::TriangleStrip, &[1 as u16, 2, 0, 3])
                .unwrap();
                
        let draw_parameters = DrawParameters {
            blend: glium::Blend::alpha_blending(),
            ..Default::default()
        };
        
        let cell_size = RectSize {
            width: 0,
            height: 0
        };
        
        let mut drawer = Self {
            display,
            harfbuzz: hb_wrapper,
            freetype: ft_wrapper,
            program,
            index_buffer,
            draw_parameters,
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
    
    /// Get the maximum number of cell per row
    fn get_line_cell_width(&self) -> u32 {
        let screen_width = self.display.get_framebuffer_dimensions().0 as f32;
        let cell_width = self.cell_size.width as f32;
        
        let mut cell_number = (screen_width / cell_width).floor() as u32;
        
        if cell_number == 0 {
            cell_number += 1;
        }
        
        cell_number
    }
    
    /// Get the maximum number of cell per column
    fn get_line_cell_height(&self) -> u32 {
        let screen_height = self.display.get_framebuffer_dimensions().1 as f32;
        let cell_height = self.cell_size.height as f32;
        
        let mut cell_number = (screen_height / cell_height).floor() as u32;
        
        if cell_number == 0 {
            cell_number += 1;
        }
        
        cell_number
    }
    
    
    // probably should redo all of this, differentiating the previous lines and the current lines
    fn character_line_to_cell_lines(&mut self, line: CharacterLine, line_cell_width: u32) -> Vec<DisplayCellLine> {
        let mut cell_lines = Vec::<DisplayCellLine>::new();
        
        for group in line.line.iter() {
            let mut rasterized: Vec<FreeTypeGlyph> = self.rasterize(&group.characters);
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
                let cells: Vec<DisplayCell> = drain.map(|ftg| DisplayCell {ftg}).collect();
                
                cell_lines.push(DisplayCellLine {
                    cells
                });
            }
        }
        
        println!("{:?}", cell_lines);
        
        cell_lines
    }
    
    fn character_lines_to_cell_lines(&mut self, lines: Vec<CharacterLine>) -> Vec<DisplayCellLine> {
        let line_cell_width = self.get_line_cell_width();
        lines
            .into_iter()
            .flat_map(|line| self.character_line_to_cell_lines(line, line_cell_width))
            .collect()
    }
    
    // really bad rn, should handle if the atlas isn't big enough
    fn prepare_atlas(&mut self, lines: &[&DisplayCellLine]) {
        for line in lines {
            for cell in &line.cells {
                self.atlas.insert(cell.ftg.size(), cell.ftg.id(), cell.ftg.data()).unwrap();
            }
        }
    }
    
    fn get_vertices_for_line(&mut self, line: &DisplayCellLine, height: u32) -> Vec<Vertex> {
        let cell_size = self.cell_size;
        let colour = [0.0, 0.0, 0.0, 1.0];
        
        // Magic happens here
        
        panic!()
    }
    
    fn draw_vertex(&self, vertex_buffer: &VertexBuffer<Vertex>, frame: &mut Frame, uniforms: impl Uniforms) {
        frame
            .draw(
                vertex_buffer,
                &self.index_buffer,
                &self.program.char_program,
                &uniforms,
                &glium::DrawParameters {
                    blend: glium::Blend::alpha_blending(),
                    ..Default::default()
                },
            )
            .unwrap();
    }
    
    pub fn render_lines(&mut self, lines: Vec<CharacterLine>, frame: &mut Frame) {
        let cell_lines = self.character_lines_to_cell_lines(lines);
        let cell_line_height = self.get_line_cell_height();
        
        let number_of_lines = (self.display.get_framebuffer_dimensions().1 / self.cell_size.height) as usize;
        
        let lines_to_render: Vec<&DisplayCellLine> = cell_lines.iter().rev().take(number_of_lines).collect();
        
        self.prepare_atlas(&lines_to_render);
        
        let mut current_height = 0;
        let mut vertices: Vec<Vertex> = vec![];
        for line in lines_to_render {
            vertices.append(&mut self.get_vertices_for_line(line, current_height));
            current_height += cell_line_height;
        }
        
        let vertex_buffer = VertexBuffer::new(self.display, &vertices).unwrap();
            
        let sampler = self.atlas.atlas
            .sampled()
            .magnify_filter(glium::uniforms::MagnifySamplerFilter::Nearest);
            
        let char_uniforms = uniform! {
            tex: sampler
        };
        
        self.draw_vertex(&vertex_buffer, frame, char_uniforms);
    }
}

