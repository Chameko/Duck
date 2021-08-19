#![warn(clippy::all, clippy::pedantic)]

mod window;
mod document;

use petgraph::{Graph};
use crossterm::terminal::{EnterAlternateScreen, LeaveAlternateScreen, Clear, ClearType, size, enable_raw_mode, disable_raw_mode};
use crossterm::{execute};
use std::io::{Write, stdout,};
use crossterm::event::{read, Event, KeyEvent, KeyCode};
use window::{WinType, Split, Window, new_vertical, new_horizontal, render, root};
use document::{Doc};

fn main() {
    let mut stdout = stdout();
    let size = size().unwrap();
    let mut tree = Graph::<Window, Split>::new();
    let root = root(&mut tree, (size.0, size.1 - 1));

    let mut doc = Doc::new();
    doc.open(String::from("poem.txt"));

    let mut doc2 = Doc::new();
    doc2.open(String::from("poem.txt"));

    let (_win1, win2, _split_op_a) = new_vertical(&mut tree, WinType::Document(doc), WinType::Empty, root, 2);
    
    let (_win3, _win4, _split_op_b) = new_horizontal(&mut tree, WinType::Text(String::from("Yas King")), WinType::Document(doc2), win2, 2);

    enable_raw_mode().unwrap();
    execute!(stdout, Clear(ClearType::All), EnterAlternateScreen );
    render(&tree, root, &mut stdout);

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
