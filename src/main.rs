extern crate petgraph;

use petgraph::{Graph, prelude::*, Direction};
use crossterm::terminal::*;
use crossterm::style::*;
use crossterm::cursor::MoveTo;
use crossterm::{queue, execute};
use std::io::{Write, stdout, Stdout};
use crossterm::event::{read, Event, KeyEvent, KeyCode};

#[derive(Clone)]
pub enum WinType {
    Text(String),
    Menu,
    Empty,
}

#[derive(Clone)]
pub struct Position(u16, u16);

// Represents a split in the window
#[derive(Clone, Debug)]
pub enum Split {
    Vertical(u16), // Percentage of the window the split takes
    Horizontal(u16), // Percentage of the window the split takes
}

pub struct SplitFull(EdgeIndex, EdgeIndex);

// Represents a drawable surface
#[derive(Clone)]
pub struct Window {
    win_t: WinType, // Type of window
    top_l: Position, // The top left of the window
    bottom_r: Position // The bottom right of the window
}

fn render(tree: &Graph<Window, Split>, root: NodeIndex, stdout: &mut Stdout) {
    for node in tree.neighbors_directed(root, Direction::Outgoing) {
        let weight = tree.node_weight(node).unwrap();
        match  weight {
            Window{win_t: WinType::Empty, top_l, bottom_r} => render(tree, node, stdout),
            Window{win_t: WinType::Text(content), top_l, bottom_r} => draw_c(String::from(content), top_l, bottom_r, stdout),
            _ => println!("window type not implemented")
        }
        match tree.edges_connecting(root, node).map(|x| x.weight()).next().unwrap() {
            Split::Vertical(col) => draw_v(col, (weight.top_l.1, weight.bottom_r.1), stdout),
            Split::Horizontal(row) => draw_h(row, (weight.top_l.0, weight.bottom_r.0), stdout),
            _ => println!("Split not supported")
        }
    }
}

fn draw_c(content: String, top_l: &Position, bottom_r: &Position, stdout: &mut Stdout) {
    queue!(stdout, MoveTo(top_l.0, top_l.1), Print(content));
}

fn draw_v(col: &u16, range: (u16, u16), stdout: &mut Stdout) {
    for row in range.0..range.1 {
        queue!(stdout, MoveTo(*col, row), Print("|".negative()));
    }
}

fn draw_h(row: &u16, range: (u16, u16), stdout: &mut Stdout) {
    for col in range.0..range.1 {
        queue!(stdout, MoveTo(col, *row), Print("-".negative()));
    }
}

fn new_vertical(tree: &mut Graph<Window, Split>, win_t: WinType, win_t2: WinType, parent: NodeIndex, percentage: u16) -> (NodeIndex, NodeIndex, SplitFull) {
    let parent_win = tree.node_weight(parent).unwrap().clone();
    let winA = tree.add_node( Window {
        win_t,
        top_l: parent_win.top_l.clone(),
        bottom_r: Position(parent_win.bottom_r.0 / percentage - 1, parent_win.bottom_r.1)
    });
    let winB = tree.add_node( Window {
        win_t: win_t2,
        top_l: Position(parent_win.bottom_r.0 / percentage + 1, parent_win.top_l.1),
        bottom_r: parent_win.bottom_r.clone(),
    });
    let split_op = SplitFull(tree.add_edge(parent, winA, Split::Vertical(parent_win.bottom_r.0 / percentage)), tree.add_edge(parent, winB, Split::Vertical(parent_win.bottom_r.0 / percentage)));

    (winA, winB, split_op)
}

fn new_horizontal(tree: &mut Graph<Window, Split>, win_t: WinType, win_t2: WinType, parent: NodeIndex, percentage: u16) -> (NodeIndex, NodeIndex, SplitFull) {
    let parent_win = tree.node_weight(parent).unwrap().clone();
    let winA = tree.add_node( Window {
        win_t,
        top_l: parent_win.top_l.clone(),
        bottom_r: Position(parent_win.top_l.0, parent_win.bottom_r.1 / percentage - 1)
    });
    let winB = tree.add_node( Window {
        win_t: win_t2,
        top_l: Position(parent_win.top_l.0, parent_win.bottom_r.1 / percentage + 1),
        bottom_r: parent_win.bottom_r.clone(),
    });
    let split_op = SplitFull(tree.add_edge(parent, winA, Split::Horizontal(parent_win.bottom_r.1 / percentage)), tree.add_edge(parent, winB, Split::Horizontal(parent_win.bottom_r.1 / percentage)));

    (winA, winB, split_op)
}

fn main() {
    let mut stdout = stdout();
    let size = size().unwrap();
    let mut tree = Graph::<Window, Split>::new();
    let root = tree.add_node( Window {
        win_t: WinType::Empty,
        top_l: Position(0, 0),
        bottom_r: Position(size.0, size.1 - 1)
    });

    let (win1, win2, split_op_A) = new_vertical(&mut tree, WinType::Text(String::from("Yas queen")), WinType::Empty, root, 2);
    
    let (win3, win4, split_op_B) = new_horizontal(&mut tree, WinType::Text(String::from("Yas King")), WinType::Text(String::from("Yas mate")), win2, 2);

    enable_raw_mode().unwrap();
    execute!(stdout, Clear(ClearType::All), EnterAlternateScreen );
    render(&tree, root, &mut stdout);
    stdout.flush().unwrap();
    loop {
        match read().unwrap() {
            Event::Key(KeyEvent{code: KeyCode::Char('q'), modifiers}) => break,
            _ => continue
        }
    }
    execute!(stdout, LeaveAlternateScreen);
}
