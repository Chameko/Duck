use crate::window::{WindowSystem};
use crossterm::event::{ Event, KeyCode, KeyEvent, KeyModifiers };

// Manage entire app
pub struct Duck {
    window_s: WindowSystem, // Holds the window system
    command_l: u16, // Holds the row to be used to enter command inputs
    status_l: u16, // Holds the row to be used to output
}

pub enum RawKey {
    Char(char),
    Up,
    Down,
    Left,
    Right,
    Enter,
    Backspace,
}

pub enum KeyBinding {
    Ctrl(RawKey),
    Alt(RawKey),
    Shift(RawKey),
    F(u8),
    Unsupported
}

impl Duck {
    pub fn to_duck_input(event: Event) {

    }

    fn quit() {
        std::process::exit(0)
    }

    fn remove() {

    }
}