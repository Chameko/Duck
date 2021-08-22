#![warn(clippy::all, clippy::pedantic)]
use petgraph::{Graph, prelude::*, Direction};
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
#[derive(Debug)]
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
    tree: Graph<Window, Split>, // Holds the graph containing the window tree
    size: (u16, u16),
}


impl WindowSystem {
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
            match self.tree.edges_connecting(root, node).next().unwrap().weight() {
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
                execute!(stdout, MoveTo(0, size().unwrap().1), Print(format!("row: {:?} row_l: {:?}", row, row_l)));
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
        // Create a clone of the parent to use as adding a node mutable borrows it, should be safe as any changes made to parent are just adding the node and not modifying the wieghts
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

        (win_a, win_b)
    }

    pub fn root(tree: &mut Graph<Window, Split>, size: (u16, u16)) -> NodeIndex {
        tree.add_node( Window {
            win_t: WinType::Empty,
            top_l: Position(0, 0),
            bottom_r: Position(size.0, size.1)
        })
    }
}