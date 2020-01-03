#![allow(dead_code)]

// I just hate implementing default when I know I'll have to remove it later :/
#![allow(clippy::new_without_default)]

#[macro_use]
extern crate glium;

pub mod pty;

// Allowed because of the implement_vertex! macro which will trigger this clippy lint, outside
// of our control
#[allow(clippy::unneeded_field_pattern)]
pub mod window;
pub mod freetype;
pub mod harfbuzz;
pub mod config;
pub mod atlas;

// Allowed because of the implement_vertex! macro which will trigger this clippy lint, outside
// of our control
#[allow(clippy::unneeded_field_pattern)]
pub mod draw;
pub mod pty_buffer;
pub mod term;
pub mod rasterizer;
pub mod window_event;
pub mod unicode;
pub mod control;
pub mod youtube;
