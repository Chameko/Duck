#![warn(clippy::all, clippy::pedantic)]
use petgraph::{Graph, prelude::*, Direction};
use crossterm::style::{Print, Stylize};
use crossterm::cursor::MoveTo;
use crossterm::{queue};
use std::io::{Stdout};

// Represents the window types
#[derive(Clone)]
pub enum WinType {
    Text(String),
    Empty,
}

#[derive(Clone)]
pub struct Position(pub u16, pub u16);

// Represents the splits created by the action of splitting a window
pub struct SplitFull(EdgeIndex, EdgeIndex);

// Represents a split in the window
#[derive(Clone, Debug)]
pub enum Split {
    Vertical(u16), // Percentage of the window the split takes
    Horizontal(u16), // Percentage of the window the split takes
}

// Represents a drawable surface
#[derive(Clone)]
pub struct Window {
    win_t: WinType, // Type of window
    top_l: Position, // The top left of the window
    bottom_r: Position // The bottom right of the window
}

pub fn render(tree: &Graph<Window, Split>, root: NodeIndex, stdout: &mut Stdout) {
    // For every node that isn't drawable it will call this function on. 
    for node in tree.neighbors_directed(root, Direction::Outgoing) {
        let weight = tree.node_weight(node).unwrap();
        match  weight {
            Window{win_t: WinType::Empty, top_l, bottom_r} => render(tree, node, stdout),
            Window{win_t: WinType::Text(content), top_l, bottom_r} => draw_c(String::from(content), top_l, bottom_r, stdout),
            _ => println!("window type not implemented")
        }
        match tree.edges_connecting(root, node).map(|x| x.weight()).next().unwrap() {
            Split::Vertical(col) => draw_v(*col, (weight.top_l.1, weight.bottom_r.1), stdout),
            Split::Horizontal(row) => draw_h(*row, (weight.top_l.0, weight.bottom_r.0), stdout),
            _ => println!("Split not supported")
        }
    }
}

// Write very basic text
fn draw_c(content: String, top_l: &Position, _bottom_r: &Position, stdout: &mut Stdout) {
    queue!(stdout, MoveTo(top_l.0, top_l.1), Print(content));
}

// Draw a vertical split
fn draw_v(col: u16, range: (u16, u16), stdout: &mut Stdout) {
    for row in range.0..range.1 {
        queue!(stdout, MoveTo(col, row), Print("|".negative()));
    }
}

// Draw a horizontal split
fn draw_h(row: u16, range: (u16, u16), stdout: &mut Stdout) {
    for col in range.0..range.1 {
        queue!(stdout, MoveTo(col, row), Print("-".negative()));
    }
}

pub fn new_vertical(tree: &mut Graph<Window, Split>, win_t: WinType, win_t2: WinType, parent: NodeIndex, percentage: u16) -> (NodeIndex, NodeIndex, SplitFull) {
    // Create a clone of the parent to use as adding a node mutable borrows it, should be safe as any changes made to parent are just adding the node and not modifying the wieghts
    let parent_win = tree.node_weight(parent).unwrap().clone();
    let win_a = tree.add_node( Window {
        win_t,
        top_l: parent_win.top_l.clone(),
        bottom_r: Position(parent_win.bottom_r.0 / percentage - 1, parent_win.bottom_r.1)
    });
    let win_b = tree.add_node( Window {
        win_t: win_t2,
        top_l: Position(parent_win.bottom_r.0 / percentage + 1, parent_win.top_l.1),
        bottom_r: parent_win.bottom_r.clone(),
    });
    let split_op = SplitFull(tree.add_edge(parent, win_a, Split::Vertical(parent_win.bottom_r.0 / percentage)), tree.add_edge(parent, win_b, Split::Vertical(parent_win.bottom_r.0 / percentage)));

    (win_a, win_b, split_op)
}

pub fn new_horizontal(tree: &mut Graph<Window, Split>, win_t: WinType, win_t2: WinType, parent: NodeIndex, percentage: u16) -> (NodeIndex, NodeIndex, SplitFull) {
    // Create a clone of the parent to use as adding a node mutable borrows it, should be safe as any changes made to parent are just adding the node and not modifying the wieghts
    let parent_win = tree.node_weight(parent).unwrap().clone();
    let win_a = tree.add_node( Window {
        win_t,
        top_l: parent_win.top_l.clone(),
        bottom_r: Position(parent_win.top_l.0, parent_win.bottom_r.1 / percentage - 1)
    });
    let win_b = tree.add_node( Window {
        win_t: win_t2,
        top_l: Position(parent_win.top_l.0, parent_win.bottom_r.1 / percentage + 1),
        bottom_r: parent_win.bottom_r.clone(),
    });
    let split_op = SplitFull(tree.add_edge(parent, win_a, Split::Horizontal(parent_win.bottom_r.1 / percentage)), tree.add_edge(parent, win_b, Split::Horizontal(parent_win.bottom_r.1 / percentage)));

    (win_a, win_b, split_op)
}

pub fn root(tree: &mut Graph<Window, Split>, size: (u16, u16)) -> NodeIndex {
    tree.add_node( Window {
        win_t: WinType::Empty,
        top_l: Position(0, 0),
        bottom_r: Position(size.0, size.1)
    })
}
