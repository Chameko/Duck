#![warn(clippy::all, clippy::pedantic)]

mod window;
mod document;
mod ducka;

use petgraph::{Graph};
use crossterm::terminal::{EnterAlternateScreen, LeaveAlternateScreen, Clear, ClearType, size, enable_raw_mode, disable_raw_mode};
use crossterm::{execute};
use std::io::{Write, stdout,};
use crossterm::event::{read, Event, KeyEvent, KeyCode};
use window::{WinType, Split, Window};
use document::{Doc};

fn main() {
    let mut stdout = stdout();
    let size = size().unwrap();
    let mut tree = Graph::<Window, Split>::new();

    enable_raw_mode().unwrap();
    execute!(stdout, Clear(ClearType::All), EnterAlternateScreen );

    stdout.flush().unwrap();
    loop {
        match read().unwrap() {
            Event::Key(KeyEvent{code: KeyCode::Char('q'), modifiers: _}) => break,
            _ => continue
        }
    }
    execute!(stdout, LeaveAlternateScreen);
    disable_raw_mode().unwrap();
}
