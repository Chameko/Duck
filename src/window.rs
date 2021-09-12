#![warn(clippy::all, clippy::pedantic)]
use crate::error::WinError;
use petgraph::{stable_graph::StableGraph, prelude::*, Direction};
use crossterm::style::{Print, Stylize};
use crossterm::cursor::MoveTo;
use crossterm::{queue, execute};
use std::io::{Stdout};
use crate::document::Doc;
use unicode_segmentation::UnicodeSegmentation;
use petgraph::dot::{Dot, Config};

type Result<T> = std::result::Result<T, WinError>;

// Represents the window types
#[derive(Debug)]
pub enum WinType {
    Text(String),
    Document(Doc),
    Empty,
}

#[derive(Clone, Debug, Copy)]
pub struct Position(pub u16, pub u16);

// Represents a split in the window
#[derive(Clone, Debug, Copy)]
pub enum Split {
    Vertical(u16), // Percentage of the window the split takes
    Horizontal(u16), // Percentage of the window the split takes
}

// Represents a drawable surface
#[derive(Debug)]
pub struct Window {
    pub win_t: WinType, // Type of window
    pub top_l: Position, // The top left of the window
    pub bottom_r: Position // The bottom right of the window
}

pub struct WindowSystem {
    pub focused: NodeIndex, // Holds the currently focused window
    tree: StableGraph<Window, Split>, // Holds the graph containing the window tree
    size: (u16, u16),
    pub root: NodeIndex
}


impl WindowSystem {
    pub fn new(size: (u16, u16)) -> WindowSystem {
        let mut tree: StableGraph<Window, Split> = StableGraph::new();

        let root = WindowSystem::root(size, &mut tree);

        WindowSystem {
            focused: root,
            tree: tree,
            root,
            size
        }
    }

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

    pub fn new_vertical(&mut self, win_t: WinType, win_t2: WinType, percentage: f32) -> (NodeIndex, NodeIndex) {
        // Clone the parents coordinates
        let parent_top_l = self.tree.node_weight(self.focused).unwrap().top_l;
        let parent_bottom_r = self.tree.node_weight(self.focused).unwrap().bottom_r;

        let area = ((parent_bottom_r.0 - parent_top_l.0) as f32, (parent_bottom_r.1 - parent_top_l.1) as f32);

        // Construct the two windows
        let win_a = self.tree.add_node( Window {
            win_t,
            top_l: parent_top_l,
            bottom_r: Position(parent_bottom_r.0 - (area.0 * percentage) as u16 - 1, parent_bottom_r.1)
        });
        let win_b = self.tree.add_node( Window {
            win_t: win_t2,
            top_l: Position(parent_bottom_r.0 - (area.0 * percentage) as u16 + 1, parent_top_l.1),
            bottom_r: parent_bottom_r,
        });

        self.tree.add_edge(self.focused, win_a, Split::Vertical(parent_bottom_r.0 - (area.0 * percentage) as u16));
        self.tree.add_edge(self.focused, win_b, Split::Vertical(parent_bottom_r.0 - (area.0 * percentage) as u16));

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

    fn root(size: (u16, u16), tree: &mut StableGraph<Window, Split>) -> NodeIndex {
        tree.add_node( Window {
            win_t: WinType::Empty,
            top_l: Position(0, 0),
            bottom_r: Position(size.0, size.1)
        })
    }

    pub fn vertical(&mut self, win_t: WinType, percentage: f32) -> Result<NodeIndex> {
        let parent = self.get_parent(&self.focused)?;
        let focused = self.focused;

        // Create copies of the various window positions
        let f_win_top_l = self.tree.node_weight(self.focused).ok_or_else(|| WinError::FindWindowError(self.focused, self.gen_graph()))?.top_l;
        let f_win_bottom_r = self.tree.node_weight(self.focused).ok_or_else(|| WinError::FindWindowError(self.focused, self.gen_graph()))?.bottom_r;

        let area = ((f_win_bottom_r.0 - f_win_top_l.0) as f32, (f_win_bottom_r.1 - f_win_top_l.1) as f32);

        // Create the new window 
        let n_win = self.tree.add_node(Window {
            win_t,
            top_l: f_win_top_l,
            bottom_r: Position(f_win_bottom_r.0 - (area.0 * percentage) as u16 - 1, f_win_bottom_r.1),
        });

        // Create the container that parent will point to
        let c_win = self.tree.add_node(Window {
            win_t: WinType::Empty,
            top_l: f_win_top_l.clone(),
            bottom_r: f_win_bottom_r.clone()
        });

        let p_to_f = self.tree.find_edge(parent, self.focused).ok_or_else(|| WinError::FindSplitError(parent, self.focused,self.gen_graph()))?;

        // Create edge between new container and parent
        self.tree.add_edge(parent, c_win, *self.tree.edge_weight(p_to_f).ok_or_else(|| WinError::GetSplitError(p_to_f,self.gen_graph()))?);

        // Remove edge between parent and focused
        self.tree.remove_edge(p_to_f).ok_or_else(|| WinError::RemoveSplitError(p_to_f,self.gen_graph()))?;

        // Resize focused and panic if it breaks
        self.tree.node_weight_mut(focused).unwrap().top_l = Position(f_win_top_l.0 + (area.0 * percentage) as u16 + 1, f_win_top_l.1);

        // Add edges between container, the new window and the focused
        self.tree.add_edge(c_win, n_win, Split::Vertical(f_win_bottom_r.0 - (area.0 * percentage) as u16));
        self.tree.add_edge(c_win, self.focused, Split::Vertical(f_win_bottom_r.0 - (area.0 * percentage) as u16));

        Ok(n_win)
    }

    pub fn remove(&mut self, win: NodeIndex) -> Result<NodeIndex> {
        let parent = self.get_parent(&win)?;
        self.tree.remove_node(win).ok_or_else(|| WinError::RemoveNodeError(win, self.gen_graph()))?;
        Ok(parent)
    }

    pub fn get_parent(&self, win: &NodeIndex) -> Result<NodeIndex> {
        self.tree.neighbors_directed(*win, Direction::Incoming).last().ok_or_else(|| WinError::FindParentError(*win, self.gen_graph()))
    }

    pub fn gen_graph(&self) -> String {
        format!("{:?}", Dot::with_config(&self.tree, &[Config::NodeIndexLabel, Config::EdgeNoLabel]))
    }
}