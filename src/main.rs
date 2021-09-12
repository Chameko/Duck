#![warn(clippy::all, clippy::pedantic)]

mod window;
mod document;
mod ducka;
mod error;

use petgraph::{Graph};
use crossterm::terminal::{EnterAlternateScreen, LeaveAlternateScreen, Clear, ClearType, size, enable_raw_mode, disable_raw_mode};
use crossterm::{execute};
use std::io::{Write, stdout,};
use crossterm::event::{read, Event, KeyEvent, KeyCode};
use window::{WinType, Split, Window};
use document::{Doc};

use crate::window::WindowSystem;

fn main() {
    let mut stdout = stdout();
    let size = size().unwrap();
    let mut tree = Graph::<Window, Split>::new();

    let mut ws = WindowSystem::new(size);

    enable_raw_mode().unwrap();
    execute!(stdout, Clear(ClearType::All), EnterAlternateScreen );

    stdout.flush().unwrap();

    let (win1, win2) = ws.new_vertical(WinType::Text(String::from("Yas queen")), WinType::Text(String::from("Yas king")), 0.5);

    ws.render(ws.root, &mut stdout);

    ws.focused = win2;

    stdout.flush().unwrap();

    loop {
        match read().unwrap() {
            Event::Key(KeyEvent{code: KeyCode::Char('q'), modifiers: _}) => break,
            Event::Key(KeyEvent{code: KeyCode::Char('v'), modifiers: _}) => {ws.vertical(WinType::Text(String::from("n")), 0.5).unwrap();},
            Event::Key(KeyEvent{code: KeyCode::Char('b'), modifiers: _}) => {ws.vertical(WinType::Text(String::from("q")), 0.5).unwrap();},
            _ => continue
        }
        ws.render(ws.root, &mut stdout);
        stdout.flush().unwrap();
    }
    execute!(stdout, LeaveAlternateScreen).unwrap();
    disable_raw_mode().unwrap();
    println!("{}", ws.gen_graph());
}
