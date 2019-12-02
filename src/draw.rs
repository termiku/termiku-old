use crate::atlas::*;
use crate::rasterizer::*;
use crate::pty_buffer::*;
use crate::config::*;

use glium::{Display, Frame, VertexBuffer, DrawParameters, Surface, index::NoIndices};
use glium::program::Program;
use glium::uniforms::Uniforms;

use std::sync::{Arc, RwLock};

#[derive(Copy, Clone, Debug)]
struct Vertex {
    position: [f32; 2],
    tex_coords: [f32; 2],
    colour: [f32; 4],
}

implement_vertex!(Vertex, position, tex_coords, colour);


pub struct Drawer<'a> {
    config: Config,
    dimensions: RectSize,
    program: ProgramWrapper,
    index_buffer: NoIndices,
    draw_parameters: DrawParameters<'a>,
    pub atlas: Atlas,
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

pub type WrappedDrawer<'a> = Arc<RwLock<Drawer<'a>>>;

impl <'a> Drawer<'a> {
    // TODO: probably should take a DrawConfig, or use a builder pattern
    pub fn new(display: &Display, config: Config) -> Self {
        let dimensions = RectSize {
            width: display.get_framebuffer_dimensions().0,
            height: display.get_framebuffer_dimensions().1,
        };
        
        // let dpi_factor = display.gl_window().window().hidpi_factor();
        let atlas = Atlas::new(display, RectSize {
            height: 1280,
            width: 720
        });
        
        let program = ProgramWrapper::new(display);
        
        let index_buffer = NoIndices(glium::index::PrimitiveType::TrianglesList);
                
        let draw_parameters = DrawParameters {
            blend: glium::Blend::alpha_blending(),
            ..Default::default()
        };
        
        let cell_size = RectSize {
            width: 0,
            height: 0
        };
        
        Self {
            config,
            dimensions,
            program,
            index_buffer,
            draw_parameters,
            atlas,
        }
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
    
    
    // TODO really bad rn, should handle if the atlas isn't big enough
    fn prepare_atlas(&mut self, lines: &[&DisplayCellLine]) {        
        for line in lines {
            for cell in &line.cells {
                self.atlas.insert(cell.ftg.size(), cell.ftg.id(), cell.ftg.data()).unwrap();
            }
        }
    }
    
    fn get_vertices_for_cell(&self, cell: &DisplayCell, cell_size: RectSize, x: u32, y: u32) -> [Vertex; 6] {
        let actual_x = x as i32;
        let actual_y = y as i32;
        
        let tex_rect = self.atlas.get(cell.ftg.id()).unwrap();
        let cell_height = cell_size.height;        
        
        let delta_cell_y = cell_height as i32 - tex_rect.size.height as i32;
        let actual_y = actual_y + delta_cell_y;
        let actual_x = actual_x + 1;
        
        let delta_glyph_y = (cell.ftg.height - cell.ftg.bearing_y) / 64;
        
        let actual_y = actual_y + delta_glyph_y as i32;
        
        let delta_glyph_x = cell.ftg.bearing_x / 64;
        let actual_x = (actual_x as i64 + delta_glyph_x) as i32;
        
        let RectSize {
            height: screen_height,
            width: screen_width
        } = self.dimensions;
        let RectSize {
            height: atlas_height,
            width: atlas_width
        } = self.atlas.size;
        
        let pos_top_left_x = ((actual_x as f32 / screen_width as f32) - 0.5 ) * 2.0;
        let pos_top_left_y = ((actual_y as f32 / screen_height as f32) - 0.5 ) * -2.0;
        
        let pos_bottom_right_x = (((actual_x + tex_rect.size.width as i32) as f32 / screen_width as f32) - 0.5 ) * 2.0;
        let pos_bottom_right_y = (((actual_y + tex_rect.size.height as i32) as f32 / screen_height as f32) - 0.5 ) * -2.0;
        
        let tex_top_left_x = tex_rect.top_left().x as f32 / atlas_width as f32;
        let tex_top_left_y = tex_rect.top_left().y as f32 / atlas_height as f32 * -1.0;
        
        let tex_bottom_right_x = tex_rect.bottom_right().x as f32 / atlas_width as f32;
        let tex_bottom_right_y = tex_rect.bottom_right().y as f32 / atlas_height as f32 * -1.0;
        
        let colour = [0.0, 0.0, 0.0, 1.0];
        
        [
            Vertex {
                position: [pos_top_left_x, pos_top_left_y],
                tex_coords: [tex_top_left_x, tex_top_left_y],
                colour
            },
            Vertex {
                position: [pos_top_left_x, pos_bottom_right_y],
                tex_coords: [tex_top_left_x, tex_bottom_right_y],
                colour
            },
            Vertex {
                position: [pos_bottom_right_x, pos_top_left_y],
                tex_coords: [tex_bottom_right_x, tex_top_left_y],
                colour
            },
            Vertex {
                position: [pos_top_left_x, pos_bottom_right_y],
                tex_coords: [tex_top_left_x, tex_bottom_right_y],
                colour
            },
            Vertex {
                position: [pos_bottom_right_x, pos_top_left_y],
                tex_coords: [tex_bottom_right_x, tex_top_left_y],
                colour
            },
            Vertex {
                position: [pos_bottom_right_x, pos_bottom_right_y],
                tex_coords: [tex_bottom_right_x, tex_bottom_right_y],
                colour
            }
        ]
    }
    
    fn get_vertices_for_line(&self, line: &DisplayCellLine, cell_size: RectSize, y: u32) -> Vec<Vertex> {
        let mut x = 0;
        
        let mut vertices: Vec<Vertex> = Vec::with_capacity(line.cells.len());  
        
        for cell in line.cells.iter() {
            vertices.extend(&self.get_vertices_for_cell(cell, cell_size, x, y));
            x += cell_size.width;
        }
        
        vertices
        
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
    
    pub fn render_lines(&mut self, lines: &[CharacterLine],
        cell_size: RectSize, display: &Display, frame: &mut Frame) {
        let cell_lines: Vec<&DisplayCellLine> = lines.iter().flat_map(|line| &line.cell_lines).collect();
        
        let cell_height = cell_size.height;
        
        let number_of_lines = (self.dimensions.height / cell_size.height) as usize;
        
        let lines_to_render: Vec<&DisplayCellLine> = cell_lines.into_iter().take(number_of_lines).rev().collect();
        
        self.prepare_atlas(&lines_to_render);
        
        let mut current_height = 0;
        let mut vertices: Vec<Vertex> = vec![];

        for line in lines_to_render {
            vertices.append(&mut self.get_vertices_for_line(line, cell_size, current_height));
            current_height += cell_height;
        }
        
        let vertex_buffer = VertexBuffer::new(display, &vertices).unwrap();
            
        let sampler = self.atlas.atlas
            .sampled()
            .magnify_filter(glium::uniforms::MagnifySamplerFilter::Nearest);
            
        let char_uniforms = uniform! {
            tex: sampler
        };
        
        self.draw_vertex(&vertex_buffer, frame, char_uniforms);
    }
}
