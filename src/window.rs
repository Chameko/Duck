#![warn(clippy::all, clippy::pedantic)]
use petgraph::{stable_graph::StableGraph, prelude::*, Direction};
use crossterm::style::{Print, Stylize};
use crossterm::cursor::MoveTo;
use crossterm::{queue, execute};
use std::io::{Stdout};
use crate::document::Doc;
use unicode_segmentation::UnicodeSegmentation;

use crossterm::terminal::size;
// Represents the window types
pub enum WinType {
    Text(String),
    Document(Doc),
    Empty,
}

#[derive(Clone, Debug)]
pub struct Position(pub u16, pub u16);

// Represents a split in the window
#[derive(Clone, Debug)]
pub enum Split {
    Vertical(u16), // Percentage of the window the split takes
    Horizontal(u16), // Percentage of the window the split takes
}

// Represents a drawable surface
pub struct Window {
    pub win_t: WinType, // Type of window
    pub top_l: Position, // The top left of the window
    pub bottom_r: Position // The bottom right of the window
}

pub struct WindowSystem {
    focused: NodeIndex, // Holds the currently focused window
    tree: StableGraph<Window, Split>, // Holds the graph containing the window tree
    size: (u16, u16),
}


impl WindowSystem {
    // Render a specific window
    pub fn render_window(&self, window: NodeIndex, stdout: &mut Stdout) {
        let window = self.tree.node_weight(window).unwrap();

        match window {
            Window{win_t: WinType::Text(content), top_l, bottom_r} => WindowSystem::draw_c(String::from(content), top_l, bottom_r, stdout),
            Window{win_t: WinType::Document(doc), top_l, bottom_r} => WindowSystem::draw_d(doc, top_l, bottom_r, stdout),
            Window{win_t: WinType::Empty, ..} => (),
        }
    }

    // Render the entire screen
    pub fn render(&self, root: NodeIndex, stdout: &mut Stdout) {
        // For every node that isn't drawable it will call this function on. 
        for node in self.tree.neighbors_directed(root, Direction::Outgoing) {
            let weight = self.tree.node_weight(node).unwrap();
            match  weight {
                Window{win_t: WinType::Empty, top_l: _, bottom_r: _} => self.render(node, stdout),
                Window{win_t: WinType::Text(content), top_l, bottom_r} => WindowSystem::draw_c(String::from(content), top_l, bottom_r, stdout),
                Window{win_t: WinType::Document(doc), top_l, bottom_r} => WindowSystem::draw_d(doc, top_l, bottom_r, stdout),
                _ => println!("window type not implemented")
            }
            match self.tree.edge_weight(self.tree.find_edge(root, node).unwrap()).unwrap() {
                Split::Vertical(col) => WindowSystem::draw_v(*col, (weight.top_l.1, weight.bottom_r.1), stdout),
                Split::Horizontal(row) => WindowSystem::draw_h(*row, (weight.top_l.0, weight.bottom_r.0), stdout),
                _ => println!("Split not supported")
            }
        }
    }

    // Write very basic text
    fn draw_c(content: String, top_l: &Position, _bottom_r: &Position, stdout: &mut Stdout) {
        queue!(stdout, MoveTo(top_l.0, top_l.1), Print(content));
    }

