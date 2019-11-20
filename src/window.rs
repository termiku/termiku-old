// A good lot of this code is taken from glium/examples/image.rs
// For now, we only want a window capable of receiving keyboard inputs as a basis for future work
use crate::bridge::spawn_process;
use crate::atlas::{Atlas, RectSize};
use crate::draw::*;

use std::sync::Arc;

use mio_extras::channel::Sender;

use glium::{glutin, Surface, Frame};

use std::io::Cursor;

use glium::glutin::event::{Event, KeyboardInput, StartCause, WindowEvent};
use glium::glutin::event_loop::{ControlFlow, EventLoop};
use std::time::{Duration, Instant};

use std::borrow::Cow;

use crate::harfbuzz::*;

use ::freetype::freetype::*;
use crate::freetype::*;
use harfbuzz::sys::*;
use harfbuzz::*;
use std::collections::HashMap;

use arrayvec::*;
use glium::index::PrimitiveType;

use rusttype::gpu_cache::Cache;
use rusttype::{point, vector, Font, Scale};

pub fn window(program: &str, args: &[&str], env: &Option<HashMap<String, String>>) {
    let events_loop = EventLoop::new();
    let window_builder = glutin::window::WindowBuilder::new()
        .with_inner_size(glutin::dpi::LogicalSize::new(1280.0, 720.0))
        .with_title("mou ikkai");
    let context_builder = glutin::ContextBuilder::new();
    
    let display = glium::Display::new(window_builder, context_builder, &events_loop).unwrap();    
    
    let font_path = "/usr/share/fonts/OTF/FiraCode-Regular.otf";
    
    let drawer = Drawer::new(&display, font_path);    

    let font_p = create_harfbuzz_font(font_path).unwrap();
    let mut buffer = create_harfbuzz_buffer("abcdefghijklmnopqrstuvwxyz");
    let buffer_p = buffer.as_ptr();
    
    let glyph_buffer = unsafe {
        harfbuzz_shape(font_p, buffer_p);
        print_harfbuzz_buffer_info(font_p, buffer_p);
        get_buffer_glyph(buffer_p)
    };
    
    let GLYPH_ID = 1169;
    let freetype_lib = init_freetype().unwrap();
    let freetype_face = new_face(freetype_lib, font_path).unwrap();
    set_char_size(freetype_face).unwrap();
    let glyph = render_glyph(freetype_face, GLYPH_ID).unwrap();
    // println!("{:?}", glyph);
    // println!();
    // glyph.print();
    
    let process_sender = spawn_process(program, args, env);
    
    let image = image::load(
        Cursor::new(&include_bytes!("../images/miku.jpg")[..]),
        image::JPEG,
    )
    .unwrap()
    .to_rgba();
    let dpi_factor = display.gl_window().window().hidpi_factor();
    let (cache_width, cache_height) = (512 * dpi_factor as u32, 512 * dpi_factor as u32);

    let char_program = program!(
    &display,
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

    //
    let dimensions = image.dimensions();
    let glium_image =
        glium::texture::RawImage2d::from_raw_rgba_reversed(&image.into_raw(), dimensions);

    let opengl_texture =
        glium::texture::CompressedSrgbTexture2d::new(&display, glium_image).unwrap();
    let vertex_buffer = {
        #[derive(Copy, Clone)]
        struct Vertex {
            position: [f32; 2],
            tex_coords: [f32; 2],
        }

        implement_vertex!(Vertex, position, tex_coords);

        glium::VertexBuffer::new(
            &display,
            &[
                Vertex {
                    position: [-1.0, -1.0],
                    tex_coords: [0.0, 0.0],
                },
                Vertex {
                    position: [-1.0, 1.0],
                    tex_coords: [0.0, 1.0],
                },
                Vertex {
                    position: [1.0, 1.0],
                    tex_coords: [1.0, 1.0],
                },
                Vertex {
                    position: [1.0, -1.0],
                    tex_coords: [1.0, 0.0],
                },
            ],
        )
        .unwrap()
    };

    let index_buffer =
        glium::IndexBuffer::new(&display, PrimitiveType::TriangleStrip, &[1 as u16, 2, 0, 3])
            .unwrap();

    let program = program!(&display,
    140 => {
        vertex: "
                       #version 140
                       uniform mat4 matrix;
                       in vec2 position;
                       in vec2 tex_coords;
                       out vec2 v_tex_coords;
                       void main() {
                           gl_Position = matrix * vec4(position, 0.0, 1.0);
                           v_tex_coords = tex_coords;
                       }
                   ",

        fragment: "
                       #version 140
                       uniform sampler2D tex;
                       in vec2 v_tex_coords;
                       out vec4 f_color;
                       void main() {
                           f_color = texture(tex, v_tex_coords);
                       }
                   "
    })
    .unwrap();
    
    let mut atlas = Atlas::new(&display, RectSize {
        width: 800,
        height: 800
    });
    
    for glyph_id in glyph_buffer.into_iter() {
        let glyph = render_glyph(freetype_face, glyph_id).unwrap();
    }
    
    let mut drawer = Drawer::new(&display, font_path);
    
    let lines = vec![
        // CharacterLine::from_string("abcdefghijklmnopqrstuvwxyz12345678901234567890234567890".to_owned()),
        CharacterLine::from_string("abc=>a<>a!=a==a===a<=>a>=".to_owned()),
        // CharacterLine::from_string("ghi".to_owned())
    ];

    start_loop(events_loop, move |events| {
        
        drawer.update_dimensions(&display);
        
        let uniforms = uniform! {
            matrix: [
                [1.0, 0.0, 0.0, 0.0],
                [0.0, 1.0, 0.0, 0.0],
                [0.0, 0.0, 1.0, 0.0],
                [0.0, 0.0, 0.0, 1.0f32]
            ],
            tex: &opengl_texture
        };

        // drawing a frame
        let mut target = display.draw();
        target.clear_color(0.0, 0.0, 0.0, 0.0);
        target
            .draw(
                &vertex_buffer,
                &index_buffer,
                &program,
                &uniforms,
                &Default::default(),
            )
            .unwrap();
        
        drawer.render_lines(&lines, &display, &mut target);
        
        target.finish().unwrap();

        let mut action = Action::Continue;
        for event in events {
            match event {
                Event::WindowEvent { event, .. } => match event {
                    WindowEvent::CloseRequested => action = Action::Stop,
                    WindowEvent::ReceivedCharacter(input) => {
                        send_char_to_process(&process_sender, *input);
                    }
                    _ => (),
                },
                _ => (),
            }
        }

        action
    });
}

pub enum Action {
    Stop,
    Continue,
}

fn start_loop<F>(event_loop: EventLoop<()>, mut callback: F) -> !
where
    F: 'static + FnMut(&Vec<Event<()>>) -> Action,
{
    let mut events_buffer = Vec::new();
    let mut next_frame_time = Instant::now();
    event_loop.run(move |event, _, control_flow| {
        let run_callback = match event {
            Event::NewEvents(cause) => match cause {
                StartCause::ResumeTimeReached { .. } | StartCause::Init => true,
                _ => false,
            },
            _ => {
                events_buffer.push(event);
                false
            }
        };

        let action = if run_callback {
            let action = callback(&events_buffer);
            next_frame_time = Instant::now() + Duration::from_nanos(16_666_667);

            events_buffer.clear();
            action
        } else {
            Action::Continue
        };

        match action {
            Action::Continue => {
                *control_flow = ControlFlow::WaitUntil(next_frame_time);
            }
            Action::Stop => *control_flow = ControlFlow::Exit,
        }
    })
}

fn send_char_to_process(process: &Sender<char>, character: char) {
    process.send(character).unwrap();
}
