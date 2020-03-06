use std::convert::TryFrom;
use std::fmt::Debug;
use std::io;

/// A node in the audio processing graph.
pub trait Node: Debug + 'static {
    /// Get a list of node inputs.
    fn inputs(&self) -> &[SignalRef];
}

/// An audio processing graph.
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
}

/// A reference to a signal in the audio processing graph.
#[derive(Debug, Copy, Clone)]
pub struct SignalRef(u32);