    fn draw_d(doc: &Doc, top_l: &Position, bottom_r: &Position, stdout: &mut Stdout) {
        let mut row = top_l.1;
        let mut col = top_l.0;
        let row_l = bottom_r.1;
        let col_l = bottom_r.0;

        for lines in &doc.rows {
            if row >= row_l {
                break;
            }
            for g in lines.string.graphemes(true).collect::<Vec<&str>>() {
                if col >= col_l {
                    break;
                }
                queue!(stdout, MoveTo(col, row), Print(g));
                col += 1;
            }
            col = top_l.0;
            row += 1;
        }
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

    pub fn new_vertical(&mut self, win_t: WinType, win_t2: WinType, percentage: u16) -> (NodeIndex, NodeIndex) {
        // Create a clone of the parent to use as adding a node mutable borrows it, should be safe as any changes made to parent are just adding the node and not modifying the weights
        let parent_top_l = self.tree.node_weight(self.focused).unwrap().top_l.clone();
        let parent_bottom_r = self.tree.node_weight(self.focused).unwrap().bottom_r.clone();

        let win_a = self.tree.add_node( Window {
            win_t,
            top_l: parent_top_l.clone(),
            bottom_r: Position(parent_bottom_r.0 / percentage - 1, parent_bottom_r.1)
        });
        let win_b = self.tree.add_node( Window {
            win_t: win_t2,
            top_l: Position(parent_bottom_r.0 / percentage + 1, parent_top_l.1),
            bottom_r: parent_bottom_r.clone(),
        });

        self.tree.add_edge(self.focused, win_a, Split::Vertical(percentage));
        self.tree.add_edge(self.focused, win_b, Split::Vertical(percentage));

        (win_a, win_b)
    }

    pub fn new_horizontal(&mut self, win_t: WinType, win_t2: WinType, percentage: u16) -> (NodeIndex, NodeIndex) {
        // Create a clone of the parent to use as adding a node mutable borrows it, should be safe as any changes made to parent are just adding the node and not modifying the wieghts
        let parent_top_l = self.tree.node_weight(self.focused).unwrap().top_l.clone();
        let parent_bottom_r = self.tree.node_weight(self.focused).unwrap().bottom_r.clone();

        let win_a = self.tree.add_node( Window {
            win_t,
            top_l: parent_top_l.clone(),
            bottom_r: Position(parent_top_l.0, parent_bottom_r.1 / percentage - 1)
        });
        let win_b = self.tree.add_node( Window {
            win_t: win_t2,
            top_l: Position(parent_top_l.0, parent_bottom_r.1 / percentage + 1),
            bottom_r: parent_bottom_r.clone(),
        });

        self.tree.add_edge(self.focused, win_a, Split::Horizontal(percentage));
        self.tree.add_edge(self.focused, win_b, Split::Horizontal(percentage));

        (win_a, win_b)
    }

    pub fn root(&mut self, size: (u16, u16)) -> NodeIndex {
        self.tree.add_node( Window {
            win_t: WinType::Empty,
            top_l: Position(0, 0),
            bottom_r: Position(size.0, size.1)
        })
    }

    pub fn vertical(&mut self, win_t: WinType, percentage: u16) -> NodeIndex {
        let focused = self.focused;
        let parent = self.get_parent(&focused);

        // Create coppies of the various window positions
        let parent_top_l = self.tree.node_weight(parent).unwrap().top_l.clone();
        let parent_bottom_r = self.tree.node_weight(parent).unwrap().bottom_r.clone();
        let f_win_top_l = self.tree.node_weight(focused).unwrap().top_l.clone();
        let f_win_bottom_r = self.tree.node_weight(focused).unwrap().bottom_r.clone();

        // Create the new window 
        let n_win = self.tree.add_node(Window {
            win_t,
            top_l: parent_top_l.clone(),
            bottom_r: Position(parent_bottom_r.0 / percentage - 1, parent_bottom_r.1)
        });

        // Create the container that parent will point to
        let c_win = self.tree.add_node(Window {
            win_t: WinType::Empty,
            top_l: f_win_top_l.clone(),
            bottom_r: f_win_bottom_r.clone()
        });

        let p_to_f = self.tree.find_edge(parent, focused).unwrap();

        // Create edge between new container and parent
        self.tree.add_edge(parent, c_win, self.tree.edge_weight(p_to_f).unwrap().clone());

        // Remove edge between parent and focused
        self.tree.remove_edge(p_to_f).unwrap();

        // Add edges between container, the new window and the focused
        self.tree.add_edge(c_win, n_win, Split::Vertical(percentage));
        self.tree.add_edge(c_win, focused, Split::Vertical(percentage));

        n_win
    }

    pub fn remove(&mut self, win: NodeIndex) -> NodeIndex {
        let parent = self.get_parent(&win);
        self.remove(win);
        parent
    }

    pub fn get_parent(&mut self, win: &NodeIndex) -> NodeIndex {
        self.tree.neighbors_directed(*win, Direction::Incoming).last().unwrap()
    }
}