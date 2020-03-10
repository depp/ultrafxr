use super::graph::{Node, NodeResult, SignalRef};
use super::program::{Function, Parameters, State};
use std::cmp::min;
use std::f32;

/// Segment of an envelope.
#[derive(Debug, Clone, Copy)]
pub enum Segment {
    /// Move to target value instantly, then hold.
    Set { value: f64 },
    /// Move to target value linearly over the given amount of time, then hold.
    Linear { time: f64, value: f64 },
    /// Move to target value exponentially with the given time constant. After
    /// one time constant, signal will decay from 1 to 1/e.
    Exponential { time_constant: f64, value: f64 },
    /// Wait the given number of seconds, extending the previous segment.
    Delay { time: f64 },
    /// Wait until the gate releases.
    Gate,
    /// Stop the synthesizer, ending audio output.
    Stop,
}

/// Envelope generator.
#[derive(Debug)]
pub struct Envelope {
    pub segments: Box<[Segment]>,
}

fn time_from(time: f32) -> usize {
    if time >= 0.0 {
        if time < usize::max_value() as f32 {
            time.round() as usize
        } else {
            usize::max_value()
        }
    } else {
        0
    }
}

impl Node for Envelope {
    fn inputs(&self) -> &[SignalRef] {
        &[]
    }
    fn instantiate(&self, parameters: &Parameters) -> NodeResult {
        let mut states = Vec::<Section>::new();
        let mut segments = Vec::<FSegment>::new();
        fn add_state(states: &mut Vec<Section>, segments: Vec<FSegment>) {
            let mut segments = segments;
            segments.shrink_to_fit();
            states.push(Section {
                generator: if states.is_empty() {
                    Generator::Constant { value: 0.0 }
                } else {
                    Generator::Passthrough { value: 0.0 }
                },
                time: Time::Done,
                index: 0,
                segments: Box::from(segments),
            });
        }
        for &seg in self.segments.iter() {
            match seg {
                Segment::Set { value } => {
                    let value = value as f32;
                    segments.push(FSegment::Set { value });
                }
                Segment::Linear { time, value } => {
                    let time = time_from((time * parameters.sample_rate) as f32);
                    let value = value as f32;
                    segments.push(FSegment::Linear { time, value });
                }
                Segment::Exponential {
                    time_constant,
                    value,
                } => {
                    let time_constant = (time_constant * parameters.sample_rate) as f32;
                    let value = value as f32;
                    segments.push(FSegment::Exponential {
                        value,
                        time_constant,
                        threshold: 0.05,
                    });
                }
                Segment::Delay { time } => {
                    let time = time_from((time * parameters.sample_rate) as f32);
                    segments.push(FSegment::Delay { time });
                }
                Segment::Gate => {
                    add_state(&mut states, segments);
                    segments = Vec::new();
                    segments.push(FSegment::Gate);
                }
                Segment::Stop => {
                    segments.push(FSegment::Stop);
                }
            }
        }
        add_state(&mut states, segments);
        states.shrink_to_fit();
        Ok(Box::new(EnvelopeF(Box::from(states))))
    }
}

#[derive(Debug, Clone, Copy, PartialEq)]
enum FSegment {
    Set {
        value: f32,
    },
    Linear {
        value: f32,
        time: usize,
    },
    Exponential {
        value: f32,
        time_constant: f32,
        threshold: f32,
    },
    Delay {
        time: usize,
    },
    Gate,
    Stop,
}

/// A generator for an infinite sequence of envelope values.
#[derive(Debug, Clone, Copy)]
enum Generator {
    Passthrough {
        value: f32,
    },
    Constant {
        value: f32,
    },
    Linear {
        value: f32,
        delta: f32,
        time: usize,
        target: f32,
    },
    Exponential {
        offset: f32,
        target: f32,
        decay: f32,
    },
}

impl Generator {
    fn value(&self) -> f32 {
        match self {
            &Generator::Passthrough { value } => value,
            &Generator::Constant { value } => value,
            &Generator::Linear { value, .. } => value,
            &Generator::Exponential { offset, target, .. } => target + offset,
        }
    }

