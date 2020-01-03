#![allow(dead_code)]

// I just hate implementing default when I know I'll have to remove it later :/
#![allow(clippy::new_without_default)]

#[macro_use]
extern crate glium;

pub mod atlas;
pub mod config;
pub mod control;
// Allowed because of the implement_vertex! macro which will trigger this clippy lint, outside
// of our control
#[allow(clippy::unneeded_field_pattern)]
pub mod draw;
pub mod freetype;
pub mod harfbuzz;
pub mod pty;
pub mod pty_buffer;
pub mod rasterizer;
pub mod term;
pub mod unicode;
// Allowed because of the implement_vertex! macro which will trigger this clippy lint, outside
// of our control
#[allow(clippy::unneeded_field_pattern)]
pub mod window;
pub mod window_event;
pub mod youtube;
