use super::graph::{Graph, SignalRef};
use std::cmp::min;
use std::error;
use std::fmt::{Debug, Display, Formatter, Result as FResult};

/// Parameters for instantiating a synthesizer program.
#[derive(Debug)]
pub struct Parameters {
    /// Audio sample rate, samples per second.
    pub sample_rate: f64,
    /// Size of audio buffers.
    pub buffer_size: usize,
}

/// Input to a synthesizer program.
#[derive(Debug)]
pub struct Input {
    /// The number of samples before the gate ends.
    pub gate: Option<usize>,
    /// MIDI note value (69 is A4, which sounds at 440 Hz).
    pub note: f32,
}

/// Audio program execution state.
pub struct State {
    gate: Option<usize>,
    note: f32,
    end: Option<usize>,
}

impl State {
    /// Get the number of samples before the gate ends.
    pub fn gate(&self) -> Option<usize> {
        self.gate
    }

    /// Get the MIDI note value.
    pub fn note(&self) -> f32 {
        self.note
    }

    /// Stop the program execution after the given number of samples.
    pub fn stop(&mut self, pos: usize) {
        self.end = Some(match self.end {
            None => pos,
            Some(oldpos) => min(pos, oldpos),
        });
    }
}

/// An audio function, consuming input buffers and filling an output buffer.
pub trait Function: Debug {
    /// Render the next output buffer.
    fn render(&mut self, output: &mut [f32], inputs: &[&[f32]], state: &mut State);
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Error {
    ContainsLoop,
    BadBuffer,
}

impl Display for Error {
    fn fmt(&self, f: &mut Formatter) -> FResult {
        f.write_str(match self {
            Error::ContainsLoop => "audio graph contains cycle",
            Error::BadBuffer => "invalid buffer size",
        })
    }
}

impl error::Error for Error {}

/// Metadata for a node in an audio program.
struct Node {
    function: Box<dyn Function>,
    input_count: usize,
    inputs: [usize; 4],
}

/// A program which can render audio.
pub struct Program {
    // The nodes are sorted in evaluation order, so each node's inputs are
    // strictly from previous nodes. The buffer is divided into chunks, with one
    // chunk for each node.
    buffer_size: usize,
    buffer: Box<[f32]>,
    nodes: Box<[Node]>,
    // If true, the program is done and has no more output.
    done: bool,
}

impl Program {
    /// Create a new program from an audio processing graph.
    pub fn new(
        graph: &Graph,
        output: SignalRef,
        parameters: &Parameters,
    ) -> Result<Self, Box<dyn error::Error>> {
        // State of a node in the graph.
        #[derive(Clone, Copy)]
        enum NodeState {
            Unvisited,
            Visiting,
            Visited(usize),
        }
        use NodeState::*;
        // Item in DFS stack.
        #[derive(Clone, Copy)]
        enum Action<'a> {
            Pre,
            Post(&'a [SignalRef]),
        }
        use Action::*;
        let gnodes = graph.nodes();
        let mut states = Vec::new();
        states.resize(gnodes.len(), NodeState::Unvisited);
        let mut stack = Vec::new();
        stack.push((output, Pre));
        let mut nodes: Vec<Node> = Vec::new();
        loop {
            let (sig, action) = match stack.pop() {
                Some(x) => x,
                None => break,
            };
            let state = &mut states[sig.0 as usize];
            match action {
                Pre => match *state {
                    Unvisited => {
                        *state = Visiting;
                        let inputs = gnodes[sig.0 as usize].inputs();
                        stack.push((sig, Post(inputs)));
                        for &input in inputs.iter() {
                            stack.push((input, Pre));
                        }
                    }
                    Visiting => {
                        return Err(Box::new(Error::ContainsLoop));
                    }
                    Visited(_) => {}
                },
                Post(inputs) => {
                    *state = Visited(nodes.len());
                    let mut input_array = [usize::max_value(); 4];
                    for (n, &input) in inputs.iter().enumerate() {
                        input_array[n] = match states[input.0 as usize] {
                            Visited(idx) => idx,
                            _ => panic!("node not visited"), // Should not happen.
                        };
                    }
                    nodes.push(Node {
                        function: gnodes[sig.0 as usize].instantiate(parameters)?,
                        input_count: inputs.len(),
                        inputs: input_array,
                    });
                }
            }
        }
        let buffer_size = parameters.buffer_size;
        if buffer_size == 0 {
            return Err(Box::new(Error::BadBuffer));
        }
        nodes.shrink_to_fit();
        let nodes = Box::<[Node]>::from(nodes);
        let mut buffer = Vec::new();
        let size = buffer_size.checked_mul(nodes.len()).unwrap();
        buffer.resize(size, Default::default());
        let buffer = Box::<[f32]>::from(buffer);
        Ok(Program {
            buffer_size,
            buffer,
            nodes,
            done: false,
        })
    }

    /// Render the next output buffer. This will return a series of full
    /// buffers, then optionally a short buffer, and then None.
    pub fn render(&mut self, input: &Input) -> Option<&[f32]> {
        if self.done {
            return None;
        }
        // TODO: Change this function so it doesn't allocate memory.
        let buffer_size = self.buffer_size;
        let buffer = &mut self.buffer[..];
        let nodes = &mut self.nodes[..];
        let mut outputs = Vec::new();
        outputs.resize(nodes.len(), Default::default());
        let mut state = State {
            note: input.note,
            gate: input.gate,
            end: None,
        };
        for (n, (node, output)) in nodes
            .iter_mut()
            .zip(buffer.chunks_mut(buffer_size))
            .enumerate()
        {
            let input_count = node.input_count;
            let mut inputs: [&[f32]; 4] = [Default::default(); 4];
            for (i, &index) in node.inputs[0..input_count].iter().enumerate() {
                debug_assert!(index < n);
                inputs[i] = outputs[index];
            }
            node.function
                .render(output, &inputs[0..input_count], &mut state);
            outputs[n] = output;
        }
        let output = buffer.chunks_exact(self.buffer_size).next_back().unwrap();
        Some(match state.end {
            Some(len) => {
                self.done = true;
                &output[..len]
            }
            None => output,
        })
    }
}
