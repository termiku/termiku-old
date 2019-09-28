// A good lot of this code is taken from glium/examples/image.rs
// For now, we only want a window capable of receiving keyboard inputs as a basis for future work
use crate::bridge::spawn_process;

use mio_extras::channel::Sender;

use glium::{glutin, Surface};

use std::io::Cursor;

use glium::glutin::event::{Event, KeyboardInput, StartCause, WindowEvent};
use glium::glutin::event_loop::{ControlFlow, EventLoop};
use std::time::{Duration, Instant};

use std::borrow::Cow;

use arrayvec::*;
use glium::index::PrimitiveType;

use rusttype::gpu_cache::Cache;
use rusttype::{point, vector, Font, PositionedGlyph, Rect, Scale};

pub fn window(program: &str, args: &[&str]) {
    let process_sender = spawn_process(program, args);
    let events_loop = EventLoop::new();
    let window_builder = glutin::window::WindowBuilder::new()
        .with_inner_size(glutin::dpi::LogicalSize::new(1280.0, 720.0))
        .with_title("mou ikkai");
    let context_builder = glutin::ContextBuilder::new();
    let display = glium::Display::new(window_builder, context_builder, &events_loop).unwrap();

    let font_data = include_bytes!(
        "/home/shinysaana/.local/share/fonts/Roboto Mono Nerd Font Complete Mono.ttf"
    );

    let image = image::load(
        Cursor::new(&include_bytes!("../images/miku.jpg")[..]),
        image::JPEG,
    )
    .unwrap()
    .to_rgba();
    let dpi_factor = display.gl_window().window().hidpi_factor();
    let (cache_width, cache_height) = (512 * dpi_factor as u32, 512 * dpi_factor as u32);
    let mut cache = Cache::builder()
        .dimensions(cache_width, cache_height)
        .build();

    let font = Font::from_bytes(font_data as &[u8]).unwrap();
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

    let char_cache_tex = glium::texture::Texture2d::with_format(
        &display,
        glium::texture::RawImage2d {
            data: Cow::Owned(vec![128u8; cache_width as usize * cache_height as usize]),
            width: cache_width,
            height: cache_height,
            format: glium::texture::ClientFormat::U8,
        },
        glium::texture::UncompressedFloatFormat::U8,
        glium::texture::MipmapsOption::NoMipmap,
    )
    .unwrap();

    start_loop(events_loop, move |events| {
        let a_glyph = font.glyph('R');
        let a_glyph_positionned = a_glyph
            .scaled(Scale::uniform(24.0 * dpi_factor as f32))
            .positioned(point(50.0, 50.0));
        cache.queue_glyph(0, a_glyph_positionned.clone());
        let glyphs = vec![a_glyph_positionned];
        cache
            .cache_queued(|rect, data| {
                char_cache_tex.main_level().write(
                    glium::Rect {
                        left: rect.min.x,
                        bottom: rect.min.y,
                        width: rect.width(),
                        height: rect.height(),
                    },
                    glium::texture::RawImage2d {
                        data: Cow::Borrowed(data),
                        width: rect.width(),
                        height: rect.height(),
                        format: glium::texture::ClientFormat::U8,
                    },
                );
            })
            .unwrap();
        let (char_vertex_buffer, char_uniforms) = {
            let sampler = char_cache_tex
                .sampled()
                .magnify_filter(glium::uniforms::MagnifySamplerFilter::Nearest);
            let char_uniforms = uniform! {
                tex: sampler
            };
            let char_vertex_buffer = {
                #[derive(Copy, Clone)]
                struct Vertex {
                    position: [f32; 2],
                    tex_coords: [f32; 2],
                    colour: [f32; 4],
                }
                implement_vertex!(Vertex, position, tex_coords, colour);
                let colour = [0.5, 0.5, 0.5, 1.0];
                let (screen_width, screen_height) = {
                    let (w, h) = display.get_framebuffer_dimensions();
                    (w as f32, h as f32)
                };
                let origin = point(0.0, 0.0);
                let vertices: Vec<Vertex> = glyphs
                    .iter()
                    .flat_map(|g| {
                        if let Ok(Some((uv_rect, screen_rect))) = cache.rect_for(0, g) {
                            let gl_rect = Rect {
                                min: origin
                                    + (vector(
                                        screen_rect.min.x as f32 / screen_width - 0.5,
                                        1.0 - screen_rect.min.y as f32 / screen_height - 0.5,
                                    )) * 2.0,
                                max: origin
                                    + (vector(
                                        screen_rect.max.x as f32 / screen_width - 0.5,
                                        1.0 - screen_rect.max.y as f32 / screen_height - 0.5,
                                    )) * 2.0,
                            };
                            arrayvec::ArrayVec::<[Vertex; 6]>::from([
                                Vertex {
                                    position: [gl_rect.min.x, gl_rect.max.y],
                                    tex_coords: [uv_rect.min.x, uv_rect.max.y],
                                    colour,
                                },
                                Vertex {
                                    position: [gl_rect.min.x, gl_rect.min.y],
                                    tex_coords: [uv_rect.min.x, uv_rect.min.y],
                                    colour,
                                },
                                Vertex {
                                    position: [gl_rect.max.x, gl_rect.min.y],
                                    tex_coords: [uv_rect.max.x, uv_rect.min.y],
                                    colour,
                                },
                                Vertex {
                                    position: [gl_rect.max.x, gl_rect.min.y],
                                    tex_coords: [uv_rect.max.x, uv_rect.min.y],
                                    colour,
                                },
                                Vertex {
                                    position: [gl_rect.max.x, gl_rect.max.y],
                                    tex_coords: [uv_rect.max.x, uv_rect.max.y],
                                    colour,
                                },
                                Vertex {
                                    position: [gl_rect.min.x, gl_rect.max.y],
                                    tex_coords: [uv_rect.min.x, uv_rect.max.y],
                                    colour,
                                },
                            ])
                        } else {
                            arrayvec::ArrayVec::new()
                        }
                    })
                    .collect();

                glium::VertexBuffer::new(&display, &vertices).unwrap()
            };
            (char_vertex_buffer, char_uniforms)
        };
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
        // target
        //     .draw(
        //         &vertex_buffer,
        //         &index_buffer,
        //         &program,
        //         &uniforms,
        //         &Default::default(),
        //     )
        //     .unwrap();
        target
            .draw(
                &char_vertex_buffer,
                glium::index::NoIndices(glium::index::PrimitiveType::TrianglesList),
                &char_program,
                &char_uniforms,
                &glium::DrawParameters {
                    blend: glium::Blend::alpha_blending(),
                    ..Default::default()
                },
            )
            .unwrap();

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

fn layout_paragraph<'a>(
    font: &'a Font,
    scale: Scale,
    width: u32,
    text: &str,
) -> Vec<PositionedGlyph<'a>> {
    let mut result = Vec::new();
    let v_metrics = font.v_metrics(scale);
    let advance_height = v_metrics.ascent - v_metrics.descent + v_metrics.line_gap;
    let mut caret = point(0.0, v_metrics.ascent);
    let mut last_glyph_id = None;
    for c in text.chars() {
        if c.is_control() {
            match c {
                '\r' => {
                    caret = point(0.0, caret.y + advance_height);
                }
                '\n' => {}
                _ => {}
            }
            continue;
        }
        let base_glyph = font.glyph(c);
        if let Some(id) = last_glyph_id.take() {
            caret.x += font.pair_kerning(scale, id, base_glyph.id());
        }
        last_glyph_id = Some(base_glyph.id());
        let mut glyph = base_glyph.scaled(scale).positioned(caret);
        if let Some(bb) = glyph.pixel_bounding_box() {
            if bb.max.x > width as i32 {
                caret = point(0.0, caret.y + advance_height);
                glyph.set_position(caret);
                last_glyph_id = None;
            }
        }
        caret.x += glyph.unpositioned().h_metrics().advance_width;
        result.push(glyph);
    }
    result
}
