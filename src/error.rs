use petgraph::graph::{EdgeIndex, NodeIndex};
use petgraph::stable_graph::StableGraph;
use std::fmt;
use crate::window::{Split, Window};

#[derive(Debug)]
pub enum WinError {
    GetSplitError(EdgeIndex, String),
    GetWindowError(NodeIndex, String),
    FindSplitError(NodeIndex, NodeIndex, String),
    FindWindowError(NodeIndex, String),
    FindParentError(NodeIndex, String),
    RemoveSplitError(EdgeIndex, String),
    RemoveNodeError(NodeIndex, String),
}

impl fmt::Display for WinError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        use WinError::*;
        match self {
            GetSplitError(err, graph) => write!(f, "Couldn't access split Index: {} data. Tree: {}", err.index(), graph),
            GetWindowError(err, graph) => write!(f, "Couldn't access window Index: {} data Tree: {}", err.index(), graph),
            FindSplitError(n1, n2, graph) => write!(f, "Could not find split between windows Index: {} and Index: {} Tree: {}", n1.index(), n2.index(), graph),
            FindWindowError(err, graph) => write!(f, "Could not find window. Index: {} Tree: {}", err.index(), graph),
            FindParentError(err, graph) => write!(f, "Couldn't find parent of window. Index: {} Tree: {}", err.index(), graph),
            RemoveSplitError(err, graph) => write!(f, "Edge attempting to be removed doesn't exist. Index: {} Tree: {}", err.index(), graph),
            RemoveNodeError(err, graph) => write!(f, "Node attempted to be removed doesn't exist. Index: {} Tree: {}", err.index(), graph)
        }
    }
}