    fn render(&mut self, output: &mut [f32]) {
        match self {
            &mut Generator::Passthrough { ref mut value } => {
                if let Some(&x) = output.last() {
                    *value = x;
                }
            }
            &mut Generator::Constant { value } => {
                for output in output.iter_mut() {
                    *output = value;
                }
            }
            &mut Generator::Linear {
                ref mut value,
                delta,
                ref mut time,
                target,
            } => {
                let mut cur = *value;
                let n = min(*time, output.len());
                for output in output[..n].iter_mut() {
                    cur += delta;
                    *output = cur;
                }
                if n < *time {
                    *value = cur;
                    *time -= n;
                } else {
                    for output in output[n..].iter_mut() {
                        *output = target;
                    }
                    *self = Generator::Constant { value: target };
                }
            }
            &mut Generator::Exponential {
                ref mut offset,
                target,
                decay,
            } => {
                let mut cur = *offset;
                for output in output.iter_mut() {
                    *output = target + cur;
                    cur *= decay;
                }
                *offset = cur;
            }
        }
    }
}

/// Result for rendering an envelope segment.
#[derive(Debug)]
enum Render {
    /// The segment is done, and this many samples were produced.
    Done(usize),
    /// The segment is not done, and filled the entire buffer.
    Continue,
}

/// The time when the current segment finishes, and the next segment starts.
#[derive(Debug, Clone, Copy)]
enum Time {
    /// Segment is done now, start next segment immediately.
    Done,
    /// Segment will run forever.
    Forever,
    /// Segment will run for a fixed amount of time.
    Timed(usize),
    /// Segment will run until gate is triggered.
    Gate,
}

/// Section of an envelope generator section. Each section is run in parallel
/// with the others, with later sections having priority.
#[derive(Debug, Clone)]
struct Section {
    generator: Generator,
    time: Time,
    index: usize,
    segments: Box<[FSegment]>,
}

impl Section {
    fn render_full(&mut self, output: &mut [f32]) -> Render {
        self.generator.render(output);
        Render::Continue
    }

    fn render_partial(&mut self, output: &mut [f32], time: usize) -> Render {
        if time < output.len() {
            self.generator.render(&mut output[..time]);
            Render::Done(time)
        } else {
            self.generator.render(output);
            self.time = Time::Timed(time - output.len());
            Render::Continue
        }
    }

    fn render_next(&mut self, output: &mut [f32], state: &State) -> Render {
        match self.time {
            Time::Done => Render::Done(0),
            Time::Forever => self.render_full(output),
            Time::Timed(time) => self.render_partial(output, time),
            Time::Gate => match state.gate() {
                Some(time) => self.render_partial(output, time),
                None => self.render_full(output),
            },
        }
    }

    fn advance(&mut self, offset: usize, state: &mut State) {
        use FSegment::*;
        let seg = match self.segments.get(self.index) {
            None => {
                self.time = Time::Forever;
                return;
            }
            Some(&seg) => seg,
        };
        self.index += 1;
        self.time = match seg {
            Set { value } => {
                self.generator = Generator::Constant { value };
                Time::Done
            }
            Linear { value, time } => {
                let target = value;
                let value = self.generator.value();
                let delta = (target - value) / (time as f32);
                self.generator = Generator::Linear {
                    value,
                    delta,
                    time,
                    target,
                };
                Time::Timed(time)
            }
            Exponential {
                value,
                time_constant,
                threshold,
            } => {
                // Curve formula:
                //     f(x) = target + offset exp(- t / time)
                //     f(x) = target + offset decay^t
                //     decay = exp(-1 / time)
                //
                // End:
                //     |f(end) - target| = threshold
                //     |offset| exp(- end / time) = threshold
                //     exp(- end / time) = threshold / |offset|
                //     end / time = log (|offset| / threshold)
                //     end = time log (|offset| / threshold)
                let target = value;
                let offset = self.generator.value() - target;
                let decay = (-1.0 / time_constant).exp();
                let time = time_from(time_constant * (offset.abs() / threshold).ln());
                self.generator = Generator::Exponential {
                    offset,
                    target,
                    decay,
                };
                Time::Timed(time)
            }
            Delay { time } => Time::Timed(time),
            Gate => Time::Gate,
            Stop => {
                state.stop(offset);
                Time::Done
            }
        };
    }

    fn render(&mut self, output: &mut [f32], state: &mut State) {
        let mut output = output;
        let mut pos = 0;
        loop {
            match self.render_next(output, state) {
                Render::Done(n) => {
                    output = &mut output[n..];
                    pos += n;
                }
                Render::Continue => return,
            }
            self.advance(pos, state);
        }
    }
}

/// Envelope function. Runs multiple envelope sections in parallel.
#[derive(Debug)]
struct EnvelopeF(Box<[Section]>);

impl Function for EnvelopeF {
    fn render(&mut self, output: &mut [f32], _inputs: &[&[f32]], state: &mut State) {
        for section in self.0.iter_mut() {
            section.render(output, state);
        }
    }
}
