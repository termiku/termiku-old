use glium::glutin::event::{ElementState, KeyboardInput, VirtualKeyCode };

const UP_CONTROL_SEQUENCE:      &str = "\x1B[A";
const DOWN_CONTROL_SEQUENCE:    &str = "\x1B[B";
const RIGHT_CONTROL_SEQUENCE:   &str = "\x1B[C";
const LEFT_CONTROL_SEQUENCE:    &str = "\x1B[D";

#[derive(Copy, Clone, Debug)]
pub enum TermikuWindowEvent {
    CharacterInput(char),
    KeyboardArrow(KeyboardArrow)
}

#[derive(Copy, Clone, Debug)]
pub enum KeyboardArrow {
    Up,
    Down,
    Right,
    Left,
}

impl KeyboardArrow {
    pub fn to_control_sequence(self) -> &'static str {
        match self {
            KeyboardArrow::Up       => UP_CONTROL_SEQUENCE,
            KeyboardArrow::Down     => DOWN_CONTROL_SEQUENCE,
            KeyboardArrow::Right    => RIGHT_CONTROL_SEQUENCE,
            KeyboardArrow::Left     => LEFT_CONTROL_SEQUENCE,
        }
    }
}

pub fn handle_keyboard_input(input: &KeyboardInput) -> Option<TermikuWindowEvent> {
    if let Some(key_code) = input.virtual_keycode {
        use VirtualKeyCode::*;
        
        match key_code {
            Up => event_if_pressed(TermikuWindowEvent::KeyboardArrow(KeyboardArrow::Up), input),
            Down => event_if_pressed(TermikuWindowEvent::KeyboardArrow(KeyboardArrow::Down), input),
            Right => event_if_pressed(TermikuWindowEvent::KeyboardArrow(KeyboardArrow::Right), input),
            Left => event_if_pressed(TermikuWindowEvent::KeyboardArrow(KeyboardArrow::Left), input),
            _ => None
        }
    } else {
        None
    }
    
}

fn event_if_pressed(
    event: TermikuWindowEvent, input: &KeyboardInput
) -> Option<TermikuWindowEvent> {
    if pressed(input) {
        Some(event)
    } else {
        None
    }
}

fn pressed(input: &KeyboardInput) -> bool {
    input.state == ElementState::Pressed
}
