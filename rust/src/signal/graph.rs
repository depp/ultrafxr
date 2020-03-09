use super::program::{Function, Parameters};
use std::convert::TryFrom;
use std::error::Error;
use std::fmt::Debug;
use std::io;

/// Result of instantiating a node.
pub type NodeResult = Result<Box<dyn Function>, Box<dyn Error>>;

/// A node in the audio processing graph description.
pub trait Node: Debug {
    /// Get a list of node inputs.
    fn inputs(&self) -> &[SignalRef];

    /// Create an instance of the node's audio function.
    fn instantiate(&self, params: &Parameters) -> NodeResult;
}

/// Description of an audio processing graph.
pub struct Graph {
    nodes: Vec<Box<dyn Node>>,
}

impl Graph {
    /// Create a new, empty graph.
    pub fn new() -> Self {
        Graph { nodes: Vec::new() }
    }

    /// Add a new node to the graph.
    pub fn add(&mut self, node: Box<dyn Node>) -> SignalRef {
        for &SignalRef(idx) in node.inputs().iter() {
            if idx as usize >= self.nodes.len() {
                panic!("node input out of range");
            }
        }
        let idx = u32::try_from(self.nodes.len()).unwrap();
        self.nodes.push(node);
        SignalRef(idx)
    }

    /// Dump the graph to a stream in text format.
    pub fn dump(&self, f: &mut dyn io::Write) {
        for (n, node) in self.nodes.iter().enumerate() {
            writeln!(f, "{}: {:?}", n, node).unwrap();
        }
    }

    /// Return all nodes in the graph.
    pub fn nodes(&self) -> &[Box<dyn Node>] {
        &self.nodes
    }
}

/// A reference to a signal in the audio processing graph.
#[derive(Debug, Copy, Clone, PartialEq, Eq)]
pub struct SignalRef(pub u32);
